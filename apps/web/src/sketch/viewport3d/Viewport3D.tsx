import { useRef, useEffect, useState, type JSX } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { TransformControls } from "three/addons/controls/TransformControls.js";
import { buildAllMeshes } from "./ExtrudeBuilder";
import type { SketchData } from "./ExtrudeBuilder";
import { wasmGetContours } from "../../wasm";

interface Props { sketches: SketchData[] }

export function Viewport3D({ sketches }: Props): JSX.Element {
  const containerRef = useRef<HTMLDivElement>(null);
  const [selectedInfo, setSelectedInfo] = useState("");
  const sceneRef = useRef<{
    scene: THREE.Scene; camera: THREE.PerspectiveCamera;
    renderer: THREE.WebGLRenderer; controls: OrbitControls;
  } | null>(null);
  const transformRef = useRef<TransformControls | null>(null);
  const meshesRef = useRef<THREE.Object3D[]>([]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    const w = container.clientWidth || 600;
    const h = container.clientHeight || 400;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x1a1a2e);

    const camera = new THREE.PerspectiveCamera(45, w / h, 0.1, 1000);
    camera.position.set(30, 20, 30);
    camera.lookAt(0, 0, 0);

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(w, h);
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    container.appendChild(renderer.domElement);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.1;
    controls.target.set(0, 0, 0);

    const tc = new TransformControls(camera, renderer.domElement);
    tc.setSize(0.8); tc.setMode("translate");
    tc.addEventListener("dragging-changed", (e) => { controls.enabled = !e.value; });
    (tc as unknown as { parent: THREE.Scene }).parent = scene; (scene.children as unknown[]).push(tc);
    transformRef.current = tc;

    scene.add(new THREE.AmbientLight(0x404060, 0.6));
    const dir = new THREE.DirectionalLight(0xffffff, 1.2);
    dir.position.set(20, 30, 20);
    dir.castShadow = true;
    dir.shadow.mapSize.width = 1024;
    dir.shadow.mapSize.height = 1024;
    scene.add(dir);
    const fill = new THREE.DirectionalLight(0x8888ff, 0.4);
    fill.position.set(-20, 10, -20);
    scene.add(fill);

    scene.add(new THREE.GridHelper(60, 20, 0x444488, 0x333366));
    scene.add(new THREE.AxesHelper(15));

    sceneRef.current = { scene, camera, renderer, controls };

    const raycaster = new THREE.Raycaster();
    const pointer = new THREE.Vector2();
    const onDown = (event: PointerEvent) => {
      const rect = renderer.domElement.getBoundingClientRect();
      pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
      raycaster.setFromCamera(pointer, camera);
      const hits = raycaster.intersectObjects(meshesRef.current, false);
      if (hits.length > 0) {
        let obj: THREE.Object3D | null = hits[0].object;
        tc.attach(obj);
        setSelectedInfo(`Sketch ${(obj.userData.sketchIndex ?? 0) + 1}`);
      } else {
        tc.detach();
        setSelectedInfo("");
      }
    };
    renderer.domElement.addEventListener("pointerdown", onDown);

    const ro = new ResizeObserver((entries) => {
      for (const e of entries) {
        camera.aspect = e.contentRect.width / e.contentRect.height;
        camera.updateProjectionMatrix();
        renderer.setSize(e.contentRect.width, e.contentRect.height);
      }
    });
    ro.observe(container);

    let animId = 0;
    const animate = () => {
      animId = requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    };
    animate();

    return () => {
      cancelAnimationFrame(animId); ro.disconnect(); renderer.domElement.removeEventListener("pointerdown", onDown);
      controls.dispose(); tc.dispose(); renderer.dispose();
    };
  }, []);

  useEffect(() => {
    const ctx = sceneRef.current;
    if (!ctx) return;
    const old = ctx.scene.getObjectByName("sketch-group");
    if (old) ctx.scene.remove(old);
    const withContours: SketchData[] = sketches.map((s) => {
      try {
        const c = wasmGetContours(s.model);
        return { ...s, contours: c as unknown as [number, number][][] };
      } catch { return s; }
    });
    const group = buildAllMeshes(withContours);
    ctx.scene.add(group);
    meshesRef.current = [];
    group.traverse((child) => { if (child instanceof THREE.Mesh) meshesRef.current.push(child); });
  }, [sketches]);

  return (
    <div style={{ display: "flex", flexDirection: "column", width: "100%", height: "100%", background: "#111", overflow: "hidden", position: "relative" }}>
      {selectedInfo && (
        <div style={{ position: "absolute", top: 8, left: 8, background: "rgba(0,0,0,0.7)", color: "#8af", padding: "2px 8px", borderRadius: 4, fontSize: 11, zIndex: 10, pointerEvents: "none" }}>
          Selected: {selectedInfo}
        </div>
      )}
      <div ref={containerRef} style={{ flex: 1, position: "relative" }} />
    </div>
  );
}
