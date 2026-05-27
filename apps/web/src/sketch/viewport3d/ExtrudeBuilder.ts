import * as THREE from "three";
import type { SketchModel } from "../model/SketchModel";
import type { Entity, LineEntity } from "../model/Entity";

function planeNormal(plane: "XY" | "XZ" | "YZ"): THREE.Vector3 {
  switch (plane) {
    case "XY": return new THREE.Vector3(0, 0, 1);
    case "XZ": return new THREE.Vector3(0, 1, 0);
    case "YZ": return new THREE.Vector3(1, 0, 0);
  }
}

function to3D(sx: number, sy: number, plane: "XY" | "XZ" | "YZ", offset: number): THREE.Vector3 {
  switch (plane) {
    case "XY": return new THREE.Vector3(sx, sy, offset);
    case "XZ": return new THREE.Vector3(sx, offset, sy);
    case "YZ": return new THREE.Vector3(offset, sx, sy);
  }
}

const COLORS = [0x4488cc, 0x44cc88, 0xcc8844, 0xcc4488, 0x88cc44, 0x8844cc];
const CIRCLE_SEG = 32;

export interface SketchData {
  model: SketchModel;
  depth: number;
  index: number;
  bevel: boolean;
  bevelSize: number;
  contours?: [number, number][][]; // pre-computed by WASM
}

function extrudeOpts(depth: number, bevel: boolean, bevelSize: number): THREE.ExtrudeGeometryOptions {
  return {
    depth,
    bevelEnabled: bevel,
    bevelThickness: bevel ? bevelSize : 0,
    bevelSize: bevel ? bevelSize * 0.5 : 0,
    bevelSegments: bevel ? 3 : 0,
  };
}

function ptsOnPlane(pts2d: [number, number][], plane: "XY" | "XZ" | "YZ", offset: number): THREE.Vector3[] {
  return pts2d.map(([x, y]) => to3D(x, y, plane, offset));
}

function lineSegments3D(pts3d: THREE.Vector3[]): THREE.BufferGeometry {
  const pos: number[] = [];
  for (let i = 0; i < pts3d.length - 1; i++) {
    pos.push(pts3d[i].x, pts3d[i].y, pts3d[i].z, pts3d[i + 1].x, pts3d[i + 1].y, pts3d[i + 1].z);
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
  return g;
}

function buildSolid(pts2d: [number, number][], depth: number, bevel: boolean, bevelSize: number, plane: "XY" | "XZ" | "YZ", offset: number, color: number): THREE.Mesh | null {
  if (pts2d.length < 3) return null;
  const shape = new THREE.Shape();
  shape.moveTo(pts2d[0][0], pts2d[0][1]);
  for (let i = 1; i < pts2d.length; i++) shape.lineTo(pts2d[i][0], pts2d[i][1]);
  const geom = new THREE.ExtrudeGeometry(shape, extrudeOpts(depth, bevel, bevelSize));
  geom.translate(0, 0, -depth / 2);
  const mat = new THREE.MeshStandardMaterial({ color, metalness: 0.2, roughness: 0.6, side: THREE.DoubleSide });
  const mesh = new THREE.Mesh(geom, mat);
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  const normal = planeNormal(plane);
  mesh.quaternion.setFromUnitVectors(new THREE.Vector3(0, 0, 1), normal);
  mesh.position.copy(to3D(0, 0, plane, offset));
  return mesh;
}

function circlePoints(cx: number, cy: number, r: number): [number, number][] {
  const pts: [number, number][] = [];
  for (let i = 0; i < CIRCLE_SEG; i++) {
    const a = (i / CIRCLE_SEG) * Math.PI * 2;
    pts.push([cx + r * Math.cos(a), cy + r * Math.sin(a)]);
  }
  return pts;
}

function arcPoints(cx: number, cy: number, r: number, sa: number, ea: number): [number, number][] {
  const n = Math.max(4, Math.ceil(CIRCLE_SEG * Math.abs(ea - sa) / (Math.PI * 2)));
  const pts: [number, number][] = [];
  for (let i = 0; i <= n; i++) {
    const a = sa + (i / n) * (ea - sa);
    pts.push([cx + r * Math.cos(a), cy + r * Math.sin(a)]);
  }
  return pts;
}

interface ChainResult { pts: [number, number][]; closed: boolean }

function walkOrderedChains(model: SketchModel): ChainResult[] {
  const lines: LineEntity[] = [];
  for (const [, e] of model.entities) {
    if (e.kind === "line" && !e.construction) lines.push(e);
  }
  if (lines.length === 0) return [];

  const adj = new Map<string, string[]>();
  for (const l of lines) {
    if (!adj.has(l.start)) adj.set(l.start, []);
    if (!adj.has(l.end)) adj.set(l.end, []);
    adj.get(l.start)!.push(l.end);
    adj.get(l.end)!.push(l.start);
  }

  const visited = new Set<string>();
  const results: ChainResult[] = [];

  function walk(start: string, isLoop: boolean) {
    if (visited.has(start)) return;
    const ordered: string[] = [];
    let cur: string | null = start;
    let prev: string | null = null;
    while (cur !== null && !visited.has(cur)) {
      visited.add(cur);
      ordered.push(cur);
      const neighbors: string[] = adj.get(cur)!.filter((n: string) => n !== prev);
      if (neighbors.length === 0) break;
      prev = cur;
      cur = neighbors[0];
      if (isLoop && cur === start) break;
    }
    const pts: [number, number][] = [];
    for (const id of ordered) {
      const p = model.points.get(id);
      if (p) pts.push([p.x, p.y]);
    }
    if (pts.length >= 2) results.push({ pts, closed: isLoop });
  }

  // Pass 1: endpoints (degree 1) — open chains
  for (const [pid, nbs] of adj) if (nbs.length === 1) walk(pid, false);
  // Pass 2: remaining — closed loops (all degree 2)
  for (const [pid] of adj) if (!visited.has(pid)) walk(pid, true);

  return results;
}

function tag(obj: THREE.Object3D, index: number) { obj.userData.sketchIndex = index; }

function buildOneSketch(data: SketchData): THREE.Group {
  const group = new THREE.Group();
  const { model, depth, index, bevel, bevelSize, contours } = data;
  const { plane, planeOffset } = model;
  const color = COLORS[index % COLORS.length];

  const chains: ChainResult[] = contours
    ? contours.map((pts) => ({ pts: pts as [number, number][], closed: pts.length >= 3 && pts[0][0] === pts[pts.length - 1][0] && pts[0][1] === pts[pts.length - 1][1] }))
    : walkOrderedChains(model);

  for (const { pts, closed } of chains) {
    if (closed && pts.length >= 3) {
      const s = buildSolid(pts, depth, bevel, bevelSize, plane, planeOffset, color);
      if (s) { tag(s, index); group.add(s); }
    } else {
      const p3 = ptsOnPlane(pts, plane, planeOffset);
      const segs = new THREE.LineSegments(lineSegments3D(p3), new THREE.LineBasicMaterial({ color }));
      tag(segs, index); group.add(segs);
    }
  }

  for (const [, e] of model.entities) {
    if (e.kind === "circle" && !e.construction) {
      const cp = model.points.get(e.center);
      if (!cp) continue;
      const s = buildSolid(circlePoints(cp.x, cp.y, e.radius), depth, bevel, bevelSize, plane, planeOffset, color);
      if (s) { tag(s, index); group.add(s); }
    }
  }

  for (const [, e] of model.entities) {
    if (e.kind === "arc" && !e.construction) {
      const cp = model.points.get(e.center);
      if (!cp) continue;
      const p3 = ptsOnPlane(arcPoints(cp.x, cp.y, e.radius, e.startAngle, e.endAngle), plane, planeOffset);
      const segs = new THREE.LineSegments(lineSegments3D(p3), new THREE.LineBasicMaterial({ color: 0x888888 }));
      tag(segs, index); group.add(segs);
    }
  }

  for (const [, e] of model.entities) {
    if (e.kind === "polyline" && !e.construction) {
      const pts: [number, number][] = [];
      for (const pid of e.points) { const p = model.points.get(pid); if (p) pts.push([p.x, p.y]); }
      if (pts.length < 2) continue;
      if (e.closed && pts.length >= 3) {
        const s = buildSolid(pts, depth, bevel, bevelSize, plane, planeOffset, color);
        if (s) { tag(s, index); group.add(s); }
      } else {
        const segs = new THREE.LineSegments(lineSegments3D(ptsOnPlane(pts, plane, planeOffset)), new THREE.LineBasicMaterial({ color }));
        tag(segs, index); group.add(segs);
      }
    }
  }

  return group;
}

export function buildAllMeshes(sketches: SketchData[]): THREE.Group {
  const group = new THREE.Group();
  group.name = "sketch-group";
  for (let i = 0; i < sketches.length; i++) {
    group.add(buildOneSketch(sketches[i]));
  }
  return group;
}
