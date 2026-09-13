pub mod protocol;
pub mod resources;
pub mod sse;
pub mod tools;

use std::io::{BufRead, Write};
use protocol::{RpcRequest, RpcResponse, PARSE_ERROR, INVALID_REQUEST, METHOD_NOT_FOUND, INTERNAL_ERROR};
use crate::core::{CommandBus, SceneState};

/// Run the MCP server over stdio (line-delimited JSON-RPC 2.0).
/// Each invocation owns its own in-memory studio command bus.
pub fn run_stdio() -> anyhow::Result<()> {
    tracing::info!("MCP stdio server started");
    let (bus, _rx) = CommandBus::new(SceneState::new());
    let bus = std::sync::Arc::new(bus);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("MCP stdin read error: {e}");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<RpcRequest>(&line) {
            Ok(req) => {
                let response = runtime.block_on(handle_request(&bus, req));
                if let Some(resp) = response {
                    let out = serde_json::to_string(&resp)?;
                    writeln!(stdout, "{out}")?;
                    stdout.flush()?;
                }
            }
            Err(_) => {
                // Malformed JSON: emit a parse error response with a null id.
                let resp = RpcResponse::error(serde_json::Value::Null, PARSE_ERROR, "Parse error");
                let out = serde_json::to_string(&resp)?;
                writeln!(stdout, "{out}")?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}

/// Handle a single JSON-RPC request, returning the response (or None for notifications).
pub async fn handle_request(bus: &std::sync::Arc<CommandBus>, req: RpcRequest) -> Option<RpcResponse> {
    if req.jsonrpc != "2.0" {
        return Some(error_with_id(req, INVALID_REQUEST, "Invalid jsonrpc version"));
    }
    let id = req.id.unwrap_or(serde_json::Value::Null);

    match req.method.as_str() {
        "initialize" => Some(RpcResponse::success(
            id,
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "subscribe": false }
                },
                "serverInfo": { "name": "sgame-studio", "version": env!("CARGO_PKG_VERSION") }
            }),
        )),
        "notifications/initialized" => None,
        "tools/list" => {
            let tools = tools::tool_definitions()
                .into_iter()
                .map(|t| serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema,
                }))
                .collect::<Vec<_>>();
            Some(RpcResponse::success(id, serde_json::json!({ "tools": tools })))
        }
        "tools/call" => {
            let name = req.params["name"].as_str().unwrap_or_default().to_string();
            let arguments = req.params.get("arguments").cloned().unwrap_or(serde_json::Value::Null);
            match tools::call_tool(bus, &name, arguments).await {
                Ok(content) => Some(RpcResponse::success(
                    id,
                    serde_json::json!({
                        "content": [ { "type": "text", "text": content.to_string() } ],
                        "isError": false
                    }),
                )),
                Err(e) => Some(RpcResponse::success(
                    id,
                    serde_json::json!({
                        "content": [ { "type": "text", "text": format!("Tool error: {e}") } ],
                        "isError": true
                    }),
                )),
            }
        }
        "resources/list" => {
            let res = resources::resource_definitions()
                .iter()
                .map(|r| serde_json::json!({
                    "uri": r.uri,
                    "name": r.name,
                    "description": r.description,
                    "mimeType": r.mime_type,
                }))
                .collect::<Vec<_>>();
            Some(RpcResponse::success(id, serde_json::json!({ "resources": res })))
        }
        "resources/read" => {
            let uri = req.params["uri"].as_str().unwrap_or_default();
            match resources::read_resource(bus, uri) {
                Ok((text, mime)) => Some(RpcResponse::success(
                    id,
                    serde_json::json!({
                        "contents": [ { "uri": uri, "mimeType": mime, "text": text } ]
                    }),
                )),
                Err(e) => Some(RpcResponse::error(id, INTERNAL_ERROR, e)),
            }
        }
        other => Some(error_with_id(
            RpcRequest { id: Some(id), jsonrpc: "2.0".into(), method: other.into(), params: serde_json::Value::Null },
            METHOD_NOT_FOUND,
            format!("Method not found: {other}"),
        )),
    }
}

fn error_with_id(req: RpcRequest, code: i64, message: impl Into<String>) -> RpcResponse {
    let id = req.id.unwrap_or(serde_json::Value::Null);
    RpcResponse::error(id, code, message)
}
