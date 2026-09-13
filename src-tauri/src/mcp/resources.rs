use crate::core::{CommandBus, EntityKind};

pub struct ResourceDef {
    pub uri: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub mime_type: &'static str,
}

pub fn resource_definitions() -> &'static [ResourceDef] {
    &[
        ResourceDef {
            uri: "sgame://project/summary",
            name: "Project Summary",
            description: "Summary of the project",
            mime_type: "application/json",
        },
        ResourceDef {
            uri: "sgame://scene/2d/hierarchy",
            name: "2D Scene Hierarchy",
            description: "Hierarchy of 2D scene",
            mime_type: "application/json",
        },
        ResourceDef {
            uri: "sgame://scene/3d/hierarchy",
            name: "3D Scene Hierarchy",
            description: "Hierarchy of 3D scene",
            mime_type: "application/json",
        },
        ResourceDef {
            uri: "sgame://audio/manifest",
            name: "Audio Manifest",
            description: "List of audio assets",
            mime_type: "application/json",
        },
        ResourceDef {
            uri: "sgame://assembler/entity",
            name: "Game Entity",
            description: "Game entity definition",
            mime_type: "application/json",
        },
        ResourceDef {
            uri: "sgame://viewport/screenshot",
            name: "Viewport Screenshot",
            description: "Screenshot of the viewport",
            mime_type: "image/png",
        },
    ]
}

fn records_of_kind(bus: &CommandBus, kind: EntityKind) -> Vec<serde_json::Value> {
    bus.with_state(|state| {
        state
            .entities
            .values()
            .filter(|r| r.kind == kind)
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "kind": r.kind.as_str(),
                    "data": r.data,
                    "operations": r.op_log,
                })
            })
            .collect()
    })
}

/// Read a resource into (content, mime_type). The viewport screenshot is not
/// available headlessly, so it reports an explicit unsupported error.
pub fn read_resource(bus: &CommandBus, uri: &str) -> Result<(String, &'static str), String> {
    match uri {
        "sgame://project/summary" => Ok((bus.state_summary_json().to_string(), "application/json")),
        "sgame://scene/2d/hierarchy" => {
            let shapes = records_of_kind(bus, EntityKind::Shape2D);
            let rigs = records_of_kind(bus, EntityKind::Rig);
            let value = serde_json::json!({
                "shapes": shapes,
                "rigs": rigs,
            });
            Ok((value.to_string(), "application/json"))
        }
        "sgame://scene/3d/hierarchy" => {
            let meshes = records_of_kind(bus, EntityKind::Mesh3D);
            let node_models = records_of_kind(bus, EntityKind::NodeModel);
            let rigs = records_of_kind(bus, EntityKind::Rig);
            let value = serde_json::json!({
                "meshes": meshes,
                "nodeModels": node_models,
                "rigs": rigs,
            });
            Ok((value.to_string(), "application/json"))
        }
        "sgame://audio/manifest" => {
            let clips = records_of_kind(bus, EntityKind::AudioClip);
            let tracks = records_of_kind(bus, EntityKind::MusicTrack);
            let value = serde_json::json!({
                "clips": clips,
                "tracks": tracks,
            });
            Ok((value.to_string(), "application/json"))
        }
        "sgame://assembler/entity" => {
            let entities = records_of_kind(bus, EntityKind::Entity);
            Ok((serde_json::json!({ "entities": entities }).to_string(), "application/json"))
        }
        "sgame://viewport/screenshot" => Err("Viewport screenshot is only available in GUI or server mode with an active viewport".into()),
        _ => Err(format!("Unknown resource: {uri}")),
    }
}
