use axum::routing::get;
use axum::{Router, Server};
use axum_macros::FromRef;
use color_eyre::eyre::Context;
use color_eyre::Result;
use config::{Config, StatefulConfig};
use diesel::Connection;
use diesel_async::pooled_connection::bb8::{Pool, PooledConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use oauth::{CsrfCache, OAuthClients};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use user::openid_connect::{CsrfNonceCache, OpenIdClients};
use user::session::SessionStore;

use crate::config::get_config;

mod config;
mod http_error;
mod integrations;
mod locations;
mod model;
mod oauth;
mod schema;
mod user;

pub type PgPool = Pool<AsyncPgConnection>;
pub type PgConn<'a> = PooledConnection<'a, AsyncPgConnection>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Clone, FromRef)]
pub struct AppState {
    pool: PgPool,
    session_store: SessionStore,
    config: StatefulConfig,
    oauth_clients: OAuthClients,
    c_cache: CsrfCache,
    openid_clients: OpenIdClients,
    cn_cache: CsrfNonceCache,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Init tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    dotenvy::dotenv().ok();
    color_eyre::install()?;

    // Load Config
    let config = Config::setup().await?;

    // Run pending migrations synchronously (async not supported)
    {
        let mut conn = diesel::pg::PgConnection::establish(&config.read().await.database_url)
            .wrap_err("could not get synchronous database connection")?;
        conn.run_pending_migrations(MIGRATIONS).unwrap();
    } // drop connection

    // Connect to database and create pool
    let manager =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.read().await.database_url);
    let pool = Pool::builder()
        .build(manager)
        .await
        .wrap_err("could not create database connection pool")?;

    // Create session store
    let session_store = SessionStore::new();
    let session_monitor = session_store.spawn_monitor_thread();

    // Create csrf cache
    let c_cache = CsrfCache::new();
    let c_cache_monitor = c_cache.spawn_monitor_thread();

    // Create csrf-nonce cache
    let cn_cache = CsrfNonceCache::new();
    let cn_monitor = cn_cache.spawn_monitor_thread();

    // App state and other things
    let addr = config.read().await.bind_address;
    let oauth_clients = OAuthClients::from_config(&*config.read().await).await;
    let openid_clients = OpenIdClients::from_config(&*config.read().await).await;
    let state = AppState {
        pool,
        session_store,
        config,
        oauth_clients,
        c_cache,
        openid_clients,
        cn_cache,
    };

    // Build routes
    let app = Router::new()
        .nest("/user", user::router())
        .nest("/oauth", oauth::router())
        .nest("/location", locations::router())
        .route("/config", get(get_config))
        .with_state(state);

    info!("Listening on {:?}", addr);
    Server::bind(&addr)
        .serve(Router::new().nest("/api", app).into_make_service())
        .await
        .unwrap();

    // Probably not going to run
    session_monitor.abort();
    c_cache_monitor.abort();
    cn_monitor.abort();

    Ok(())
}
