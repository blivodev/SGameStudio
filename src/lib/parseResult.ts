export interface EntityRef { id: string; kind: string }
export function parseEntityRef(result: unknown): EntityRef | null {
  if (typeof result !== "object" || result === null) return null;
  const r = result as Record<string, unknown>;
  if (typeof r.entityId !== "string") return null;
  return { id: r.entityId, kind: typeof r.kind === "string" ? r.kind : "unknown" };
}