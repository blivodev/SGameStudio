import { useEffect, useRef } from 'react';
import { parseEntityRef } from '../../lib/parseResult';
import { dispatchCommand } from '../../services/commandBridge';
import { useStudioStore } from '../../store/studioStore';
import { Button, Panel } from '../../components/ui/primitives';

export default function Studio2D() {
  const { entities, addEntity, setLastError } = useStudioStore();
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const dispatchShape = async (shapeType: "rectangle" | "circle" | "star") => {
    const n = entities.length + 1;
    try {
      const res = await dispatchCommand("create_2d_shape", {
        shapeType,
        name: `shape2d_${n}`,
        x: 120,
        y: 120,
        width: 140,
        height: 140,
        fillColor: "#7c3aed",
        strokeColor: "#f0f6fc",
        strokeWidth: 2
      });
      const ref = parseEntityRef(res.result);
      if (ref) addEntity({ ...ref, name: `shape2d_${n}`, label: shapeType });
    } catch (e) {
      setLastError(String(e));
    }
  };

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const parent = canvas.parentElement;
    if (!parent) return;
    
    const draw = () => {
      const w = parent.clientWidth, h = parent.clientHeight;
      canvas.width = w;
      canvas.height = h;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      
      ctx.fillStyle = "#0d1117";
      ctx.fillRect(0, 0, w, h);
      ctx.strokeStyle = "rgba(124,58,237,0.15)";
      ctx.lineWidth = 1;
      
      for (let x = 0; x < w; x += 20) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, h);
        ctx.stroke();
      }
      
      for (let y = 0; y < h; y += 20) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(w, y);
        ctx.stroke();
      }
      
      const shapes = entities.filter((e) => e.kind === "shape2d");
      const colors = ["#7c3aed", "#00f0ff", "#ff0055", "#fbbf24"];
      
      shapes.forEach((_s, i) => {
        ctx.fillStyle = colors[i % colors.length];
        ctx.globalAlpha = 0.8;
        ctx.fillRect(40 + (i % 4) * 160, 40 + Math.floor(i / 4) * 160, 140, 140);
      });
      
      ctx.globalAlpha = 1;
    };
    
    const ro = new ResizeObserver(draw);
    ro.observe(parent);
    draw();
    return () => ro.disconnect();
  }, [entities]);

  return (
    <div className="flex h-full min-h-0">
      <div className="w-44 shrink-0 border-r border-border p-2">
        <Panel title="Tools">
          <div className="flex flex-col gap-2">
            <Button onClick={() => dispatchShape("rectangle")}>Add Rectangle</Button>
            <Button onClick={() => dispatchShape("circle")}>Add Circle</Button>
            <Button onClick={() => dispatchShape("star")}>Add Star</Button>
          </div>
        </Panel>
      </div>
      
      <div className="flex-1 min-w-0 p-2">
        <Panel title="Canvas">
          <canvas ref={canvasRef} className="h-full w-full" />
        </Panel>
      </div>
      
      <div className="w-52 shrink-0 border-l border-border p-2">
        <Panel title="Layers">
          <div className="flex flex-col gap-1">
            {entities
              .filter(e => e.kind === "shape2d")
              .concat(entities.filter(e => e.kind !== "shape2d"))
              .map((e) => (
                <div key={e.id} className="flex items-center justify-between rounded px-2 py-1 text-xs">
                  <span>{e.name}</span>
                  <span className="rounded bg-surfaceAlt px-1">{e.kind}</span>
                </div>
              ))
            }
          </div>
        </Panel>
      </div>
    </div>
  );
}