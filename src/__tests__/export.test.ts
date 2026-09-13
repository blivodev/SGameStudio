import { describe, it, expect, vi, beforeEach } from 'vitest';
import { exportMesh } from '../services/export';

function jsonResponse(body: unknown, ok: boolean, status: number) {
  return { ok, status, json: async () => body } as Response;
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe('mesh export service', () => {
  it('calls the export endpoint with the entity and format', async () => {
    const payload = {
      entity: 'mesh3d_1',
      format: 'obj',
      fileName: 'mesh3d_1.obj',
      mimeType: 'model/obj',
      sizeBytes: 1234,
      dataBase64: 'YWJj',
      download: false,
    };
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse(payload, true, 200));
    vi.stubGlobal('fetch', fetchMock);

    const res = await exportMesh('mesh3d_1', 'obj');
    expect(res.fileName).toBe('mesh3d_1.obj');
    const [url] = fetchMock.mock.calls[0];
    expect(url).toContain('/api/v1/mesh/export');
    expect(url).toContain('entity=mesh3d_1');
    expect(url).toContain('format=obj');
  });

  it('throws a readable error when the backend rejects', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({ error: 'Unsupported mesh format' }, false, 400));
    vi.stubGlobal('fetch', fetchMock);
    await expect(exportMesh('x', 'obj')).rejects.toThrow(/Unsupported mesh format/);
  });
});
