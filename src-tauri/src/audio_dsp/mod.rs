#![allow(dead_code)]

pub mod sfxr;
pub mod wav;

use crate::core::commands::SfxPreset;

pub use sfxr::{SfxrParams, Waveform};

/// Height (seconds) of a media-size rendered sound cap per preset types.
pub const DEFAULT_SAMPLE_RATE: u32 = 44100;

/// Build a deterministic `SfxrParams` for one of the studio SFX presets.
pub fn params_for_preset(preset: &SfxPreset, pitch_variation: f64) -> SfxrParams {
    // Pitch variation shifts the base frequency within +/- one octave.
    let octave = 2.0f32.powf(pitch_variation.clamp(-1.0, 1.0) as f32);
    let mut p = match preset {
        SfxPreset::Laser => SfxrParams {
            waveform: Waveform::Saw,
            duration: 0.3,
            start_frequency: 660.0,
            min_frequency: 0.05,
            slide: -1.0,
            vibrato_depth: 0.0,
            sustain: 0.0,
            lowpass_cutoff: 0.0,
            ..Default::default()
        },
        SfxPreset::Explosion => SfxrParams {
            waveform: Waveform::Noise,
            duration: 0.6,
            start_frequency: 400.0,
            min_frequency: 0.4,
            slide: -0.7,
            sustain: 0.0,
            lowpass_cutoff: 3500.0,
            ..Default::default()
        },
        SfxPreset::Powerup => SfxrParams {
            waveform: Waveform::Square,
            duration: 0.4,
            start_frequency: 300.0,
            min_frequency: 0.9,
            slide: 0.7,
            sustain: 0.4,
            lowpass_cutoff: 0.0,
            ..Default::default()
        },
        SfxPreset::Hit => SfxrParams {
            waveform: Waveform::Square,
            duration: 0.15,
            start_frequency: 200.0,
            min_frequency: 0.0,
            slide: -0.5,
            sustain: 0.0,
            punch: 1.0,
            lowpass_cutoff: 2000.0,
            ..Default::default()
        },
        SfxPreset::Jump => SfxrParams {
            waveform: Waveform::Square,
            duration: 0.25,
            start_frequency: 250.0,
            min_frequency: 0.7,
            slide: 0.6,
            sustain: 0.0,
            lowpass_cutoff: 4000.0,
            ..Default::default()
        },
        SfxPreset::Coin => SfxrParams {
            waveform: Waveform::Square,
            duration: 0.2,
            start_frequency: 880.0,
            min_frequency: 1.0,
            slide: 0.0,
            sustain: 0.2,
            lowpass_cutoff: 0.0,
            ..Default::default()
        },
        SfxPreset::Blip => SfxrParams {
            waveform: Waveform::Square,
            duration: 0.1,
            start_frequency: 1200.0,
            min_frequency: 1.0,
            sustain: 0.0,
            lowpass_cutoff: 0.0,
            ..Default::default()
        },
        SfxPreset::Footstep => SfxrParams {
            waveform: Waveform::Noise,
            duration: 0.12,
            start_frequency: 300.0,
            min_frequency: 0.1,
            slide: -0.8,
            sustain: 0.0,
            punch: 0.8,
            lowpass_cutoff: 1500.0,
            ..Default::default()
        },
    };
    p.start_frequency = (p.start_frequency * octave).clamp(20.0, 8000.0);
    p
}

/// Render a preset to mono f32 samples.
pub fn render_preset(preset: &SfxPreset, pitch_variation: f64, sample_rate: u32) -> Vec<f32> {
    let params = params_for_preset(preset, pitch_variation);
    sfxr::render(&params, sample_rate)
}
