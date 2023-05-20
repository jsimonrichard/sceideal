use axum::routing::post;
use axum::Router;
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;
use diesel::result::DatabaseErrorKind;
use diesel::{insert_into, prelude::*};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use typeshare::typeshare;

use crate::http_error::HttpError;
use crate::model::{NewLocalLogin, PermissionLevel};
use crate::{
    model::{LocalLogin, NewUser, User},
    schema::{local_logins, users},
    user::{
        session::{SessionData, SessionStore},
        UserData,
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
) -> Result<(CookieJar, Json<UserData>), HttpError> {
    let conn = &mut pool.get().await?;

    // Create the user
    let new_user_data = NewUser {
        email: &create_user.email,
        email_verified: false,
        phone_number: create_user.phone_number.as_ref().map(String::as_str),
        fname: &create_user.fname,
        lname: &create_user.lname,
        bio: None,
        profile_image: None,
        permission_level: PermissionLevel::Student,
    };

    let id: i32 = insert_into(users::table)
        .values(&new_user_data)
        .returning(users::id)
        .get_result(conn)
        .await
        .map_err(|e| {
            if let diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) = e {
                HttpError::internal("a user with this email already exists")
            } else {
                e.into()
            }
        })?;

    // Create the login method for the user
    let new_local_login = NewLocalLogin {
        user_id: id,
        hash: &bcrypt::hash(create_user.password, bcrypt::DEFAULT_COST)?,
    };

    insert_into(local_logins::table)
        .values(new_local_login)
        .execute(conn)
        .await?;

    // Return user data
    let user = User::get(id, conn)
        .await?
        .ok_or(HttpError::internal("user not found"))?;
    let user_data = user.to_user_data(conn).await?;

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
) -> Result<(CookieJar, Json<UserData>), HttpError> {
    let conn = &mut pool.get().await?;

    let LoginData { email, password } = login_data;
    let user = User::find(&email, conn)
        .await?
        .ok_or(HttpError::internal("user not found"))?;

    if let Some(local_login_info) = LocalLogin::belonging_to(&user)
        .first::<LocalLogin>(conn)
        .await
        .optional()?
    {
        if bcrypt::verify(password, &local_login_info.hash)? {
            let user_data = user.to_user_data(conn).await?;
            jar = session.insert(SessionData::new(user.id), jar).await;
            return Ok((jar, Json(user_data)));
        }
    }

    Err(HttpError::internal("user not found"))
}
