use std::time::Duration;

use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use chrono::NaiveDateTime;
use cookie::Cookie;
use cookie::{time::Duration as CookieDuration, SameSite};
use diesel::*;
use diesel::{insert_into, result::DatabaseErrorKind, Insertable, Queryable};
use rand::{distributions::Alphanumeric, rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;
use typeshare::typeshare;

use crate::{model::User, schema::users, AppState, PgConn, SessionStore};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_user))
        // .route("/sign-up", post(sign_up))
        // .route("/login", post(login))
        .route("/logout", post(logout))
}

impl User {
    pub fn find(name_query: &str, connection: &mut PgConn) -> Result<Self, UserError> {
        use crate::schema::users::dsl::*;
        users
            .filter(email.eq(&name_query))
            .first(connection)
            .map_err(|e| {
                trace!("Couldn't find user: {}", e.to_string());
                UserError::UserNotFound
            })
    }

    pub fn get(id_: i32, connection: &mut PgConn) -> Result<Self, UserError> {
        use crate::schema::users::dsl::*;
        users.filter(id.eq(id_)).first(connection).map_err(|e| {
            trace!("Couldn't find get: {}", e.to_string());
            UserError::UserNotFound
        })
    }

    // pub fn create(
    //     username: &str,
    //     email: &str,
    //     password: &str,
    //     connection: &mut PgConn,
    // ) -> Result<(), UserError> {
    //     insert_into(users::table)
    //         .values(&NewUser {
    //             username,
    //             email,
    //             hash: &bcrypt::hash(password, bcrypt::DEFAULT_COST)
    //                 .map_err(|e| UserError::Other(e.to_string()))?,
    //         })
    //         .execute(connection)
    //         .map_err(|e| {
    //             if let diesel::result::Error::DatabaseError(
    //                 DatabaseErrorKind::UniqueViolation,
    //                 info,
    //             ) = e
    //             {
    //                 UserError::UserAlreadyExists(
    //                     info.constraint_name()
    //                         .and_then(|c| c.split('_').nth(1))
    //                         .unwrap_or("username or password")
    //                         .to_string(),
    //                 )
    //             } else {
    //                 UserError::DatabaseError(e.to_string())
    //             }
    //         })?;
    //     Ok(())
    // }
}

struct UserFromParts {
    user: User,
    jar: CookieJar,
}

#[async_trait]
impl FromRequestParts<AppState> for UserFromParts {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Get cookie value
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        let sid = jar
            .get(SESSION_COOKIE_NAME)
            .ok_or(StatusCode::FORBIDDEN)?
            .value()
            .to_string();

        // Get associated user_id if the seesion exists
        let user_id = *(state
            .session_store
            .get(&sid)
            .await
            .ok_or(StatusCode::FORBIDDEN)?);

        // Restart session
        state
            .session_store
            .insert(
                sid.clone(),
                state.session_store.remove(&sid).await.unwrap(),
                Duration::from_secs(SESSION_TTL),
            )
            .await;

        // Get the user
        let conn = &mut state
            .pool
            .get()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(UserFromParts {
            user: User::get(user_id, conn).map_err(|e| e.get_status_code())?,
            jar,
        })
    }
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("No user with that username, email, or password was found")]
    UserNotFound,
    #[error("A user with this {0} already exists")]
    UserAlreadyExists(String),
    #[error("Database error: `{0}`")]
    DatabaseError(String),
    #[error("An error occured: `{0}`")]
    Other(String),
}

impl UserError {
    fn get_status_code(&self) -> StatusCode {
        match self {
            UserError::UserNotFound => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for UserError {
    fn into_response(self) -> axum::response::Response {
        (
            self.get_status_code(),
            [(header::CONTENT_TYPE, "text/plain")],
            self.to_string(),
        )
            .into_response()
    }
}

impl IntoResponse for User {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_user(UserFromParts { user, jar }: UserFromParts) -> (CookieJar, User) {
    (jar, user)
}

const SESSION_COOKIE_NAME: &str = "sid";
const SESSION_TTL: u64 = 3600 * 5; // 5 hrs in seconds

#[axum_macros::debug_handler(state = AppState)]
async fn logout(
    State(session): State<SessionStore>,
    jar: CookieJar,
) -> Result<CookieJar, UserError> {
    let sid = jar
        .get(SESSION_COOKIE_NAME)
        .ok_or(UserError::UserNotFound)?
        .value()
        .to_string();
    session.remove(&sid).await;
    Ok(jar.remove(Cookie::named(SESSION_COOKIE_NAME)))
}

async fn start_session(
    user_id: i32,
    jar: CookieJar,
    rng: StdRng,
    session: SessionStore,
) -> CookieJar {
    let sid: String = rng
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    session
        .insert(sid.clone(), user_id, Duration::from_secs(SESSION_TTL))
        .await;

    // Cookie
    let cookie = Cookie::build(SESSION_COOKIE_NAME, sid)
        .max_age(CookieDuration::seconds(SESSION_TTL as i64))
        .path("/")
        .secure(cfg!(not(debug_assertions)))
        .same_site(if cfg!(debug_assertions) {
            SameSite::Lax
        } else {
            SameSite::Strict
        })
        .http_only(true)
        .finish();
    jar.add(cookie)
}
