import { useEffect, useRef, useState } from "react";
import { Play, Pause, Film, Volume2, Sparkles } from "lucide-react";
import { Button, Panel } from "../../components/ui/primitives";

const MAX_FRAMES = 90;

type Lane = "visual" | "audio";

interface Clip {
  id: string;
  lane: Lane;
  label: string;
  start: number;
  end: number;
  active: boolean;
  color: string;
}

const INITIAL_CLIPS: Clip[] = [
  { id: "v1", lane: "visual", label: "Scene A", start: 0, end: 40, active: true, color: "#7c3aed" },
  { id: "v2", lane: "visual", label: "Scene B", start: 40, end: 80, active: true, color: "#22d3ee" },
  { id: "a1", lane: "audio", label: "Music loop", start: 0, end: 60, active: true, color: "#f43f5e" },
  { id: "a2", lane: "audio", label: "Voiceover", start: 60, end: 85, active: true, color: "#fbbf24" },
];

export default function VideoStudio() {
  const [clips, setClips] = useState<Clip[]>(INITIAL_CLIPS);
  const [frame, setFrame] = useState(0);
  const [playing, setPlaying] = useState(false);
  const seq = useRef(5);

  useEffect(() => {
    if (!playing) return;
    const id = window.setInterval(() => setFrame((f) => (f >= MAX_FRAMES ? 0 : f + 1)), 80);
    return () => window.clearInterval(id);
  }, [playing]);

  const addClip = (lane: Lane) => {
    const n = seq.current++;
    const colors: Record<Lane, string> = { visual: "#7c3aed", audio: "#f43f5e" };
    setClips((prev) => [
      ...prev,
      { id: `${lane}${n}`, lane, label: lane === "visual" ? `Shot ${n}` : `Track ${n}`, start: 20, end: 50, active: true, color: colors[lane] },
    ]);
  };

  const toggleClip = (id: string) => setClips((prev) => prev.map((c) => (c.id === id ? { ...c, active: !c.active } : c)));

  const activeVisual = clips.filter((c) => c.lane === "visual" && c.active && frame >= c.start && frame <= c.end);
  const activeAudio = clips.filter((c) => c.lane === "audio" && c.active && frame >= c.start && frame <= c.end);

  const renderLane = (lane: Lane) => (
    <div className="relative h-12 rounded border border-border bg-background/40">
      {clips
        .filter((c) => c.lane === lane)
        .map((c) => {
          const left = (c.start / MAX_FRAMES) * 100;
          const width = ((c.end - c.start) / MAX_FRAMES) * 100;
          const lit = c.active && frame >= c.start && frame <= c.end;
          return (
            <button
              key={c.id}
              type="button"
              onClick={() => toggleClip(c.id)}
              title={c.label}
              className={`absolute top-1 flex h-10 items-center justify-center overflow-hidden rounded border text-[11px] font-medium transition-opacity ${lit ? "" : "opacity-40"}`}
              style={{ left: `${left}%`, width: `${width}%`, background: c.color, borderColor: lit ? "#ffffff" : "transparent" }}
            >
              <span className="truncate px-1">{c.label}</span>
            </button>
          );
        })}
      <div className="pointer-events-none absolute inset-y-0 w-px bg-white" style={{ left: `${(frame / MAX_FRAMES) * 100}%` }} />
    </div>
  );

  const ratio = frame / MAX_FRAMES;

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex items-center gap-3 border-b border-border p-2">
        <Button size="sm" variant={playing ? "default" : "accent"} onClick={() => setPlaying((p) => !p)}>
          {playing ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />} {playing ? "Pause" : "Play"}
        </Button>
        <Button size="sm" variant="ghost" onClick={() => addClip("visual")}><Film className="h-4 w-4" /> Add Shot</Button>
        <Button size="sm" variant="ghost" onClick={() => addClip("audio")}><Volume2 className="h-4 w-4" /> Add Track</Button>
        <span className="text-xs text-muted">Frame {frame}/{MAX_FRAMES}</span>
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-[140px_1fr] gap-3 overflow-y-auto p-3">
        <Panel title="Preview">
          <div className="flex h-full flex-col items-center justify-center text-center">
            {activeVisual.length > 0 ? (
              <>
                <Sparkles className="mb-2 h-6 w-6 text-accent-2" />
                <p className="text-sm text-text">{activeVisual.map((c) => c.label).join(" → ")}</p>
                <p className="text-[11px] text-muted">Now playing on the visual track.</p>
              </>
            ) : (
              <p className="text-xs text-muted">No active shot at this frame.</p>
            )}
            <div className="mt-4 h-2 w-32 rounded-full bg-surfaceAlt">
              <div className="h-full rounded-full bg-accent/60" style={{ width: `${ratio * 100}%` }} />
            </div>
            {activeAudio.map((c) => (
              <div key={c.id} className="mt-2 flex items-center gap-1 text-[11px] text-muted">
                <Volume2 className="h-3 w-3" /> {c.label}
              </div>
            ))}
          </div>
        </Panel>

        <Panel title="Timeline — click a clip to toggle it on/off">
          <div className="flex h-full flex-col gap-3 py-1">
            <div className="flex items-center gap-2 text-[11px] text-muted"><Film className="h-3 w-3" /> Video</div>
            {renderLane("visual")}
            <div className="flex items-center gap-2 text-[11px] text-muted"><Volume2 className="h-3 w-3" /> Audio</div>
            {renderLane("audio")}
            <div className="relative mt-2 h-5 rounded bg-surfaceAlt">
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
            <p className="text-[11px] text-muted">
              Motion/video timeline preview. Clip in/out points, per-clip toggling, and transport play/scrub.
              Final render (mp4/webm export) is planned — this is the editing timeline.
            </p>
          </div>
        </Panel>
      </div>
    </div>
  );
}
