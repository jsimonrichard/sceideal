use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use diesel::prelude::*;
use diesel::{insert_into, result::DatabaseErrorKind};
use diesel_async::pooled_connection::bb8::RunError;
use diesel_async::RunQueryDsl;
use futures::{stream::FuturesUnordered, StreamExt};
use openidconnect::{
    core::{CoreClient, CoreRequestTokenError, CoreResponseType},
    reqwest::async_http_client,
    AuthenticationFlow, AuthorizationCode, ClaimsVerificationError, ClientId, ClientSecret,
    CsrfToken, EndSessionUrl, IssuerUrl, Nonce, ProviderMetadataWithLogout, RedirectUrl,
};
use retainer::Cache;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::{trace, warn};
use typeshare::typeshare;

use crate::{
    config::{Config, StatefulConfig},
    model::{NewOAuthLogin, NewUser, OAuthLogin, PermissionLevel, User},
    schema::{oauth_logins, users},
    user::session::{OAuthRecord, SessionData, SessionStore},
    AppState, PgPool,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:provider/generate_url", get(get_oauth_url))
        .route("/:provider/callback", get(oauth_callback))
}

#[derive(Clone)]
pub struct OAuthClients(Arc<HashMap<String, OAuthClientRecord>>);

pub struct OAuthClientRecord {
    pub client: CoreClient,
    pub end_session_endpoint: EndSessionUrl,
}

impl OAuthClients {
    pub async fn from_config(config: &Config) -> Self {
        let futures: FuturesUnordered<_> = config
            .oauth_providers
            .iter()
            .map(|(k, v)| async move {
                let client_id = ClientId::new(v.client_id.clone());
                let client_secret = v.client_secret.clone().map(ClientSecret::new);
                let issuer_url = if let Ok(url) = IssuerUrl::new(v.issuer_url.clone()) {
                    url
                } else {
                    warn!("Invalid issuer url for {}", k);
                    return None;
                };

                let provider_metadata =
                    match ProviderMetadataWithLogout::discover_async(issuer_url, async_http_client)
                        .await
                    {
                        Ok(metadata) => metadata,
                        Err(e) => {
                            warn!("Failed to discover OpenId provider {k}: {e}");
                            return None;
                        }
                    };

                let redirect_url = match RedirectUrl::new(format!(
                    "{}/api/user/oauth/{k}/callback",
                    config.base_url
                )) {
                    Ok(url) => url,
                    Err(e) => {
                        warn!("Invalid redirect URL: {e}");
                        return None;
                    }
                };

                let end_session_endpoint = if let Some(url) =
                    &provider_metadata.additional_metadata().end_session_endpoint
                {
                    url.clone()
                } else {
                    warn!("Failed to discover the end session endpoint for {k}");
                    return None;
                };

                let client =
                    CoreClient::from_provider_metadata(provider_metadata, client_id, client_secret)
                        .set_redirect_uri(redirect_url);

                Some((
                    k.clone(),
                    OAuthClientRecord {
                        client,
                        end_session_endpoint,
                    },
                ))
            })
            .collect();

        OAuthClients(Arc::new(
            futures
                .filter_map(|r| async { r })
                .collect::<HashMap<String, OAuthClientRecord>>()
                .await,
        ))
    }

    pub async fn get(&self, provider_: &str) -> Option<&OAuthClientRecord> {
        self.0.get(provider_)
    }
}

#[derive(Clone)]
pub struct CachableCsrfToken(CsrfToken);

impl PartialEq for CachableCsrfToken {
    fn eq(&self, other: &Self) -> bool {
        self.0.secret() == other.0.secret()
    }
}

impl Eq for CachableCsrfToken {}

impl PartialOrd for CachableCsrfToken {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.secret().partial_cmp(other.0.secret())
    }
}

impl Ord for CachableCsrfToken {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.secret().cmp(other.0.secret())
    }
}

#[derive(Clone)]
pub struct CsrfNonceCache(Arc<Cache<CachableCsrfToken, Nonce>>);

impl CsrfNonceCache {
    pub fn new() -> Self {
        Self(Arc::new(Cache::new()))
    }

    pub fn spawn_monitor_thread(&self) -> JoinHandle<()> {
        let store = self.0.clone();
        tokio::spawn(async move { store.monitor(4, 0.25, Duration::from_secs(3)).await })
    }
}

const NONCE_TIMEOUT: u64 = 3600; // 1 hour in seconds

#[axum_macros::debug_handler(state = AppState)]
async fn get_oauth_url(
    State(oauth_clients): State<OAuthClients>,
    State(cn_cache): State<CsrfNonceCache>,
    Path(provider_): Path<String>,
) -> Result<String, StatusCode> {
    let client_record = oauth_clients
        .get(&provider_)
        .await
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
    let (authorize_url, csrf_state, nonce) = client_record
        .client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // .add_scope(Scope::new(let CodeStatePair { code, state } = pair;
        //     "https://www.googleapis.com/auth/calendar.events.readonly".to_string(),
        // ))
        // .add_scope(Scope::new(
        //     "https://www.googleapis.com/auth/calendar.readonly".to_string(),
        // ))
        .url();

    // Add state to cache
    cn_cache
        .0
        .insert(
            CachableCsrfToken(csrf_state),
            nonce,
            Duration::from_secs(NONCE_TIMEOUT),
        )
        .await;

    Ok(authorize_url.to_string())
}

pub struct OAuthRedirect;

impl IntoResponse for OAuthRedirect {
    fn into_response(self) -> Response {
        (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response()
    }
}

#[derive(Error, Debug)]
pub enum OAuthError {
    #[error("No provider by that name exists")]
    MissingProvider,
    #[error("Invalid OAuth State")]
    InvalidOAuthState,
    #[error("OAuth Provider error: {0}")]
    ProviderError(String),
    #[error("The OAuth provider couldn't produce required information")]
    MissingInformation,
    #[error("A user with this email already exists but is not logged in. To connect this OAuth provider to that user, please log in as that user first.")]
    UserAlreadyExists,
    #[error("A database error has occurred: {0}")]
    DatabaseError(String),
    #[error("A database pool error has occurred: {0}")]
    PoolError(String),
    #[error("No account exists and sign ups are not allowed; or, if your are trying to connect a provider to an existing account, sign into that account first")]
    SignUpDisallowed,
}

impl From<diesel::result::Error> for OAuthError {
    fn from(value: diesel::result::Error) -> Self {
        OAuthError::DatabaseError(value.to_string())
    }
}

impl From<RunError> for OAuthError {
    fn from(value: RunError) -> Self {
        OAuthError::PoolError(value.to_string())
    }
}

impl<RE: std::error::Error> From<CoreRequestTokenError<RE>> for OAuthError {
    fn from(value: CoreRequestTokenError<RE>) -> Self {
        OAuthError::ProviderError(value.to_string())
    }
}

impl From<ClaimsVerificationError> for OAuthError {
    fn from(value: ClaimsVerificationError) -> Self {
        OAuthError::ProviderError(value.to_string())
    }
}

#[typeshare]
#[derive(Serialize)]
pub struct OAuthErrorMessage {
    error_msg: String,
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::FOUND,
            [(
                header::LOCATION,
                format!(
                    "/?{}",
                    serde_urlencoded::to_string(OAuthErrorMessage {
                        error_msg: self.to_string()
                    })
                    .unwrap_or_default()
                ),
            )],
        )
            .into_response()
    }
}

#[derive(Deserialize)]
pub struct CodeStatePair {
    code: AuthorizationCode,
    state: CsrfToken,
}
#[allow(clippy::too_many_arguments)]
#[axum_macros::debug_handler(state = AppState)]
async fn oauth_callback(
    Path(provider_): Path<String>,
    Query(pair): Query<CodeStatePair>,
    State(oauth_clients): State<OAuthClients>,
    State(cn_cache): State<CsrfNonceCache>,
    State(pool): State<PgPool>,
    mut jar: CookieJar,
    State(config): State<StatefulConfig>,
    State(session): State<SessionStore>,
) -> Result<(CookieJar, OAuthRedirect), OAuthError> {
    // Verify oauth
    let CodeStatePair { code, state } = pair;
    let client_record = oauth_clients
        .get(&provider_)
        .await
        .ok_or(OAuthError::MissingProvider)?;
    let nonce = cn_cache
        .0
        .remove(&CachableCsrfToken(state))
        .await
        .ok_or(OAuthError::InvalidOAuthState)?;
    let token_response = client_record
        .client
        .exchange_code(code)
        .request_async(async_http_client)
        .await?;
    let id_token_verifier = client_record.client.id_token_verifier();

    let id_token_claims = token_response
        .extra_fields()
        .id_token()
        .expect("Server did not return an ID token")
        .claims(&id_token_verifier, &nonce)?;

    let email = id_token_claims
        .email()
        .ok_or(OAuthError::MissingInformation)?
        .to_string();

    // Get the current user if one is present
    let conn = &mut pool.get().await?;
    if let Some(user) = User::from_jar(&jar, &session, conn).await? {
        // Add this provider to the session
        session
            .update(
                &jar,
                OAuthRecord {
                    provider: provider_.clone(),
                    token_response,
                },
            )
            .await;

        // Add this provier to the database
        // This may fail, but that's okay
        let new_oauth_login = &NewOAuthLogin {
            user_id: user.id,
            provider: &provider_,
            associated_email: &email,
        };
        insert_into(oauth_logins::table)
            .values(new_oauth_login)
            .execute(conn)
            .await?;

        Ok((jar, OAuthRedirect))
    } else if let Some(oauth_login) = oauth_logins::dsl::oauth_logins
        .find((&provider_, &email))
        .get_result::<OAuthLogin>(conn)
        .await
        .optional()?
    {
        // Otherwise try to sign in
        let user: User = users::dsl::users
            .find(oauth_login.user_id)
            .get_result(conn)
            .await?;

        // Start session
        jar = session
            .insert(
                SessionData {
                    user_id: user.id,
                    oauth_records: vec![OAuthRecord {
                        provider: provider_.clone(),
                        token_response,
                    }],
                },
                jar,
            )
            .await;

        Ok((jar, OAuthRedirect))
    } else if config.read().await.allow_signups {
        // Sign up

        // Create the user
        let new_user_data = NewUser {
            email: &email,
            email_verified: id_token_claims.email_verified().unwrap_or_default(),
            phone_number: id_token_claims.phone_number().map(|p| p.as_str()),
            fname: id_token_claims
                .given_name()
                .ok_or(OAuthError::MissingInformation)?
                .iter()
                .next()
                .map(|(_, n)| n.as_str())
                .ok_or(OAuthError::MissingInformation)?,
            lname: id_token_claims
                .family_name()
                .ok_or(OAuthError::MissingInformation)?
                .iter()
                .next()
                .map(|(_, n)| n.as_str())
                .ok_or(OAuthError::MissingInformation)?,
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
                if let diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) =
                    e
                {
                    OAuthError::UserAlreadyExists
                } else {
                    e.into()
                }
            })?;

        // Add this provier to the database
        let new_oauth_login = &NewOAuthLogin {
            user_id: id,
            provider: &provider_,
            associated_email: &email,
        };
        insert_into(oauth_logins::table)
            .values(new_oauth_login)
            .execute(conn)
            .await?;

        // Start session
        jar = session
            .insert(
                SessionData {
                    user_id: id,
                    oauth_records: vec![OAuthRecord {
                        provider: provider_.clone(),
                        token_response,
                    }],
                },
                jar,
            )
            .await;

        Ok((jar, OAuthRedirect))
    } else {
        Err(OAuthError::SignUpDisallowed)
    }
}
