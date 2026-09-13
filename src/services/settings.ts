import { apiBase } from './commandBridge';

interface AiRoute {
  provider: string;
  model: string;
}

interface StudioSettings {
  aiRouting: Record<string, AiRoute>;
  exportDefaults: {
    defaultEngine: string;
    audioFormat: string;
    meshFormat: string;
  };
}

export const DEFAULT_SETTINGS: StudioSettings = {
  aiRouting: {
    chat_assistant: { provider: "deepinfra", model: "Meta-Llama-3.1-70B-Instruct" },
    vector_2d_generation: { provider: "openrouter", model: "claude-3.5-sonnet" },
    model_3d_generation: { provider: "comfyui_local", model: "trellis" },
    texture_pbr: { provider: "deepinfra", model: "sdxl" },
    vocal: { provider: "local_piper", model: "en_US-lessac-medium" },
    music: { provider: "local_audiocraft", model: "musicgen-small" },
  },
  exportDefaults: {
    defaultEngine: "godot4",
    audioFormat: "ogg",
    meshFormat: "glb",
  },
};

export async function loadSettings(): Promise<StudioSettings> {
  try {
    const res = await fetch(`${apiBase()}/api/v1/settings`);
    if (res.ok) return (await res.json()) as StudioSettings;
    return DEFAULT_SETTINGS;
  } catch {
    return DEFAULT_SETTINGS;
  }
}