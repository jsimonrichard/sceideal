use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{extract::State, Json};
use color_eyre::{eyre::eyre, Result};
use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl};
use openidconnect::IssuerUrl;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use typeshare::typeshare;

use crate::{model::OAuthProvision, AppState};

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
    pub integrations: HashMap<String, Provider>,
    pub live_reloading: bool,
}

#[derive(Deserialize)]
pub struct Provider {
    pub client_id: ClientId,
    pub client_secret: Option<ClientSecret>,
    #[serde(flatten)]
    pub urls: ProviderUrls,
    pub provides: Vec<OAuthProvision>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum ProviderUrls {
    OAuthOnly {
        auth_url: AuthUrl,
        token_url: TokenUrl,
    },
    OpenIdConnect {
        issuer_url: IssuerUrl,
    },
}

impl ProviderUrls {
    pub fn is_open_id(&self) -> bool {
        matches!(self, Self::OpenIdConnect { .. })
    }

    pub fn is_oauth_only(&self) -> bool {
        matches!(self, Self::OAuthOnly { .. })
    }
}

pub type StatefulConfig = Arc<RwLock<Config>>;

impl Config {
    pub async fn setup() -> Result<Arc<RwLock<Self>>> {
        let config_path = std::env::var("CONFIG_FILE")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                dirs::config_local_dir().map(|mut p| {
                    p.push("sceideal");
                    p.push("config.toml");
                    p
                })
            })
            .ok_or(eyre!("Could not get config directory."))?;

        let config = Arc::new(RwLock::new(Self::load(&config_path).await?));

        // Live reload if appropriate
        // if config.read().await.live_reloading {
        //     let (tx, mut rx) = channel(1);
        //     let mut watcher = recommended_watcher(move |res| {
        //         debug!("File watcher event: {:?}", res);
        //         Handle::current().block_on(async {
        //             tx.send(res).await.unwrap();
        //         });
        //     })?;

        //     watcher.watch(&config_path, RecursiveMode::NonRecursive)?;

        //     let config = config.clone();
        //     let config_path = config_path.clone();
        //     tokio::spawn(async move {
        //         debug!("Watching {:?}", config_path);
        //         while let Some(res) = rx.recv().await {
        //             let event = match res {
        //                 Ok(event) => event,
        //                 Err(e) => {
        //                     error!("Error watching config file: {:?}", e);
        //                     continue;
        //                 }
        //             };

        //             if event.kind.is_modify() {
        //                 debug!("Config file changed!");
        //                 match Self::load(&config_path).await {
        //                     Ok(new_config) => *config.write().await = new_config,
        //                     Err(e) => {
        //                         error!("Error loading config file: {:?}", e);
        //                     }
        //                 }
        //             }
        //         }
        //     });
        // }

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
    pub oauth_providers: Vec<String>, // TODO
}

impl From<&Config> for PublicConfig {
    fn from(value: &Config) -> Self {
        PublicConfig {
            redirect_to_first_oauth_provider: value.redirect_to_first_oauth_provider,
            oauth_providers: value.integrations.keys().map(String::to_owned).collect(),
        }
    }
}

#[axum_macros::debug_handler(state = AppState)]
pub async fn get_config(State(config): State<Arc<RwLock<Config>>>) -> Json<PublicConfig> {
    Json(PublicConfig::from(&*config.read().await))
}
