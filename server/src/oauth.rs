use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use chrono::Local;
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use futures::{stream::FuturesUnordered, StreamExt};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken, RedirectUrl,
    TokenResponse,
};
use retainer::Cache;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tracing::warn;
use typeshare::typeshare;

use crate::{
    config::{Config, ProviderUrls},
    model::{NewOAuthConnection, OAuthProvision},
    schema::oauth_connections,
    user::UserFromParts,
    AppState, PgPool,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:provider/generate_url/:provides", get(get_oauth_url))
        .route("/:provider/callback", get(oauth_callback))
}

#[derive(Clone)]
pub struct OAuthClients(Arc<HashMap<String, BasicClient>>);

impl OAuthClients {
    pub async fn from_config(config: &Config) -> Self {
        let futures: FuturesUnordered<_> = config
            .integrations
            .iter()
            .filter(|(_, p)| p.urls.is_oauth_only())
            .map(|(k, v)| async move {
                let client_id = v.client_id.clone();
                let client_secret = v.client_secret.as_ref().cloned();

                let (auth_url, token_url) = if let ProviderUrls::OAuthOnly {
                    auth_url,
                    token_url,
                } = v.urls.clone()
                {
                    (auth_url, token_url)
                } else {
                    unreachable!();
                };

                let redirect_url =
                    match RedirectUrl::new(format!("{}/api/oauth/{k}/callback", config.base_url)) {
                        Ok(url) => url,
                        Err(e) => {
                            warn!("Invalid redirect URL: {e}");
                            return None;
                        }
                    };

                let client = BasicClient::new(client_id, client_secret, auth_url, Some(token_url))
                    .set_redirect_uri(redirect_url);

                Some((k.clone(), client))
            })
            .collect();

        OAuthClients(Arc::new(
            futures
                .filter_map(|r| async { r })
                .collect::<HashMap<String, BasicClient>>()
                .await,
        ))
    }

    pub async fn get(&self, provider_: &str) -> Option<&BasicClient> {
        self.0.get(provider_)
    }
}

#[derive(Clone)]
pub struct CachableCsrfToken(pub CsrfToken);

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

struct CsrfCacheRecord {
    user_id: i32,
    provides: OAuthProvision,
}

#[derive(Clone)]
pub struct CsrfCache(Arc<Cache<CachableCsrfToken, CsrfCacheRecord>>);

impl CsrfCache {
    pub fn new() -> Self {
        Self(Arc::new(Cache::new()))
    }

    pub fn spawn_monitor_thread(&self) -> JoinHandle<()> {
        let store = self.0.clone();
        tokio::spawn(async move { store.monitor(4, 0.25, Duration::from_secs(3)).await })
    }

    async fn insert(&self, token: CsrfToken, record: CsrfCacheRecord) {
        self.0
            .insert(
                CachableCsrfToken(token),
                record,
                Duration::from_secs(CSRF_TIMEOUT),
            )
            .await;
    }
}

const CSRF_TIMEOUT: u64 = 3600; // 1 hour in seconds

#[axum_macros::debug_handler(state = AppState)]
async fn get_oauth_url(
    State(oauth_clients): State<OAuthClients>,
    State(c_cache): State<CsrfCache>,
    Path((provider_, provides_)): Path<(String, OAuthProvision)>,
    UserFromParts { user, jar }: UserFromParts,
) -> Result<(CookieJar, String), StatusCode> {
    if provides_ == OAuthProvision::Auth {
        return Err(StatusCode::NOT_FOUND);
    }

    let client = oauth_clients
        .get(&provider_)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // .add_scopes(config.read().await.integrations.get(&provider_).provides)
        .url();

    c_cache
        .insert(
            csrf_state,
            CsrfCacheRecord {
                user_id: user.id,
                provides: provides_,
            },
        )
        .await;

    Ok((jar, authorize_url.to_string()))
}

pub struct OAuthRedirect;

impl IntoResponse for OAuthRedirect {
    fn into_response(self) -> Response {
        (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct OAuthError(pub String);

impl From<OAuthError> for String {
    fn from(val: OAuthError) -> Self {
        val.0
    }
}

macro_rules! impl_oauth_error {
    // (
    //     $($names:ident)::+
    //     $(< $( $lt:tt $( : $($clt:ident)::+ $(+ $($dlt:ident)::+ )* )? ),+ >)? ,
    //     $format:literal
    // ) => {
    //     impl $(< $( $lt $( : $($clt)::+ $(+ $($dlt)::+ )* )? ),+ >)?
    //         From<$($names)::+ $(< $($lt),+ >)? > for $crate::oauth::OAuthError {
    //         fn from(value: $($names)::+ $(< $($lt),+ >)?) -> Self {
    //             Self(format!($format, value.to_string()))
    //         }
    //     }
    // };

    (
        $($names:ident)::+
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)? ,
        $format:literal
    ) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            From<$($names)::+ $(< $($lt),+ >)? > for $crate::oauth::OAuthError {
            fn from(value: $($names)::+ $(< $($lt),+ >)?) -> Self {
                Self(format!($format, value.to_string()))
            }
        }
    };
}

pub(crate) use impl_oauth_error;

impl_oauth_error!(diesel::result::Error, "Database error: {}");
impl_oauth_error!(
    diesel_async::pooled_connection::bb8::RunError,
    "Pool error: {}"
);

use oauth2::ErrorResponse;
use std::error::Error;
impl_oauth_error!(
    oauth2::RequestTokenError<RE: Error + 'static, T: ErrorResponse + 'static>,
    "Provider error: {}"
);

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
                        error_msg: self.into()
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
    State(c_cache): State<CsrfCache>,
    State(pool): State<PgPool>,
    UserFromParts { user, jar }: UserFromParts,
) -> Result<(CookieJar, OAuthRedirect), OAuthError> {
    let CodeStatePair { code, state } = pair;

    // Verify CSRF token
    let csrf_record = c_cache
        .0
        .remove(&CachableCsrfToken(state))
        .await
        .ok_or(OAuthError("Invalid OAuth State".to_string()))?;
    if csrf_record.user_id != user.id {
        return Err(OAuthError("Invalid OAuth State".to_string()));
    }

    let client = oauth_clients
        .get(&provider_)
        .await
        .ok_or(OAuthError("No provider by that name exists".to_string()))?;

    let token_response = client
        .exchange_code(code)
        .request_async(async_http_client)
        .await?;

    // Save OAuth connection
    let conn = &mut pool.get().await?;

    let access_token_expires = token_response.expires_in().and_then(|d| {
        let chrono_duration = chrono::Duration::from_std(d).ok()?;
        Some(Local::now().naive_local() + chrono_duration)
    });
    let new_oauth_login = &NewOAuthConnection {
        user_id: user.id,
        provider: &provider_,
        provides: &csrf_record.provides,
        access_token: token_response.access_token().secret(),
        access_token_expires: access_token_expires.as_ref(),
        refresh_token: token_response.refresh_token().map(|t| t.secret().as_str()),
        refresh_token_expires: None,
        oid_subject: None,
    };
    insert_into(oauth_connections::table)
        .values(new_oauth_login)
        .execute(conn)
        .await?;

    Ok((jar, OAuthRedirect))
}
