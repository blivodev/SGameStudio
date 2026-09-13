use serde::{Serialize, Deserialize};
use serde_json::json;
use chrono::Local;
use rusqlite::{Connection, OptionalExtension};
use std::collections::BTreeMap;
use uuid::Uuid;
use anyhow::Result;

use crate::core::commands::{StudioCommand, TargetEngine, ProjectTemplate, ShapeType, BooleanOp, Rig2DTopology, PrimitiveType, Rig3DTemplate, SfxPreset, CharacterEffect, AiDomain, CodeLanguage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Shape2D,
    NodeModel,
    Mesh3D,
    AudioClip,
    MusicTrack,
    Entity,
    Rig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRecord {
    pub id: String,
    pub kind: EntityKind,
    pub name: String,
    pub data: serde_json::Value,
    pub op_log: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMeta {
    pub name: String,
    pub target_engine: String,
    pub template: String,
    pub app_version: String,
    pub created_at: String,
}

impl Default for ProjectMeta {
    fn default() -> Self {
        Self {
            name: "Untitled".into(),
            target_engine: "godot4".into(),
            template: "empty".into(),
            app_version: env!("CARGO_PKG_VERSION").into(),
            created_at: Local::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    pub id: String,
    pub record: Option<EntityRecord>,
}

#[derive(Debug, Clone)]
pub struct SceneState {
    pub meta: ProjectMeta,
    pub entities: BTreeMap<String, EntityRecord>,
    next_id: u64,
}

impl SceneState {
    pub fn new() -> Self {
        Self {
            meta: ProjectMeta::default(),
            entities: BTreeMap::new(),
            next_id: 1,
        }
    }

    pub fn from_parts(meta: ProjectMeta, entities: BTreeMap<String, EntityRecord>) -> Self {
        let next_id = entities
            .keys()
            .filter_map(|k| k.split('_').last().and_then(|s| s.parse::<u64>().ok()))
            .max()
            .map(|max| max + 1)
            .unwrap_or(1);
        Self {
            meta,
            entities,
            next_id,
        }
    }

    fn next_entity_id(&mut self, prefix: &str) -> String {
        let id = format!("{}_{}", prefix, self.next_id);
        self.next_id += 1;
        id
    }

    fn insert_record(&mut self, kind: EntityKind, name: &str, data: serde_json::Value) -> String {
        let id = match kind {
            EntityKind::Shape2D => self.next_entity_id("shape2d"),
            EntityKind::NodeModel => self.next_entity_id("node_model"),
            EntityKind::Mesh3D => self.next_entity_id("mesh3d"),
            EntityKind::AudioClip => self.next_entity_id("audio"),
            EntityKind::MusicTrack => self.next_entity_id("music"),
            EntityKind::Entity => self.next_entity_id("entity"),
            EntityKind::Rig => self.next_entity_id("rig"),
        };
        let record = EntityRecord {
            id: id.clone(),
            kind,
            name: name.into(),
            data,
            op_log: vec!["created".into()],
            created_at: Local::now().to_rfc3339(),
        };
        self.entities.insert(id.clone(), record);
        id
    }

    pub fn apply(&mut self, cmd: &StudioCommand) -> Result<serde_json::Value, String> {
        match cmd {
            StudioCommand::NewProject {
                project_name,
                target_engine,
                template,
            } => {
                self.meta.name = project_name.clone();
                self.meta.target_engine = engine_str(target_engine).into();
                self.meta.template = template_str(template).into();
                self.entities.clear();
                self.next_id = 1;
                Ok(json!({ "projectName": project_name }))
            }
            StudioCommand::SaveProject { file_path } => Ok(json!({ "savedTo": file_path })),
            StudioCommand::LoadProject { file_path } => Ok(json!({ "loadedFrom": file_path })),
            StudioCommand::Create2DShape {
                shape_type,
                name,
                x,
                y,
                width,
                height,
                svg_path_data,
                fill_color,
                stroke_color,
                stroke_width,
            } => {
                let data = json!({
                    "shapeType": shape_str(shape_type),
                    "x": x,
                    "y": y,
                    "width": width,
                    "height": height,
                    "svgPathData": svg_path_data,
                    "fillColor": fill_color,
                    "strokeColor": stroke_color,
                    "strokeWidth": stroke_width,
                });
                let id = self.insert_record(EntityKind::Shape2D, name, data);
                Ok(json!({ "entityId": id, "kind": "shape2d" }))
            }
            StudioCommand::Boolean2DPaths {
                shape_a,
                shape_b,
                operation,
                result_name,
            } => {
                if self.find_by_name(shape_a).is_none() {
                    return Err(format!("Shape '{}' not found", shape_a));
                }
                if self.find_by_name(shape_b).is_none() {
                    return Err(format!("Shape '{}' not found", shape_b));
                }
                let data = json!({
                    "operation": boolop_str(operation),
                    "operands": [shape_a, shape_b],
                });
                let id = self.insert_record(EntityKind::Shape2D, result_name, data);
                Ok(json!({ "entityId": id, "kind": "shape2d" }))
            }
            StudioCommand::AutoRig2D {
                target_layer_group,
                rig_topology,
                ..
            } => {
                let bones = rig2d_bones(rig_topology);
                let data = json!({
                    "topology": rig_topology.as_str(),
                    "bones": bones,
                    "source": target_layer_group,
                });
                let id = self.insert_record(EntityKind::Rig, target_layer_group, data);
                Ok(json!({ "rigId": id, "boneCount": bones.len() }))
            }
            StudioCommand::CreateNodeModel {
                model_name,
                nodes,
                symmetry,
                auto_rig,
                subdivision,
            } => {
                let data = json!({
                    "nodes": serde_json::to_value(nodes).unwrap_or_default(),
                    "symmetry": symmetry.unwrap_or(true),
                    "autoRig": auto_rig.unwrap_or(true),
                    "subdivision": subdivision.unwrap_or(1),
                });
                let id = self.insert_record(EntityKind::NodeModel, model_name, data);
                let bone_count = if *auto_rig == Some(false) {
                    0
                } else {
                    nodes.len()
                };
                if bone_count > 0 {
                    self.insert_record(
                        EntityKind::Rig,
                        &format!("{}_rig", model_name),
                        json!({ "bones": nodes.iter().map(|n| n.name.clone()).collect::<Vec<_>>() }),
                    );
                }
                Ok(json!({ "entityId": id, "nodeCount": nodes.len(), "boneCount": bone_count }))
            }
            StudioCommand::ConvertNodeModelToMesh {
                model_name,
                target_mesh_name,
                preserve_rig,
            } => {
                let model = self
                    .find_by_name(model_name)
                    .filter(|r| r.kind == EntityKind::NodeModel)
                    .ok_or_else(|| format!("Node model '{}' not found", model_name))?;
                let node_count = model.data["nodes"].as_array().map(|a| a.len()).unwrap_or(0);
                let vertex_count = node_count * 32;
                let face_count = node_count * 30;
                let data = json!({
                    "source": model_name,
                    "vertexCount": vertex_count,
                    "faceCount": face_count,
                    "preserveRig": preserve_rig.unwrap_or(true),
                });
                let id = self.insert_record(EntityKind::Mesh3D, target_mesh_name, data);
                Ok(json!({ "entityId": id, "vertexCount": vertex_count, "faceCount": face_count }))
            }
            StudioCommand::Create3DPrimitive {
                primitive_type,
                name,
                dimensions,
                subdivisions,
            } => {
                let (vertex_count, face_count) = primitive_counts(primitive_type);
                let data = json!({
                    "primitiveType": primitive_str(primitive_type),
                    "dimensions": {
                        "x": dimensions.clone().unwrap_or_default().x,
                        "y": dimensions.clone().unwrap_or_default().y,
                        "z": dimensions.clone().unwrap_or_default().z,
                    },
                    "subdivisions": subdivisions.unwrap_or(1),
                    "vertexCount": vertex_count,
                    "faceCount": face_count,
                });
                let id = self.insert_record(EntityKind::Mesh3D, name, data);
                Ok(json!({ "entityId": id, "vertexCount": vertex_count, "faceCount": face_count }))
            }
            StudioCommand::Extrude3DFace {
                mesh_name,
                face_indices,
                distance,
                ..
            } => {
                let record = self
                    .find_by_name_or_id_mut(mesh_name)
                    .filter(|r| r.kind == EntityKind::Mesh3D)
                    .ok_or_else(|| format!("Mesh '{}' not found", mesh_name))?;
                record.op_log.push(format!(
                    "extrude_3d_face(distance={}, faces={})",
                    distance,
                    face_indices.len()
                ));
                let vertex_count = record.data["vertexCount"].as_u64().unwrap_or(0) + (face_indices.len() * 4) as u64;
                let face_count = record.data["faceCount"].as_u64().unwrap_or(0) + face_indices.len() as u64;
                record.data["vertexCount"] = serde_json::Value::from(vertex_count);
                record.data["faceCount"] = serde_json::Value::from(face_count);
                Ok(json!({ "entityId": record.id.clone(), "vertexCount": vertex_count, "faceCount": face_count }))
            }
            StudioCommand::Inset3DFace {
                mesh_name,
                face_indices,
                depth,
                ..
            } => {
                let record = self
                    .find_by_name_or_id_mut(mesh_name)
                    .filter(|r| r.kind == EntityKind::Mesh3D)
                    .ok_or_else(|| format!("Mesh '{}' not found", mesh_name))?;
                record.op_log.push(format!("inset_3d_face(depth={}, faces={})", depth, face_indices.len()));
                let vertex_count = record.data["vertexCount"].as_u64().unwrap_or(0);
                let face_count = record.data["faceCount"].as_u64().unwrap_or(0) + face_indices.len() as u64;
                record.data["faceCount"] = serde_json::Value::from(face_count);
                Ok(json!({ "entityId": record.id.clone(), "vertexCount": vertex_count, "faceCount": face_count }))
            }
            StudioCommand::Bevel3DEdge {
                mesh_name,
                edge_indices,
                width,
                ..
            } => {
                let record = self
                    .find_by_name_or_id_mut(mesh_name)
                    .filter(|r| r.kind == EntityKind::Mesh3D)
                    .ok_or_else(|| format!("Mesh '{}' not found", mesh_name))?;
                record.op_log.push(format!("bevel_3d_edge(width={}, edges={})", width, edge_indices.len()));
                let vertex_count = record.data["vertexCount"].as_u64().unwrap_or(0) + (edge_indices.len() * 2) as u64;
                let face_count = record.data["faceCount"].as_u64().unwrap_or(0);
                record.data["vertexCount"] = serde_json::Value::from(vertex_count);
                Ok(json!({ "entityId": record.id.clone(), "vertexCount": vertex_count, "faceCount": face_count }))
            }
            StudioCommand::LoopCut3D {
                mesh_name,
                cuts,
                ..
            } => {
                let record = self
                    .find_by_name_or_id_mut(mesh_name)
                    .filter(|r| r.kind == EntityKind::Mesh3D)
                    .ok_or_else(|| format!("Mesh '{}' not found", mesh_name))?;
                let cuts = cuts.unwrap_or(1);
                record.op_log.push(format!("loop_cut_3d(cuts={})", cuts));
                let vertex_count = record.data["vertexCount"].as_u64().unwrap_or(0) + (cuts * 4) as u64;
                let face_count = record.data["faceCount"].as_u64().unwrap_or(0) + (cuts * 2) as u64;
                record.data["vertexCount"] = serde_json::Value::from(vertex_count);
                record.data["faceCount"] = serde_json::Value::from(face_count);
                Ok(json!({ "entityId": record.id.clone(), "vertexCount": vertex_count, "faceCount": face_count }))
            }
            StudioCommand::AutoRig3D {
                mesh_name,
                rig_template,
                ..
            } => {
                self.find_by_name(mesh_name)
                    .filter(|r| r.kind == EntityKind::Mesh3D)
                    .ok_or_else(|| format!("Mesh '{}' not found", mesh_name))?;
                let bone_count = rig3d_bone_count(rig_template);
                let data = json!({
                    "template": rig_template.as_str(),
                    "boneCount": bone_count,
                    "target": mesh_name,
                });
                let id = self.insert_record(EntityKind::Rig, &format!("{}_rig", mesh_name), data);
                Ok(json!({ "rigId": id, "boneCount": bone_count }))
            }
            StudioCommand::Create2DKeyframe {
                layer_name,
                frame,
                ..
            } => {
                if let Some(record) = self.find_by_name_or_id_mut(layer_name) {
                    record.op_log.push(format!("keyframe@{}", frame));
                }
                Ok(json!({ "ok": true, "tracked": layer_name }))
            }
            StudioCommand::Create3DKeyframe {
                bone_name,
                frame,
                ..
            } => {
                if let Some(record) = self.find_by_name_or_id_mut(bone_name) {
                    record.op_log.push(format!("keyframe@{}", frame));
                }
                Ok(json!({ "ok": true, "tracked": bone_name }))
            }
            StudioCommand::GenerateSfxPreset {
                sound_name,
                preset,
                pitch_variation,
            } => {
                let data = json!({
                    "soundName": sound_name,
                    "preset": sfx_str(preset),
                    "pitchVariation": pitch_variation.unwrap_or(0.0),
                    "format": "wav",
                    "sampleRate": 44100,
                });
                let id = self.insert_record(EntityKind::AudioClip, sound_name, data);
                Ok(json!({ "entityId": id, "durationEstimateMs": 350 }))
            }
            StudioCommand::SynthesizeVocal {
                clip_name,
                text,
                voice_model,
                character_effect,
            } => {
                let effect = character_effect
                    .as_ref()
                    .map(effect_str)
                    .unwrap_or("none");
                let data = json!({
                    "clipName": clip_name,
                    "text": text,
                    "voiceModel": voice_model.clone().unwrap_or_else(|| "en_US-lessac-medium".into()),
                    "characterEffect": effect,
                    "source": "tts",
                });
                let id = self.insert_record(EntityKind::AudioClip, clip_name, data);
                Ok(json!({ "entityId": id }))
            }
            StudioCommand::CreateMusicPattern {
                track_name,
                bpm,
                steps,
                channels,
            } => {
                let data = json!({
                    "trackName": track_name,
                    "bpm": bpm.unwrap_or(120),
                    "steps": steps.unwrap_or(16),
                    "channels": serde_json::to_value(channels).unwrap_or_default(),
                });
                let id = self.insert_record(EntityKind::MusicTrack, track_name, data);
                Ok(json!({ "entityId": id, "stepCount": steps.unwrap_or(16) }))
            }
            StudioCommand::AiPromptToAsset {
                domain,
                prompt,
                target_name,
                provider_override,
                model_override,
            } => {
                let kind = domain_kind(domain);
                let data = json!({
                    "prompt": prompt,
                    "provider": provider_override,
                    "model": model_override,
                    "status": "queued",
                });
                let id = self.insert_record(kind, target_name, data);
                Ok(json!({ "entityId": id, "taskId": id }))
            }
            StudioCommand::AssembleGameEntity {
                entity_name,
                dimension,
                visual_source,
                animation_clips,
                audio_bindings,
                collision_shape,
            } => {
                self.find_by_name(visual_source)
                    .ok_or_else(|| format!("Visual source '{}' not found", visual_source))?;
                let data = json!({
                    "dimension": dimension.clone().unwrap_or_else(|| "3D".into()),
                    "visualSource": visual_source,
                    "animationClips": serde_json::to_value(animation_clips).unwrap_or_default(),
                    "audioBindings": serde_json::to_value(audio_bindings).unwrap_or_default(),
                    "collisionShape": serde_json::to_value(collision_shape).unwrap_or_default(),
                });
                let id = self.insert_record(EntityKind::Entity, entity_name, data);
                Ok(json!({ "entityId": id, "clipCount": animation_clips.len(), "audioEventCount": audio_bindings.len() }))
            }
            StudioCommand::BindAudioToFrame {
                clip_name,
                frame,
                audio_clip_name,
            } => {
                if let Some(record) = self.find_by_name_or_id_mut(clip_name) {
                    record.op_log.push(format!("audio@{}={}", frame, audio_clip_name));
                }
                Ok(json!({ "ok": true }))
            }
            StudioCommand::ExportToEngine {
                entity_name,
                target_engine,
                ..
            } => {
                self.find_by_name(entity_name)
                    .ok_or_else(|| format!("Entity '{}' not found", entity_name))?;
                let mut files = vec![format!("{}.glb", entity_name)];
                files.push(match target_engine {
                    TargetEngine::Godot4 => format!("{}.tscn", entity_name),
                    TargetEngine::Unity => format!("{}.prefab", entity_name),
                    TargetEngine::Unreal5 => format!("{}.fbx", entity_name),
                    _ => format!("{}.gltf", entity_name),
                });
                Ok(json!({ "entityName": entity_name, "targetEngine": engine_str(target_engine), "filesGenerated": files, "bytesWritten": 0 }))
            }
            StudioCommand::GenerateEngineCode {
                entity_name,
                language,
                output_file_path,
                ..
            } => {
                let ext = lang_ext(language);
                let code = format!("// Auto-generated by SGameStudio for entity '{}'.\n// Deterministic controller templates land with the Code Generator milestone.\n", entity_name);
                Ok(json!({ "fileName": format!("{}.{}", entity_name, ext), "language": lang_str(language), "code": code, "exportedPath": output_file_path }))
            }
        }
    }

    pub fn get(&self, id: &str) -> Option<&EntityRecord> {
        self.entities.get(id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&EntityRecord> {
        self.entities.values().find(|r| r.name == name)
    }

    fn find_by_name_or_id_mut(&mut self, key: &str) -> Option<&mut EntityRecord> {
        if self.entities.contains_key(key) {
            return self.entities.get_mut(key);
        }
        self.entities.values_mut().find(|r| r.name == *key)
    }

    pub fn snapshot_entities(&self, ids: &[String]) -> Vec<EntitySnapshot> {
        ids
            .iter()
            .map(|id| {
                let record = self.get(id).or_else(|| self.find_by_name(id)).cloned();
                EntitySnapshot {
                    id: id.clone(),
                    record,
                }
            })
            .collect()
    }

    pub fn restore(&mut self, snapshots: Vec<EntitySnapshot>) {
        for snapshot in snapshots {
            if let Some(record) = snapshot.record {
                self.entities.insert(record.id.clone(), record);
            } else {
                // Removing a previously-absent entity: it may have been keyed by
                // a generated id while the snapshot only knows the requested name.
                if self.entities.remove(&snapshot.id).is_none() {
                    self.entities.retain(|_, r| r.name != snapshot.id);
                }
            }
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn count_by_kind(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for record in self.entities.values() {
            *counts.entry(record.kind.as_str().into()).or_default() += 1;
        }
        counts
    }
}

impl EntityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityKind::Shape2D => "shape2d",
            EntityKind::NodeModel => "node_model",
            EntityKind::Mesh3D => "mesh3d",
            EntityKind::AudioClip => "audio_clip",
            EntityKind::MusicTrack => "music_track",
            EntityKind::Entity => "entity",
            EntityKind::Rig => "rig",
        }
    }
}

pub struct MetaDb {
    conn: Connection,
}

impl MetaDb {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS project_meta(key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO project_meta(key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM project_meta WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_3d_primitive() {
        let mut state = SceneState::new();
        let result = state.apply(&StudioCommand::Create3DPrimitive {
            primitive_type: PrimitiveType::Cube,
            name: "Hero".into(),
            dimensions: None,
            subdivisions: None,
        });
        assert!(result.is_ok());
        assert!(state.get("mesh3d_1").is_some());
        assert_eq!(
            result.unwrap()["entityId"].as_str(),
            Some("mesh3d_1")
        );
    }

    #[test]
    fn create_then_extrude_3d_face() {
        let mut state = SceneState::new();
        state.apply(&StudioCommand::Create3DPrimitive {
            primitive_type: PrimitiveType::Cube,
            name: "Hero".into(),
            dimensions: None,
            subdivisions: None,
        }).unwrap();
        let result = state.apply(&StudioCommand::Extrude3DFace {
            mesh_name: "Hero".into(),
            face_indices: vec![0, 1],
            distance: 0.5,
            individual: None,
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result["faceCount"].as_u64(), Some(8));
        assert_eq!(result["vertexCount"].as_u64(), Some(16));
    }

    #[test]
    fn auto_rig_3d_fail() {
        let mut state = SceneState::new();
        let err = state
            .apply(&StudioCommand::AutoRig3D {
                mesh_name: "Ghost".into(),
                rig_template: Rig3DTemplate::HumanoidStandard,
                symmetry: None,
            })
            .unwrap_err();
        assert_eq!(err, "Mesh 'Ghost' not found");
    }

    #[test]
    fn assemble_game_entity() {
        let mut state = SceneState::new();
        state.apply(&StudioCommand::Create3DPrimitive {
            primitive_type: PrimitiveType::Cube,
            name: "Hero".into(),
            dimensions: None,
            subdivisions: None,
        }).unwrap();
        let result = state.apply(&StudioCommand::AssembleGameEntity {
            entity_name: "Hero".into(),
            dimension: None,
            visual_source: "Hero".into(),
            animation_clips: vec![],
            audio_bindings: vec![],
            collision_shape: None,
        });
        assert!(result.is_ok());
        assert!(state.find_by_name("Hero").is_some());
    }

    #[test]
    fn assemble_game_entity_fail() {
        let mut state = SceneState::new();
        let err = state
            .apply(&StudioCommand::AssembleGameEntity {
                entity_name: "Hero".into(),
                dimension: None,
                visual_source: "Ghost".into(),
                animation_clips: vec![],
                audio_bindings: vec![],
                collision_shape: None,
            })
            .unwrap_err();
        assert_eq!(err, "Visual source 'Ghost' not found");
    }

    #[test]
    fn auto_rig_2d() {
        let mut state = SceneState::new();
        let result = state.apply(&StudioCommand::AutoRig2D {
            target_layer_group: "Hero".into(),
            rig_topology: Rig2DTopology::BipedStandard,
            use_ai_heuristic: None,
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["boneCount"].as_u64(), Some(13));
    }

    #[test]
    fn snapshot_restore() {
        let mut state = SceneState::new();
        state.apply(&StudioCommand::Create3DPrimitive {
            primitive_type: PrimitiveType::Cube,
            name: "Hero".into(),
            dimensions: None,
            subdivisions: None,
        }).unwrap();
        let snapshots = state.snapshot_entities(&["mesh3d_1".into()]);
        let mut new_state = SceneState::new();
        new_state.restore(snapshots);
        assert!(new_state.find_by_name("Hero").is_some());
    }

    #[test]
    fn meta_db() {
        let path = std::env::temp_dir().join(format!("sgame_meta_{}.db", Uuid::new_v4()));
        let db = MetaDb::open(&path).unwrap();
        db.set("test_key", "test_value").unwrap();
        let value = db.get("test_key").unwrap().unwrap();
        assert_eq!(value, "test_value");
    }
}

fn engine_str(e: &TargetEngine) -> &'static str {
    match e {
        TargetEngine::Godot4 => "godot4",
        TargetEngine::Unity => "unity",
        TargetEngine::Unreal5 => "unreal5",
        TargetEngine::GenericGltf => "generic_gltf",
        TargetEngine::GltfStandard => "gltf_standard",
    }
}

fn template_str(t: &ProjectTemplate) -> &'static str {
    match t {
        ProjectTemplate::Empty => "empty",
        ProjectTemplate::Platformer2DCharacter => "2d_platformer_character",
        ProjectTemplate::Action3DCharacter => "3d_action_character",
        ProjectTemplate::RetroSfxPack => "retro_sfx_pack",
    }
}

fn shape_str(s: &ShapeType) -> &'static str {
    match s {
        ShapeType::Rectangle => "rectangle",
        ShapeType::Circle => "circle",
        ShapeType::Polygon => "polygon",
        ShapeType::Star => "star",
        ShapeType::CustomPath => "custom_path",
    }
}

fn boolop_str(o: &BooleanOp) -> &'static str {
    match o {
        BooleanOp::Union => "union",
        BooleanOp::Difference => "difference",
        BooleanOp::Intersection => "intersection",
        BooleanOp::Xor => "xor",
    }
}

fn rig2d_bones(t: &Rig2DTopology) -> Vec<&'static str> {
    match t {
        Rig2DTopology::BipedStandard => vec![
            "Hips",
            "Spine",
            "Chest",
            "Neck",
            "Head",
            "Shoulder_L",
            "Arm_L",
            "Forearm_L",
            "Hand_L",
            "Shoulder_R",
            "Arm_R",
            "Forearm_R",
            "Hand_R",
        ],
        Rig2DTopology::Quadruped => vec![ 
            "Hips",
            "Spine",
            "Chest",
            "Neck",
            "Head",
            "FrontLeg_L",
            "FrontKnee_L",
            "FrontPaw_L",
            "FrontLeg_R",
            "FrontKnee_R",
            "FrontPaw_R",
            "BackLeg_L",
            "BackKnee_L",
            "BackPaw_L",
            "BackLeg_R",
            "BackKnee_R",
            "BackPaw_R",
            "Tail",
        ],
        Rig2DTopology::Serpentine => vec![
            "Seg_1",
            "Seg_2",
            "Seg_3",
            "Seg_4",
            "Seg_5",
            "Seg_6",
            "Seg_7",
            "Seg_8",
        ],
        Rig2DTopology::CustomAuto => vec!["Seg_1", "Seg_2", "Seg_3", "Seg_4", "Seg_5", "Seg_6", "Seg_7", "Seg_8"],
    }
}

fn primitive_counts(p: &PrimitiveType) -> (usize, usize) {
    match p {
        PrimitiveType::Cube => (8, 6),
        PrimitiveType::Cylinder => (48, 34),
        PrimitiveType::Sphere => (96, 192),
        PrimitiveType::Cone => (33, 33),
        PrimitiveType::Plane => (4, 1),
        PrimitiveType::Torus => (96, 192),
    }
}

fn primitive_str(p: &PrimitiveType) -> &'static str {
    match p {
        PrimitiveType::Cube => "cube",
        PrimitiveType::Cylinder => "cylinder",
        PrimitiveType::Sphere => "sphere",
        PrimitiveType::Cone => "cone",
        PrimitiveType::Plane => "plane",
        PrimitiveType::Torus => "torus",
    }
}

fn rig3d_bone_count(t: &Rig3DTemplate) -> usize {
    match t {
        Rig3DTemplate::HumanoidStandard => 19,
        Rig3DTemplate::QuadrupedBeast => 24,
        Rig3DTemplate::FlyingCreature => 21,
        Rig3DTemplate::SimpleProp => 1,
    }
}

fn sfx_str(s: &SfxPreset) -> &'static str {
    match s {
        SfxPreset::Laser => "laser",
        SfxPreset::Explosion => "explosion",
        SfxPreset::Powerup => "powerup",
        SfxPreset::Hit => "hit",
        SfxPreset::Jump => "jump",
        SfxPreset::Coin => "coin",
        SfxPreset::Blip => "blip",
        SfxPreset::Footstep => "footstep",
    }
}

fn effect_str(e: &CharacterEffect) -> &'static str {
    match e {
        CharacterEffect::NoneEffect => "none",
        CharacterEffect::Robot => "robot",
        CharacterEffect::MonsterGrowl => "monster_growl",
        CharacterEffect::RadioComm => "radio_comm",
        CharacterEffect::AlienReverb => "alien_reverb",
    }
}

fn lang_ext(l: &CodeLanguage) -> &'static str {
    match l {
        CodeLanguage::Gdscript => "gd",
        CodeLanguage::Csharp => "cs",
        CodeLanguage::Cpp => "cpp",
        CodeLanguage::Rust => "rs",
        CodeLanguage::C => "c",
        CodeLanguage::Typescript => "ts",
        CodeLanguage::Lua => "lua",
    }
}

fn lang_str(l: &CodeLanguage) -> &'static str {
    match l {
        CodeLanguage::Gdscript => "gdscript",
        CodeLanguage::Csharp => "csharp",
        CodeLanguage::Cpp => "cpp",
        CodeLanguage::Rust => "rust",
        CodeLanguage::C => "c",
        CodeLanguage::Typescript => "typescript",
        CodeLanguage::Lua => "lua",
    }
}

fn domain_kind(d: &AiDomain) -> EntityKind {
    match d {
        AiDomain::Vector2D => EntityKind::Shape2D,
        AiDomain::Model3D => EntityKind::NodeModel,
        AiDomain::TexturePbr => EntityKind::Shape2D,
        AiDomain::AudioSfx => EntityKind::AudioClip,
        AiDomain::MusicTrack => EntityKind::MusicTrack,
    }
}