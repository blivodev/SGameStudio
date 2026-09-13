use std::sync::Arc;
use axum::{extract::{State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use futures::{SinkExt, StreamExt};
use crate::core::CommandBus;

pub fn ws_router(bus: Arc<CommandBus>) -> Router {
    Router::new().route("/ws", get(ws_handler)).with_state(bus)
}

async fn ws_handler(
    State(bus): State<Arc<CommandBus>>,
    ws: WebSocketUpgrade
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, bus))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, bus: Arc<CommandBus>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = bus.subscribe();
    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let text = serde_json::to_string(&event).unwrap_or_default();
                    if sender.send(axum::extract::ws::Message::text(text)).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    });
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {}
    });
    tokio::select! {
        _ = &mut send_task => {},
        _ = &mut recv_task => {}
    }
}
