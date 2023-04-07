use axum::{Router, Server};
use axum_macros::FromRef;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rand::rngs::StdRng;
use rand::SeedableRng;
use retainer::Cache;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod auth;
mod schema;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgConn = PooledConnection<ConnectionManager<PgConnection>>;
pub type SessionStore = Arc<Cache<String, i32>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Clone, FromRef)]
pub struct AppState {
    pool: PgPool,
    session_store: Arc<Cache<String, i32>>,
    rng: StdRng,
}

#[tokio::main]
pub async fn main() {
    dotenvy::dotenv().ok();

    // Init tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Connect to database and create pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::new(manager).expect("could not create database connection pool");

    // Run pending migrations
    {
        let mut conn = pool
            .get()
            .expect("Couldn't retrieve a database connection from the pool");
        conn.run_pending_migrations(MIGRATIONS).unwrap();
    } // drop connection

    // Create session store
    let session_store: Arc<Cache<String, i32>> = Arc::new(Cache::new());
    // Remove sessions when they expire
    let session_clone = session_store.clone();
    let monitor =
        tokio::spawn(async move { session_clone.monitor(4, 0.25, Duration::from_secs(3)).await });

    // App state
    let state = AppState {
        pool,
        session_store,
        rng: StdRng::from_entropy(),
    };

    // Build routes
    let app = Router::new()
        .nest("/user", auth::router())
        .with_state(state);

    let addr: SocketAddr = env::var("ADDR")
        .expect("ADDR must be set")
        .parse()
        .expect("bad ADDR format");
    info!("Listening on {:?}", addr);
    Server::bind(&addr)
        .serve(Router::new().nest("/api", app).into_make_service())
        .await
        .unwrap();

    // Probably not going to run
    monitor.abort();
}
