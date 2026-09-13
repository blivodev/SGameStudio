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
