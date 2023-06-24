use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use chrono::Local;
use diesel::prelude::*;
use diesel::{insert_into, result::DatabaseErrorKind};
use diesel_async::RunQueryDsl;
use futures::{stream::FuturesUnordered, StreamExt};
use oauth2::TokenResponse;
use openidconnect::{
    core::{CoreClient, CoreResponseType},
    reqwest::async_http_client,
    AuthenticationFlow, AuthorizationCode, ClaimsVerificationError, CsrfToken, EndSessionUrl,
    Nonce, ProviderMetadataWithLogout, RedirectUrl,
};
use retainer::Cache;
use serde::Deserialize;
use tokio::task::JoinHandle;
use tracing::warn;

use crate::{
    config::{Config, ProviderUrls, StatefulConfig},
    model::{NewOAuthConnection, NewUser, OAuthConnection, OAuthProvision, User},
    oauth::{impl_oauth_error, OAuthError, OAuthRedirect},
    schema::{oauth_connections, users},
    user::session::{SessionData, SessionStore},
    AppState, PgPool,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:provider/generate_url", get(get_oauth_url))
        .route("/:provider/callback", get(openid_callback))
}

#[derive(Clone)]
pub struct OpenIdClients(Arc<HashMap<String, OpenIdClientRecord>>);

pub struct OpenIdClientRecord {
    pub client: CoreClient,
    pub end_session_endpoint: Option<EndSessionUrl>,
}

impl OpenIdClients {
    pub async fn from_config(config: &Config) -> Self {
        let futures: FuturesUnordered<_> = config
            .integrations
            .iter()
            .filter(|(_, p)| p.provides.contains(&OAuthProvision::Auth) || p.urls.is_open_id())
            .map(|(k, v)| async move {
                let issuer_url = if let ProviderUrls::OpenIdConnect { ref issuer_url } = v.urls {
                    issuer_url.clone()
                } else {
                    warn!("Auth providers must specify an issuer URL (and omit the auth and token URLs)");
                    return None;
                };

                let client_id = v.client_id.clone();
                let client_secret = v.client_secret.as_ref().cloned();

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
                
                let end_session_endpoint = provider_metadata
                    .additional_metadata()
                    .end_session_endpoint
                    .as_ref().cloned();

                let redirect_url = match RedirectUrl::new(format!(
                    "{}/api/user/openid/{k}/callback",
                    config.base_url
                )) {
                    Ok(url) => url,
                    Err(e) => {
                        warn!("Invalid redirect URL: {e}");
                        return None;
                    }
                };

                let client =
                    CoreClient::from_provider_metadata(provider_metadata, client_id, client_secret)
                        .set_redirect_uri(redirect_url);

                Some((
                    k.clone(),
                    OpenIdClientRecord {
                        client,
                        end_session_endpoint,
                    },
                ))
            })
            .collect();

        OpenIdClients(Arc::new(
            futures
                .filter_map(|r| async { r })
                .collect::<HashMap<String, OpenIdClientRecord>>()
                .await,
        ))
    }

    pub async fn get(&self, provider_: &str) -> Option<&OpenIdClientRecord> {
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
        tokio::spawn(async move {
            store
                .monitor(4, 0.25, std::time::Duration::from_secs(3))
                .await
        })
    }
}

const NONCE_TIMEOUT: u64 = 3600; // 1 hour in seconds

#[axum_macros::debug_handler(state = AppState)]
async fn get_oauth_url(
    State(oid_clients): State<OpenIdClients>,
    State(cn_cache): State<CsrfNonceCache>,
    Path(provider_): Path<String>,
) -> Result<String, StatusCode> {
    let client_record = oid_clients
        .get(&provider_)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
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
            std::time::Duration::from_secs(NONCE_TIMEOUT),
        )
        .await;

    Ok(authorize_url.to_string())
}

impl_oauth_error!(
    ClaimsVerificationError,
    "Open Id Connect Claim Verification Error: {}"
);

#[derive(Deserialize)]
pub struct CodeStatePair {
    code: AuthorizationCode,
    state: CsrfToken,
}
#[allow(clippy::too_many_arguments)]
#[axum_macros::debug_handler(state = AppState)]
async fn openid_callback(
    Path(provider_): Path<String>,
    Query(pair): Query<CodeStatePair>,
    State(oid_clients): State<OpenIdClients>,
    State(cn_cache): State<CsrfNonceCache>,
    State(pool): State<PgPool>,
    mut jar: CookieJar,
    State(config): State<StatefulConfig>,
    State(session): State<SessionStore>,
) -> Result<(CookieJar, OAuthRedirect), OAuthError> {
    // Verify openid
    let CodeStatePair { code, state } = pair;
    let client_record = oid_clients
        .get(&provider_)
        .await
        .ok_or(OAuthError("No provider by that name exists".to_string()))?;
    let nonce = cn_cache
        .0
        .remove(&CachableCsrfToken(state))
        .await
        .ok_or(OAuthError("Invalid OAuth State".to_string()))?;
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

    let oid_subject_: &str = id_token_claims.subject();

    // Get the current user if one is present
    let conn = &mut pool.get().await?;
    if let Some(user) = User::from_jar(&jar, &session, conn).await? {
        if client_record.end_session_endpoint.is_some() {
            session.add_rp_provider(&jar, provider_.clone()).await;
        }

        // Add this provier to the database
        let access_token_expires = token_response.expires_in().and_then(|d| {
            let chrono_duration = chrono::Duration::from_std(d).ok()?;
            Some(Local::now().naive_local() + chrono_duration)
        });
        let new_oauth_login = &NewOAuthConnection {
            user_id: user.id,
            provider: &provider_,
            provides: &OAuthProvision::Auth,
            access_token: token_response.access_token().secret(),
            access_token_expires: access_token_expires.as_ref(),
            refresh_token: token_response.refresh_token().map(|t| t.secret().as_str()),
            refresh_token_expires: None,
            oid_subject: Some(oid_subject_),
        };
        insert_into(oauth_connections::table)
            .values(new_oauth_login)
            .execute(conn)
            .await?;

        Ok((jar, OAuthRedirect))
    } else if let Some(oid_connection) = {
        use crate::schema::oauth_connections::dsl::*;
        oauth_connections
            .filter(provider.eq(&provider_))
            .filter(oid_subject.eq(&oid_subject_))
            .first::<OAuthConnection>(conn)
            .await
            .optional()?
    } {
        // Otherwise try to sign in
        let user: User = users::dsl::users
            .find(oid_connection.user_id)
            .get_result(conn)
            .await?;

        // Start session
        jar = session
            .insert(
                SessionData {
                    user_id: user.id,
                    rp_logout_providers_with_open_sessions: vec![provider_],
                },
                jar,
            )
            .await;

        Ok((jar, OAuthRedirect))
    } else if config.read().await.allow_signups {
        // Sign up

        // Create the user
        let missing_info_error = OAuthError(
            "The OpenId Connect provider couldn't produce required information".to_string(),
        );
        let email = id_token_claims
            .email()
            .ok_or_else(|| missing_info_error.clone())?;
        let new_user_data = NewUser {
            email,
            email_verified: id_token_claims.email_verified().unwrap_or_default(),
            phone_number: id_token_claims.phone_number().map(|p| p.as_str()),
            fname: id_token_claims
                .given_name()
                .ok_or_else(|| missing_info_error.clone())?
                .iter()
                .next()
                .map(|(_, n)| n.as_str())
                .ok_or_else(|| missing_info_error.clone())?,
            lname: id_token_claims
                .family_name()
                .ok_or_else(|| missing_info_error.clone())?
                .iter()
                .next()
                .map(|(_, n)| n.as_str())
                .ok_or(missing_info_error)?,
            bio: None,
            profile_image: None,
            permission_level: None,
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
                    OAuthError("A user with that email already exists".to_string())
                } else {
                    e.into()
                }
            })?;

        // Add this provier to the database
        let access_token_expires = token_response.expires_in().and_then(|d| {
            let chrono_duration = chrono::Duration::from_std(d).ok()?;
            Some(Local::now().naive_local() + chrono_duration)
        });
        let new_oauth_login = &NewOAuthConnection {
            user_id: id,
            provider: &provider_,
            provides: &OAuthProvision::Auth,
            access_token: token_response.access_token().secret(),
            access_token_expires: access_token_expires.as_ref(),
            refresh_token: token_response.refresh_token().map(|t| t.secret().as_str()),
            refresh_token_expires: None,
            oid_subject: Some(oid_subject_),
        };
        insert_into(oauth_connections::table)
            .values(new_oauth_login)
            .execute(conn)
            .await?;

        // Start session
        jar = session
            .insert(
                SessionData {
                    user_id: id,
                    rp_logout_providers_with_open_sessions: vec![provider_],
                },
                jar,
            )
            .await;

        Ok((jar, OAuthRedirect))
    } else {
        Err(OAuthError(
            "Automatic sign-ups have been disallowed".to_string(),
        ))
    }
}
