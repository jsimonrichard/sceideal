use std::{sync::Arc, time::Duration};

use axum_extra::extract::CookieJar;
use cookie::{Cookie, SameSite};
use openidconnect::core::CoreTokenResponse;
use rand::{distributions::Alphanumeric, Rng};
use retainer::{entry::CacheReadGuard, Cache};
use tokio::task::JoinHandle;

pub const SESSION_COOKIE_NAME: &str = "sid";
pub const SESSION_TTL: u64 = 3600 * 5; // 5 hrs in seconds

pub struct OAuthRecord {
    pub provider: String,
    pub token_response: CoreTokenResponse,
}

pub struct SessionData {
    pub user_id: i32,

    // Use in place of a hashmap because the number of providers is small
    // and the number of sessions is large
    pub oauth_records: Vec<OAuthRecord>,
}

impl SessionData {
    pub fn new(user_id: i32) -> Self {
        Self {
            user_id,
            oauth_records: Vec::new(),
        }
    }

    pub fn update_oauth_records(&mut self, oauth_record: OAuthRecord) {
        if let Some(i) = self
            .oauth_records
            .iter()
            .position(|r| r.provider == oauth_record.provider)
        {
            self.oauth_records[i] = oauth_record;
        } else {
            self.oauth_records.push(oauth_record);
        }
    }

    pub fn get_oauth_token(&self, provider: String) -> Option<&CoreTokenResponse> {
        self.oauth_records
            .iter()
            .filter(|r| r.provider == provider)
            .map(|r| &r.token_response)
            .next()
    }
}

#[derive(Clone)]
pub struct SessionStore(Arc<Cache<String, SessionData>>);

impl SessionStore {
    pub fn new() -> Self {
        Self(Arc::new(Cache::new()))
    }

    pub fn spawn_monitor_thread(&self) -> JoinHandle<()> {
        let store = self.0.clone();
        tokio::spawn(async move { store.monitor(4, 0.25, Duration::from_secs(3)).await })
    }

    pub async fn insert(&self, session_data: SessionData, jar: CookieJar) -> CookieJar {
        let sid: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        self.0
            .insert(sid.clone(), session_data, Duration::from_secs(SESSION_TTL))
            .await;

        // Cookie
        let cookie = Cookie::build(SESSION_COOKIE_NAME, sid)
            .max_age(cookie::time::Duration::seconds(SESSION_TTL as i64))
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

    pub async fn get(&self, jar: &CookieJar) -> Option<CacheReadGuard<SessionData>> {
        let sid = jar.get(SESSION_COOKIE_NAME)?.value().to_string();
        self.0.get(&sid).await
    }

    pub async fn reup(&self, mut jar: CookieJar) -> CookieJar {
        if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
            let sid = cookie.value().to_string();
            self.0
                .set_expiration(&sid, Duration::from_secs(SESSION_TTL))
                .await;
            // cookie.set_max_age()
            // jar = jar.add(cookie);
        }
        jar
    }

    pub async fn remove(&self, mut jar: CookieJar) -> (Option<SessionData>, CookieJar) {
        let mut session_data = None;
        if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
            session_data = self.0.remove(&cookie.value().to_string()).await;
        }

        jar = jar.remove(Cookie::named(SESSION_COOKIE_NAME));
        (session_data, jar)
    }

    pub async fn update(&self, jar: &CookieJar, oauth_record: OAuthRecord) {
        if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
            let sid = cookie.value().to_string();
            self.0
                .update(&sid, |session_data| {
                    session_data.update_oauth_records(oauth_record)
                })
                .await;
        }
    }
}
