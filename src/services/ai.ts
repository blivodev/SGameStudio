import { apiBase } from './commandBridge';

export interface AiChatResult {
  reply: string;
  task: string;
}

/**
 * Send a message to the in-app AI assistant through the backend's configured
 * chat provider (Settings → AI Provider API Keys). Resolves the reply text.
 */
export async function chatWithAssistant(message: string): Promise<AiChatResult> {
  const res = await fetch(`${apiBase()}/api/v1/ai/chat`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message, task: "chat" }),
    signal: AbortSignal.timeout(45000),
  });
  const data = await res.json().catch(() => null);
  if (!res.ok) {
    const error = (data as { error?: string } | null)?.error ?? `AI chat failed: HTTP ${res.status}`;
    throw new Error(error);
  }
  return data as AiChatResult;
}
