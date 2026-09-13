use serde_json::{json, Value};
use crate::core::{CommandBus, CommandEnvelope, Issuer, StudioCommand};

pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

/// Register all MCP tools, one per `StudioCommand`, matching
/// `docs/MCP_AND_API_SPECIFICATION.md` (24 command tools + scene summary = 25).
pub fn tool_definitions() -> Vec<ToolDef> {
    let str_t = |desc: &'static str| json!({ "type": "string", "description": desc });

    vec![
        ToolDef {
            name: "sgame_new_project",
            description: "Create a new SGameStudio project.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "projectName": str_t("Name of the project."),
                    "targetEngine": { "type": "string", "enum": ["godot4", "unity", "unreal5", "generic_gltf"], "default": "godot4" },
                    "template": { "type": "string", "enum": ["empty", "2d_platformer_character", "3d_action_character", "retro_sfx_pack"], "default": "empty" }
                },
                "required": ["projectName"]
            }),
        },
        ToolDef {
            name: "sgame_save_project",
            description: "Save the current project to an .sgsproj bundle.",
            input_schema: json!({
                "type": "object",
                "properties": { "filePath": str_t("Absolute destination path (.sgsproj).") },
                "required": ["filePath"]
            }),
        },
        ToolDef {
            name: "sgame_load_project",
            description: "Load a project from an .sgsproj bundle.",
            input_schema: json!({
                "type": "object",
                "properties": { "filePath": str_t("Absolute source path (.sgsproj).") },
                "required": ["filePath"]
            }),
        },
        ToolDef {
            name: "sgame_get_scene_summary",
            description: "Get a JSON summary of the current scene state.",
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        ToolDef {
            name: "sgame_create_2d_shape",
            description: "Create a 2D vector shape or custom Bezier path.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "shapeType": { "type": "string", "enum": ["rectangle", "circle", "polygon", "star", "custom_path"] },
                    "name": str_t("Layer/shape identifier."),
                    "x": { "type": "number", "description": "Center X." },
                    "y": { "type": "number", "description": "Center Y." },
                    "width": { "type": "number" },
                    "height": { "type": "number" },
                    "svgPathData": str_t("SVG d-attribute for custom_path."),
                    "fillColor": str_t("Hex or rgba color."),
                    "strokeColor": str_t("Stroke color."),
                    "strokeWidth": { "type": "number", "default": 2.0 }
                },
                "required": ["shapeType", "name", "x", "y"]
            }),
        },
        ToolDef {
            name: "sgame_boolean_2d_paths",
            description: "Execute a 2D Boolean path operation on two shapes.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "shapeA": str_t("Primary shape."),
                    "shapeB": str_t("Operand shape."),
                    "operation": { "type": "string", "enum": ["union", "difference", "intersection", "xor"] },
                    "resultName": str_t("Name for the result.")
                },
                "required": ["shapeA", "shapeB", "operation", "resultName"]
            }),
        },
        ToolDef {
            name: "sgame_auto_rig_2d",
            description: "Auto-detect joints and generate a 2D skeletal rig with heat diffusion skinning.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "targetLayerGroup": str_t("Layer group containing the character."),
                    "rigTopology": { "type": "string", "enum": ["biped_standard", "quadruped", "serpentine", "custom_auto"], "default": "biped_standard" },
                    "useAIHeuristic": { "type": "boolean", "default": true }
                },
                "required": ["targetLayerGroup"]
            }),
        },
        ToolDef {
            name: "sgame_create_node_model",
            description: "Generate a watertight 3D mesh and inherent skeletal rig from a Dust3D-style node tree.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "modelName": str_t("Unique name."),
                    "nodes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "name": { "type": "string" },
                                "radius": { "type": "number" },
                                "position": { "type": "array", "items": { "type": "number" }, "minItems": 3, "maxItems": 3 },
                                "parentId": { "type": "string" },
                                "color": { "type": "string" }
                            },
                            "required": ["id", "radius", "position"]
                        }
                    },
                    "symmetry": { "type": "boolean", "default": true },
                    "autoRig": { "type": "boolean", "default": true },
                    "subdivision": { "type": "integer", "default": 1, "minimum": 0, "maximum": 3 }
                },
                "required": ["modelName", "nodes"]
            }),
        },
        ToolDef {
            name: "sgame_convert_node_model_to_mesh",
            description: "Bake a node model into a polygon mesh for face/edge editing.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "modelName": str_t("Source node model."),
                    "targetMeshName": str_t("Resulting polygon mesh name."),
                    "preserveRig": { "type": "boolean", "default": true }
                },
                "required": ["modelName", "targetMeshName"]
            }),
        },
        ToolDef {
            name: "sgame_create_3d_primitive",
            description: "Create a basic 3D mesh primitive (Wings3D/SketchUp style).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "primitiveType": { "type": "string", "enum": ["cube", "cylinder", "sphere", "cone", "plane", "torus"] },
                    "name": str_t("Unique mesh name."),
                    "dimensions": {
                        "type": "object",
                        "properties": {
                            "x": { "type": "number", "default": 1.0 },
                            "y": { "type": "number", "default": 1.0 },
                            "z": { "type": "number", "default": 1.0 }
                        }
                    },
                    "subdivisions": { "type": "integer", "default": 1 }
                },
                "required": ["primitiveType", "name"]
            }),
        },
        ToolDef {
            name: "sgame_extrude_3d_face",
            description: "Extrude selected faces along their normal or a custom direction.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "meshName": str_t("Target mesh."),
                    "faceIndices": { "type": "array", "items": { "type": "integer" } },
                    "distance": { "type": "number", "description": "Extrusion distance." },
                    "individual": { "type": "boolean", "default": false }
                },
                "required": ["meshName", "faceIndices", "distance"]
            }),
        },
        ToolDef {
            name: "sgame_inset_3d_face",
            description: "Inset selected faces with depth control.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "meshName": str_t("Target mesh."),
                    "faceIndices": { "type": "array", "items": { "type": "integer" } },
                    "depth": { "type": "number", "description": "Inset depth." }
                },
                "required": ["meshName", "faceIndices", "depth"]
            }),
        },
        ToolDef {
            name: "sgame_bevel_3d_edge",
            description: "Bevel selected edges with a given width.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "meshName": str_t("Target mesh."),
                    "edgeIndices": { "type": "array", "items": { "type": "integer" } },
                    "width": { "type": "number", "description": "Bevel width." }
                },
                "required": ["meshName", "edgeIndices", "width"]
            }),
        },
        ToolDef {
            name: "sgame_loop_cut_3d",
            description: "Insert a loop cut across the selected edge ring.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "meshName": str_t("Target mesh."),
                    "edgeIndices": { "type": "array", "items": { "type": "integer" } },
                    "cuts": { "type": "integer", "description": "Number of cuts." }
                },
                "required": ["meshName", "edgeIndices"]
            }),
        },
        ToolDef {
            name: "sgame_auto_rig_3d",
            description: "Auto-fit a 3D armature into a mesh and compute smooth skinning weights.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "meshName": str_t("Target mesh."),
                    "rigTemplate": { "type": "string", "enum": ["humanoid_standard", "quadruped_beast", "flying_creature", "simple_prop"], "default": "humanoid_standard" },
                    "symmetry": { "type": "boolean", "default": true }
                },
                "required": ["meshName"]
            }),
        },
        ToolDef {
            name: "sgame_create_2d_keyframe",
            description: "Add or modify a 2D animation keyframe on a layer.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "layerName": str_t("Layer/shape name."),
                    "frame": { "type": "integer" },
                    "properties": { "type": "object", "description": "Keyed properties." }
                },
                "required": ["layerName", "frame"]
            }),
        },
        ToolDef {
            name: "sgame_create_3d_keyframe",
            description: "Add or modify a 3D skeletal keyframe on a bone.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "boneName": str_t("Bone name."),
                    "frame": { "type": "integer" },
                    "rotation": { "type": "array", "items": { "type": "number" }, "minItems": 3, "maxItems": 3 },
                    "position": { "type": "array", "items": { "type": "number" }, "minItems": 3, "maxItems": 3 }
                },
                "required": ["boneName", "frame"]
            }),
        },
        ToolDef {
            name: "sgame_generate_sfx_preset",
            description: "Generate an instant procedural game sound effect using the sfxr DSP engine.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "soundName": str_t("Identifier for the sound."),
                    "preset": { "type": "string", "enum": ["laser", "explosion", "powerup", "hit", "jump", "coin", "blip", "footstep"] },
                    "pitchVariation": { "type": "number", "default": 0.0 }
                },
                "required": ["soundName", "preset"]
            }),
        },
        ToolDef {
            name: "sgame_synthesize_vocal",
            description: "Generate a character vocal line from text using neural TTS and DSP voice effects.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipName": str_t("Clip identifier."),
                    "text": str_t("Speech text."),
                    "voiceModel": { "type": "string", "default": "en_US-lessac-medium" },
                    "characterEffect": { "type": "string", "enum": ["none", "robot", "monster_growl", "radio_comm", "alien_reverb"], "default": "none" }
                },
                "required": ["clipName", "text"]
            }),
        },
        ToolDef {
            name: "sgame_create_music_pattern",
            description: "Compose a pattern in the audio tracker with notes, instruments, and tempo.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trackName": { "type": "string" },
                    "bpm": { "type": "integer", "default": 120 },
                    "steps": { "type": "integer", "default": 16 },
                    "channels": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "channelIndex": { "type": "integer" },
                                "instrument": { "type": "string", "enum": ["synth_lead", "bass", "chiptune_square", "drums"] },
                                "notes": { "type": "array", "items": { "type": "string" } }
                            }
                        }
                    }
                },
                "required": ["trackName", "channels"]
            }),
        },
        ToolDef {
            name: "sgame_ai_prompt_to_asset",
            description: "Generate an asset via the configured AI provider pipeline.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "enum": ["2d_vector", "3d_model", "texture_pbr", "audio_sfx", "music_track"] },
                    "prompt": str_t("Generation prompt."),
                    "targetName": str_t("Asset name."),
                    "providerOverride": { "type": "string" },
                    "modelOverride": { "type": "string" }
                },
                "required": ["domain", "prompt", "targetName"]
            }),
        },
        ToolDef {
            name: "sgame_assemble_game_entity",
            description: "Assemble components into a unified Game Asset Blueprint.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entityName": str_t("Name of the game entity."),
                    "dimension": { "type": "string", "enum": ["2D", "3D"], "default": "3D" },
                    "visualSource": str_t("Mesh name or 2D layer group name."),
                    "animationClips": { "type": "array", "items": { "type": "string" } },
                    "audioBindings": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "clipName": { "type": "string" },
                                "frame": { "type": "integer" },
                                "audioClipName": { "type": "string" }
                            },
                            "required": ["clipName", "frame", "audioClipName"]
                        }
                    },
                    "collisionShape": {
                        "type": "object",
                        "properties": {
                            "type": { "type": "string", "enum": ["box", "capsule", "sphere", "convex_hull"] },
                            "radius": { "type": "number", "default": 0.5 },
                            "height": { "type": "number", "default": 1.8 }
                        },
                        "required": ["type"]
                    }
                },
                "required": ["entityName", "visualSource"]
            }),
        },
        ToolDef {
            name: "sgame_bind_audio_to_frame",
            description: "Bind an audio event marker to a specific animation frame.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipName": str_t("Animation clip name."),
                    "frame": { "type": "integer" },
                    "audioClipName": str_t("Audio clip to trigger.")
                },
                "required": ["clipName", "frame", "audioClipName"]
            }),
        },
        ToolDef {
            name: "sgame_export_to_engine",
            description: "Export the assembled game asset package to a target engine.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entityName": str_t("Assembled entity."),
                    "targetEngine": { "type": "string", "enum": ["godot4", "unity", "unreal5", "gltf_standard"] },
                    "outputDirectory": str_t("Destination directory."),
                    "embedAudio": { "type": "boolean", "default": true },
                    "optimizeTextures": { "type": "boolean", "default": true }
                },
                "required": ["entityName", "targetEngine", "outputDirectory"]
            }),
        },
        ToolDef {
            name: "sgame_generate_engine_code",
            description: "Generate boilerplate character controller or entity script for a target engine and language.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entityName": str_t("Assembled entity."),
                    "targetEngine": { "type": "string", "enum": ["godot4", "unity", "unreal5", "bevy", "raylib", "web_html5", "defold_love2d"] },
                    "language": { "type": "string", "enum": ["gdscript", "csharp", "cpp", "rust", "c", "typescript", "lua"] },
                    "gameplayStyle": { "type": "string", "enum": ["platformer_2d", "top_down_2d", "third_person_3d", "first_person_3d", "stationary_npc", "custom_prompt"], "default": "platformer_2d" },
                    "customPrompt": str_t("Optional AI-assisted instructions."),
                    "outputFilePath": str_t("Optional path to write the script file.")
                },
                "required": ["entityName", "targetEngine", "language"]
            }),
        },
    ]
}

/// Execute an MCP tool call against the Command Bus.
pub async fn call_tool(bus: &CommandBus, name: &str, arguments: Value) -> Result<Value, String> {
    if name == "sgame_get_scene_summary" {
        return Ok(bus.state_summary_json());
    }

    let def = tool_definitions()
        .into_iter()
        .find(|t| t.name == name)
        .ok_or_else(|| format!("Unknown tool: {name}"))?;
    let description: &'static str = def.description;

    let command_name = name.strip_prefix("sgame_").unwrap_or(name);
    let wrapper = serde_json::json!({ "command": command_name, "payload": arguments });
    let cmd: StudioCommand = serde_json::from_value(wrapper).map_err(|e| {
        format!("invalid arguments for {name}: {e} (tool '{}' expects {}", name, description)
    })?;
    let envelope = CommandEnvelope::new(Issuer::Mcp, cmd);
    let result = bus.dispatch(envelope).await;
    Ok(json!({
        "status": result.status,
        "commandId": result.command_id,
        "result": result.result,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::SceneState;

    #[tokio::test]
    async fn test_call_tool_create_primitive() {
        let (bus, _rx) = CommandBus::new(SceneState::new());
        let args = serde_json::json!({ "primitiveType": "cube", "name": "McpCube" });
        let result = call_tool(&bus, "sgame_create_3d_primitive", args).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "success");
        assert_eq!(
            value["result"]["entityId"].as_str(),
            Some("mesh3d_1")
        );
    }

    #[tokio::test]
    async fn test_call_tool_unknown() {
        let (bus, _rx) = CommandBus::new(SceneState::new());
        assert!(call_tool(&bus, "unknown_tool", Value::Null).await.is_err());
    }

    #[tokio::test]
    async fn test_call_tool_scene_summary() {
        let (bus, _rx) = CommandBus::new(SceneState::new());
        let args = serde_json::json!({ "primitiveType": "cube", "name": "McpCube" });
        call_tool(&bus, "sgame_create_3d_primitive", args).await.unwrap();
        let summary = call_tool(&bus, "sgame_get_scene_summary", Value::Null).await;
        assert!(summary.is_ok());
        assert_eq!(summary.unwrap()["entityCount"].as_u64(), Some(1));
    }

    #[test]
    fn all_command_tools_present() {
        // Every 24 StudioCommand names (minus scene summary which is separate)
        // must have a corresponding tool definition.
        let defs = tool_definitions();
        let names: std::collections::HashSet<&str> = defs.iter().map(|d| d.name).collect();
        for name in [
            "sgame_new_project", "sgame_save_project", "sgame_load_project",
            "sgame_create_2d_shape", "sgame_boolean_2d_paths", "sgame_auto_rig_2d",
            "sgame_create_node_model", "sgame_convert_node_model_to_mesh",
            "sgame_create_3d_primitive", "sgame_extrude_3d_face", "sgame_inset_3d_face",
            "sgame_bevel_3d_edge", "sgame_loop_cut_3d", "sgame_auto_rig_3d",
            "sgame_create_2d_keyframe", "sgame_create_3d_keyframe",
            "sgame_generate_sfx_preset", "sgame_synthesize_vocal", "sgame_create_music_pattern",
            "sgame_ai_prompt_to_asset", "sgame_assemble_game_entity", "sgame_bind_audio_to_frame",
            "sgame_export_to_engine", "sgame_generate_engine_code",
        ] {
            assert!(names.contains(name), "missing tool definition for {name}");
        }
        assert!(names.contains("sgame_get_scene_summary"));
        assert_eq!(names.len(), 25);
    }
}
