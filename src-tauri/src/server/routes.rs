use std::sync::Arc;
use axum::{routing::{get, post}, Router, Json, extract::{State, Query}, http::StatusCode};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use crate::core::{CommandBus, CommandEnvelope, CommandResult, Issuer, StudioCommand};
use crate::core::commands::{CodeLanguage, SfxPreset, TargetEngine};

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
        .route("/api/v1/audio/export", get(export_audio_endpoint))
        .route("/api/v1/settings", get(get_settings).post(apply_settings))
        .route("/api/v1/ai/chat", post(ai_chat))
        .route("/api/v1/ai/agent", post(ai_agent))
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

#[derive(serde::Deserialize)]
struct AudioExportQuery {
    #[serde(default)]
    preset: Option<String>,
    #[serde(default)]
    pitch: f64,
}

/// GET /api/v1/audio/export?preset=explosion&pitch=0.0
/// Synthesizes an sfxr sound-effect preset and returns it as a WAV file (base64).
/// This gives the Audio Studio a real, downloadable audio export without any
/// external command-line encoder.
async fn export_audio_endpoint(
    Query(query): Query<AudioExportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let preset_name = query.preset.as_deref().unwrap_or("blip");
    let preset = serde_json::from_str::<SfxPreset>(&format!("\"{preset_name}\""))
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unknown SFX preset: {preset_name}") })),
            )
        })?;
    let sample_rate = crate::audio_dsp::DEFAULT_SAMPLE_RATE;
    let samples = crate::audio_dsp::render_preset(&preset, query.pitch, sample_rate);
    let wav = crate::audio_dsp::wav::encode_wav_16bit(&samples, sample_rate);
    let file_name = format!("{preset_name}.wav");
    Ok(Json(serde_json::json!({
        "preset": preset_name,
        "fileName": file_name,
        "mimeType": "audio/wav",
        "sampleRate": sample_rate,
        "durationSeconds": samples.len() as f64 / sample_rate as f64,
        "sizeBytes": wav.len(),
        "dataBase64": crate::settings::encode_b64(&wav),
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

#[derive(serde::Deserialize)]
struct AiAgentBody {
    prompt: String,
}

/// Extract the first balanced `[...]` JSON array from model output, ignoring
/// markdown fences and leading/trailing prose so responses stay robust.
fn extract_json_array(s: &str) -> &str {
    let s = s.trim();
    if let Some(start) = s.find('[') {
        if let Some(end) = s.rfind(']') {
            if end > start {
                return &s[start..=end];
            }
        }
    }
    s
}

/// Agentic builder: ask the chat provider for a structured `{command, payload}`
/// plan and execute each step against the live Command Bus. This lets the AI
/// create assets inside the app, not just reply with text.
async fn ai_agent(
    State(bus): State<Arc<CommandBus>>,
    Json(body): Json<AiAgentBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let prompt = body.prompt.trim();
    if prompt.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "empty prompt" }))));
    }
    let config = crate::ai_gateway::RoutingConfig::default_config();
    let providers = crate::ai_gateway::configured_providers();
    let resolved = config.resolve_with_fallback(crate::ai_gateway::AiTask::ChatAssistant);
    let needs_key = match resolved {
        Some((provider_name, _)) => !crate::ai_gateway::provider_usable(&provider_name),
        None => true,
    };
    if needs_key {
        return Err((StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({ "error": "AI provider is not configured. Add an API key in Settings → AI Provider API Keys." }))));
    }
    let sys = "You are SGameStudio's agentic builder. Given a user request, reply with ONLY a JSON array of one or more tool calls, no prose and no markdown fences, using this exact schema: [{\"command\":\"<name>\",\"payload\":{...}}]. Allowed commands: create_2d_shape, create_3d_primitive, generate_sfx_preset, create_music_pattern, assemble_game_entity, export_to_engine, generate_engine_code. Payloads must be camelCase and match each command's fields (example create_3d_primitive: {\"primitiveType\":\"cube\",\"name\":\"Robot\",\"dimensions\":{\"x\":1,\"y\":1,\"z\":1}}).";
    let combined = format!("{sys}\n\nUser request: {prompt}");
    let reply = crate::ai_gateway::run_task(&config, &providers, crate::ai_gateway::AiTask::ChatAssistant, &combined)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let extracted = extract_json_array(&reply);
    let plan: Vec<serde_json::Value> = serde_json::from_str(extracted)
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": format!("AI returned an unparseable plan: {e}") }))))?;
    let mut results = Vec::new();
    for item in plan {
        match serde_json::from_value::<StudioCommand>(item) {
            Ok(cmd) => {
                let name = cmd.command_name();
                let res = bus.dispatch(CommandEnvelope::new(Issuer::Ai, cmd)).await;
                results.push(serde_json::json!({ "command": name, "status": res.status, "result": res.result }));
            }
            Err(e) => results.push(serde_json::json!({ "command": null, "error": format!("could not execute step: {e}") })),
        }
    }
    Ok(Json(serde_json::json!({ "results": results })))
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

    #[test]
    fn sfx_preset_names_parse_and_render_wav() {
        use crate::core::commands::SfxPreset;
        // The query param is parsed as a snake_case string -> variant.
        assert!(matches!(serde_json::from_str::<SfxPreset>("\"explosion\"").unwrap(), SfxPreset::Explosion));
        assert!(matches!(serde_json::from_str::<SfxPreset>("\"footstep\"").unwrap(), SfxPreset::Footstep));
        assert!(serde_json::from_str::<SfxPreset>("\"nope\"").is_err());

        // Rendering a preset yields a valid RIFF/WAVE WAV file.
        let samples = crate::audio_dsp::render_preset(&SfxPreset::Blip, 0.0, 44100);
        assert!(!samples.is_empty());
        let wav = crate::audio_dsp::wav::encode_wav_16bit(&samples, 44100);
        assert!(wav.len() >= 44);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[test]
    fn extract_json_array_handles_fences_and_prose() {
        let with_fence = "Here you go:\n```json\n[{\"command\":\"create_3d_primitive\",\"payload\":{}}]\n```\nLet me know.";
        assert_eq!(extract_json_array(with_fence), r#"[{"command":"create_3d_primitive","payload":{}}]"#);
        let raw = "[{\"a\":1}]";
        assert_eq!(extract_json_array(raw), raw);
        let no_brackets = "sorry";
        assert_eq!(extract_json_array(no_brackets), "sorry");
    }

    #[test]
    fn agent_payload_deserializes_to_command() {
        // Matches StudioCommand's #[serde(tag="command", content="payload")] shape.
        let json = r#"{"command":"create_3d_primitive","payload":{"primitiveType":"cube","name":"Robot","dimensions":{"x":1,"y":1,"z":1}}}"#;
        let cmd: StudioCommand = serde_json::from_str(json).expect("command+payload should deserialize");
        assert_eq!(cmd.command_name(), "sgame_create_3d_primitive");
    }
}
