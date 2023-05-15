use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use color_eyre::{eyre::eyre, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tokio::{
    runtime::Handle,
    sync::{mpsc::channel, RwLock},
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub bind_address: SocketAddr,
    pub openid_providers: HashMap<String, OpenIdProvider>,
    pub live_reloading: bool,
}

#[derive(Serialize, Deserialize)]
pub struct OpenIdProvider {
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: String,
}

impl Config {
    pub async fn setup() -> Result<Arc<RwLock<Self>>> {
        let config_path = std::env::var("CONFIG_PATH")
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
