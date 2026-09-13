import { apiBase } from './commandBridge';

interface AiRoute {
  provider: string;
  model: string;
  endpoint?: string;
}

interface ProviderStatus {
  configured: boolean;
  keyMasked?: string | null;
}

export interface StudioSettings {
  aiRouting: Record<string, AiRoute>;
  providers: Record<string, ProviderStatus>;
  exportDefaults: {
    defaultEngine: string;
    audioFormat: string;
    meshFormat: string;
  };
}

export const PROVIDER_NAMES = ["openrouter", "deepinfra", "openai", "anthropic", "gemini"] as const;
export type ProviderId = (typeof PROVIDER_NAMES)[number];

export const PROVIDER_LABELS: Record<string, string> = {
  openrouter: "OpenRouter",
  deepinfra: "DeepInfra",
  openai: "OpenAI",
  anthropic: "Anthropic",
  gemini: "Google Gemini",
};

export const DEFAULT_SETTINGS: StudioSettings = {
  aiRouting: {
    chat_assistant: { provider: "deepinfra", model: "Meta-Llama-3.1-70B-Instruct" },
    vector_2d_generation: { provider: "openrouter", model: "claude-3.5-sonnet" },
    model_3d_generation: { provider: "comfyui_local", model: "trellis" },
    texture_pbr: { provider: "deepinfra", model: "sdxl" },
    vocal: { provider: "local_piper", model: "en_US-lessac-medium" },
    music: { provider: "local_audiocraft", model: "musicgen-small" },
  },
  providers: Object.fromEntries(PROVIDER_NAMES.map((p) => [p, { configured: false, keyMasked: null }])),
  exportDefaults: {
    defaultEngine: "godot4",
    audioFormat: "ogg",
    meshFormat: "glb",
  },
};

export async function loadSettings(): Promise<StudioSettings> {
  try {
    const res = await fetch(`${apiBase()}/api/v1/settings`, { signal: AbortSignal.timeout(4000) });
    if (res.ok) {
      const data = (await res.json()) as Partial<StudioSettings>;
      return {
        ...DEFAULT_SETTINGS,
        ...data,
        providers: { ...DEFAULT_SETTINGS.providers, ...(data.providers ?? {}) },
      };
    }
    return DEFAULT_SETTINGS;
  } catch {
    return DEFAULT_SETTINGS;
  }
}

export interface SaveKeyResult {
  provider: string;
  configured: boolean;
  keyMasked?: string | null;
}

export async function saveProviderKey(provider: string, apiKey: string): Promise<SaveKeyResult> {
  const res = await fetch(`${apiBase()}/api/v1/settings`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ provider, apiKey }),
  });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new Error((body as { error?: string })?.error ?? `Failed to save key: HTTP ${res.status}`);
  }
  return (await res.json()) as SaveKeyResult;
}

export async function clearProviderKey(provider: string): Promise<SaveKeyResult> {
  const res = await fetch(`${apiBase()}/api/v1/settings`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ provider, clear: true }),
  });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new Error((body as { error?: string })?.error ?? `Failed to clear key: HTTP ${res.status}`);
  }
  return (await res.json()) as SaveKeyResult;
}
