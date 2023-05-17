use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{extract::State, Json};
use color_eyre::{eyre::eyre, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tokio::{
    runtime::Handle,
    sync::{mpsc::channel, RwLock},
};
use typeshare::typeshare;

use crate::AppState;

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub bind_address: SocketAddr,
    /// For OAuth redirects; should include http/https schema
    pub base_url: String,
    #[serde(default)]
    pub allow_signups: bool,
    #[serde(default)]
    pub redirect_to_first_oauth_provider: bool,
    pub oauth_providers: HashMap<String, OAuthProvider>,
    pub live_reloading: bool,
}

#[derive(Deserialize)]
pub struct OAuthProvider {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub issuer_url: String,
}

pub type StatefulConfig = Arc<RwLock<Config>>;

impl Config {
    pub async fn setup() -> Result<Arc<RwLock<Self>>> {
        let config_path = std::env::var("CONFIG_FILE")
            .ok()
            .map(PathBuf::from)
            .or(dirs::config_local_dir().map(|mut p| {
                p.push("sceideal");
                p.push("config.toml");
                p
            }))
            .ok_or(eyre!("Could not get config directory."))?;

        let config = Arc::new(RwLock::new(Self::load(&config_path).await?));

        // Live reload if appropriate
        if config.read().await.live_reloading {
            let (tx, mut rx) = channel(1);
            let mut watcher = RecommendedWatcher::new(
                move |res| {
                    Handle::current().block_on(async {
                        tx.send(res).await.unwrap();
                    });
                },
                notify::Config::default(),
            )?;

            watcher.watch(&config_path, RecursiveMode::NonRecursive)?;

            let config = config.clone();
            let config_path = config_path.clone();
            tokio::spawn(async move {
                while let Some(res) = rx.recv().await {
                    let event = match res {
                        Ok(event) => event,
                        Err(e) => {
                            eprintln!("Error watching config file: {:?}", e);
                            continue;
                        }
                    };

                    if event.kind.is_modify() {
                        match Self::load(&config_path).await {
                            Ok(new_config) => *config.write().await = new_config,
                            Err(e) => {
                                eprintln!("Error loading config file: {:?}", e);
                            }
                        }
                    }
                }
            });
        }

        Ok(config)
    }

    async fn load(path: &Path) -> Result<Self> {
        let config_str = String::from_utf8(tokio::fs::read(path).await?)?;
        Ok(toml::from_str(&config_str)?)
    }
}

#[typeshare]
#[derive(Serialize)]
pub struct PublicConfig {
    pub redirect_to_first_oauth_provider: bool,
    pub oauth_providers: Vec<String>,
}

impl From<&Config> for PublicConfig {
    fn from(value: &Config) -> Self {
        PublicConfig {
            redirect_to_first_oauth_provider: value.redirect_to_first_oauth_provider,
            oauth_providers: value
                .oauth_providers
                .keys()
                .into_iter()
                .map(String::to_owned)
                .collect(),
        }
    }
}

#[axum_macros::debug_handler(state = AppState)]
pub async fn get_config(State(config): State<Arc<RwLock<Config>>>) -> Json<PublicConfig> {
    Json(PublicConfig::from(&*config.read().await))
}
