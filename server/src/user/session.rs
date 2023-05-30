use std::{sync::Arc, time::Duration};

use axum_extra::extract::CookieJar;
use cookie::{Cookie, SameSite};
use rand::{distributions::Alphanumeric, Rng};
use retainer::{entry::CacheReadGuard, Cache};
use tokio::task::JoinHandle;

pub const SESSION_COOKIE_NAME: &str = "sid";
pub const SESSION_TTL: u64 = 3600 * 5; // 5 hrs in seconds

pub struct SessionData {
    pub user_id: i32,
    pub rp_logout_providers_with_open_sessions: Vec<String>,
}

impl SessionData {
    pub fn new(user_id: i32) -> Self {
        Self {
            user_id,
            rp_logout_providers_with_open_sessions: Vec::new(),
        }
    }

    fn add_rp_providers(&mut self, provider: String) {
        if !self
            .rp_logout_providers_with_open_sessions
            .contains(&provider)
        {
            self.rp_logout_providers_with_open_sessions.push(provider);
        }
    }
}

#[derive(Clone)]
pub struct SessionStore(Arc<Cache<String, SessionData>>);

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

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

    pub async fn reup(&self, jar: CookieJar) -> CookieJar {
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

    pub async fn add_rp_provider(&self, jar: &CookieJar, provider: String) {
        if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
            let sid = cookie.value().to_string();
            self.0
                .update(&sid, |session_data| session_data.add_rp_providers(provider))
                .await;
        }
    }
}
