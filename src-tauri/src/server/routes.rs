use std::sync::Arc;
use axum::{routing::{get, post}, Router, Json, extract::{State, Query}, http::StatusCode};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use crate::core::{CommandBus, CommandEnvelope, CommandResult, Issuer, StudioCommand};
use crate::core::commands::{CodeLanguage, TargetEngine};

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum DispatchBody {
    Envelope(CommandEnvelope),
    Bare(StudioCommand)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveBody {
    file_path: String
}

#[derive(serde::Deserialize)]
struct GenCodeQuery {
    entity: String,
    language: Option<String>,
    engine: Option<String>,
    #[serde(rename = "gameplayStyle")]
    gameplay_style: Option<String>
}

pub fn router(bus: Arc<CommandBus>) -> Router {
    Router::new()
        .route("/api/v1/commands/dispatch", post(dispatch_command))
        .route("/api/v1/health", get(health))
        .route("/api/v1/state/summary", get(state_summary))
        .route("/api/v1/project/save", post(project_save))
        .route("/api/v1/assets/generate-code", get(generate_code))
        .route("/api/v1/mesh/export", get(export_mesh_endpoint))
        .route("/api/v1/settings", get(get_settings).post(apply_settings))
        .route("/api/v1/ai/chat", post(ai_chat))
        .with_state(bus)
}

async fn dispatch_command(
    State(bus): State<Arc<CommandBus>>,
    Json(body): Json<serde_json::Value>
) -> Json<CommandResult> {
    let envelope = match serde_json::from_value::<DispatchBody>(body) {
        Ok(DispatchBody::Envelope(e)) => e,
        Ok(DispatchBody::Bare(cmd)) => CommandEnvelope::new(Issuer::Rest, cmd),
        Err(_) => return Json(CommandResult::failure("", "Invalid command format", 0.0)),
    };
    Json(bus.dispatch(envelope).await)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "sgame-studio", "version": env!("CARGO_PKG_VERSION") }))
}

async fn state_summary(
    State(bus): State<Arc<CommandBus>>
) -> Json<serde_json::Value> {
    Json(bus.state_summary_json())
}

async fn project_save(
    State(bus): State<Arc<CommandBus>>,
    Json(body): Json<SaveBody>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match bus.save_to_zip(std::path::Path::new(&body.file_path)) {
        Ok(_) => Ok(Json(serde_json::json!({ "status": "completed", "savedTo": body.file_path }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))
    }
}

async fn generate_code(
    State(bus): State<Arc<CommandBus>>,
    Query(query): Query<GenCodeQuery>
) -> Json<serde_json::Value> {
    let command = StudioCommand::GenerateEngineCode {
        entity_name: query.entity,
        target_engine: serde_json::from_value(serde_json::json!(query.engine.unwrap_or_else(|| "godot4".to_string())))
            .unwrap_or(TargetEngine::Godot4),
        language: serde_json::from_value(serde_json::json!(query.language.unwrap_or_else(|| "gdscript".to_string())))
            .unwrap_or(CodeLanguage::Gdscript),
        gameplay_style: None,
        custom_prompt: None,
        output_file_path: None,
    };
    let result = bus.dispatch(CommandEnvelope::new(Issuer::Rest, command)).await;
    Json(serde_json::json!({ "status": result.status, "fileName": result.result.get("fileName"), "code": result.result.get("code") }))
}

#[derive(serde::Deserialize)]
struct ExportQuery {
    entity: String,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    download: bool,
}

/// GET /api/v1/mesh/export?entity=<id-or-name>&format=obj
/// Export a mesh to the requested format and return the file as base64.
async fn export_mesh_endpoint(
    State(bus): State<Arc<CommandBus>>,
    Query(query): Query<ExportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let format = crate::export::MeshFormat::from_str(query.format.as_deref().unwrap_or("obj"))
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unsupported mesh format: {:?}", query.format) })),
            )
        })?;
    let artifact = bus
        .with_state(|s| crate::export::export_mesh(s, &query.entity, format))
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;
    let data_b64 = B64.encode(&artifact.bytes);
    Ok(Json(serde_json::json!({
        "entity": query.entity,
        "format": format.extension(),
        "fileName": artifact.file_name,
        "mimeType": artifact.mime,
        "sizeBytes": artifact.bytes.len(),
        "dataBase64": data_b64,
        "download": query.download,
    })))
}

/// GET /api/v1/settings — the full settings payload (AI routing + which
/// providers have keys configured, masked) read straight from the store.
async fn get_settings() -> Json<serde_json::Value> {
    Json(crate::settings::settings_response())
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsBody {
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    clear: bool,
}

/// PUT /api/v1/settings
/// Body: { "provider": "openrouter", "apiKey": "sk-..." }
/// Saves the key (obfuscated at rest via the settings store).
async fn apply_settings(Json(body): Json<SettingsBody>) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let Some(provider) = body.provider.as_deref() else {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "missing provider" }))));
    };
    let result = crate::settings::apply_provider_settings(
        provider,
        body.api_key.as_deref(),
        body.base_url.as_deref(),
        body.clear,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(result))
}

#[derive(serde::Deserialize)]
struct AiChatBody {
    message: String,
    #[serde(default)]
    task: Option<String>,
}

/// POST /api/v1/ai/chat — run the in-app assistant through the configured chat
/// provider. Uses `resolve_with_fallback` so the storage API-key (set via the
/// Settings dialog, SGAME_<PROVIDER>_KEY, or providers.json) is picked up live.
async fn ai_chat(
    Json(body): Json<AiChatBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let message = body.message.trim();
    if message.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "empty message" }))));
    }
    let task = match body.task.as_deref() {
        Some("chat") | None => crate::ai_gateway::AiTask::ChatAssistant,
        Some(other) => {
            return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("unknown task: {other}") }))));
        }
    };
    let config = crate::ai_gateway::RoutingConfig::default_config();
    let providers = crate::ai_gateway::configured_providers();
    // Resolve the provider to check availability before dispatching, so a
    // missing API key yields a clear "configure a key" message instead of a
    // confusing upstream HTTP error.
    let resolved = config.resolve_with_fallback(task);
    let needs_key = match resolved {
        Some((provider_name, _)) => !crate::ai_gateway::provider_usable(&provider_name),
        None => true,
    };
    if needs_key {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "AI provider is not configured. Add an API key in Settings → AI Provider API Keys."
            })),
        ));
    }
    match crate::ai_gateway::run_task(&config, &providers, task, message).await {
        Ok(reply) => Ok(Json(serde_json::json!({ "reply": reply, "task": task.config_key() }))),
        Err(crate::ai_gateway::AiError::NotConfigured(key)) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": format!("AI provider for '{key}' is not configured. Add an API key in Settings → AI Provider API Keys.")
            })),
        )),
        Err(e) => Err((StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() })))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The documented REST contract sends camelCase: { provider, apiKey, baseUrl, clear }.
    #[test]
    fn settings_body_accepts_camelcase() {
        let body = r#"{"provider":"openrouter","apiKey":"sk-abc123","baseUrl":"https://openrouter.ai/api/v1"}"#;
        let parsed: SettingsBody = serde_json::from_str(body).expect("camelCase settings body should parse");
        assert_eq!(parsed.provider.as_deref(), Some("openrouter"));
        assert_eq!(parsed.api_key.as_deref(), Some("sk-abc123"));
        assert_eq!(parsed.base_url.as_deref(), Some("https://openrouter.ai/api/v1"));
        assert!(!parsed.clear);
    }

    #[test]
    fn settings_body_clear_flag() {
        let body = r#"{"provider":"openai","clear":true}"#;
        let parsed: SettingsBody = serde_json::from_str(body).unwrap();
        assert!(parsed.clear);
        assert_eq!(parsed.provider.as_deref(), Some("openai"));
    }

    #[test]
    fn ai_chat_body_parses_message_and_task() {
        let body: AiChatBody = serde_json::from_str(r#"{"message":"hello","task":"chat"}"#).unwrap();
        assert_eq!(body.message, "hello");
        assert_eq!(body.task.as_deref(), Some("chat"));
        let def: AiChatBody = serde_json::from_str(r#"{"message":"hi"}"#).unwrap();
        assert_eq!(def.task, None);
    }
}
