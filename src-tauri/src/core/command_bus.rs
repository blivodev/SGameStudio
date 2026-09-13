use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use crate::core::commands::{CommandEnvelope, CommandResult};
use crate::core::state_store::{EntitySnapshot, SceneState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum StudioEvent {
    #[serde(rename_all = "camelCase")]
    CommandExecuted { command_id: String, ok: bool },
    #[serde(rename_all = "camelCase")]
    SceneMutated { command: String, affected_entities: Vec<String> },
    #[serde(rename_all = "camelCase")]
    EntityCreated { id: String, kind: String },
    #[serde(rename_all = "camelCase")]
    AiTaskProgress { task_id: String, progress: f64, status_message: String },
}

#[derive(Debug, Clone)]
struct UndoEntry { description: String, before: Vec<EntitySnapshot>, after: Vec<EntitySnapshot> }

pub struct CommandBus {
    state: std::sync::RwLock<SceneState>,
    event_tx: broadcast::Sender<StudioEvent>,
    undo_stack: std::sync::RwLock<Vec<UndoEntry>>,
    redo_stack: std::sync::RwLock<Vec<UndoEntry>>,
}

impl CommandBus {
    pub fn new(state: SceneState) -> (Self, broadcast::Receiver<StudioEvent>) {
        let (event_tx, event_rx) = broadcast::channel(256);
        let bus = Self {
            state: std::sync::RwLock::new(state),
            event_tx,
            undo_stack: std::sync::RwLock::new(Vec::new()),
            redo_stack: std::sync::RwLock::new(Vec::new()),
        };
        (bus, event_rx)
    }

    pub async fn dispatch(&self, envelope: CommandEnvelope) -> CommandResult {
        let start = std::time::Instant::now();
        let cid = envelope.command_id.clone();
        let ms = || start.elapsed().as_secs_f64() * 1000.0;

        if let Err(e) = envelope.command.validate() {
            let _ = self.event_tx.send(StudioEvent::CommandExecuted { command_id: cid.clone(), ok: false });
            return CommandResult::failure(cid, e, ms());
        }

        let affected = envelope.command.affected_entity_ids();
        let before = self.state.read().unwrap().snapshot_entities(&affected);

        // Scope the write guard so it is released before the read below.
        let outcome = {
            let mut guard = self.state.write().unwrap();
            guard.apply(&envelope.command)
        };

        match outcome {
            Ok(value) => {
                let after = self.state.read().unwrap().snapshot_entities(&affected);
                let description = envelope.command.command_name().to_string();
                self.undo_stack.write().unwrap().push(UndoEntry { description: description.clone(), before, after });
                self.redo_stack.write().unwrap().clear();

                let _ = self.event_tx.send(StudioEvent::CommandExecuted { command_id: cid.clone(), ok: true });
                let _ = self.event_tx.send(StudioEvent::SceneMutated { command: description, affected_entities: affected });

                if let Some(id) = value.get("entityId").and_then(|v| v.as_str()) {
                    let kind = value.get("kind").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let _ = self.event_tx.send(StudioEvent::EntityCreated { id: id.to_string(), kind });
                }

                CommandResult::success(cid, value, ms())
            }
            Err(e) => {
                let _ = self.event_tx.send(StudioEvent::CommandExecuted { command_id: cid.clone(), ok: false });
                CommandResult::failure(cid, e, ms())
            }
        }
    }

    pub async fn undo(&self) -> CommandResult {
        if let Some(entry) = self.undo_stack.write().unwrap().pop() {
            let affected = entry.before.iter().filter_map(|s| s.record.as_ref().map(|_| s.id.clone())).collect::<Vec<_>>();
            let description = entry.description.clone();
            self.state.write().unwrap().restore(entry.before.clone());
            self.redo_stack.write().unwrap().push(entry);
            let _ = self.event_tx.send(StudioEvent::SceneMutated { command: "undo".to_string(), affected_entities: affected });
            CommandResult::success("undo", serde_json::json!({ "undone": description }), 0.0)
        } else {
            CommandResult::failure("undo", "Nothing to undo", 0.0)
        }
    }

    pub async fn redo(&self) -> CommandResult {
        if let Some(entry) = self.redo_stack.write().unwrap().pop() {
            let affected = entry.after.iter().filter_map(|s| s.record.as_ref().map(|_| s.id.clone())).collect::<Vec<_>>();
            let description = entry.description.clone();
            self.state.write().unwrap().restore(entry.after.clone());
            self.undo_stack.write().unwrap().push(entry);
            let _ = self.event_tx.send(StudioEvent::SceneMutated { command: "redo".to_string(), affected_entities: affected });
            CommandResult::success("redo", serde_json::json!({ "redone": description }), 0.0)
        } else {
            CommandResult::failure("redo", "Nothing to redo", 0.0)
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<StudioEvent> {
        self.event_tx.subscribe()
    }

    pub fn state_summary_json(&self) -> serde_json::Value {
        let state = self.state.read().unwrap();
        serde_json::json!({ "meta": serde_json::to_value(&state.meta).unwrap_or(serde_json::Value::Null), "entityCount": state.entity_count(), "counts": state.count_by_kind() })
    }

    pub fn with_state<R>(&self, f: impl FnOnce(&SceneState) -> R) -> R {
        let state = self.state.read().unwrap();
        f(&state)
    }

    pub fn save_to_zip(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let state = self.state.read().unwrap();
        crate::core::project_zip::save_project(&state, path)
    }

    pub fn load_from_zip(path: &std::path::Path) -> anyhow::Result<Self> {
        let state = crate::core::project_zip::load_project(path)?;
        Ok(CommandBus::new(state).0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::commands::{StudioCommand, Issuer, PrimitiveType};
    

    #[tokio::test]
    async fn test_dispatch_create_primitive() {
        let (bus, _) = CommandBus::new(SceneState::new());
        let cmd = StudioCommand::Create3DPrimitive {
            primitive_type: PrimitiveType::Cube,
            name: "Hero".to_string(),
            dimensions: None,
            subdivisions: None,
        };
        let result = bus.dispatch(CommandEnvelope::new(Issuer::Gui, cmd)).await;
        assert_eq!(result.status, "success");
        assert_eq!(bus.state_summary_json()["entityCount"], 1);
    }

    #[tokio::test]
    async fn test_undo_redo() {
        let (bus, _) = CommandBus::new(SceneState::new());
        let cmd = StudioCommand::Create3DPrimitive {
            primitive_type: PrimitiveType::Cube,
            name: "Hero".to_string(),
            dimensions: None,
            subdivisions: None,
        };
        bus.dispatch(CommandEnvelope::new(Issuer::Gui, cmd)).await;
        assert_eq!(bus.state_summary_json()["entityCount"], 1);

        let undo_result = bus.undo().await;
        assert_eq!(undo_result.status, "success");
        assert_eq!(bus.state_summary_json()["entityCount"], 0);

        let redo_result = bus.redo().await;
        assert_eq!(redo_result.status, "success");
        assert_eq!(bus.state_summary_json()["entityCount"], 1);
    }

    #[tokio::test]
    async fn test_dispatch_invalid_command() {
        let (bus, _) = CommandBus::new(SceneState::new());
        let cmd = StudioCommand::Create3DPrimitive {
            primitive_type: PrimitiveType::Cube,
            name: "".to_string(),
            dimensions: None,
            subdivisions: None,
        };
        let result = bus.dispatch(CommandEnvelope::new(Issuer::Gui, cmd)).await;
        assert_eq!(result.status, "failure");
    }
}