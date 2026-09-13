pub mod routes;
pub mod websocket;

use std::net::IpAddr;
use std::sync::Arc;
use crate::core::CommandBus;

pub async fn run_server(bus: Arc<CommandBus>, host: IpAddr, port: u16) -> anyhow::Result<()> {
    let app = routes::router(bus.clone())
        .merge(websocket::ws_router(bus.clone()))
        .merge(crate::mcp::sse::router(bus.clone()));
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    tracing::info!("SGameStudio headless server listening on http://{}:{}", host, port);
    axum::serve(listener, app).await?;
    Ok(())
}
