// End-to-end headless flow against the real library crate.
use sgame_studio::{CommandBus, CommandEnvelope, Issuer, SceneState, StudioCommand};
use sgame_studio::core::commands::{PrimitiveType, ProjectTemplate, SfxPreset, TargetEngine};

#[tokio::test]
async fn full_headless_flow() {
    let (bus, mut rx) = CommandBus::new(SceneState::new());
    // 1. new project
    let r = bus.dispatch(CommandEnvelope::new(Issuer::Cli, StudioCommand::NewProject { project_name: "Demo".into(), target_engine: TargetEngine::Godot4, template: ProjectTemplate::Empty })).await;
    assert_eq!(r.status, "success");
    // 2. create cube + sfx
    let r = bus.dispatch(CommandEnvelope::new(Issuer::Rest, StudioCommand::Create3DPrimitive { primitive_type: PrimitiveType::Cube, name: "Hero".into(), dimensions: None, subdivisions: None })).await;
    assert_eq!(r.status, "success");
    let entity_id = r.result.get("entityId").and_then(|v| v.as_str()).unwrap().to_string();
    assert_eq!(entity_id, "mesh3d_1");
    let r = bus.dispatch(CommandEnvelope::new(Issuer::Mcp, StudioCommand::GenerateSfxPreset { sound_name: "laser_1".into(), preset: SfxPreset::Laser, pitch_variation: Some(0.1) })).await;
    assert_eq!(r.status, "success");
    // 3. assemble entity referencing "Hero"
    let r = bus.dispatch(CommandEnvelope::new(Issuer::Gui, StudioCommand::AssembleGameEntity { entity_name: "HeroEntity".into(), dimension: Some("3D".into()), visual_source: "Hero".into(), animation_clips: vec!["Idle".into(), "Walk".into()], audio_bindings: vec![], collision_shape: None })).await;
    assert_eq!(r.status, "success");
    assert_eq!(bus.state_summary_json()["entityCount"].as_u64(), Some(3));
    // 4. events were broadcast
    let mut seen = false;
    while let Ok(ev) = rx.try_recv() { if matches!(ev, sgame_studio::StudioEvent::SceneMutated { .. }) { seen = true; } }
    assert!(seen, "expected at least one SceneMutated event");
    // 5. undo removes assembly, redo restores it
    let _ = bus.undo().await;
    assert_eq!(bus.state_summary_json()["entityCount"].as_u64(), Some(2));
    let _ = bus.redo().await;
    assert_eq!(bus.state_summary_json()["entityCount"].as_u64(), Some(3));
    // 6. zip roundtrip preserves state
    let path = std::env::temp_dir().join(format!("sgame_it_{}.sgsproj", uuid::Uuid::new_v4()));
    bus.save_to_zip(&path).expect("save");
    let loaded = sgame_studio::core::project_zip::load_project(&path).expect("load");
    assert_eq!(loaded.entity_count(), 3);
    assert_eq!(loaded.meta.name, "Demo");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn validation_failures() {
    let (bus, _) = CommandBus::new(SceneState::new());
    // Empty name
    let r = bus.dispatch(CommandEnvelope::new(Issuer::Cli, StudioCommand::Create3DPrimitive { primitive_type: PrimitiveType::Cube, name: "".into(), dimensions: None, subdivisions: None })).await;
    assert_eq!(r.status, "failure");
    // Unknown mesh
    let r = bus.dispatch(CommandEnvelope::new(Issuer::Cli, StudioCommand::Extrude3DFace { mesh_name: "Unknown".into(), face_indices: vec![0], distance: 1.0, individual: None })).await;
    assert_eq!(r.status, "failure");
}