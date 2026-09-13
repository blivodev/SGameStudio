import { describe, it, expect, vi, beforeEach } from 'vitest';
import { chatWithAssistant, runAgent, formatAgentStep } from '../services/ai';

function jsonResponse(body: unknown, ok: boolean, status: number) {
  return { ok, status, json: async () => body } as Response;
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe('ai chat service', () => {
  it('posts the message and returns the reply', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({ reply: 'hello back', task: 'chat_assistant' }, true, 200));
    vi.stubGlobal('fetch', fetchMock);

    const res = await chatWithAssistant('hello');
    expect(res.reply).toBe('hello back');
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toContain('/api/v1/ai/chat');
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body)).toEqual({ message: 'hello', task: 'chat' });
  });

  it('throws a readable error when not configured (503)', async () => {
    const err = { error: 'AI provider is not configured. Add an API key in Settings → AI Provider API Keys.' };
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse(err, false, 503));
    vi.stubGlobal('fetch', fetchMock);

    await expect(chatWithAssistant('hi')).rejects.toThrow(/not configured/);
  });
});

describe('AI agent service', () => {
  it('posts the build prompt and returns executed step results', async () => {
    const payload = {
      results: [{ command: 'sgame_create_3d_primitive', status: 'success', result: { entityId: 'mesh3d_1' } }],
    };
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse(payload, true, 200));
    vi.stubGlobal('fetch', fetchMock);

    const res = await runAgent('make a box');
    expect(res.results).toHaveLength(1);
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toContain('/api/v1/ai/agent');
    expect(JSON.parse(init.body).prompt).toBe('make a box');
  });

  it('throws a readable error when the agent provider is unconfigured', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse({ error: 'AI provider is not configured.' }, false, 503)));
    await expect(runAgent('x')).rejects.toThrow(/not configured/);
  });

  it('formats step results into readable lines', () => {
    expect(formatAgentStep({ command: 'sgame_create_3d_primitive', status: 'success', result: { entityId: 'mesh3d_1' } })).toContain('mesh3d_1');
    expect(formatAgentStep({ error: 'boom' })).toContain('boom');
    expect(formatAgentStep({ command: 'sgame_generate_sfx_preset', status: 'success', result: {} })).toContain('sgame_generate_sfx_preset');
  });
});
