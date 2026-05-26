import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import init, {
  hello,
  build_mesh,
} from "../kpe-wasm/kpe_wasm.js";
import { SketchUI } from "./sketch-ui";
import type { ToolType } from "./sketch-engine";

type TriangleMesh = {
  vertices: [number, number, number][];
  normals: [number, number, number][];
  uvs: [number, number][];
  triangles: [number, number, number][];
};

// ── Recipes ──────────────────────────────────────────────────────

function makeRecipe(scene: any) {
  return {
    version: "0.1.0",
    metadata: { name: "KPE Demo", author: null, description: null, created_at: null, tags: [] },
    blocks: {},
    scene,
    joints: [],
    constraints: [],
    materials: {},
    precision: null,
  };
}

const RECIPES: Record<string, { label: string; recipe: any }> = {
  csg: {
    label: "CSG",
    recipe: makeRecipe({
      id: "root",
      node_type: "Compound",
      transform: null,
      children: [
        { id: "box1", node_type: { Box: { width: 2, height: 2, depth: 2 } }, transform: null, children: [], operations: [] },
        { id: "sphere1", node_type: { Sphere: { radius: 1.3 } }, transform: { translation: [0.8, 0, 0.8], rotation: null, scale: null }, children: [], operations: [] },
      ],
      operations: [],
    }),
  },
  extrude: {
    label: "Extrude",
    recipe: makeRecipe({
      id: "root",
      node_type: "Compound",
      transform: null,
      children: [
        {
          id: "profile",
          node_type: {
            Sketch: {
              plane: "XY",
              primitives: [
                { Rectangle: { x: -1.5, y: -0.75, width: 3, height: 1.5 } },
                { Circle: { cx: 0, cy: 0, radius: 0.4 } },
              ],
            },
          },
          transform: null,
          children: [],
          operations: [],
        },
        {
          id: "block",
          node_type: { Extrude: { sketch_id: "profile", distance: 4, cap: true } },
          transform: null,
          children: [],
          operations: [],
        },
      ],
      operations: [],
    }),
  },
  revolve: {
    label: "Revolve",
    recipe: makeRecipe({
      id: "root",
      node_type: "Compound",
      transform: null,
      children: [
        {
          id: "profile",
          node_type: {
            Sketch: {
              plane: "XY",
              primitives: [
                {
                  Polygon: {
                    points: [
                      [0, 0],
                      [0.3, 0], [0.3, 0.5], [0.5, 0.7], [0.5, 1.5],
                      [0.3, 1.7], [0.3, 2.5], [0.4, 2.7], [0.4, 3.5],
                      [0.3, 3.7], [0.3, 4.5], [0.5, 4.7], [0.5, 5.0],
                      [0.3, 5.0], [0.3, 5.5], [0, 6.0]
                    ],
                  },
                },
              ],
            },
          },
          transform: null,
          children: [],
          operations: [],
        },
        {
          id: "leg",
          node_type: { Revolve: { sketch_id: "profile", angle: 6.283185307179586, segments: 48, axis: "Y", cap: false } },
          transform: null,
          children: [],
          operations: [],
        },
      ],
      operations: [],
    }),
  },
  groups: {
    label: "Groups",
    recipe: makeRecipe({
      id: "root",
      node_type: "Compound",
      transform: null,
      children: [
        {
          id: "rotated-group",
          node_type: "Compound",
          transform: { translation: null, rotation: [0, 45, 0], scale: null },
          children: [
            {
              id: "box",
              node_type: { Box: { width: 2, height: 1, depth: 0.5 } },
              transform: null,
              children: [],
              operations: [],
            },
            {
              id: "nested-group",
              node_type: "Compound",
              transform: { translation: [2.5, 0.5, 0], rotation: null, scale: null },
              children: [
                {
                  id: "sphere",
                  node_type: { Sphere: { radius: 0.8 } },
                  transform: null,
                  children: [],
                  operations: [],
                },
              ],
              operations: [],
            },
          ],
          operations: [],
        },
      ],
      operations: [],
    }),
  },
  sweep: {
    label: "Sweep",
    recipe: makeRecipe({
      id: "root",
      node_type: "Compound",
      transform: null,
      children: [
        {
          id: "wire",
          node_type: {
            Sketch: {
              plane: "YZ",
              primitives: [
                { Circle: { cx: 0, cy: 0, radius: 0.15, segments: 12 } },
              ],
            },
          },
          transform: null,
          children: [],
          operations: [],
        },
        {
          id: "spring",
          node_type: { Sweep: { sketch_id: "wire", path: { Helix: { radius: 1.5, pitch: 0.8, turns: 5 } }, segments: 120, cap: false } },
          transform: null,
          children: [],
          operations: [],
        },
      ],
      operations: [],
    }),
  },
};

// ── Three.js helpers ─────────────────────────────────────────────

function meshToThree(mesh: TriangleMesh, color: number) {
  const verts = new Float32Array(mesh.vertices.flat());
  const idx = new Uint32Array(mesh.triangles.flat());
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(verts, 3));
  geometry.setIndex(new THREE.BufferAttribute(idx, 1));
  geometry.computeVertexNormals();

  const mat = new THREE.MeshStandardMaterial({
    color,
    roughness: 0.35,
    side: THREE.DoubleSide,
    flatShading: true,
    polygonOffset: true,
    polygonOffsetFactor: 1,
    polygonOffsetUnits: 1,
  });
  const obj = new THREE.Mesh(geometry, mat);
  obj.castShadow = true;
  obj.receiveShadow = true;

  const wireMat = new THREE.LineBasicMaterial({ color: 0x000000, transparent: true, opacity: 0.15 });
  const wireObj = new THREE.LineSegments(new THREE.WireframeGeometry(geometry), wireMat);
  obj.add(wireObj);

  return obj;
}

// ── Main ─────────────────────────────────────────────────────────

async function main() {
  await init();
  const statusEl = document.getElementById("status")!;
  statusEl.textContent = hello();

  // Renderer
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.2;
  document.getElementById("app")!.appendChild(renderer.domElement);

  // Scene
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x111122);

  const camera = new THREE.PerspectiveCamera(45, innerWidth / innerHeight, 0.1, 100);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.08;
  controls.target.set(0, 0, 0);

  // Lights
  scene.add(new THREE.AmbientLight(0x404060, 0.5));
  const dl = new THREE.DirectionalLight(0xffffff, 2.5);
  dl.position.set(8, 12, 6);
  dl.castShadow = true;
  scene.add(dl);
  const fill = new THREE.DirectionalLight(0x8888ff, 0.6);
  fill.position.set(-4, 2, -6);
  scene.add(fill);

  // Ground
  const ground = new THREE.Mesh(
    new THREE.PlaneGeometry(20, 20),
    new THREE.MeshStandardMaterial({ color: 0x222244, roughness: 0.8, metalness: 0.1 }),
  );
  ground.rotation.x = -Math.PI / 2;
  ground.position.y = -1.5;
  ground.receiveShadow = true;
  scene.add(ground);

  // Grid
  const grid = new THREE.GridHelper(14, 20, 0x666688, 0x444466);
  grid.position.y = -1.5;
  scene.add(grid);

  // Editor
  const editor = document.getElementById("editor") as HTMLTextAreaElement;
  const triCount = document.getElementById("tri-count")!;
  const buildBtn = document.getElementById("build-btn")!;

  let meshGroup = new THREE.Group();
  scene.add(meshGroup);

  function buildFromRecipe(recipeJson: string) {
    try {
      const json = build_mesh(recipeJson);
      const mesh: TriangleMesh = JSON.parse(json);

      // Clear previous
      scene.remove(meshGroup);
      meshGroup.traverse((c) => {
        if (c instanceof THREE.Mesh) {
          c.geometry.dispose();
          if (Array.isArray(c.material)) c.material.forEach((m) => m.dispose());
          else c.material.dispose();
        }
      });

      meshGroup = new THREE.Group();
      const obj = meshToThree(mesh, 0x88aaff);
      meshGroup.add(obj);
      scene.add(meshGroup);

      const count = mesh.triangles.length;
      triCount.textContent = `${count} triangles`;
      statusEl.textContent = `${count} triangles · OK`;
    } catch (e: any) {
      statusEl.textContent = `Error: ${e}`;
    }
  }

  // Demo buttons
  let currentDemo = "csg";

  const updateCamera = () => {
    switch (currentDemo) {
      case "csg": camera.position.set(6, 4, 8); break;
      case "extrude": camera.position.set(6, 4, 8); break;
      case "revolve": camera.position.set(8, 6, 8); break;
      case "groups": camera.position.set(5, 3, 6); break;
      case "sweep": camera.position.set(5, 4, 6); break;
    }
    controls.target.set(0, 0, 0);
    controls.update();
  };
  updateCamera();

  const demoButtons = document.querySelectorAll<HTMLButtonElement>("[data-demo]");
  demoButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
      demoButtons.forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      currentDemo = btn.dataset.demo!;
      updateCamera();
      const entry = RECIPES[currentDemo];
      const json = JSON.stringify(entry.recipe, null, 2);
      editor.value = json;
      buildFromRecipe(json);
    });
  });

  // Load default
  const defaultJson = JSON.stringify(RECIPES.csg.recipe, null, 2);
  editor.value = defaultJson;
  buildFromRecipe(defaultJson);

  // Build button
  buildBtn.addEventListener("click", () => {
    buildFromRecipe(editor.value);
  });

  // ── Sketch Mode ──────────────────────────────────────────────────

  const toolbar = document.getElementById("toolbar")!;
  const sketchBtn = document.getElementById("sketch-btn")!;
  const sketchUI = new SketchUI(scene, renderer);
  let isSketchMode = false;

  const sketchToolBtns = document.querySelectorAll<HTMLButtonElement>("[data-sketch-tool]");

  function activateSketchMode() {
    isSketchMode = true;
    toolbar.classList.add("sketch-mode");
    controls.enabled = false;
    sketchUI.activate();
    statusEl.textContent = "Sketch mode — draw on the grid";
    panel.classList.add("collapsed");
    scene.remove(meshGroup);
  }

  function deactivateSketchMode() {
    isSketchMode = false;
    toolbar.classList.remove("sketch-mode");
    controls.enabled = true;
    sketchUI.deactivate();
    scene.add(meshGroup);
    updateCamera();
    statusEl.textContent = `${triCount.textContent || "OK"}`;
    panel.classList.remove("collapsed");
  }

  sketchBtn.addEventListener("click", () => {
    if (isSketchMode) {
      deactivateSketchMode();
      sketchBtn.classList.remove("active");
    } else {
      activateSketchMode();
      sketchBtn.classList.add("active");
    }
  });

  sketchToolBtns.forEach((btn) => {
    btn.addEventListener("click", () => {
      sketchToolBtns.forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      const tool = btn.dataset.sketchTool as ToolType;
      sketchUI.setTool(tool);
      statusEl.textContent = `Tool: ${tool}`;
    });
  });

  sketchUI.onExtrudeJSON = (_json: string) => {
    const engine = sketchUI.engineInstance;
    const distance = parseFloat((document.getElementById("extrude-distance") as HTMLInputElement).value) || 3.0;
    const meshJson = engine.extrude(distance);
    if (!meshJson) return;
    try {
      const resultMesh: TriangleMesh = JSON.parse(meshJson);
      scene.remove(meshGroup);
      meshGroup.traverse((c) => {
        if (c instanceof THREE.Mesh) {
          c.geometry.dispose();
          if (Array.isArray(c.material)) c.material.forEach((m) => m.dispose());
          else c.material.dispose();
        }
      });
      meshGroup = new THREE.Group();

      const obj = meshToThree(resultMesh, 0x88aaff);

      // Orient the mesh according to the selected plane
      if (sketchUI.plane === "XY") {
        obj.rotation.x = Math.PI / 2;
        obj.position.z += sketchUI.offset;
      } else if (sketchUI.plane === "XZ") {
        // Default orientation in engine is XZ anyway? Wait, engine extrudes from XY plane conceptually?
        // Let's rely on how the engine outputs, and adjust.
        // Actually the engine outputs 'mesh' based on points in 2D.
        // It extrudes along Z in the engine.
        // Our 3D mapping: X -> X, Y -> Z. So we might need rotation.
        // Assuming engine extrudes in Z. In three.js we want it in Y if it's on XZ floor.
        obj.rotation.x = -Math.PI / 2;
        obj.position.y += sketchUI.offset;
      } else if (sketchUI.plane === "YZ") {
        obj.rotation.y = Math.PI / 2;
        obj.rotation.x = -Math.PI / 2;
        obj.position.x += sketchUI.offset;
      }

      meshGroup.add(obj);
      scene.add(meshGroup);
      const count = resultMesh.triangles?.length || 0;
      triCount.textContent = `${count} triangles`;
      statusEl.textContent = `Extruded · ${count} tris`;
    } catch (e: any) {
      statusEl.textContent = `Extrude error: ${e}`;
    }
  };

  // Properties Panel Handlers
  const planeSelect = document.getElementById("sketch-plane") as HTMLSelectElement;
  const offsetInput = document.getElementById("plane-offset") as HTMLInputElement;
  const extrudeInput = document.getElementById("extrude-distance") as HTMLInputElement;
  const visualizeBtn = document.getElementById("sketch-visualize") as HTMLButtonElement;

  planeSelect.addEventListener("change", () => {
    if (isSketchMode) {
      sketchUI.plane = planeSelect.value as any;
      sketchUI.buildGrid();
      sketchUI.updateCamera();
    }
  });

  offsetInput.addEventListener("input", () => {
    if (isSketchMode) {
      sketchUI.offset = parseFloat(offsetInput.value) || 0;
      sketchUI.buildGrid();
      sketchUI.updateCamera();
    }
  });

  visualizeBtn.addEventListener("click", () => {
    if (isSketchMode) {
      sketchUI.onExtrudeJSON!(sketchUI.engineInstance.state.docJson);
      deactivateSketchMode();
      sketchBtn.classList.remove("active");
    }
  });

  // Build from sketch
  buildBtn.addEventListener("click", () => {
    if (isSketchMode) {
      const engine = sketchUI.engineInstance;
      const meshJson = engine.extrude(3.0);
      if (meshJson) {
        editor.value = meshJson;
        try {
          const mesh: TriangleMesh = JSON.parse(meshJson);
          scene.remove(meshGroup);
          meshGroup.traverse((c) => {
            if (c instanceof THREE.Mesh) {
              c.geometry.dispose();
              if (Array.isArray(c.material)) c.material.forEach((m) => m.dispose());
              else c.material.dispose();
            }
          });
          meshGroup = new THREE.Group();
          const obj = meshToThree(mesh, 0x88aaff);
          if (sketchUI.plane === "XY") {
            obj.rotation.x = Math.PI / 2;
            obj.position.z += sketchUI.offset;
          } else if (sketchUI.plane === "XZ") {
            obj.rotation.x = -Math.PI / 2;
            obj.position.y += sketchUI.offset;
          } else if (sketchUI.plane === "YZ") {
            obj.rotation.y = Math.PI / 2;
            obj.rotation.x = -Math.PI / 2;
            obj.position.x += sketchUI.offset;
          }
          meshGroup.add(obj);
          scene.add(meshGroup);
          triCount.textContent = `${mesh.triangles?.length || 0} triangles`;
          statusEl.textContent = `Built from sketch`;
        } catch (e: any) {
          statusEl.textContent = `Build error: ${e}`;
        }
      }
    }
  });

  // Toggle panel
  const panel = document.getElementById("panel")!;
  document.getElementById("toggle-panel")!.addEventListener("click", () => {
    if (!isSketchMode) panel.classList.toggle("collapsed");
  });

  // Keyboard shortcut
  editor.addEventListener("keydown", (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      buildFromRecipe(editor.value);
    }
  });

  addEventListener("resize", () => {
    camera.aspect = innerWidth / innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(innerWidth, innerHeight);
  });

  // Animate
  function animate() {
    requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
    if (sketchUI.active && sketchUI.activeCamera) {
      renderer.render(scene, sketchUI.activeCamera);
    }
  }
  animate();
}

main();
