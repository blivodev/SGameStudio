use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShapeType {
    Rectangle,
    Circle,
    Polygon,
    Star,
    CustomPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BooleanOp {
    Union,
    Difference,
    Intersection,
    Xor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rig2DTopology {
    BipedStandard,
    Quadruped,
    Serpentine,
    CustomAuto,
}

impl Rig2DTopology {
    pub fn as_str(&self) -> &'static str {
        match self {
            Rig2DTopology::BipedStandard => "biped_standard",
            Rig2DTopology::Quadruped => "quadruped",
            Rig2DTopology::Serpentine => "serpentine",
            Rig2DTopology::CustomAuto => "custom_auto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveType {
    Cube,
    Cylinder,
    Sphere,
    Cone,
    Plane,
    Torus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rig3DTemplate {
    HumanoidStandard,
    QuadrupedBeast,
    FlyingCreature,
    SimpleProp,
}

impl Rig3DTemplate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Rig3DTemplate::HumanoidStandard => "humanoid_standard",
            Rig3DTemplate::QuadrupedBeast => "quadruped_beast",
            Rig3DTemplate::FlyingCreature => "flying_creature",
            Rig3DTemplate::SimpleProp => "simple_prop",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SfxPreset {
    Laser,
    Explosion,
    Powerup,
    Hit,
    Jump,
    Coin,
    Blip,
    Footstep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterEffect {
    #[serde(rename="none")]
    NoneEffect,
    Robot,
    #[serde(rename="monster_growl")]
    MonsterGrowl,
    #[serde(rename="radio_comm")]
    RadioComm,
    #[serde(rename="alien_reverb")]
    AlienReverb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackerInstrument {
    #[serde(rename="synth_lead")]
    SynthLead,
    Bass,
    #[serde(rename="chiptune_square")]
    ChiptuneSquare,
    Drums,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiDomain {
    #[serde(rename="2d_vector")]
    Vector2D,
    #[serde(rename="3d_model")]
    Model3D,
    #[serde(rename="texture_pbr")]
    TexturePbr,
    AudioSfx,
    MusicTrack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetEngine {
    Godot4,
    Unity,
    Unreal5,
    #[serde(rename="generic_gltf")]
    GenericGltf,
    #[serde(rename="gltf_std")]
    GltfStandard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeLanguage {
    Gdscript,
    Csharp,
    Cpp,
    Rust,
    C,
    Typescript,
    Lua,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameplayStyle {
    #[serde(rename="platformer_2d")]
    Platformer2D,
    #[serde(rename="top_down_2d")]
    TopDown2D,
    #[serde(rename="third_person_3d")]
    ThirdPerson3D,
    #[serde(rename="first_person_3d")]
    FirstPerson3D,
    #[serde(rename="stationary_npc")]
    StationaryNpc,
    #[serde(rename="custom_prompt")]
    CustomPrompt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionKind {
    Box,
    Capsule,
    Sphere,
    #[serde(rename="convex_hull")]
    ConvexHull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectTemplate {
    Empty,
    #[serde(rename="2d_platformer_character")]
    Platformer2DCharacter,
    #[serde(rename="3d_action_character")]
    Action3DCharacter,
    RetroSfxPack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Issuer {
    Gui,
    Rest,
    WebSocket,
    Mcp,
    Cli,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dimensions {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0, z: 1.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeSpec {
    pub id: String,
    pub name: String,
    pub radius: f64,
    pub position: [f64; 3],
    pub parent_id: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackerChannel {
    pub channel_index: u32,
    pub instrument: TrackerInstrument,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioBinding {
    pub clip_name: String,
    pub frame: u32,
    pub audio_clip_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollisionShapeSpec {
    #[serde(rename="type")]
    pub kind: CollisionKind,
    pub radius: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command", content = "payload")]
pub enum StudioCommand {
    #[serde(rename = "new_project", rename_all = "camelCase")]
    NewProject { project_name: String, target_engine: TargetEngine, template: ProjectTemplate },
    #[serde(rename = "save_project", rename_all = "camelCase")]
    SaveProject { file_path: String },
    #[serde(rename = "load_project", rename_all = "camelCase")]
    LoadProject { file_path: String },
    #[serde(rename = "create_2d_shape", rename_all = "camelCase")]
    Create2DShape { shape_type: ShapeType, name: String, x: f64, y: f64, width: Option<f64>, height: Option<f64>, svg_path_data: Option<String>, fill_color: Option<String>, stroke_color: Option<String>, stroke_width: Option<f64> },
    #[serde(rename = "boolean_2d_paths", rename_all = "camelCase")]
    Boolean2DPaths { shape_a: String, shape_b: String, operation: BooleanOp, result_name: String },
    #[serde(rename = "auto_rig_2d", rename_all = "camelCase")]
    AutoRig2D { target_layer_group: String, rig_topology: Rig2DTopology, use_ai_heuristic: Option<bool> },
    #[serde(rename = "create_node_model", rename_all = "camelCase")]
    CreateNodeModel { model_name: String, nodes: Vec<NodeSpec>, symmetry: Option<bool>, auto_rig: Option<bool>, subdivision: Option<u8> },
    #[serde(rename = "convert_node_model_to_mesh", rename_all = "camelCase")]
    ConvertNodeModelToMesh { model_name: String, target_mesh_name: String, preserve_rig: Option<bool> },
    #[serde(rename = "create_3d_primitive", rename_all = "camelCase")]
    Create3DPrimitive { primitive_type: PrimitiveType, name: String, dimensions: Option<Dimensions>, subdivisions: Option<u32> },
    #[serde(rename = "extrude_3d_face", rename_all = "camelCase")]
    Extrude3DFace { mesh_name: String, face_indices: Vec<u32>, distance: f64, individual: Option<bool> },
    #[serde(rename = "inset_3d_face", rename_all = "camelCase")]
    Inset3DFace { mesh_name: String, face_indices: Vec<u32>, depth: f64 },
    #[serde(rename = "bevel_3d_edge", rename_all = "camelCase")]
    Bevel3DEdge { mesh_name: String, edge_indices: Vec<u32>, width: f64 },
    #[serde(rename = "loop_cut_3d", rename_all = "camelCase")]
    LoopCut3D { mesh_name: String, edge_indices: Vec<u32>, cuts: Option<u32> },
    #[serde(rename = "auto_rig_3d", rename_all = "camelCase")]
    AutoRig3D { mesh_name: String, rig_template: Rig3DTemplate, symmetry: Option<bool> },
    #[serde(rename = "create_2d_keyframe", rename_all = "camelCase")]
    Create2DKeyframe { layer_name: String, frame: u32, properties: serde_json::Value },
    #[serde(rename = "create_3d_keyframe", rename_all = "camelCase")]
    Create3DKeyframe { bone_name: String, frame: u32, rotation: [f64; 3], position: [f64; 3] },
    #[serde(rename = "generate_sfx_preset", rename_all = "camelCase")]
    GenerateSfxPreset { sound_name: String, preset: SfxPreset, pitch_variation: Option<f64> },
    #[serde(rename = "synthesize_vocal", rename_all = "camelCase")]
    SynthesizeVocal { clip_name: String, text: String, voice_model: Option<String>, character_effect: Option<CharacterEffect> },
    #[serde(rename = "create_music_pattern", rename_all = "camelCase")]
    CreateMusicPattern { track_name: String, bpm: Option<u32>, steps: Option<u32>, channels: Vec<TrackerChannel> },
    #[serde(rename = "ai_prompt_to_asset", rename_all = "camelCase")]
    AiPromptToAsset { domain: AiDomain, prompt: String, target_name: String, provider_override: Option<String>, model_override: Option<String> },
    #[serde(rename = "assemble_game_entity", rename_all = "camelCase")]
    AssembleGameEntity { entity_name: String, dimension: Option<String>, visual_source: String, animation_clips: Vec<String>, audio_bindings: Vec<AudioBinding>, collision_shape: Option<CollisionShapeSpec> },
    #[serde(rename = "bind_audio_to_frame", rename_all = "camelCase")]
    BindAudioToFrame { clip_name: String, frame: u32, audio_clip_name: String },
    #[serde(rename = "export_to_engine", rename_all = "camelCase")]
    ExportToEngine { entity_name: String, target_engine: TargetEngine, output_directory: String, embed_audio: Option<bool>, optimize_textures: Option<bool> },
    #[serde(rename = "generate_engine_code", rename_all = "camelCase")]
    GenerateEngineCode { entity_name: String, target_engine: TargetEngine, language: CodeLanguage, gameplay_style: Option<GameplayStyle>, custom_prompt: Option<String>, output_file_path: Option<String> },
}

impl StudioCommand {
    pub fn command_name(&self) -> &'static str {
        match self {
            StudioCommand::NewProject { .. } => "sgame_new_project",
            StudioCommand::SaveProject { .. } => "sgame_save_project",
            StudioCommand::LoadProject { .. } => "sgame_load_project",
            StudioCommand::Create2DShape { .. } => "sgame_create_2d_shape",
            StudioCommand::Boolean2DPaths { .. } => "sgame_boolean_2d_paths",
            StudioCommand::AutoRig2D { .. } => "sgame_auto_rig_2d",
            StudioCommand::CreateNodeModel { .. } => "sgame_create_node_model",
            StudioCommand::ConvertNodeModelToMesh { .. } => "sgame_convert_node_model_to_mesh",
            StudioCommand::Create3DPrimitive { .. } => "sgame_create_3d_primitive",
            StudioCommand::Extrude3DFace { .. } => "sgame_extrude_3d_face",
            StudioCommand::Inset3DFace { .. } => "sgame_inset_3d_face",
            StudioCommand::Bevel3DEdge { .. } => "sgame_bevel_3d_edge",
            StudioCommand::LoopCut3D { .. } => "sgame_loop_cut_3d",
            StudioCommand::AutoRig3D { .. } => "sgame_auto_rig_3d",
            StudioCommand::Create2DKeyframe { .. } => "sgame_create_2d_keyframe",
            StudioCommand::Create3DKeyframe { .. } => "sgame_create_3d_keyframe",
            StudioCommand::GenerateSfxPreset { .. } => "sgame_generate_sfx_preset",
            StudioCommand::SynthesizeVocal { .. } => "sgame_synthesize_vocal",
            StudioCommand::CreateMusicPattern { .. } => "sgame_create_music_pattern",
            StudioCommand::AiPromptToAsset { .. } => "sgame_ai_prompt_to_asset",
            StudioCommand::AssembleGameEntity { .. } => "sgame_assemble_game_entity",
            StudioCommand::BindAudioToFrame { .. } => "sgame_bind_audio_to_frame",
            StudioCommand::ExportToEngine { .. } => "sgame_export_to_engine",
            StudioCommand::GenerateEngineCode { .. } => "sgame_generate_engine_code",
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        fn ensure_nonempty(field: &str, value: &str) -> Result<(), String> {
            if value.trim().is_empty() {
                Err(format!("{} cannot be empty or whitespace", field))
            } else {
                Ok(())
            }
        }

        match self {
            StudioCommand::NewProject { project_name, .. } => ensure_nonempty("project_name", project_name),
            StudioCommand::SaveProject { file_path } => ensure_nonempty("file_path", file_path),
            StudioCommand::LoadProject { file_path } => ensure_nonempty("file_path", file_path),
            StudioCommand::Create2DShape { name, .. } => ensure_nonempty("name", name),
            StudioCommand::Boolean2DPaths { shape_a, shape_b, result_name, .. } => {
                ensure_nonempty("shape_a", shape_a)?;
                ensure_nonempty("shape_b", shape_b)?;
                ensure_nonempty("result_name", result_name)
            }
            StudioCommand::AutoRig2D { target_layer_group, .. } => ensure_nonempty("target_layer_group", target_layer_group),
            StudioCommand::CreateNodeModel { model_name, nodes, .. } => {
                ensure_nonempty("model_name", model_name)?;
                if nodes.is_empty() {
                    return Err("nodes cannot be empty".into());
                }
                Ok(())
            }
            StudioCommand::ConvertNodeModelToMesh { model_name, target_mesh_name, .. } => {
                ensure_nonempty("model_name", model_name)?;
                ensure_nonempty("target_mesh_name", target_mesh_name)
            }
            StudioCommand::Create3DPrimitive { name, .. } => ensure_nonempty("name", name),
            StudioCommand::Extrude3DFace { distance, face_indices, .. } => {
                if *distance < 0.0 {
                    return Err("distance cannot be negative".into());
                }
                if face_indices.is_empty() {
                    return Err("face_indices cannot be empty".into());
                }
                Ok(())
            }
            StudioCommand::Inset3DFace { depth, face_indices, .. } => {
                if *depth < 0.0 {
                    return Err("depth cannot be negative".into());
                }
                if face_indices.is_empty() {
                    return Err("face_indices cannot be empty".into());
                }
                Ok(())
            }
            StudioCommand::Bevel3DEdge { width, edge_indices, .. } => {
                if *width < 0.0 {
                    return Err("width cannot be negative".into());
                }
                if edge_indices.is_empty() {
                    return Err("edge_indices cannot be empty".into());
                }
                Ok(())
            }
            StudioCommand::LoopCut3D { edge_indices, .. } => {
                if edge_indices.is_empty() {
                    return Err("edge_indices cannot be empty".into());
                }
                Ok(())
            }
            StudioCommand::AutoRig3D { mesh_name, .. } => ensure_nonempty("mesh_name", mesh_name),
            StudioCommand::Create2DKeyframe { layer_name, .. } => ensure_nonempty("layer_name", layer_name),
            StudioCommand::Create3DKeyframe { bone_name, .. } => ensure_nonempty("bone_name", bone_name),
            StudioCommand::GenerateSfxPreset { sound_name, pitch_variation, .. } => {
                ensure_nonempty("sound_name", sound_name)?;
                if let Some(pitch) = pitch_variation {
                    if *pitch < -1.0 || *pitch > 1.0 {
                        return Err("pitch_variation outside [-1.0,1.0]".into());
                    }
                }
                Ok(())
            }
            StudioCommand::SynthesizeVocal { clip_name, text, .. } => {
                ensure_nonempty("clip_name", clip_name)?;
                ensure_nonempty("text", text)
            }
            StudioCommand::CreateMusicPattern { track_name, channels, .. } => {
                ensure_nonempty("track_name", track_name)?;
                if channels.is_empty() {
                    return Err("channels cannot be empty".into());
                }
                Ok(())
            }
            StudioCommand::AiPromptToAsset { prompt, target_name, .. } => {
                ensure_nonempty("prompt", prompt)?;
                ensure_nonempty("target_name", target_name)
            }
            StudioCommand::AssembleGameEntity { entity_name, visual_source, .. } => {
                ensure_nonempty("entity_name", entity_name)?;
                ensure_nonempty("visual_source", visual_source)
            }
            StudioCommand::BindAudioToFrame { clip_name, audio_clip_name, .. } => {
                ensure_nonempty("clip_name", clip_name)?;
                ensure_nonempty("audio_clip_name", audio_clip_name)
            }
            StudioCommand::ExportToEngine { entity_name, output_directory, .. } => {
                ensure_nonempty("entity_name", entity_name)?;
                ensure_nonempty("output_directory", output_directory)
            }
            StudioCommand::GenerateEngineCode { entity_name, .. } => ensure_nonempty("entity_name", entity_name),
        }
    }

    pub fn affected_entity_ids(&self) -> Vec<String> {
        match self {
            StudioCommand::NewProject { .. } | StudioCommand::SaveProject { .. } | StudioCommand::LoadProject { .. } => vec![],
            StudioCommand::Create2DShape { name, .. } => vec![name.clone()],
            StudioCommand::Boolean2DPaths { result_name, .. } => vec![result_name.clone()],
            StudioCommand::AutoRig2D { target_layer_group, .. } => vec![target_layer_group.clone()],
            StudioCommand::CreateNodeModel { model_name, .. } => vec![model_name.clone()],
            StudioCommand::ConvertNodeModelToMesh { target_mesh_name, .. } => vec![target_mesh_name.clone()],
            StudioCommand::Create3DPrimitive { name, .. } => vec![name.clone()],
            StudioCommand::Extrude3DFace { mesh_name, .. }
            | StudioCommand::Inset3DFace { mesh_name, .. }
            | StudioCommand::Bevel3DEdge { mesh_name, .. }
            | StudioCommand::LoopCut3D { mesh_name, .. }
            | StudioCommand::AutoRig3D { mesh_name, .. } => vec![mesh_name.clone()],
            StudioCommand::Create2DKeyframe { layer_name, .. } => vec![layer_name.clone()],
            StudioCommand::Create3DKeyframe { bone_name, .. } => vec![bone_name.clone()],
            StudioCommand::GenerateSfxPreset { sound_name, .. } => vec![sound_name.clone()],
            StudioCommand::SynthesizeVocal { clip_name, .. } => vec![clip_name.clone()],
            StudioCommand::CreateMusicPattern { track_name, .. } => vec![track_name.clone()],
            StudioCommand::AiPromptToAsset { target_name, .. } => vec![target_name.clone()],
            StudioCommand::AssembleGameEntity { entity_name, .. } 
            | StudioCommand::BindAudioToFrame { clip_name: entity_name, .. }
            | StudioCommand::ExportToEngine { entity_name, .. }
            | StudioCommand::GenerateEngineCode { entity_name, .. } => vec![entity_name.clone()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandEnvelope {
    pub command_id: String,
    pub issuer: Issuer,
    #[serde(flatten)]
    pub command: StudioCommand,
}

impl CommandEnvelope {
    pub fn new(issuer: Issuer, command: StudioCommand) -> Self {
        Self {
            command_id: uuid::Uuid::new_v4().to_string(),
            issuer,
            command,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub status: String,
    pub command_id: String,
    pub execution_time_ms: f64,
    pub result: serde_json::Value,
}

impl CommandResult {
    pub fn success(command_id: impl Into<String>, result: serde_json::Value, execution_time_ms: f64) -> Self {
        Self {
            status: "success".into(),
            command_id: command_id.into(),
            execution_time_ms,
            result,
        }
    }
    pub fn failure(command_id: impl Into<String>, error: impl Into<String>, execution_time_ms: f64) -> Self {
        Self {
            status: "failure".into(),
            command_id: command_id.into(),
            execution_time_ms,
            result: serde_json::json!({ "error": error.into() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    #[test]
    fn test_parse_rest_spec_example() {
        let json = r#"{"command":"create_3d_primitive","payload":{"primitiveType":"cube","name":"Knight_Shield","dimensions":{"x":0.8,"y":1.2,"z":0.1}}}"#;
        let cmd: StudioCommand = from_str(json).unwrap();
        assert!(matches!(cmd, StudioCommand::Create3DPrimitive { .. }));
        assert_eq!(cmd.command_name(), "sgame_create_3d_primitive");
    }

    #[test]
    fn test_serialize_roundtrip() {
        let cmd = StudioCommand::GenerateSfxPreset {
            sound_name: "laser_1".to_string(),
            preset: SfxPreset::Laser,
            pitch_variation: Some(0.2),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("generate_sfx_preset"));
        assert!(json.contains("soundName"));
        let cmd2: StudioCommand = from_str(&json).unwrap();
        assert_eq!(cmd, cmd2);
    }

    #[test]
    fn test_validate() {
        let cmd = StudioCommand::Create3DPrimitive {
            primitive_type: PrimitiveType::Cube,
            name: "".to_string(),
            dimensions: None,
            subdivisions: None,
        };
        assert!(cmd.validate().is_err());

        let cmd = StudioCommand::GenerateSfxPreset {
            sound_name: "laser_1".to_string(),
            preset: SfxPreset::Laser,
            pitch_variation: Some(2.0),
        };
        assert!(cmd.validate().is_err());

        let cmd = StudioCommand::Create3DPrimitive {
            primitive_type: PrimitiveType::Cube,
            name: "X".to_string(),
            dimensions: None,
            subdivisions: None,
        };
        assert!(cmd.validate().is_ok());
    }
}
