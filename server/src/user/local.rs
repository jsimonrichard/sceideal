use axum::routing::post;
use axum::Router;
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;
use diesel::result::DatabaseErrorKind;
use diesel::{insert_into, prelude::*};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use typeshare::typeshare;

use crate::model::NewLocalLogin;
use crate::{
    model::{LocalLogin, NewUser, User},
    schema::{local_logins, users},
    user::{
        session::{SessionData, SessionStore},
        UserData, UserError,
    },
    AppState, PgPool,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sign-up", post(sign_up))
        .route("/login", post(login))
}

#[typeshare]
#[derive(Deserialize)]
pub struct CreateUser {
    email: String,
    phone_number: Option<String>,
    fname: String,
    lname: String,
    password: String,
}

#[axum_macros::debug_handler(state = AppState)]
async fn sign_up(
    State(pool): State<PgPool>,
    State(session): State<SessionStore>,
    mut jar: CookieJar,
    Json(create_user): Json<CreateUser>,
) -> Result<(CookieJar, Json<UserData>), UserError> {
    let conn = &mut pool.get().await.map_err(UserError::from_pool_error)?;

    let CreateUser {
        email,
        phone_number,
        fname,
        lname,
        password,
    } = &create_user;

    // Create the user
    let new_user_data = NewUser {
        email,
        phone_number: phone_number.as_ref().map(String::as_str),
        fname,
        lname,
        bio: None,
        profile_image: None,
    };

    let id: i32 = insert_into(users::table)
        .values(&new_user_data)
        .returning(users::id)
        .get_result(conn)
        .await
        .map_err(|e| {
            if let diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) = e {
                UserError::UserAlreadyExists
            } else {
                UserError::from_diesel_error(e)
            }
        })?;

    // Create the login method for the user
    let new_local_login = NewLocalLogin {
        user_id: id,
        hash: &bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| UserError::Other(e.to_string()))?,
    };

    insert_into(local_logins::table)
        .values(new_local_login)
        .execute(conn)
        .await
        .map_err(UserError::from_diesel_error)?;

    // Return user data
    let user = User::get(id, conn)
        .await
        .map_err(UserError::from_diesel_error)?
        .ok_or(UserError::UserNotFound)?;
    let user_data = user
        .to_user_data(conn)
        .await
        .map_err(UserError::from_diesel_error)?;

    // Start session
    jar = session.insert(SessionData::new(user.id), jar).await;

    Ok((jar, Json(user_data)))
}

#[typeshare]
#[derive(Deserialize)]
pub struct LoginData {
    email: String,
    password: String,
}
#[axum_macros::debug_handler(state = AppState)]
async fn login(
    State(pool): State<PgPool>,
    State(session): State<SessionStore>,
    mut jar: CookieJar,
    Json(login_data): Json<LoginData>,
) -> Result<(CookieJar, Json<UserData>), UserError> {
    let conn = &mut pool.get().await.map_err(UserError::from_pool_error)?;

    let LoginData { email, password } = login_data;
    let user = User::find(&email, conn)
        .await
        .map_err(UserError::from_diesel_error)?
        .ok_or(UserError::UserNotFound)?;

    if let Some(local_login_info) = LocalLogin::belonging_to(&user)
        .first::<LocalLogin>(conn)
        .await
        .optional()
        .map_err(UserError::from_diesel_error)?
    {
        if bcrypt::verify(password, &local_login_info.hash).map_err(|_| UserError::UserNotFound)? {
            let user_data = user
                .to_user_data(conn)
                .await
                .map_err(UserError::from_diesel_error)?;
            jar = session.insert(SessionData::new(user.id), jar).await;
            return Ok((jar, Json(user_data)));
        }
    }

    Err(UserError::UserNotFound)
}
