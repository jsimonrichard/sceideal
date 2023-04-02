use cfg_if::cfg_if;
pub mod app;
pub mod auth;
pub mod error_template;
pub mod fileserv;

cfg_if! { if #[cfg(feature = "hydrate")] {
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::app::*;

    #[wasm_bindgen]
    pub fn hydrate() {
        // initializes logging using the `log` crate
        _ = console_log::init_with_level(log::Level::Debug);
        console_error_panic_hook::set_once();

        leptos::mount_to_body(move |cx| {
            view! { cx, <App/> }
        });
    }
}}

cfg_if! { if #[cfg(feature = "ssr")] {
    use diesel::pg::PgConnection;
    use diesel::r2d2::{Pool, PooledConnection, ConnectionManager};
    use leptos::*;
    use axum_sessions::SessionHandle;

    pub mod schema;

    pub type PooledPgConnection = PooledConnection<ConnectionManager<PgConnection>>;

    pub fn get_connection(cx: Scope) -> Result<PooledPgConnection, ServerFnError> {
        Ok(use_context::<Pool<ConnectionManager<PgConnection>>>(cx)
            .ok_or(ServerFnError::ServerError("Pool missing.".to_string()))?
            .get()
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?)
    }

    pub fn get_session(cx: Scope) -> Result<SessionHandle, ServerFnError> {
        Ok(use_context::<SessionHandle>(cx)
            .ok_or(ServerFnError::ServerError("Session missing.".to_string()))?)
    }
}}
