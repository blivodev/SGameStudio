import { useEffect, useRef } from 'react';
import * as THREE from 'three';
import { parseEntityRef } from '../../lib/parseResult';
import { dispatchCommand } from '../../services/commandBridge';
import { useStudioStore } from '../../store/studioStore';
import { Button } from '../../components/ui/primitives';

export default function Studio3D() {
  const { entities, addEntity, setLastError } = useStudioStore();
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
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
      .map((_e, i) => {
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
        mesh.position.set(-2 + (i % 5) * 1.1, 0.5, Math.floor(i / 5) * 1.1);
        scene.add(mesh);
        return mesh;
      });

    return () => {};
  }, [entities]);

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
    <div className="flex flex-col h-full">
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
  );
}