import { useStudioStore } from '../store/studioStore';

export type CommandResult = { status: string; commandId: string; executionTimeMs: number; result: unknown };

export function apiBase(): string {
  return import.meta.env.VITE_SGAME_API_BASE ?? "http://127.0.0.1:8080";
}

export function buildDispatchBody(command: string, payload: Record<string, unknown>): string {
  return JSON.stringify({ command, payload });
}

export async function dispatchCommand(command: string, payload: Record<string, unknown>): Promise<CommandResult> {
  const res = await fetch(`${apiBase()}/api/v1/commands/dispatch`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: buildDispatchBody(command, payload),
  });
  if (!res.ok) throw new Error(`Command ${command} failed: HTTP ${res.status}`);
  return (await res.json()) as CommandResult;
}

export async function checkHealth(): Promise<boolean> {
  try {
    const r = await fetch(`${apiBase()}/api/v1/health`, { signal: AbortSignal.timeout(3000) });
    return r.ok;
  } catch {
    return false;
  }
}

export async function dispatchAndTrack(command: string, payload: Record<string, unknown>, label: string): Promise<CommandResult> {
  const result = await dispatchCommand(command, payload);
  if (result.result && typeof result.result === 'object' && 'entityId' in result.result) {
    const { entityId } = result.result as { entityId: string };
    useStudioStore.getState().addEntity({
      id: entityId,
      kind: command.startsWith("create_") ? command.replace("create_", "") : "unknown",
      name: entityId,
      label,
    });
  }
  return result;
}