import { useEffect, useState } from 'react';
import { parseEntityRef } from '../../lib/parseResult';
import { dispatchCommand } from '../../services/commandBridge';
import { useStudioStore } from '../../store/studioStore';
import { Panel, Button } from '../../components/ui/primitives';

export default function Assembler() {
  const [entityName, setEntityName] = useState<string>("");
  const [dimension, setDimension] = useState<string>("3D");
  const [visualSource, setVisualSource] = useState<string>("");
  const [clipsText, setClipsText] = useState<string>("Idle, Walk, Attack");
  const [collision, setCollision] = useState<string>("capsule");
  const [radius, setRadius] = useState<string>("0.5");
  const [height, setHeight] = useState<string>("1.8");
  const [engine, setEngine] = useState<string>("godot4");
  const [outputDir, setOutputDir] = useState<string>("./dist/export");
  const [exportedFiles, setExportedFiles] = useState<string[]>([]);
  const [selectedEntity, setSelectedEntity] = useState<string>("");

  const { entities, addEntity, setLastError } = useStudioStore();

  const assemble = async () => {
    try {
      const res = await dispatchCommand("assemble_game_entity", {
        entityName,
        dimension,
        visualSource,
        animationClips: clipsText.split(",").map((s) => s.trim()).filter(Boolean),
        audioBindings: [],
        collisionShape: {
          type: collision,
          radius: Number(radius),
          height: Number(height)
        }
      });
      const ref = parseEntityRef(res.result);
      if (ref) addEntity({ ...ref, name: entityName, label: "Entity" });
      setSelectedEntity(entityName);
    } catch (e) {
      setLastError(String(e));
    }
  };

  const exportEntity = async () => {
    try {
      const res = await dispatchCommand("export_to_engine", {
        entityName: selectedEntity,
        targetEngine: engine,
        outputDirectory: outputDir,
        embedAudio: true,
        optimizeTextures: true
      });
      const r = res.result as { filesGenerated?: string[] };
      setExportedFiles(r.filesGenerated ?? []);
    } catch (e) {
      setLastError(String(e));
    }
  };

  useEffect(() => {
    if (!selectedEntity && entities.some(e => e.kind === "entity")) {
      const firstEntity = entities.find(e => e.kind === "entity");
      if (firstEntity) setSelectedEntity(firstEntity.name);
    }
  }, [entities]);

  return (
    <div className="grid grid-cols-2 gap-3 h-full overflow-y-auto p-3">
      <Panel title="Entity Composer">
        <div className="flex flex-col gap-3">
          <div>
            <label className="text-xs text-muted">Entity Name</label>
            <input
              type="text"
              value={entityName}
              onChange={(e) => setEntityName(e.target.value)}
              className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
            />
          </div>

          <div>
            <label className="text-xs text-muted">Dimension</label>
            <select
              value={dimension}
              onChange={(e) => setDimension(e.target.value)}
              className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
            >
              <option value="2D">2D</option>
              <option value="3D">3D</option>
            </select>
          </div>

          <div>
            <label className="text-xs text-muted">Visual Source</label>
            <input
              type="text"
              placeholder="e.g. mesh3d_1"
              value={visualSource}
              onChange={(e) => setVisualSource(e.target.value)}
              className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
            />
          </div>

          <div>
            <label className="text-xs text-muted">Animation Clips</label>
            <input
              type="text"
              value={clipsText}
              onChange={(e) => setClipsText(e.target.value)}
              className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
            />
          </div>

          <div>
            <label className="text-xs text-muted">Collision Shape</label>
            <select
              value={collision}
              onChange={(e) => setCollision(e.target.value)}
              className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
            >
              <option value="box">Box</option>
              <option value="capsule">Capsule</option>
              <option value="sphere">Sphere</option>
            </select>
          </div>

          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="text-xs text-muted">Radius</label>
              <input
                type="number"
                step="0.1"
                value={radius}
                onChange={(e) => setRadius(e.target.value)}
                className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
              />
            </div>
            <div>
              <label className="text-xs text-muted">Height</label>
              <input
                type="number"
                step="0.1"
                value={height}
                onChange={(e) => setHeight(e.target.value)}
                className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
              />
            </div>
          </div>

          <Button
            onClick={assemble}
            disabled={!entityName.trim() || !visualSource.trim()}
          >
            Assemble Entity
          </Button>
        </div>
      </Panel>

      <Panel title="Export">
        <div className="flex flex-col gap-3">
          <div>
            <label className="text-xs text-muted">Entity</label>
            <select
              value={selectedEntity}
              onChange={(e) => setSelectedEntity(e.target.value)}
              className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
            >
              {entities
                .filter(e => e.kind === "entity")
                .map(e => (
                  <option key={e.id} value={e.name}>{e.name}</option>
                ))
              }
            </select>
          </div>

          <div>
            <label className="text-xs text-muted">Engine</label>
            <select
              value={engine}
              onChange={(e) => setEngine(e.target.value)}
              className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
            >
              <option value="godot4">Godot 4</option>
              <option value="unity">Unity</option>
              <option value="unreal5">Unreal Engine 5</option>
              <option value="gltf_standard">glTF 2.0</option>
            </select>
          </div>

          <div>
            <label className="text-xs text-muted">Output Directory</label>
            <input
              type="text"
              value={outputDir}
              onChange={(e) => setOutputDir(e.target.value)}
              className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
            />
          </div>

          <Button onClick={exportEntity} disabled={!selectedEntity}>
            Export
          </Button>

          {exportedFiles.length > 0 && (
            <div className="flex flex-col gap-1 text-xs">
              <span className="text-muted">Exported files:</span>
              {exportedFiles.map((f, i) => (
                <div key={i} className="font-mono">{f}</div>
              ))}
            </div>
          )}
        </div>
      </Panel>

      <Panel title="Generated Code" className="col-span-2">
        <p className="text-xs text-muted">
          Deterministic controller code generation (GDScript, C#, C++, Rust, TypeScript, Lua) lands with the Code Generator milestone.
        </p>
        <Button disabled>Copy</Button>
      </Panel>
    </div>
  );
}