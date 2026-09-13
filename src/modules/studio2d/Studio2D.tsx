import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Play, Pause, MousePointer2, Square, Circle, Star as StarIcon, Trash2 } from "lucide-react";
import { Button, Panel } from "../../components/ui/primitives";
import { dispatchCommand } from "../../services/commandBridge";
import { lerpPosition, upsertKeyframe } from "../../lib/anim";
import { useStudioStore } from "../../store/studioStore";

type Tool = "select" | "rect" | "ellipse" | "star";
type ShapeKind = "rect" | "ellipse" | "star";

interface LocalShape {
  id: string;
  kind: ShapeKind;
  name: string;
  x: number;
  y: number;
  w: number;
  h: number;
  fill: string;
}

interface Keyframe {
  time: number;
  x: number;
  y: number;
}

const TOOL_COLORS: Record<Tool, string> = { select: "#22d3ee", rect: "#7c3aed", ellipse: "#22d3ee", star: "#f43f5e" };
const PALETTE = ["#7c3aed", "#22d3ee", "#f43f5e", "#fbbf24", "#34d399", "#94a3b8"];
const MAX_FRAMES = 48;

function shapeAtPoint(shapes: LocalShape[], px: number, py: number): LocalShape | null {
  for (let i = shapes.length - 1; i >= 0; i--) {
    const s = shapes[i];
    const inside =
      s.kind === "ellipse"
        ? ((px - s.x) / (s.w / 2)) ** 2 + ((py - s.y) / (s.h / 2)) ** 2 <= 1
        : px >= s.x - s.w / 2 && px <= s.x + s.w / 2 && py >= s.y - s.h / 2 && py <= s.y + s.h / 2;
    if (inside) return s;
  }
  return null;
}

export default function Studio2D() {
  const { addEntity, setLastError } = useStudioStore();
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const [tool, setTool] = useState<Tool>("select");
  const [shapes, setShapes] = useState<LocalShape[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [keyframes, setKeyframes] = useState<Record<string, Keyframe[]>>({});

  const [frame, setFrame] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [drag, setDrag] = useState<{ id: string; startX: number; startY: number; origX: number; origY: number } | null>(null);
  const [draw, setDraw] = useState<{ kind: ShapeKind; startX: number; startY: number } | null>(null);

  const selected = shapes.find((s) => s.id === selectedId) ?? null;

  const addLocalShape = (kind: ShapeKind, x: number, y: number, w: number, h: number) => {
    const id = crypto.randomUUID();
    const n = shapes.length + 1;
    const shape: LocalShape = { id, kind, name: `shape2d_${n}`, x, y, w, h, fill: PALETTE[n % PALETTE.length] };
    setShapes((prev) => [...prev, shape]);
    setSelectedId(id);
    // Persist to the backend so it also appears in Layers / the scene store.
    dispatchCommand("create_2d_shape", {
      shapeType: kind,
      name: shape.name,
      x,
      y,
      width: w,
      height: h,
      fillColor: shape.fill,
      strokeColor: "#f0f6fc",
      strokeWidth: 2,
    })
      .then((res) => {
        const ref = (res.result as { entityId?: string })?.entityId;
        if (ref) addEntity({ id: ref, kind: "shape2d", name: shape.name, label: kind });
      })
      .catch((e) => setLastError(String(e)));
  };

  const interpolate = (t: number): { x: number; y: number } | null => {
    if (!selectedId) return null;
    return lerpPosition(keyframes[selectedId] ?? [], t);
  };

  // Playback loop
  useEffect(() => {
    if (!playing) return;
    const id = window.setInterval(() => {
      setFrame((f) => (f >= MAX_FRAMES ? 0 : f + 1));
    }, 80);
    return () => window.clearInterval(id);
  }, [playing]);

  // Keep the drawing loop simple: redraw each render.
  const redraw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const parent = canvas.parentElement;
    if (!parent) return;
    const w = parent.clientWidth;
    const h = parent.clientHeight;
    canvas.width = w;
    canvas.height = h;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.fillStyle = "#0d1117";
    ctx.fillRect(0, 0, w, h);

    // grid
    ctx.strokeStyle = "rgba(124,58,237,0.12)";
    ctx.lineWidth = 1;
    for (let x = 20; x < w; x += 20) {
      ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, h); ctx.stroke();
    }
    for (let y = 20; y < h; y += 20) {
      ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
    }

    const drawShape = (s: LocalShape, ex: number, ey: number, isSel: boolean) => {
      ctx.beginPath();
      if (s.kind === "ellipse") ctx.ellipse(ex, ey, s.w / 2, s.h / 2, 0, 0, Math.PI * 2);
      else if (s.kind === "star") {
        const spikes = 5;
        const outer = s.w / 2;
        const inner = s.w / 4;
        for (let i = 0; i < spikes * 2; i++) {
          const r = i % 2 === 0 ? outer : inner;
          const a = (i * Math.PI) / spikes - Math.PI / 2;
          const px = ex + r * Math.cos(a);
          const py = ey + r * Math.sin(a);
          if (i === 0) ctx.moveTo(px, py);
          else ctx.lineTo(px, py);
        }
        ctx.closePath();
      } else {
        ctx.rect(ex - s.w / 2, ey - s.h / 2, s.w, s.h);
      }
      ctx.fillStyle = s.fill;
      ctx.globalAlpha = 0.85;
      ctx.fill();
      ctx.globalAlpha = 1;
      ctx.strokeStyle = isSel ? "#ffffff" : "rgba(240,246,252,0.25)";
      ctx.lineWidth = isSel ? 2 : 1;
      ctx.stroke();
    };

    shapes.forEach((s) => {
      const isSelected = s.id === selectedId;
      let pos: { x: number; y: number } | null = null;
      if (isSelected && playing) pos = interpolate(frame);
      if (isSelected && drag && drag.id === s.id && !playing) pos = { x: s.x, y: s.y };
      const ex = pos ? pos.x : s.x;
      const ey = pos ? pos.y : s.y;
      drawShape(s, ex, ey, isSelected);
    });

    // selection box
    if (selected && !playing) {
      ctx.strokeStyle = "#22d3ee";
      ctx.setLineDash([4, 4]);
      ctx.strokeRect(selected.x - selected.w / 2 - 5, selected.y - selected.h / 2 - 5, selected.w + 10, selected.h + 10);
      ctx.setLineDash([]);
    }
  }, [shapes, selectedId, playing, frame, drag, keyframes, selected]);

  useEffect(() => {
    redraw();
    const parent = canvasRef.current?.parentElement;
    if (!parent) return;
    const ro = new ResizeObserver(redraw);
    ro.observe(parent);
    return () => ro.disconnect();
  }, [redraw]);

  const onPointerDown = (e: React.PointerEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current!;
    const rect = canvas.getBoundingClientRect();
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;

    if (tool === "select") {
      const hit = shapeAtPoint(shapes, px, py);
      setSelectedId(hit ? hit.id : null);
      if (hit) setDrag({ id: hit.id, startX: px, startY: py, origX: hit.x, origY: hit.y });
    } else {
      const kind: ShapeKind = tool === "rect" ? "rect" : tool === "ellipse" ? "ellipse" : "star";
      setDraw({ kind, startX: px, startY: py });
      setSelectedId(null);
    }
  };

  const onPointerMove = (e: React.PointerEvent<HTMLCanvasElement>) => {
    if (drag) {
      const rect = canvasRef.current!.getBoundingClientRect();
      const px = e.clientX - rect.left;
      const py = e.clientY - rect.top;
      setShapes((prev) =>
        prev.map((s) => (s.id === drag.id ? { ...s, x: drag.origX + (px - drag.startX), y: drag.origY + (py - drag.startY) } : s)),
      );
    }
  };

  const onPointerUp = () => {
    if (draw) {
      const rect = canvasRef.current!.getBoundingClientRect();
      const px = draw.startX;
      const py = draw.startY;
      const w = Math.max(24, 70);
      const h = Math.max(24, 70);
      let nx = px + w / 2;
      let ny = py + h / 2;
      const shapeBelowTop = nx - w / 2 > rect.width || ny - h / 2 > rect.height;
      if (shapeBelowTop) {
        nx = Math.min(nx, rect.width - w / 2);
        ny = Math.min(ny, rect.height - h / 2);
      }
      addLocalShape(draw.kind, nx, ny, w, h);
      setDraw(null);
    }
    setDrag(null);
  };

  const addKeyframe = () => {
    if (!selected || !selectedId) return;
    const sid = selectedId;
    setKeyframes((prev) => ({
      ...prev,
      [sid]: upsertKeyframe(prev[sid] ?? [], { time: frame, x: selected.x, y: selected.y }),
    }));
  };

  const deleteKeyframes = (id: string) => {
    setKeyframes((prev) => {
      const cp = { ...prev };
      delete cp[id];
      return cp;
    });
  };

  const ratio = useMemo(() => frame / Math.max(1, MAX_FRAMES), [frame]);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex min-h-0 flex-1">
        <div className="w-44 shrink-0 border-r border-border p-2 space-y-2">
          <Panel title="Tools">
            <div className="flex flex-col gap-1.5">
              {(["select", "rect", "ellipse", "star"] as Tool[]).map((t) => (
                <Button key={t} size="sm" variant={tool === t ? "accent" : "ghost"} className="w-full justify-start gap-2" onClick={() => setTool(t)}>
                  {t === "select" ? <MousePointer2 className="h-4 w-4" /> : t === "rect" ? <Square className="h-4 w-4" /> : t === "ellipse" ? <Circle className="h-4 w-4" /> : <StarIcon className="h-4 w-4" />}
                  {t[0].toUpperCase() + t.slice(1)}
                </Button>
              ))}
            </div>
            {selected && (
              <Button size="sm" variant="ghost" className="w-full justify-start gap-2 text-red-400 hover:text-red-300" onClick={() => { deleteKeyframes(selected.id); setShapes((prev) => prev.filter((s) => s.id !== selected.id)); setSelectedId(null); }}>
                <Trash2 className="h-4 w-4" /> Delete
              </Button>
            )}
          </Panel>
          <Panel title={selected ? "Keyframes" : "Keyframes"}>
            {selected ? (
              <div className="space-y-2 text-xs">
                <p className="text-muted">Frame {frame} — record position keyframes.</p>
                <Button size="sm" className="w-full" onClick={addKeyframe}>
                  Record Keyframe @ {frame}
                </Button>
                <div className="flex flex-wrap gap-1">
                  {(selectedId ? keyframes[selectedId] ?? [] : []).map((k) => (
                    <span key={k.time} className="rounded bg-surfaceAlt px-1.5 py-0.5 font-mono text-[10px]">
                      f{k.time}
                    </span>
                  ))}
                </div>
                {((selectedId ? keyframes[selectedId] ?? [] : []).length === 0) && <p className="text-[11px] text-muted">No keyframes yet for this shape.</p>}
              </div>
            ) : (
              <p className="text-xs text-muted">Select a shape to animate its position over time.</p>
            )}
          </Panel>
        </div>

        <div className="min-w-0 flex-1 p-2">
          <Panel title="Canvas — drag to draw / move">
            <canvas
              ref={canvasRef}
              className="h-full w-full touch-none"
              onPointerDown={onPointerDown}
              onPointerMove={onPointerMove}
              onPointerUp={onPointerUp}
              onPointerLeave={onPointerUp}
            />
          </Panel>
        </div>

        <div className="w-52 shrink-0 border-l border-border p-2">
          <Panel title="Layers">
            <div className="flex flex-col gap-1 text-xs">
              {shapes.length === 0 && <span className="text-muted">Draw a shape to begin.</span>}
              {shapes.map((s) => (
                <button
                  key={s.id}
                  type="button"
                  onClick={() => setSelectedId(s.id)}
                  className={`flex items-center justify-between rounded border px-2 py-1 text-left ${s.id === selectedId ? "border-accent" : "border-border"}`}
                >
                  <span className="truncate">{s.name}</span>
                  <span className="flex items-center gap-1 text-muted">
                    {(keyframes[s.id] ?? []).length > 0 && <span className="text-accent-2">◆{keyframes[s.id].length}</span>}
                    {s.kind}
                  </span>
                </button>
              ))}
            </div>
          </Panel>
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-3 border-t border-border px-3 py-2">
        <Button size="sm" variant={playing ? "default" : "accent"} onClick={() => setPlaying((p) => !p)} aria-label={playing ? "Pause" : "Play"}>
          {playing ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
        </Button>
        <span className="text-xs text-muted">Frame {frame}/{MAX_FRAMES}</span>
        <div className="relative h-5 flex-1 rounded bg-surfaceAlt">
          <div className="absolute inset-y-0 left-0 rounded bg-accent/40" style={{ width: `${ratio * 100}%` }} />
          <input
            type="range"
            min={0}
            max={MAX_FRAMES}
            value={frame}
            onChange={(e) => { setPlaying(false); setFrame(Number(e.target.value)); }}
            className="absolute inset-0 h-full w-full cursor-pointer opacity-0"
            aria-label="Timeline scrubber"
          />
        </div>
        <span className="text-[11px] text-muted">{TOOL_COLORS[tool] ? "●" : ""} {selected ? `Animating ${selected.name}` : "No selection"}</span>
      </div>
    </div>
  );
}
