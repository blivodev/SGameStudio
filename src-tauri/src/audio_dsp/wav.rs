use std::io::Write;

/// Convert a mono f32 sample in range [-1, 1] to 16-bit signed PCM.
pub fn samples_to_i16(sample: f32) -> i16 {
    let clamped = sample.clamp(-1.0, 1.0);
    (clamped * i16::MAX as f32) as i16
}

/// Encode mono f32 samples as a complete 16-bit PCM WAV file (44.1k default in caller).
pub fn encode_wav_16bit(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_len = samples.len();
    let bytes_per_sample = 2u32;
    let byte_rate = sample_rate * bytes_per_sample;
    let data_bytes = data_len as u32 * bytes_per_sample;

    let mut wav = Vec::with_capacity(44 + data_bytes as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());          // fmt chunk size
    wav.extend_from_slice(&1u16.to_le_bytes());           // PCM
    wav.extend_from_slice(&1u16.to_le_bytes());           // mono
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&(bytes_per_sample as u16).to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes());          // bits per sample
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_bytes.to_le_bytes());

    // Write samples via a rolling buffer to avoid re-allocating per sample.
    let mut tmp = [0u8; 2];
    for &sample in samples {
        let v = samples_to_i16(sample);
        tmp.copy_from_slice(&v.to_le_bytes());
        wav.extend_from_slice(&tmp);
    }
    wav
}

/// Write mono f32 samples as a 16-bit PCM WAV file to disk.
pub fn write_wav(path: &std::path::Path, samples: &[f32], sample_rate: u32) -> anyhow::Result<()> {
    if samples.is_empty() || sample_rate == 0 {
        anyhow::bail!("refusing to write empty or invalid WAV");
    }
    let bytes = encode_wav_16bit(samples, sample_rate);
    let mut file = std::fs::File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_meta() {
        let samples = vec![0.0f32, 0.5, -0.5, 1.0, -1.0];
        let wav = encode_wav_16bit(&samples, 44100);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(wav.len(), 44 + 5 * 2);
    }

    #[test]
    fn sample_clamp() {
        assert_eq!(samples_to_i16(1.0), i16::MAX);
        assert_eq!(samples_to_i16(-1.0), -i16::MAX);
        assert_eq!(samples_to_i16(0.0), 0);
    }
}
