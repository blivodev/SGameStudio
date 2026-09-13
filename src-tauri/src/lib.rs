pub mod core;
pub mod server;
pub mod mcp;
pub mod mesh_kernel;
pub mod export;
pub mod audio_dsp;
pub mod ai_gateway;

#[cfg(feature = "gui")]
pub mod prelude;

pub use core::{CommandBus, CommandEnvelope, CommandResult, Issuer, SceneState, StudioCommand, StudioEvent};

/// Headless self-diagnostics (`sgame-studio test`).
pub fn run_self_diagnostics() -> anyhow::Result<()> {
    tracing::info!("SGameStudio self-diagnostics starting");
    let mut failures: u32 = 0;
    // 1. Command bus dispatch + undo/redo symmetry
    {
        let (bus, _rx) = CommandBus::new(SceneState::new());
        let env = CommandEnvelope::new(core::Issuer::Cli, StudioCommand::Create3DPrimitive { primitive_type: core::commands::PrimitiveType::Cube, name: "TestCube".into(), dimensions: None, subdivisions: None });
        let res = futures::executor::block_on(bus.dispatch(env));
        let ok1 = res.status == "success";
        let _ = futures::executor::block_on(bus.undo());
        let count_after_undo = bus.state_summary_json()["entityCount"].as_u64().unwrap_or(u64::MAX);
        let _ = futures::executor::block_on(bus.redo());
        let count_after_redo = bus.state_summary_json()["entityCount"].as_u64().unwrap_or(0);
        let ok = ok1 && count_after_undo == 0 && count_after_redo == 1;
        if ok { println!("  [PASS] command_bus  : dispatch + undo/redo symmetry"); } else { failures += 1; println!("  [FAIL] command_bus  : dispatch + undo/redo symmetry"); }
    }
    // 2. .sgsproj zip roundtrip
    {
        let mut state = SceneState::new();
        let cmd = StudioCommand::Create3DPrimitive { primitive_type: core::commands::PrimitiveType::Sphere, name: "RoundBot".into(), dimensions: None, subdivisions: None };
        let _ = state.apply(&cmd);
        let path = std::env::temp_dir().join(format!("sgame_diag_{}.sgsproj", uuid::Uuid::new_v4()));
        let saved = core::project_zip::save_project(&state, &path);
        let loaded = saved.and_then(|_| core::project_zip::load_project(&path));
        match loaded { Ok(ls) => if ls.entity_count() == state.entity_count() && ls.meta.name == state.meta.name { println!("  [PASS] project_zip : .sgsproj save/load roundtrip"); } else { failures += 1; println!("  [FAIL] project_zip : roundtrip mismatch"); }, Err(e) => { failures += 1; println!("  [FAIL] project_zip : {e}"); } }
        let _ = std::fs::remove_file(&path);
    }
    // 3. Command schema conformance (MCP/REST JSON)
    {
        let json = r#"{"command":"create_3d_primitive","payload":{"primitiveType":"cube","name":"Knight_Shield","dimensions":{"x":0.8,"y":1.2,"z":0.1}}}"#;
        match serde_json::from_str::<StudioCommand>(json) { Ok(c) => println!("  [PASS] commands    : REST schema parse ({})", c.command_name()), Err(e) => { failures += 1; println!("  [FAIL] commands    : schema parse: {e}"); } }
    }
    // 4. Audio DSP: render an sfxr preset and validate the WAV header
    {
        let preset = core::commands::SfxPreset::Explosion;
        let samples = audio_dsp::render_preset(&preset, 0.0, 44100);
        if samples.is_empty() {
            failures += 1;
            println!("  [FAIL] audio_dsp   : preset rendered empty buffer");
        } else {
            let wav = audio_dsp::wav::encode_wav_16bit(&samples, 44100);
            let ok_header = wav.len() >= 44 && &wav[0..4] == b"RIFF" && &wav[8..12] == b"WAVE";
            let max_amp = samples.iter().fold(0.0f32, |a, &s| a.max(s.abs()));
            if ok_header && max_amp <= 1.0 {
                println!("  [PASS] audio_dsp   : sfxr synthesis + WAV header ({} samples)", samples.len());
            } else {
                failures += 1;
                println!("  [FAIL] audio_dsp   : invalid WAV or clipping (max_amp={max_amp})");
            }
        }
    }
    println!();
    println!("+--------------------------------------+");
    println!("| SGameStudio self-diagnostics         |");
    println!("| failures: {failures:<3}                            |");
    println!("+--------------------------------------+");
    if failures == 0 { Ok(()) } else { anyhow::bail!("{failures} diagnostics failed") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_pass() {
        crate::run_self_diagnostics().expect("self diagnostics should pass");
    }
}