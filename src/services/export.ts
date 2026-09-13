import { apiBase } from './commandBridge';

export type MeshFormatId = "obj" | "stl" | "glb" | "ply" | "stl_bin" | "ply_bin";

export interface MeshExportResult {
  entity: string;
  format: string;
  fileName: string;
  mimeType: string;
  sizeBytes: number;
  dataBase64: string;
  download: boolean;
}

/**
 * Export a mesh entity through the backend to the requested format.
 * Returns the raw artifact (base64-encoded bytes) without forcing a download.
 */
export async function exportMesh(entity: string, format: MeshFormatId): Promise<MeshExportResult> {
  const url = `${apiBase()}/api/v1/mesh/export?entity=${encodeURIComponent(entity)}&format=${encodeURIComponent(format)}&download=false`;
  const res = await fetch(url, { signal: AbortSignal.timeout(20000) });
  const data = (await res.json().catch(() => null)) as MeshExportResult | null;
  if (!res.ok) {
    throw new Error((data as { error?: string } | null)?.error ?? `Export failed: HTTP ${res.status}`);
  }
  return data!;
}

function base64ToBytes(b64: string): Uint8Array {
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

/** Trigger a browser download from a base64-encoded file payload. */
export function downloadAssetResult(result: MeshExportResult): void {
  const bytes = base64ToBytes(result.dataBase64);
  const blob = new Blob([bytes], { type: result.mimeType || "application/octet-stream" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = result.fileName;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}
