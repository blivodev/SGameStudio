use std::io::{Read, Write};
use std::path::Path;
use crate::core::state_store::{ProjectMeta, SceneState, EntityRecord};

pub fn save_project(state: &SceneState, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let file = std::fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("project.json", options)?;
    zip.write_all(serde_json::to_vec_pretty(&state.meta)?.as_slice())?;

    zip.start_file("entities.json", options)?;
    let entities = state.entities.values().collect::<Vec<_>>();
    zip.write_all(serde_json::to_vec_pretty(&serde_json::json!({ "entities": entities }))?.as_slice())?;

    for (name, bytes) in [
        ("scene2d/vectors.svg", "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>".as_bytes()),
        ("scene2d/rig2d.json", b"{}".as_slice()),
        ("scene2d/anims2d.json", b"{}".as_slice()),
        ("scene3d/model.glb", b"glTF placeholder".as_slice()),
        ("scene3d/rig3d.json", b"{}".as_slice()),
        ("scene3d/anims3d.json", b"{}".as_slice()),
        ("audio/sfx/.keep", b"".as_slice()),
        ("audio/vocals/.keep", b"".as_slice()),
        ("audio/music/.keep", b"".as_slice()),
        ("thumbnails/preview.webp", b"".as_slice()),
    ] {
        zip.start_file(name, options)?;
        zip.write_all(bytes)?;
    }

    zip.finish()?;
    Ok(())
}

pub fn load_project(path: &Path) -> anyhow::Result<SceneState> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut meta_json = String::new();
    archive.by_name("project.json")?.read_to_string(&mut meta_json)?;
    let meta: ProjectMeta = serde_json::from_str(&meta_json)?;

    let mut entities_json = String::new();
    archive.by_name("entities.json")?.read_to_string(&mut entities_json)?;
    let entities_value = serde_json::from_str::<serde_json::Value>(&entities_json)?;
    let records: Vec<EntityRecord> = serde_json::from_value(entities_value["entities"].clone())?;

    let mut entities = std::collections::BTreeMap::new();
    let mut max_id = 0u64;
    for r in records {
        if let Some(n) = r.id.rsplit('_').next().and_then(|s| s.parse::<u64>().ok()) {
            max_id = max_id.max(n);
        }
        entities.insert(r.id.clone(), r);
    }

    Ok(SceneState::from_parts(meta, entities))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::commands::{StudioCommand, Issuer, PrimitiveType};
    use std::env;
    use uuid::Uuid;

    #[test]
    fn test_save_and_load_project() {
        let mut state = SceneState::new();
        let cmd = StudioCommand::Create3DPrimitive {
            primitive_type: PrimitiveType::Cube,
            name: "Hero".to_string(),
            dimensions: None,
            subdivisions: None,
        };
        state.apply(&cmd).unwrap();

        let temp_path = env::temp_dir().join(format!("sgame_{}.sgsproj", Uuid::new_v4()));
        save_project(&state, &temp_path).unwrap();

        let loaded = load_project(&temp_path).unwrap();
        assert_eq!(loaded.meta.name, state.meta.name);
        assert_eq!(loaded.entity_count(), 1);
    }
}