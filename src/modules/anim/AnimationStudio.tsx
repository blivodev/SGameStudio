import { useCallback, useEffect, useRef, useState } from "react";
import { Play, Pause, Plus } from "lucide-react";
import { Button, Panel } from "../../components/ui/primitives";
import { useStudioStore } from "../../store/studioStore";
import { lerpPosition, upsertKeyframe, EASING_IDS, EASING_LABELS, type EasingId, type Keyframe } from "../../lib/anim";

const MAX_FRAMES = 60;

interface Subject {
  name: string;
  kind: string;
  color: string;
}

const COLORS = ["#7c3aed", "#22d3ee", "#f43f5e", "#fbbf24", "#34d399"];

export default function AnimationStudio() {
  const { entities } = useStudioStore();
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const all = entities.map((e, i) => ({ name: e.name, kind: e.kind, color: COLORS[i % COLORS.length] }));
  const [subjects, setSubjects] = useState<Subject[]>(all);
  const [name, setName] = useState<string>("");
  const [keyframes, setKeyframes] = useState<Record<string, Keyframe[]>>({});
  const [frame, setFrame] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [ease, setEase] = useState<EasingId>("linear");
  const [marker, setMarker] = useState<{ x: number; y: number } | null>(null);

  // sync the subject list from the scene store
  useEffect(() => {
    setSubjects(all);
    if (!name && all.length > 0) setName(all[0].name);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [entities]);

  const subject = subjects.find((s) => s.name === name) ?? null;

  useEffect(() => {
    if (!playing) return;
    const id = window.setInterval(() => setFrame((f) => (f >= MAX_FRAMES ? 0 : f + 1)), 90);
    return () => window.clearInterval(id);
  }, [playing]);

  const pos = name ? lerpPosition(keyframes[name] ?? [], frame, ease) : null;
  const markerPos = pos ?? marker;

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
    ctx.strokeStyle = "rgba(124,58,237,0.12)";
    ctx.lineWidth = 1;
    for (let x = 20; x < w; x += 20) { ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, h); ctx.stroke(); }
    for (let y = 20; y < h; y += 20) { ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke(); }

    // keyframe ghosts (onion-skin style)
    if (name) {
      (keyframes[name] ?? []).forEach((k) => {
        ctx.beginPath();
        ctx.arc(k.x, k.y, 4, 0, Math.PI * 2);
        ctx.fillStyle = "rgba(255,255,255,0.5)";
        ctx.fill();
        ctx.strokeStyle = "rgba(34,211,238,0.6)";
        ctx.stroke();
      });
    }

    if (markerPos && subject) {
      ctx.beginPath();
      ctx.arc(markerPos.x, markerPos.y, 14, 0, Math.PI * 2);
      ctx.fillStyle = subject.color;
      ctx.globalAlpha = 0.85;
      ctx.fill();
      ctx.globalAlpha = 1;
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.fillStyle = "#ffffff";
      ctx.font = "10px monospace";
      ctx.textAlign = "center";
      ctx.fillText(subject.name, markerPos.x, markerPos.y - 22);
    }
  }, [markerPos, name, keyframes, subject]);

  useEffect(() => {
    redraw();
    const parent = canvasRef.current?.parentElement;
    if (!parent) return;
    const ro = new ResizeObserver(redraw);
    ro.observe(parent);
    return () => ro.disconnect();
  }, [redraw]);

  const onCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!name || playing) return;
    const rect = canvasRef.current!.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    setMarker({ x, y });
    setKeyframes((prev) => ({ ...prev, [name]: upsertKeyframe(prev[name] ?? [], { time: frame, x, y }) }));
  };

  const addKeyframeHere = () => {
    if (!name || !marker) return;
    setKeyframes((prev) => ({ ...prev, [name]: upsertKeyframe(prev[name] ?? [], { time: frame, x: marker.x, y: marker.y }) }));
  };

  const addSubject = () => {
    const n = `anim_${subjects.length + 1}`;
    const subj: Subject = { name: n, kind: "custom", color: COLORS[subjects.length % COLORS.length] };
    setKeyframes((prev) => ({ ...prev, [n]: [{ time: 0, x: 80, y: 80 }, { time: MAX_FRAMES, x: 360, y: 240 }] }));
    setSubjects((prev) => [...prev, subj]);
    setName(n);
  };

  const ratio = frame / MAX_FRAMES;

  return (
    <div className="flex h-full min-h-0">
      <div className="flex min-h-0 flex-1 flex-col">
        <div className="flex gap-2 border-b border-border p-2">
          <Button size="sm" variant={playing ? "default" : "accent"} onClick={() => setPlaying((p) => !p)}>
            {playing ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />} {playing ? "Pause" : "Play"}
          </Button>
          <Button size="sm" variant="ghost" onClick={addSubject}>
            <Plus className="h-4 w-4" /> New Subject
          </Button>
          <span className="self-center text-xs text-muted">Click the canvas to place a keyframe for the selected subject.</span>
        </div>
        <div className="relative min-h-0 flex-1 p-2">
          <Panel title="Animation Canvas (2D / 3D motion preview)">
            <canvas ref={canvasRef} className="h-full w-full cursor-crosshair" onClick={onCanvasClick} />
          </Panel>
        </div>
      </div>

      <div className="w-64 shrink-0 border-l border-border p-2">
        <Panel title="Subjects & Keyframes">
          <div className="flex flex-col gap-3">
            <div>
              <label className="text-[11px] text-muted">Subject</label>
              <select value={name} onChange={(e) => setName(e.target.value)} className="w-full rounded border border-border bg-background px-2 py-1 text-sm">
                {subjects.length === 0 && <option value="">No subjects</option>}
                {subjects.map((s) => (
                  <option key={s.name} value={s.name}>{s.name} ({s.kind})</option>
                ))}
              </select>
            </div>

            <div className="flex items-center justify-between gap-2">
              <label className="text-[11px] text-muted">Ease</label>
              <select value={ease} onChange={(e) => setEase(e.target.value as EasingId)} className="rounded border border-border bg-background px-1.5 py-0.5 text-[11px]" aria-label="Easing">
                {EASING_IDS.map((id) => (
                  <option key={id} value={id}>{EASING_LABELS[id]}</option>
                ))}
              </select>
            </div>

            <Button size="sm" disabled={!marker} onClick={addKeyframeHere}>
              Keyframe @ Frame {frame}
            </Button>

            <div className="flex flex-wrap gap-1">
              {(name ? keyframes[name] ?? [] : []).map((k) => (
                <span key={k.time} className="rounded bg-surfaceAlt px-1.5 py-0.5 font-mono text-[10px]">f{k.time}</span>
              ))}
            </div>
            {(name ? (keyframes[name] ?? []).length : 0) === 0 && (
              <p className="text-[11px] text-muted">No keyframes. Click the canvas, then Record.</p>
            )}

            <div className="border-t border-border pt-2">
              <div className="relative h-5 rounded bg-surfaceAlt">
                <div className="absolute inset-y-0 left-0 rounded bg-accent/40" style={{ width: `${ratio * 100}%` }} />
                <input
                  type="range"
                  min={0}
                  max={MAX_FRAMES}
                  value={frame}
                  onChange={(e) => { setPlaying(false); setFrame(Number(e.target.value)); }}
                  className="absolute inset-0 h-full w-full cursor-pointer opacity-0"
                  aria-label="Timeline"
                />
              </div>
              <div className="mt-1 text-center text-[11px] text-muted">Frame {frame}/{MAX_FRAMES}</div>
            </div>
          </div>
        </Panel>
      </div>

      <div className="w-52 shrink-0 border-l border-border p-2">
        <Panel title="Scene Subjects">
          <div className="flex flex-col gap-1 text-xs">
            {subjects.length === 0 && <span className="text-muted">Add a subject or create 2D/3D assets first.</span>}
            {subjects.map((s) => (
              <button key={s.name} type="button" onClick={() => setName(s.name)} className={`flex items-center gap-2 rounded border px-2 py-1 text-left ${s.name === name ? "border-accent" : "border-border"}`}>
                <span className="h-2 w-2 rounded-full" style={{ background: s.color }} />
                <span className="truncate">{s.name}</span>
              </button>
            ))}
          </div>
        </Panel>
      </div>
    </div>
  );
}
