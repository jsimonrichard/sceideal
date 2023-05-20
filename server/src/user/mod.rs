use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use chrono::NaiveDateTime;
use color_eyre::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use futures::TryStreamExt;
use openidconnect::{url::Url, LogoutRequest, RedirectUrl};
use serde::Serialize;
use typeshare::typeshare;

mod local;
pub mod oauth;
pub mod session;

use crate::{
    config::StatefulConfig,
    model::{LocalLogin, OAuthLogin, PermissionLevel, User},
    http_error::HttpError,
    AppState, PgConn, PgPool, SessionStore,
};

use self::oauth::OAuthClients;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/local", local::router())
        .nest("/oauth", oauth::router())
        .route("/", get(get_user))
        .route("/logout", post(logout))
}

impl User {
    pub async fn find(
        name_query: &str,
        connection: &mut PgConn<'_>,
    ) -> Result<Option<Self>, diesel::result::Error> {
        use crate::schema::users::dsl::*;
        users
            .filter(email.eq(&name_query))
            .first(connection)
            .await
            .optional()
    }

    pub async fn get(
        id_: i32,
        connection: &mut PgConn<'_>,
    ) -> Result<Option<Self>, diesel::result::Error> {
        use crate::schema::users::dsl::*;
        users.find(id_).get_result(connection).await.optional()
    }

    pub async fn from_jar(
        jar: &CookieJar,
        session: &SessionStore,
        connection: &mut PgConn<'_>,
    ) -> Result<Option<Self>, diesel::result::Error> {
        let user_id_ = if let Some(session_data) = session.get(&jar).await {
            session_data.user_id
        } else {
            return Ok(None);
        };

        User::get(user_id_, connection).await
    }

    pub async fn to_user_data(
        &self,
        conn: &mut PgConn<'_>,
    ) -> Result<UserData, diesel::result::Error> {
        let local_login_opt: Option<LocalLoginData> = LocalLogin::belonging_to(&self)
            .first::<LocalLogin>(conn)
            .await
            .optional()?
            .map(|l| l.into());
        let oauth_login_list = OAuthLogin::belonging_to(&self)
            .load_stream::<OAuthLogin>(conn)
            .await?
            .try_fold(Vec::new(), |mut acc, item| {
                acc.push(item.into());
                futures::future::ready(Ok(acc))
            })
            .await?;
        Ok(UserData {
            email: self.email.clone(),
            email_verified: self.email_verified,
            phone_number: self.phone_number.clone(),
            fname: self.fname.clone(),
            lname: self.lname.clone(),
            bio: self.bio.clone(),
            profile_image: self.profile_image.clone(),
            permission_level: self.permission_level,
            joined_on: self.joined_on,
            updated_at: self.updated_at,
            last_login: self.last_login,
            local_login: local_login_opt,
            oauth_providers: oauth_login_list,
        })
    }
}

pub struct UserFromParts {
    user: User,
    jar: CookieJar,
}

#[async_trait]
impl FromRequestParts<AppState> for UserFromParts {
    type Rejection = HttpError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Get cookie value
        let mut jar = CookieJar::from_request_parts(parts, state).await.unwrap();

        // Get the user
        let conn = &mut state.pool.get().await?;
        // Gets user and updates session TTL
        let user = User::from_jar(&jar, &state.session_store, conn)
            .await?
            .ok_or(HttpError::WithCode {
                code: StatusCode::FORBIDDEN,
                msg: "No user found",
            })?;

        // Update cookie TTL
        jar = state.session_store.reup(jar).await;

        Ok(UserFromParts { user, jar })
    }
}

#[typeshare]
#[derive(Serialize)]
pub struct UserData {
    pub email: String,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub fname: String,
    pub lname: String,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub permission_level: PermissionLevel,
    #[typeshare(serialized_as = "String")]
    pub joined_on: NaiveDateTime,
    #[typeshare(serialized_as = "String")]
    pub updated_at: NaiveDateTime,
    #[typeshare(serialized_as = "Option<String>")]
    pub last_login: Option<NaiveDateTime>,
    pub local_login: Option<LocalLoginData>,
    pub oauth_providers: Vec<OAuthLoginData>,
}

#[typeshare]
#[derive(Serialize)]
pub struct LocalLoginData {
    #[typeshare(serialized_as = "String")]
    updated_at: NaiveDateTime,
}

impl From<LocalLogin> for LocalLoginData {
    fn from(value: LocalLogin) -> Self {
        LocalLoginData {
            updated_at: value.updated_at,
        }
    }
}

#[typeshare]
#[derive(Serialize)]
pub struct OAuthLoginData {
    pub provider: String,
    pub associated_email: String,
    #[typeshare(serialized_as = "String")]
    pub updated_at: NaiveDateTime,
}

impl From<OAuthLogin> for OAuthLoginData {
    fn from(value: OAuthLogin) -> Self {
        OAuthLoginData {
            provider: value.provider,
            associated_email: value.associated_email,
            updated_at: value.updated_at,
        }
    }
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_user(
    UserFromParts { user, jar }: UserFromParts,
    State(pool): State<PgPool>,
) -> Result<(CookieJar, Json<UserData>), HttpError> {
    let mut conn = pool.get().await?;
    let user_data = user.to_user_data(&mut conn).await?;
    Ok((jar, Json(user_data)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn logout(
    State(session): State<SessionStore>,
    State(oauth_clients): State<OAuthClients>,
    State(config): State<StatefulConfig>,
    jar: CookieJar,
) -> (CookieJar, Json<Option<Url>>) {
    let (session_data, jar) = session.remove(jar).await;

    let mut logout_url = None;
    if let Some(record) = session_data.and_then(|data| data.oauth_records.into_iter().next()) {
        if let Some(client_record) = oauth_clients.get(&record.provider).await {
            if let Ok(redirect_url) = RedirectUrl::new(config.read().await.base_url.clone()) {
                logout_url = Some(
                    LogoutRequest::from(client_record.end_session_endpoint.clone())
                        .set_redirect_uri(redirect_url)
                        .url(),
                );
            }
        }
    }

    (jar, Json(logout_url))
}
