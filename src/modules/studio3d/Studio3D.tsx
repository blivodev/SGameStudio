import { useEffect, useMemo, useRef, useState } from 'react';
import * as THREE from 'three';
import { Play, Pause } from 'lucide-react';
import { parseEntityRef } from '../../lib/parseResult';
import { dispatchCommand } from '../../services/commandBridge';
import { exportMesh, downloadAssetResult, type MeshFormatId } from '../../services/export';
import { lerpPosition3, upsertKeyframe3, EASING_IDS, EASING_LABELS, type EasingId, type Keyframe3 } from '../../lib/anim';
import { useStudioStore } from '../../store/studioStore';
import { Button, Panel } from '../../components/ui/primitives';

const MAX_3D_FRAMES = 60;

export default function Studio3D() {
  const { entities, addEntity, setLastError } = useStudioStore();
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [selectedMesh, setSelectedMesh] = useState<string>("");
  const [exporting, setExporting] = useState<MeshFormatId | null>(null);
  const [exportNote, setExportNote] = useState<string | null>(null);

  const [pose, setPose] = useState<Record<string, { x: number; y: number; z: number }>>({});
  const [poseKeyframes, setPoseKeyframes] = useState<Record<string, Keyframe3[]>>({});
  const [frame3d, setFrame3d] = useState(0);
  const [playing3d, setPlaying3d] = useState(false);
  const [ease3d, setEase3d] = useState<EasingId>("linear");

  const meshes = useMemo(() => entities.filter((e) => e.kind === "mesh3d"), [entities]);

  useEffect(() => {
    if (!selectedMesh && meshes.length > 0) setSelectedMesh(meshes[0].name);
  }, [meshes, selectedMesh]);

  const doExport = async (format: MeshFormatId) => {
    if (!selectedMesh) return;
    setExporting(format);
    setExportNote(null);
    try {
      const res = await exportMesh(selectedMesh, format);
      downloadAssetResult(res);
      setExportNote(`Exported ${selectedMesh} → ${res.fileName} (${(res.sizeBytes / 1024).toFixed(1)} KB)`);
    } catch (e) {
      setLastError(String(e));
      setExportNote((e as Error).message);
    } finally {
      setExporting(null);
    }
  };

  const recordPose = () => {
    if (!selectedMesh) return;
    const p = pose[selectedMesh] ?? { x: 0, y: 0, z: 0 };
    setPoseKeyframes((prev) => ({ ...prev, [selectedMesh]: upsertKeyframe3(prev[selectedMesh] ?? [], { time: frame3d, x: p.x, y: p.y, z: p.z }) }));
  };

  const setAxis = (axis: "x" | "y" | "z", value: number) => {
    if (!selectedMesh) return;
    setPose((prev) => {
      const cur = prev[selectedMesh] ?? { x: 0, y: 0, z: 0 };
      return { ...prev, [selectedMesh]: { ...cur, [axis]: value } };
    });
  };

  const threeRef = useRef<{
    renderer: THREE.WebGLRenderer;
    scene: THREE.Scene;
    camera: THREE.PerspectiveCamera;
    meshes: THREE.Mesh[];
    raf: number;
    yaw: number;
    pitch: number;
    radius: number;
    dragging: boolean;
  } | null>(null);

  const addPrimitive = async (primitiveType: "cube" | "cylinder" | "sphere" | "torus") => {
    const n = entities.filter(e => e.kind === "mesh3d").length + 1;
    try {
      const res = await dispatchCommand("create_3d_primitive", {
        primitiveType,
        name: `mesh3d_${n}`,
        dimensions: { x: 1, y: 1, z: 1 }
      });
      const ref = parseEntityRef(res.result);
      if (ref) addEntity({ ...ref, name: `mesh3d_${n}`, label: primitiveType });
    } catch (e) {
      setLastError(String(e));
    }
  };

  useEffect(() => {
    if (!threeRef.current) {
      const scene = new THREE.Scene();
      scene.background = new THREE.Color(0x0d1117);
      
      const camera = new THREE.PerspectiveCamera(60, 1, 0.1, 100);
      
      const renderer = new THREE.WebGLRenderer({ 
        canvas: canvasRef.current!, 
        antialias: true 
      });
      
      const ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
      scene.add(ambientLight);
      
      const directionalLight = new THREE.DirectionalLight(0xffffff, 1.0);
      directionalLight.position.set(3, 5, 2);
      scene.add(directionalLight);
      
      const gridHelper = new THREE.GridHelper(10, 20, 0x30363d, 0x161b22);
      scene.add(gridHelper);
      
      const meshes: THREE.Mesh[] = [];
      
      const animate = () => {
        const { yaw, pitch, radius, scene, camera, renderer } = threeRef.current!;
        camera.position.set(
          radius * Math.cos(pitch) * Math.sin(yaw),
          radius * Math.sin(pitch),
          radius * Math.cos(pitch) * Math.cos(yaw)
        );
        camera.lookAt(0, 0, 0);
        renderer.render(scene, camera);
      };
      
      threeRef.current = {
        renderer,
        scene,
        camera,
        meshes,
        raf: 0,
        yaw: 0.8,
        pitch: 0.5,
        radius: 6,
        dragging: false
      };
      
      threeRef.current.raf = requestAnimationFrame(function renderLoop() {
        animate();
        threeRef.current!.raf = requestAnimationFrame(renderLoop);
      });
    }

    return () => {
      if (threeRef.current) {
        cancelAnimationFrame(threeRef.current.raf);
        threeRef.current.renderer.dispose();
        threeRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    const { scene } = threeRef.current!;
    threeRef.current!.meshes.forEach(mesh => {
      if (Array.isArray(mesh.material)) {
        mesh.material.forEach(m => m.dispose());
      } else {
        mesh.material.dispose();
      }
      mesh.geometry.dispose();
      scene.remove(mesh);
    });

    threeRef.current!.meshes = entities
      .filter(e => e.kind === "mesh3d")
      .map((e, i) => {
        let geometry: THREE.BufferGeometry;
        const color = 0x7c3aed;
        const material = new THREE.MeshStandardMaterial({
          color,
          roughness: 0.4,
          metalness: 0.2
        });

        switch (i % 4) {
          case 0: geometry = new THREE.BoxGeometry(1, 1, 1); break;
          case 1: geometry = new THREE.CylinderGeometry(0.5, 0.5, 1, 32); break;
          case 2: geometry = new THREE.SphereGeometry(0.5, 32, 32); break;
          case 3: geometry = new THREE.TorusGeometry(0.5, 0.2, 16, 32); break;
          default: geometry = new THREE.BoxGeometry(1, 1, 1);
        }

        const mesh = new THREE.Mesh(geometry, material);
        mesh.name = e.name;
        const gx = -2 + (i % 5) * 1.1;
        const gy = 0.5;
        const gz = Math.floor(i / 5) * 1.1;
        mesh.position.set(gx, gy, gz);
        setPose((prev) => (e.name in prev ? prev : { ...prev, [e.name]: { x: gx, y: gy, z: gz } }));
        scene.add(mesh);
        return mesh;
      });

    return () => {};
  }, [entities]);

  // 3D playback loop
  useEffect(() => {
    if (!playing3d) return;
    const id = window.setInterval(() => setFrame3d((f) => (f >= MAX_3D_FRAMES ? 0 : f + 1)), 90);
    return () => window.clearInterval(id);
  }, [playing3d]);

  // Apply scratch poses or keyframed motion to each mesh.
  useEffect(() => {
    const list = threeRef.current?.meshes ?? [];
    list.forEach((mesh) => {
      const kfs = poseKeyframes[mesh.name];
      if (kfs && kfs.length > 0) {
        const p = lerpPosition3(kfs, frame3d, ease3d);
        if (p) mesh.position.set(p.x, p.y, p.z);
      } else if (pose[mesh.name]) {
        const p2 = pose[mesh.name];
        mesh.position.set(p2.x, p2.y, p2.z);
      }
    });
  }, [entities, pose, poseKeyframes, frame3d, ease3d]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    
    const handleResize = () => {
      const { clientWidth, clientHeight } = container;
      const { renderer, camera } = threeRef.current!;
      
      renderer.setSize(clientWidth, clientHeight, false);
      camera.aspect = clientWidth / clientHeight;
      camera.updateProjectionMatrix();
    };
    
    const ro = new ResizeObserver(handleResize);
    ro.observe(container);
    
    const handleMouseDown = () => {
      threeRef.current!.dragging = true;
    };
    
    const handleMouseMove = (e: MouseEvent) => {
      if (!threeRef.current!.dragging) return;
      threeRef.current!.yaw -= e.movementX * 0.005;
      threeRef.current!.pitch = Math.max(
        0.05,
        Math.min(Math.PI / 2 - 0.05, threeRef.current!.pitch + e.movementY * 0.005)
      );
    };
    
    const handleMouseUp = () => {
      threeRef.current!.dragging = false;
    };
    
    const handleWheel = (e: WheelEvent) => {
      threeRef.current!.radius = Math.max(
        2,
        Math.min(20, threeRef.current!.radius + e.deltaY * 0.005)
      );
      e.preventDefault();
    };
    
    container.addEventListener("mousedown", handleMouseDown);
    container.addEventListener("mousemove", handleMouseMove);
    container.addEventListener("mouseup", handleMouseUp);
    container.addEventListener("mouseleave", handleMouseUp);
    container.addEventListener("wheel", handleWheel, { passive: false });
    
    return () => {
      ro.disconnect();
      container.removeEventListener("mousedown", handleMouseDown);
      container.removeEventListener("mousemove", handleMouseMove);
      container.removeEventListener("mouseup", handleMouseUp);
      container.removeEventListener("mouseleave", handleMouseUp);
      container.removeEventListener("wheel", handleWheel);
    };
  }, []);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex min-h-0 flex-1">
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex gap-2 p-2 border-b border-border">
          <Button onClick={() => addPrimitive("cube")}>Add Cube</Button>
          <Button onClick={() => addPrimitive("cylinder")}>Add Cylinder</Button>
          <Button onClick={() => addPrimitive("sphere")}>Add Sphere</Button>
          <Button onClick={() => addPrimitive("torus")}>Add Torus</Button>
        </div>

        <div ref={containerRef} className="relative flex-1 min-h-0">
          <canvas ref={canvasRef} className="h-full w-full" />
        </div>
      </div>

      <div className="w-64 shrink-0 border-l border-border p-2">
        <Panel title="Export Mesh">
          <div className="flex flex-col gap-3">
            <div>
              <label className="text-[11px] text-muted">Mesh</label>
              <select
                value={selectedMesh}
                onChange={(e) => setSelectedMesh(e.target.value)}
                className="w-full rounded border border-border bg-background px-2 py-1 text-sm"
              >
                {meshes.length === 0 && <option value="">No meshes yet</option>}
                {meshes.map((m) => (
                  <option key={m.id} value={m.name}>{m.name}</option>
                ))}
              </select>
            </div>

            <div className="grid grid-cols-2 gap-2">
              {(["obj", "stl", "glb", "ply"] as MeshFormatId[]).map((fmt) => (
                <Button key={fmt} size="sm" disabled={!selectedMesh || exporting !== null} onClick={() => doExport(fmt)}>
                  {exporting === fmt ? "…" : fmt.toUpperCase()}
                </Button>
              ))}
            </div>

            {exportNote && <p className="text-[11px] text-muted">{exportNote}</p>}
            <p className="text-[11px] text-muted">
              Exports real geometry via /api/v1/mesh/export (OBJ, STL, GLB, PLY) and downloads the file.
            </p>
          </div>
        </Panel>

        <div className="mt-2">
          <Panel title="Pose & Animate">
            <div className="flex flex-col gap-3">
              <div>
                <label className="text-[11px] text-muted">Pose target</label>
                <select
                  value={selectedMesh}
                  onChange={(e) => setSelectedMesh(e.target.value)}
                  className="w-full rounded border border-border bg-background px-2 py-1 text-sm"
                >
                  {meshes.length === 0 && <option value="">No meshes</option>}
                  {meshes.map((m) => (
                    <option key={m.id} value={m.name}>{m.name}</option>
                  ))}
                </select>
              </div>

              {(["x", "y", "z"] as const).map((axis) => {
                const val = selectedMesh ? (pose[selectedMesh] ?? { x: 0, y: 0, z: 0 })[axis] : 0;
                return (
                  <div key={axis} className="flex items-center gap-2">
                    <span className="w-4 font-mono text-[11px] uppercase text-muted">{axis}</span>
                    <input
                      type="range"
                      min={-3}
                      max={3}
                      step={0.1}
                      value={val}
                      onChange={(e) => setAxis(axis, Number(e.target.value))}
                      className="min-w-0 flex-1"
                      aria-label={`${axis} axis`}
                    />
                    <span className="w-10 text-right font-mono text-[11px]">{val.toFixed(1)}</span>
                  </div>
                );
              })}

              <Button size="sm" disabled={!selectedMesh} onClick={recordPose}>
                Record Pose @ Frame {frame3d}
              </Button>

              <div className="flex items-center justify-between gap-2">
                <label className="text-[11px] text-muted">Ease</label>
                <select value={ease3d} onChange={(e) => setEase3d(e.target.value as EasingId)} className="rounded border border-border bg-background px-1.5 py-0.5 text-[11px]" aria-label="3D easing">
                  {EASING_IDS.map((id) => (
                    <option key={id} value={id}>{EASING_LABELS[id]}</option>
                  ))}
                </select>
              </div>

              <div className="flex flex-wrap gap-1">
                {(selectedMesh ? poseKeyframes[selectedMesh] ?? [] : []).map((k) => (
                  <span key={k.time} className="rounded bg-surfaceAlt px-1.5 py-0.5 font-mono text-[10px]">f{k.time}</span>
                ))}
                {(selectedMesh ? (poseKeyframes[selectedMesh] ?? []).length : 0) === 0 && (
                  <span className="text-[11px] text-muted">No poses recorded. Scrub the timeline, move the sliders, then Record.</span>
                )}
              </div>
            </div>
          </Panel>
        </div>

        <div className="mt-2">
          <Panel title="Scene">
            <div className="flex flex-col gap-1 text-xs">
              {meshes.length === 0 && <span className="text-muted">Add a primitive to populate the scene.</span>}
              {meshes.map((m, i) => (
                <div key={m.id} className="flex items-center justify-between rounded border border-border px-2 py-1">
                  <span>{m.name}</span>
                  <span className="text-muted">{["cube", "cylinder", "sphere", "torus"][i % 4]}</span>
                </div>
              ))}
            </div>
          </Panel>
        </div>
      </div>
      </div>

      <footer className="flex shrink-0 items-center gap-3 border-t border-border px-3 py-2">
        <Button size="sm" variant={playing3d ? "default" : "accent"} onClick={() => setPlaying3d((p) => !p)}>
          {playing3d ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />} {playing3d ? "Pause" : "Play"}
        </Button>
        <span className="text-xs text-muted">Frame {frame3d}/{MAX_3D_FRAMES}</span>
        <div className="relative h-5 flex-1 rounded bg-surfaceAlt">
          <div className="absolute inset-y-0 left-0 rounded bg-accent/40" style={{ width: `${(frame3d / MAX_3D_FRAMES) * 100}%` }} />
          <input
            type="range"
            min={0}
            max={MAX_3D_FRAMES}
            value={frame3d}
            onChange={(e) => { setPlaying3d(false); setFrame3d(Number(e.target.value)); }}
            className="absolute inset-0 h-full w-full cursor-pointer opacity-0"
            aria-label="3D timeline"
          />
        </div>
        <span className="text-[11px] text-muted">{selectedMesh ? `Animating ${selectedMesh}` : "No mesh selected"}</span>
      </footer>
    </div>
  );
}