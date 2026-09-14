import { useEffect, useState } from 'react';
import { Loader2, Copy } from 'lucide-react';
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
  const [codeLang, setCodeLang] = useState<string>("gdscript");
  const [style, setStyle] = useState<string>("third_person_3d");
  const [generatedCode, setGeneratedCode] = useState<string>("");
  const [generatedName, setGeneratedName] = useState<string>("");
  const [generating, setGenerating] = useState(false);

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

  const generateCode = async () => {
    if (!selectedEntity) return;
    setGenerating(true);
    try {
      const res = await dispatchCommand("generate_engine_code", {
        entityName: selectedEntity,
        targetEngine: engine,
        language: codeLang,
        gameplayStyle: style,
        customPrompt: null,
      });
      const r = res.result as { code?: string; fileName?: string };
      setGeneratedCode(r.code ?? "");
      setGeneratedName(r.fileName ?? "");
    } catch (e) {
      setLastError(String(e));
    } finally {
      setGenerating(false);
    }
  };

  const copyCode = async () => {
    try {
      await navigator.clipboard.writeText(generatedCode);
    } catch {
      // clipboard may be unavailable in restricted webviews — safe to ignore
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
        <div className="flex flex-wrap items-end gap-3">
          <div>
            <label className="text-xs text-muted">Language</label>
            <select
              value={codeLang}
              onChange={(e) => setCodeLang(e.target.value)}
              className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
            >
              <option value="gdscript">GDScript</option>
              <option value="csharp">C#</option>
              <option value="cpp">C++</option>
              <option value="rust">Rust</option>
              <option value="c">C</option>
              <option value="typescript">TypeScript</option>
              <option value="lua">Lua</option>
            </select>
          </div>
          <div>
            <label className="text-xs text-muted">Gameplay Style</label>
            <select
              value={style}
              onChange={(e) => setStyle(e.target.value)}
              className="w-full rounded border border-border bg-surface px-2 py-1 text-sm"
            >
              <option value="platformer_2d">2D Platformer</option>
              <option value="top_down_2d">2D Top-Down</option>
              <option value="third_person_3d">3D Third-Person</option>
              <option value="first_person_3d">3D First-Person</option>
            </select>
          </div>
          <Button onClick={generateCode} disabled={!selectedEntity || generating}>
            {generating ? <Loader2 className="mr-1 inline h-4 w-4 animate-spin" /> : null}
            Generate Code
          </Button>
          <Button variant="ghost" onClick={copyCode} disabled={!generatedCode}>
            <Copy className="mr-1 inline h-4 w-4" /> Copy
          </Button>
        </div>
        {generatedCode ? (
          <pre className="mt-2 max-h-72 overflow-auto rounded border border-border bg-background p-3 text-xs text-text">
            <code>{generatedCode}</code>
          </pre>
        ) : (
          <p className="mt-2 text-xs text-muted">
            Pick a language and gameplay style, then Generate to write the character controller for the selected entity against this engine.
            {generatedName ? <span className="text-accent-2"> File: {generatedName}</span> : null}
          </p>
        )}
      </Panel>
    </div>
  );
}