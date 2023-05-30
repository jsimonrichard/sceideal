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
use openidconnect::{url::Url, LogoutRequest, PostLogoutRedirectUrl};
use serde::Serialize;
use typeshare::typeshare;

mod local;
pub mod openid_connect;
pub mod session;

use crate::{
    config::StatefulConfig,
    http_error::HttpError,
    model::{LocalLogin, OAuthConnection, OAuthProvision, PermissionLevel, User},
    AppState, PgConn, PgPool, SessionStore,
};

use self::openid_connect::OpenIdClients;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/local", local::router())
        .nest("/openid", openid_connect::router())
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
        let user_id_ = if let Some(session_data) = session.get(jar).await {
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
        let oauth_login_list = OAuthConnection::belonging_to(&self)
            .load_stream::<OAuthConnection>(conn)
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
    pub user: User,
    pub jar: CookieJar,
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

pub struct TeacherFromParts(pub UserFromParts);

#[async_trait]
impl FromRequestParts<AppState> for TeacherFromParts {
    type Rejection = HttpError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_from_parts = UserFromParts::from_request_parts(parts, state).await?;
        match user_from_parts.user.permission_level {
            PermissionLevel::Teacher | PermissionLevel::Admin => {
                Ok(TeacherFromParts(user_from_parts))
            }
            _ => Err(HttpError::forbidden("insufficient permissions")),
        }
    }
}

pub struct AdminFromParts(pub UserFromParts);

#[async_trait]
impl FromRequestParts<AppState> for AdminFromParts {
    type Rejection = HttpError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_from_parts = UserFromParts::from_request_parts(parts, state).await?;
        match user_from_parts.user.permission_level {
            PermissionLevel::Admin => Ok(AdminFromParts(user_from_parts)),
            _ => Err(HttpError::forbidden("insufficient permissions")),
        }
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
    pub oauth_providers: Vec<OAuthConnectionData>,
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
pub struct OAuthConnectionData {
    pub provider: String,
    pub provides: OAuthProvision,
    #[typeshare(serialized_as = "String")]
    pub created_on: NaiveDateTime,
    #[typeshare(serialized_as = "String")]
    pub updated_at: NaiveDateTime,
}

impl From<OAuthConnection> for OAuthConnectionData {
    fn from(value: OAuthConnection) -> Self {
        Self {
            provider: value.provider,
            provides: value.provides,
            created_on: value.created_on,
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
    State(oid_clients): State<OpenIdClients>,
    State(config): State<StatefulConfig>,
    jar: CookieJar,
) -> (CookieJar, Json<Option<Url>>) {
    let (session_data, jar) = session.remove(jar).await;

    let mut logout_url = None;
    if let Some(provider) = session_data.and_then(|data| {
        data.rp_logout_providers_with_open_sessions
            .into_iter()
            .next()
    }) {
        if let Some(client_record) = oid_clients.get(&provider).await {
            if let Ok(redirect_url) =
                PostLogoutRedirectUrl::new(config.read().await.base_url.clone())
            {
                logout_url = client_record.end_session_endpoint.as_ref().map(|url| {
                    LogoutRequest::from(url.clone())
                        .set_post_logout_redirect_uri(redirect_url)
                        .http_get_url()
                });
            }
        }
    }

    (jar, Json(logout_url))
}
