use cfg_if::cfg_if;

cfg_if! { if #[cfg(feature="ssr")] {
    use std::env;
    use std::sync::Arc;
    use leptos::*;
    use axum::{Router, Server, Extension};
    use axum::routing::post;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use diesel::pg::PgConnection;
    use diesel::r2d2::{ConnectionManager, Pool};
    use sceideal::app::*;
    use sceideal::fileserv::file_and_error_handler;
    use tracing::{info, Level};
    use tracing_subscriber::FmtSubscriber;
    use axum_sessions::{async_session::MemoryStore, SessionLayer};
    use rand::Rng;

    #[tokio::main]
    pub async fn main() {
        dotenvy::dotenv().ok();

        // Init tracing
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

        // Get Cargo.toml config
        let conf = get_configuration(None).await.expect("couldn't get Cargo.toml");
        let addr = conf.leptos_options.site_addr;

        // Connect to database and create pool
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::new(manager).expect("could not create database connection pool");

        // Create session store
        let store = MemoryStore::new();
        let secret = rand::thread_rng().gen::<[u8; 128]>();
        let session_layer = SessionLayer::new(store, &secret).with_secure(false);

        // Build routes
        let leptos_routes = generate_route_list(|cx| view! { cx, <App/> }).await;
        let app = Router::new()
            .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
            .leptos_routes(conf.leptos_options.clone(), leptos_routes, |cx| view! { cx, <App/> })
            .fallback(file_and_error_handler)
            .layer(Extension(Arc::new(conf.leptos_options)))
            .layer(Extension(pool))
            .layer(session_layer);

        info!("Listening on {:?}", addr);
        Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
} else {
    // Can't run anything in a non-ssr environment
    pub fn main() {}
}}
