import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Play, Pause, MousePointer2, Square, Circle, Star as StarIcon, Trash2, Pencil, Eraser, Undo2, Bone } from "lucide-react";
import { Button, Panel } from "../../components/ui/primitives";
import { dispatchCommand } from "../../services/commandBridge";
import { lerpPosition, upsertKeyframe, EASING_IDS, EASING_LABELS, type EasingId } from "../../lib/anim";
import { useStudioStore } from "../../store/studioStore";

type Tool = "select" | "rect" | "ellipse" | "star" | "brush" | "eraser" | "bone";
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

interface Stroke {
  id: string;
  points: { x: number; y: number }[];
  color: string;
  size: number;
  eraser: boolean;
}

interface BoneJoint {
  x: number;
  y: number;
  parent: number | null;
}

interface Keyframe {
  time: number;
  x: number;
  y: number;
}

const TOOL_COLORS: Record<Tool, string> = {
  select: "#22d3ee",
  rect: "#7c3aed",
  ellipse: "#22d3ee",
  star: "#f43f5e",
  brush: "#34d399",
  eraser: "#ef4444",
  bone: "#fbbf24",
};
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
  const [ease, setEase] = useState<EasingId>("linear");
  const [onion, setOnion] = useState(false);
  const [drag, setDrag] = useState<{ id: string; startX: number; startY: number; origX: number; origY: number } | null>(null);
  const [draw, setDraw] = useState<{ kind: ShapeKind; startX: number; startY: number } | null>(null);

  const [strokes, setStrokes] = useState<Stroke[]>([]);
  const [brushColor, setBrushColor] = useState(PALETTE[4]);
  const [brushSize, setBrushSize] = useState(6);
  const [painting, setPainting] = useState<{ points: { x: number; y: number }[]; eraser: boolean } | null>(null);

  const [armature, setArmature] = useState<BoneJoint[]>([]);
  const [rigJoint, setRigJoint] = useState<number | null>(null);
  const [rigDrag, setRigDrag] = useState<number | null>(null);

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
    return lerpPosition(keyframes[selectedId] ?? [], t, ease);
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

    // Freehand paint strokes (eraser strokes punch through to the background).
    const renderStroke = (st: Stroke) => {
      if (st.points.length === 0) return;
      ctx.beginPath();
      ctx.moveTo(st.points[0].x, st.points[0].y);
      for (let i = 1; i < st.points.length; i++) ctx.lineTo(st.points[i].x, st.points[i].y);
      ctx.strokeStyle = st.eraser ? "#0d1117" : st.color;
      ctx.lineWidth = st.size;
      ctx.lineCap = "round";
      ctx.lineJoin = "round";
      ctx.stroke();
    };
    strokes.forEach(renderStroke);
    if (painting && painting.points.length > 0) {
      const live: Stroke = { id: "live", points: painting.points, color: painting.eraser ? "#0d1117" : brushColor, size: brushSize, eraser: painting.eraser };
      renderStroke(live);
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
      if (isSelected && onion) {
        const sks = selectedId ? keyframes[selectedId] ?? [] : [];
        sks.forEach((k) => {
          ctx.beginPath();
          ctx.arc(k.x, k.y, 4, 0, Math.PI * 2);
          ctx.fillStyle = "rgba(255,255,255,0.5)";
          ctx.fill();
          ctx.strokeStyle = "rgba(34,211,238,0.6)";
          ctx.stroke();
        });
      }
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

    // Skeleton (rig): bones as lines, joints as circles.
    armature.forEach((j) => {
      if (j.parent != null) {
        const p = armature[j.parent];
        if (p) {
          ctx.beginPath();
          ctx.moveTo(p.x, p.y);
          ctx.lineTo(j.x, j.y);
          ctx.strokeStyle = "#fbbf24";
          ctx.lineWidth = 3;
          ctx.stroke();
        }
      }
    });
    armature.forEach((j, i) => {
      ctx.beginPath();
      ctx.arc(j.x, j.y, i === rigJoint ? 7 : 5, 0, Math.PI * 2);
      ctx.fillStyle = i === rigJoint ? "#ffffff" : "#fbbf24";
      ctx.strokeStyle = "#0d1117";
      ctx.lineWidth = 2;
      ctx.fill();
      ctx.stroke();
    });
  }, [shapes, selectedId, playing, frame, drag, keyframes, selected, ease, onion, strokes, painting, brushColor, brushSize, armature, rigJoint]);

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

    if (tool === "bone") {
      setSelectedId(null);
      setArmature((prev) => [...prev, { x: px, y: py, parent: prev.length === 0 ? null : prev.length - 1 }]);
      setRigJoint(armature.length);
      return;
    }
    if (tool === "select") {
      // Joints take priority over shapes: click a joint to select/pose it.
      let j = -1;
      let best = 12;
      armature.forEach((pt, i) => {
        const d = Math.hypot(px - pt.x, py - pt.y);
        if (d < best) { best = d; j = i; }
      });
      if (j >= 0) {
        setRigJoint(j);
        setRigDrag(j);
        return;
      }
      const hit = shapeAtPoint(shapes, px, py);
      setSelectedId(hit ? hit.id : null);
      if (hit) setDrag({ id: hit.id, startX: px, startY: py, origX: hit.x, origY: hit.y });
    } else if (tool === "brush" || tool === "eraser") {
      setSelectedId(null);
      setPainting({ points: [{ x: px, y: py }], eraser: tool === "eraser" });
    } else {
      const kind: ShapeKind = tool === "rect" ? "rect" : tool === "ellipse" ? "ellipse" : "star";
      setDraw({ kind, startX: px, startY: py });
      setSelectedId(null);
    }
  };

  const onPointerMove = (e: React.PointerEvent<HTMLCanvasElement>) => {
    if (painting) {
      const rect = canvasRef.current!.getBoundingClientRect();
      const px = e.clientX - rect.left;
      const py = e.clientY - rect.top;
      setPainting((prev) => (prev ? { ...prev, points: [...prev.points, { x: px, y: py }] } : prev));
      return;
    }
    if (rigDrag != null) {
      const rect = canvasRef.current!.getBoundingClientRect();
      const px = e.clientX - rect.left;
      const py = e.clientY - rect.top;
      setArmature((prev) => prev.map((j, i) => (i === rigDrag ? { ...j, x: px, y: py } : j)));
      return;
    }
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
    if (painting && painting.points.length > 0) {
      setStrokes((prev) => [
        ...prev,
        { id: crypto.randomUUID(), points: painting.points, color: brushColor, size: brushSize, eraser: painting.eraser },
      ]);
      setPainting(null);
      setDrag(null);
      return;
    }
    setPainting(null);
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
    setRigDrag(null);
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

  const removeJoint = (idx: number) => {
    setArmature((prev) => {
      const oldParent = prev[idx]?.parent ?? null;
      const next = prev.filter((_, i) => i !== idx);
      return next.map((j) => {
        let par = j.parent;
        if (par != null) {
          if (par === idx) par = oldParent;
          else if (par > idx) par = par - 1;
        }
        return { ...j, parent: par };
      });
    });
    if (rigJoint === idx) setRigJoint(null);
  };

  const ratio = useMemo(() => frame / Math.max(1, MAX_FRAMES), [frame]);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex min-h-0 flex-1">
        <div className="w-44 shrink-0 border-r border-border p-2 space-y-2">
          <Panel title="Tools">
            <div className="flex flex-col gap-1.5">
              {(["select", "rect", "ellipse", "star", "brush", "eraser", "bone"] as Tool[]).map((t) => (
                <Button key={t} size="sm" variant={tool === t ? "accent" : "ghost"} className="w-full justify-start gap-2" onClick={() => setTool(t)}>
                  {t === "select" ? <MousePointer2 className="h-4 w-4" /> : t === "rect" ? <Square className="h-4 w-4" /> : t === "ellipse" ? <Circle className="h-4 w-4" /> : t === "star" ? <StarIcon className="h-4 w-4" /> : t === "brush" ? <Pencil className="h-4 w-4" /> : t === "eraser" ? <Eraser className="h-4 w-4" /> : <Bone className="h-4 w-4" />}
                  {t[0].toUpperCase() + t.slice(1)}
                </Button>
              ))}
            </div>

            {(tool === "brush" || tool === "eraser") && (
              <div className="mt-2 space-y-2 rounded border border-border p-2">
                <div>
                  <label className="text-[11px] text-muted">Brush size · {brushSize}px</label>
                  <input type="range" min={2} max={40} value={brushSize} onChange={(e) => setBrushSize(Number(e.target.value))} className="w-full" aria-label="Brush size" />
                </div>
                {tool === "brush" && (
                  <div>
                    <label className="text-[11px] text-muted">Color</label>
                    <div className="mt-1 flex flex-wrap gap-1">
                      {PALETTE.map((c) => (
                        <button key={c} type="button" onClick={() => setBrushColor(c)} aria-label={`Color ${c}`} className={`h-5 w-5 rounded border ${brushColor === c ? "border-white" : "border-transparent"}`} style={{ background: c }} />
                      ))}
                      <input type="color" value={brushColor} onChange={(e) => setBrushColor(e.target.value)} className="h-5 w-5 cursor-pointer rounded border border-border bg-transparent p-0" aria-label="Custom color" />
                    </div>
                  </div>
                )}
              </div>
            )}

            <div className="mt-2 flex flex-col gap-1.5">
              <Button size="sm" variant="ghost" className="w-full justify-start gap-2" onClick={() => setStrokes((s) => s.slice(0, -1))} disabled={strokes.length === 0}>
                <Undo2 className="h-4 w-4" /> Undo stroke
              </Button>
              <Button size="sm" variant="ghost" className="w-full justify-start gap-2 text-red-400 hover:text-red-300" onClick={() => setStrokes([])} disabled={strokes.length === 0}>
                <Trash2 className="h-4 w-4" /> Clear paint
              </Button>
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
                <div className="flex items-center justify-between gap-2">
                  <label className="text-[11px] text-muted">Ease</label>
                  <select
                    value={ease}
                    onChange={(e) => setEase(e.target.value as EasingId)}
                    className="rounded border border-border bg-background px-1.5 py-0.5 text-[11px]"
                    aria-label="Keyframe easing"
                  >
                    {EASING_IDS.map((id) => (
                      <option key={id} value={id}>{EASING_LABELS[id]}</option>
                    ))}
                  </select>
                </div>
                <button
                  type="button"
                  onClick={() => setOnion((o) => !o)}
                  className={`flex w-full items-center justify-center gap-1 rounded border px-2 py-1 text-[11px] ${onion ? "border-accent bg-accent/20" : "border-border"}`}
                >
                  <span className={`h-2 w-2 rounded-full ${onion ? "bg-accent-2" : "bg-muted/40"}`} />
                  Onion skin {onion ? "on" : "off"}
                </button>
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

          <Panel title="Rig (bones)">
            <div className="space-y-2 text-xs">
              {tool === "bone" ? (
                <p className="text-muted">Click the canvas to place joints — each new joint chains onto the last to form a skeleton.</p>
              ) : (
                <p className="text-muted">Use the <b>Bone</b> tool to add joints, then the <b>Select</b> tool to click &amp; drag a joint to pose it.</p>
              )}
              <div className="flex flex-col gap-1">
                {armature.length === 0 && <span className="text-muted">No bones yet.</span>}
                {armature.map((j, i) => (
                  <div key={i} className={`flex items-center justify-between rounded border px-1.5 py-0.5 ${rigJoint === i ? "border-accent" : "border-border"}`}>
                    <span className="font-mono">j{i} ({Math.round(j.x)},{Math.round(j.y)})</span>
                    <button type="button" onClick={() => removeJoint(i)} className="text-red-400 hover:text-red-300" aria-label={`Delete joint ${i}`}>✕</button>
                  </div>
                ))}
              </div>
              <Button size="sm" variant="ghost" className="w-full justify-start gap-2 text-red-400 hover:text-red-300" onClick={() => { setArmature([]); setRigJoint(null); }} disabled={armature.length === 0}>
                <Trash2 className="h-4 w-4" /> Clear rig
              </Button>
            </div>
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
