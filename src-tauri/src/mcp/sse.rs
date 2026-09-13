use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use axum::{
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use futures::stream::{self, Stream};
use tokio::sync::broadcast;
use crate::core::CommandBus;
use crate::mcp::protocol::{RpcRequest, RpcResponse, PARSE_ERROR};

/// Simplest shared session registry for the SSE transport.
pub struct McpSessionStore {
    sessions: RwLock<HashMap<String, broadcast::Sender<RpcResponse>>>,
}

impl McpSessionStore {
    pub fn new() -> Self {
        Self { sessions: RwLock::new(HashMap::new()) }
    }

    fn register(&self, id: String) -> broadcast::Receiver<RpcResponse> {
        let (tx, rx) = broadcast::channel(256);
        self.sessions.write().unwrap().insert(id, tx);
        rx
    }

    fn sender(&self, id: &str) -> Option<broadcast::Sender<RpcResponse>> {
        self.sessions.read().unwrap().get(id).cloned()
    }
}

#[derive(serde::Deserialize)]
struct SessionQuery {
    session_id: Option<String>,
}

#[derive(Clone)]
pub struct McpState {
    pub bus: Arc<CommandBus>,
    pub store: Arc<McpSessionStore>,
}

impl McpState {
    pub fn new(bus: Arc<CommandBus>) -> Self {
        Self { bus, store: Arc::new(McpSessionStore::new()) }
    }
}

pub fn router(bus: Arc<CommandBus>) -> Router {
    let state = McpState::new(bus);
    Router::new()
        .route("/mcp/sse", get(handle_sse))
        .route("/mcp/message", post(handle_message))
        .with_state(state)
}

/// Open an SSE event stream for a new MCP session.
async fn handle_sse(
    State(state): State<McpState>,
) -> impl IntoResponse {
    let session_id = uuid::Uuid::new_v4().to_string();
    let rx = state.store.register(session_id.clone());
    let endpoint = format!("http://127.0.0.1:8080/mcp/message?sessionId={session_id}");

    let stream: std::pin::Pin<Box<dyn Stream<Item = Result<String, std::convert::Infallible>> + Send>> = {
        let seed: (Option<String>, broadcast::Receiver<RpcResponse>) =
            (Some(format!("event: endpoint\ndata: {endpoint}\n\n")), rx);
        Box::pin(stream::unfold(seed, |(mut first, mut rx)| async move {
            if let Some(f) = first.take() {
                return Some((Ok(f), (None, rx)));
            }
            loop {
                match rx.recv().await {
                    Ok(resp) => {
                        let data = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".into());
                        return Some((Ok(format!("event: message\ndata: {data}\n\n")), (None, rx)));
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => return None,
                }
            }
        }))
    };

    Response::builder()
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// Receive a JSON-RPC message for a session and broadcast its response over SSE.
async fn handle_message(
    State(state): State<McpState>,
    Query(query): Query<SessionQuery>,
    body: String,
) -> Response {
    let session_id = match query.session_id {
        Some(id) if !id.is_empty() => id,
        _ => return (StatusCode::BAD_REQUEST, "missing sessionId").into_response(),
    };
    let Some(tx) = state.store.sender(&session_id) else {
        return (StatusCode::NOT_FOUND, "unknown session").into_response();
    };

    let response = match serde_json::from_str::<RpcRequest>(&body) {
        Ok(req) => crate::mcp::handle_request(&state.bus, req).await,
        Err(_) => Some(RpcResponse::error(serde_json::Value::Null, PARSE_ERROR, "Parse error")),
    };

    if let Some(resp) = response {
        let _ = tx.send(resp);
    }
    (StatusCode::ACCEPTED, "accepted").into_response()
}
