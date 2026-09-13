#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Waveform { Square, Saw, Sine, Triangle, Noise }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxrParams {
    pub waveform: Waveform,
    pub seed: u64,
    pub duration: f32,          // seconds, clamped 0.05..2.0
    pub start_frequency: f32,   // Hz, 20..8000
    pub min_frequency: f32,     // fraction of start (0..1)
    pub slide: f32,             // -1..1 frequency slide amount
    pub vibrato_depth: f32,     // 0..1
    pub vibrato_rate: f32,      // Hz 0..50
    pub attack: f32,            // 0..1 fraction of duration
    pub sustain: f32,           // 0..1
    pub punch: f32,             // 0..1 initial volume punch
    pub lowpass_cutoff: f32,    // Hz 0..8000 (0 = off)
    pub repeat_hz: f32,         // retrigger rate 0..=40 (0 = off)
}

impl Default for SfxrParams {
    fn default() -> Self {
        Self {
            waveform: Waveform::Square,
            seed: 0,
            duration: 0.35,
            start_frequency: 880.0,
            min_frequency: 0.0,
            slide: 0.0,
            vibrato_depth: 0.0,
            vibrato_rate: 0.0,
            attack: 0.0,
            sustain: 0.3,
            punch: 0.0,
            lowpass_cutoff: 22050.0,
            repeat_hz: 0.0,
        }
    }
}

impl SfxrParams {
    fn clamp(&mut self) {
        self.duration = self.duration.clamp(0.05, 2.0);
        self.start_frequency = self.start_frequency.clamp(20.0, 8000.0);
        self.min_frequency = self.min_frequency.clamp(0.0, 1.0);
        self.slide = self.slide.clamp(-1.0, 1.0);
        self.vibrato_depth = self.vibrato_depth.clamp(0.0, 1.0);
        self.vibrato_rate = self.vibrato_rate.clamp(0.0, 50.0);
        self.attack = self.attack.clamp(0.0, 1.0);
        self.sustain = self.sustain.clamp(0.0, 1.0);
        self.punch = self.punch.clamp(0.0, 1.0);
        self.repeat_hz = self.repeat_hz.clamp(0.0, 40.0);
    }
}

/// Deterministic pseudo-random generator (xorshift64).
struct Rng(u64);

impl Rng {
    fn next_f(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Render a classic sfxr-style waveform into interleaved mono f32 samples in range [-1, 1].
pub fn render(params: &SfxrParams, sample_rate: u32) -> Vec<f32> {
    let mut p = params.clone();
    p.clamp();

    let n = (sample_rate as f32 * p.duration) as usize;
    let mut out = Vec::with_capacity(n);
    let mut rng = Rng(p.seed.wrapping_mul(0x9E3779B97F4A7C15));
    let inv_duration = 1.0 / (n as f32);

    // One-pole lowpass state.
    let mut lp = 0.0f32;
    let beta = if p.lowpass_cutoff > 0.0 {
        // Normalized cutoff for a one-pole filter; approximate.
        let w = (p.lowpass_cutoff / sample_rate as f32 * std::f32::consts::TAU).min(std::f32::consts::PI);
        1.0 - w.cos()
    } else {
        0.0
    };

    let mut phase = 0.0f32;
    let mut vibrato_phase = 0.0f32;

    for i in 0..n {
        let t = i as f32 * inv_duration;

        // Envelope: fast attack (near-instant with punch), then decay/gate toward sustain.
        let env = if t < p.attack {
            t / p.attack.max(1e-4)
        } else {
            let after_attack = (t - p.attack) / (1.0 - p.attack).max(1e-4);
            1.0 - after_attack * (1.0 - p.sustain)
        };
        // A tiny fade-in avoids clicks when attack is zero.
        let fade_in = (t * sample_rate as f32 * 0.01).min(1.0);

        // Re-trigger when repeat_hz is set.
        if p.repeat_hz > 0.0 {
            let period = (sample_rate as f32 / p.repeat_hz).round() as usize;
            if period > 0 && i % period == 0 {
                phase = 0.0;
                lp = 0.0;
            }
        }

        // Frequency with slide toward min, plus vibrato.
        let mut freq = p.start_frequency * (1.0 + p.slide * t);
        let min_f = p.start_frequency * p.min_frequency;
        if freq < min_f { freq = min_f; }
        vibrato_phase += p.vibrato_rate / sample_rate as f32 * std::f32::consts::TAU;
        freq *= 1.0 + vibrato_phase.sin() * p.vibrato_depth * 0.5;

        phase += freq / sample_rate as f32;
        if phase >= 1.0 { phase -= phase.floor(); }

        let sample = match p.waveform {
            Waveform::Square => if phase < 0.5 { 1.0 } else { -1.0 },
            Waveform::Saw => 2.0 * phase - 1.0,
            Waveform::Sine => (phase * std::f32::consts::TAU).sin(),
            Waveform::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
            Waveform::Noise => 2.0 * rng.next_f() - 1.0,
        };

        // Lowpass.
        if p.lowpass_cutoff > 0.0 && p.lowpass_cutoff < 10000.0 {
            lp += beta * (sample - lp);
        } else {
            lp = sample;
        }

        // Initial volume "punch" spike.
        let punch_env = (1.0 - t * 30.0).max(0.0) * p.punch;
        let v = (lp * env + punch_env).clamp(-1.0, 1.0);
        out.push(v * fade_in);
    }

    out
}
