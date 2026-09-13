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

export interface AgentStepResult {
  command?: string | null;
  status?: string;
  result?: unknown;
  error?: string;
}

export interface AgentResult {
  results: AgentStepResult[];
}

/** Format a single executed step into a short, human-readable line. */
export function formatAgentStep(step: AgentStepResult): string {
  if (step.error) return `⚠ ${step.error}`;
  const name = step.command ?? "step";
  const status = step.status ?? "done";
  const entityId = (step.result as { entityId?: string } | null)?.entityId;
  return `• ${name} — ${status}${entityId ? ` (${entityId})` : ""}`;
}

/**
 * Run the in-app AI as an agent: the backend asks the chat provider for a
 * structured command plan, executes each step on the Command Bus, and returns
 * the results. Lets the AI create assets inside the app, not just reply.
 */
export async function runAgent(prompt: string): Promise<AgentResult> {
  const res = await fetch(`${apiBase()}/api/v1/ai/agent`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ prompt }),
    signal: AbortSignal.timeout(60000),
  });
  const data = await res.json().catch(() => null);
  if (!res.ok) {
    const error = (data as { error?: string } | null)?.error ?? `Agent failed: HTTP ${res.status}`;
    throw new Error(error);
  }
  return data as AgentResult;
}

