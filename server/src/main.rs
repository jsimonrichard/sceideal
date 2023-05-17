use std::sync::Arc;

use axum::routing::get;
use axum::{Router, Server};
use axum_macros::FromRef;
use config::{Config, StatefulConfig};
use diesel::Connection;
use diesel_async::pooled_connection::bb8::{Pool, PooledConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use retainer::Cache;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use user::oauth::{CsrfNonceCache, OAuthClients};
use user::session::SessionStore;

use crate::config::get_config;

mod config;
mod model;
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
    cn_cache: CsrfNonceCache,
}

#[tokio::main]
pub async fn main() {
    dotenvy::dotenv().ok();

    // Init tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Load Config
    let config = Config::setup().await.expect("Could not load config");

    // Run pending migrations synchronously (async not supported)
    {
        let mut conn = diesel::pg::PgConnection::establish(&config.read().await.database_url)
            .expect("could not get synchronous database connection");
        conn.run_pending_migrations(MIGRATIONS).unwrap();
    } // drop connection

    // Connect to database and create pool
    let manager =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.read().await.database_url);
    let pool = Pool::builder()
        .build(manager)
        .await
        .expect("could not create database connection pool");

    // Create session store
    let session_store = SessionStore::new();
    let session_monitor = session_store.spawn_monitor_thread();

    // Create csrf-nonce cache
    let cn_cache = CsrfNonceCache::new();
    let cn_monitor = cn_cache.spawn_monitor_thread();

    // App state and other things
    let addr = config.read().await.bind_address;
    let oauth_clients = OAuthClients::from_config(&*config.read().await).await;
    let state = AppState {
        pool,
        session_store,
        config,
        oauth_clients,
        cn_cache,
    };

    // Build routes
    let app = Router::new()
        .nest("/user", user::router())
        .route("/config", get(get_config))
        .with_state(state);

    info!("Listening on {:?}", addr);
    Server::bind(&addr)
        .serve(Router::new().nest("/api", app).into_make_service())
        .await
        .unwrap();

    // Probably not going to run
    session_monitor.abort();
    cn_monitor.abort();
}
