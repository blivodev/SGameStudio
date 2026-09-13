import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  loadSettings,
  saveProviderKey,
  clearProviderKey,
  DEFAULT_SETTINGS,
} from '../services/settings';

function jsonResponse(body: unknown, ok = true) {
  return {
    ok,
    status: ok ? 200 : 500,
    json: async () => body,
  } as Response;
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe('settings service', () => {
  it('loadSettings parses backend payload and merges defaults', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        aiRouting: { chat_assistant: { provider: 'deepinfra', model: 'test-model' } },
        exportDefaults: { defaultEngine: 'godot4', audioFormat: 'ogg', meshFormat: 'glb' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    const s = await loadSettings();
    expect(s.aiRouting.chat_assistant.model).toBe('test-model');
    expect(s.providers.openrouter).toBeDefined();
    expect(s.exportDefaults.meshFormat).toBe('glb');
  });

  it('loadSettings falls back to defaults on failure', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('offline')));

    const s = await loadSettings();
    expect(s.exportDefaults.meshFormat).toBe('glb');
    expect(DEFAULT_SETTINGS.aiRouting.chat_assistant).toBeDefined();
  });

  it('saveProviderKey sends camelCase apiKey over POST', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({ provider: 'openrouter', configured: true }));
    vi.stubGlobal('fetch', fetchMock);

    const res = await saveProviderKey('openrouter', 'sk-live-1234');
    expect(res.configured).toBe(true);
    const [, init] = fetchMock.mock.calls[0];
    expect(init.method).toBe('POST');
    const body = JSON.parse(init.body as string);
    expect(body.provider).toBe('openrouter');
    expect(body.apiKey).toBe('sk-live-1234');
    expect(body.clear).toBeUndefined();
  });

  it('clearProviderKey sends clear flag', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({ provider: 'openai', configured: false }));
    vi.stubGlobal('fetch', fetchMock);

    await clearProviderKey('openai');
    const [, init] = fetchMock.mock.calls[0];
    const body = JSON.parse(init.body as string);
    expect(body.provider).toBe('openai');
    expect(body.clear).toBe(true);
  });
});
