pub mod routes;
pub mod websocket;

use std::net::IpAddr;
use std::sync::Arc;
use axum::http::{header, Method};
use tower_http::cors::{Any, CorsLayer};
use crate::core::CommandBus;

/// Default port the embedded backend listens on so the desktop UI's REST/WS
/// client (which targets http://127.0.0.1:8080) can reach it. Overridable via
/// the `SGAME_PORT` env var (and the GUI automatically mirrors that value).
pub fn default_port() -> u16 {
    std::env::var("SGAME_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}

pub async fn run_server(bus: Arc<CommandBus>, host: IpAddr, port: u16) -> anyhow::Result<()> {
    // The desktop UI runs in a WebView on a different origin (http://localhost:1420
    // in dev, tauri://localhost in the packaged app), so cross-origin REST/WS calls
    // to the loopback backend must be allowed. Permissive is safe because the server
    // only ever binds to 127.0.0.1 by default.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT]);
    let app = routes::router(bus.clone())
        .merge(websocket::ws_router(bus.clone()))
        .merge(crate::mcp::sse::router(bus.clone()))
        .layer(cors);
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    tracing::info!("SGameStudio headless server listening on http://{}:{}", host, port);
    axum::serve(listener, app).await?;
    Ok(())
}
