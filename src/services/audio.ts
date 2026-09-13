import { apiBase } from './commandBridge';

export interface AudioExportResult {
  preset: string;
  fileName: string;
  mimeType: string;
  sampleRate: number;
  durationSeconds: number;
  sizeBytes: number;
  dataBase64: string;
}

export type SfxPresetName =
  | "laser"
  | "explosion"
  | "powerup"
  | "hit"
  | "jump"
  | "coin"
  | "blip"
  | "footstep";

export const SFX_PRESETS: SfxPresetName[] = ["laser", "explosion", "powerup", "hit", "jump", "coin", "blip", "footstep"];

/** Synthesize an sfxr preset on the backend and return the WAV payload. */
export async function exportAudioPreset(preset: SfxPresetName, pitch = 0): Promise<AudioExportResult> {
  const url = `${apiBase()}/api/v1/audio/export?preset=${encodeURIComponent(preset)}&pitch=${encodeURIComponent(String(pitch))}`;
  const res = await fetch(url, { signal: AbortSignal.timeout(15000) });
  const data = (await res.json().catch(() => null)) as AudioExportResult | null;
  if (!res.ok) {
    throw new Error((data as { error?: string } | null)?.error ?? `Audio export failed: HTTP ${res.status}`);
  }
  return data!;
}

function base64ToBytes(b64: string): Uint8Array {
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

/** Trigger a browser download of the exported audio payload. */
export function downloadAudioResult(result: AudioExportResult): void {
  const bytes = base64ToBytes(result.dataBase64);
  const blob = new Blob([bytes], { type: result.mimeType || "audio/wav" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = result.fileName;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}
