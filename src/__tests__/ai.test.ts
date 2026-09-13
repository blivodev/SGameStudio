import { describe, it, expect, vi, beforeEach } from 'vitest';
import { chatWithAssistant } from '../services/ai';

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
