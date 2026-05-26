import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import init, {
  hello,
  build_mesh,
  csg_union,
  csg_subtract,
  csg_intersect,
} from "../kpe-wasm/kpe_wasm.js";

type TriangleMesh = {
  vertices: [number, number, number][];
  normals: [number, number, number][];
  uvs: [number, number][];
  triangles: [number, number, number][];
};

function meshToThree(mesh: TriangleMesh, color: number, ghost: boolean = false) {
  const verts = new Float32Array(mesh.vertices.flat());
  const idx = new Uint32Array(mesh.triangles.flat());
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(verts, 3));
  geometry.setIndex(new THREE.BufferAttribute(idx, 1));
  geometry.computeVertexNormals();

  const mat = new THREE.MeshStandardMaterial({
    color,
    roughness: 0.4,
    metalness: 0.1,
    side: THREE.DoubleSide,
    transparent: ghost,
    opacity: ghost ? 0.2 : 1.0,
    polygonOffset: true,
    polygonOffsetFactor: 1,
    polygonOffsetUnits: 1,
  });
  const obj = new THREE.Mesh(geometry, mat);
  obj.castShadow = !ghost;
  obj.receiveShadow = !ghost;

  // Add wireframe to show the mesh properly
  const wireMat = new THREE.LineBasicMaterial({
    color: ghost ? 0xffffff : 0x000000,
    transparent: true,
    opacity: ghost ? 0.1 : 0.3,
  });
  const wireObj = new THREE.LineSegments(new THREE.WireframeGeometry(geometry), wireMat);
  obj.add(wireObj);

  return obj;
}

function makeBox(id: string, w: number, h: number, d: number) {
  return {
    id,
    node_type: { Box: { width: w, height: h, depth: d } },
    transform: null as any,
    children: [] as any[],
    operations: [] as any[],
  };
}

function makeSphere(id: string, r: number) {
  return {
    id,
    node_type: { Sphere: { radius: r } },
    transform: null as any,
    children: [] as any[],
    operations: [] as any[],
  };
}

function makeRecipe(children: any[]) {
  return {
    version: "0.1.0",
    metadata: { name: "CSG Demo", author: null, description: null, created_at: null, tags: [] },
    blocks: {},
    scene: {
      id: "root",
      node_type: "Compound",
      transform: null,
      children,
      operations: [],
    },
    joints: [],
    constraints: [],
    materials: {},
    precision: null,
  };
}

async function main() {
  await init();
  const statusEl = document.getElementById("status");
  if (statusEl) statusEl.textContent = hello();

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
  camera.position.set(6, 4, 8);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.08;
  controls.target.set(0, 0, 0);
  controls.update();

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

  // Build source meshes
  const boxNode = makeBox("box1", 2, 2, 2);
  const sphereNode = makeSphere("sphere1", 1.3);
  sphereNode.transform = { translation: [0.8, 0, 0.8], rotation: null, scale: null };

  const boxRecipe = makeRecipe([boxNode]);
  const sphereRecipe = makeRecipe([sphereNode]);

  const boxJson = build_mesh(JSON.stringify(boxRecipe));
  const sphereJson = build_mesh(JSON.stringify(sphereRecipe));

  const boxMesh: TriangleMesh = JSON.parse(boxJson);
  const sphereMesh: TriangleMesh = JSON.parse(sphereJson);

  // Ghost originals
  const ghostGroup = new THREE.Group();
  ghostGroup.position.x = -7.0; // Prevent z-fighting with the Union result at x=0

  const ghostBox = meshToThree(boxMesh, 0x4488ff, true);
  ghostGroup.add(ghostBox);

  const ghostSphere = meshToThree(sphereMesh, 0xff8844, true);
  ghostGroup.add(ghostSphere);

  scene.add(ghostGroup);

  // Ghost Label
  const ghostCanvas = document.createElement("canvas");
  ghostCanvas.width = 256;
  ghostCanvas.height = 64;
  const ghostCtx = ghostCanvas.getContext("2d")!;
  ghostCtx.fillStyle = "#fff";
  ghostCtx.font = "bold 28px sans-serif";
  ghostCtx.textAlign = "center";
  ghostCtx.fillText("Originals", 128, 42);
  const ghostTex = new THREE.CanvasTexture(ghostCanvas);
  const ghostSprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: ghostTex, depthWrite: false }));
  ghostSprite.scale.set(2, 0.5, 1);
  ghostSprite.position.set(0, 2.8, 0); // Relative to group
  ghostGroup.add(ghostSprite);

  // CSG results
  const results: [string, typeof csg_union, number, number][] = [
    ["Union", csg_union, 0x44ff88, 0],
    ["Subtract", csg_subtract, 0xff4488, 3.5],
    ["Intersect", csg_intersect, 0xffcc44, -3.5],
  ];

  for (const [label, fn, color, x] of results) {
    const json = fn(boxJson, sphereJson);
    const mesh: TriangleMesh = JSON.parse(json);
    const obj = meshToThree(mesh, color);
    obj.position.x = x;
    scene.add(obj);

    // Label sprite
    const canvas = document.createElement("canvas");
    canvas.width = 256;
    canvas.height = 64;
    const ctx = canvas.getContext("2d")!;
    ctx.fillStyle = "#fff";
    ctx.font = "bold 28px sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(label, 128, 42);
    const tex = new THREE.CanvasTexture(canvas);
    const sprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: tex, depthWrite: false }));
    sprite.scale.set(2, 0.5, 1);
    sprite.position.set(x, 2.8, 0);
    scene.add(sprite);
  }

  // Grid
  const grid = new THREE.GridHelper(14, 20, 0x666688, 0x444466);
  grid.position.y = -1.5;
  scene.add(grid);

  // Resize
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
  }
  animate();
}

main();
