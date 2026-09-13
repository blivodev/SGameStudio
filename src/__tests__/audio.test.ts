import { describe, it, expect, vi, beforeEach } from 'vitest';
import { exportAudioPreset } from '../services/audio';

function jsonResponse(body: unknown, ok: boolean, status: number) {
  return { ok, status, json: async () => body } as Response;
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe('audio export service', () => {
  it('requests the preset WAV from the backend and returns the payload', async () => {
    const payload = {
      preset: 'explosion',
      fileName: 'explosion.wav',
      mimeType: 'audio/wav',
      sampleRate: 44100,
      durationSeconds: 0.6,
      sizeBytes: 52964,
      dataBase64: 'UklGRg==',
    };
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse(payload, true, 200));
    vi.stubGlobal('fetch', fetchMock);

    const res = await exportAudioPreset('explosion');
    expect(res.fileName).toBe('explosion.wav');
    expect(res.mimeType).toBe('audio/wav');
    const [url] = fetchMock.mock.calls[0];
    expect(url).toContain('/api/v1/audio/export');
    expect(url).toContain('preset=explosion');
  });

  it('throws a readable error on unknown preset', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({ error: 'Unknown SFX preset: nope' }, false, 400));
    vi.stubGlobal('fetch', fetchMock);
    await expect(exportAudioPreset('laser')).rejects.toThrow(/Unknown SFX preset/);
  });
});
