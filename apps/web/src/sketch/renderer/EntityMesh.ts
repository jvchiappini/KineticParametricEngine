import * as THREE from "three";
import type { SketchModel } from "../model/SketchModel";
import type { Entity } from "../model/Entity";
import { getPointCoords } from "../model/SketchModel";

const CONSTRUCTION_COLOR = 0x00bcd4;
const NORMAL_COLOR = 0x4a9eff;
const CONSTRAINED_COLOR = 0x43e97b;
const SELECTED_COLOR = 0xffffff;
const ARC_SEGMENTS = 48;

function makeLineMaterial(
  color: number,
  dashed: boolean,
  opacity: number
): THREE.LineBasicMaterial | THREE.LineDashedMaterial {
  if (dashed) {
    return new THREE.LineDashedMaterial({
      color,
      dashSize: 4,
      gapSize: 3,
      opacity,
      transparent: opacity < 1,
    });
  }
  return new THREE.LineBasicMaterial({
    color,
    opacity,
    transparent: opacity < 1,
  });
}

function lineFromPoints(
  pts: number[],
  mat: THREE.LineBasicMaterial | THREE.LineDashedMaterial
): THREE.Line {
  const geo = new THREE.BufferGeometry();
  geo.setAttribute(
    "position",
    new THREE.BufferAttribute(new Float32Array(pts), 3)
  );
  const line = new THREE.Line(geo, mat);
  if (mat instanceof THREE.LineDashedMaterial) {
    line.computeLineDistances();
  }
  return line;
}

function buildPointMesh(x: number, y: number): THREE.Object3D {
  const geo = new THREE.CircleGeometry(3, 8);
  const mat = new THREE.MeshBasicMaterial({ color: NORMAL_COLOR });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.position.set(x, y, 0);
  return mesh;
}

function buildLineMesh(
  x1: number, y1: number,
  x2: number, y2: number,
  mat: THREE.LineBasicMaterial | THREE.LineDashedMaterial
): THREE.Object3D {
  return lineFromPoints([x1, y1, 0, x2, y2, 0], mat);
}

function buildCircleMesh(
  cx: number, cy: number, r: number,
  mat: THREE.LineBasicMaterial | THREE.LineDashedMaterial
): THREE.Object3D {
  const pts: number[] = [];
  const steps = 48;
  for (let i = 0; i <= steps; i++) {
    const theta = (i / steps) * Math.PI * 2;
    pts.push(cx + r * Math.cos(theta), cy + r * Math.sin(theta), 0);
  }
  return lineFromPoints(pts, mat);
}

function buildArcMesh(
  cx: number, cy: number, r: number,
  startAngle: number, endAngle: number,
  mat: THREE.LineBasicMaterial | THREE.LineDashedMaterial
): THREE.Object3D {
  const pts: number[] = [];
  const span = endAngle - startAngle;
  for (let i = 0; i <= ARC_SEGMENTS; i++) {
    const theta = startAngle + (i / ARC_SEGMENTS) * span;
    pts.push(cx + r * Math.cos(theta), cy + r * Math.sin(theta), 0);
  }
  return lineFromPoints(pts, mat);
}

function buildPolylineMesh(
  coords: { x: number; y: number }[],
  closed: boolean,
  mat: THREE.LineBasicMaterial | THREE.LineDashedMaterial
): THREE.Object3D {
  const pts: number[] = [];
  for (const p of coords) pts.push(p.x, p.y, 0);
  if (closed && coords.length > 0) {
    pts.push(coords[0].x, coords[0].y, 0);
  }
  return lineFromPoints(pts, mat);
}

export function createEntityMesh(
  entity: Entity,
  model: SketchModel,
  isSelected: boolean,
  isConstrained: boolean
): THREE.Object3D {
  const color = isSelected
    ? SELECTED_COLOR
    : isConstrained
      ? CONSTRAINED_COLOR
      : NORMAL_COLOR;
  const isConstruction =
    "construction" in entity && entity.construction === true;
  const opacity = isConstruction ? 0.4 : 1;
  const mat = makeLineMaterial(color, isConstruction, opacity);

  switch (entity.kind) {
    case "point":
      return buildPointMesh(entity.x, entity.y);

    case "line": {
      const s = getPointCoords(model, entity.start);
      const e = getPointCoords(model, entity.end);
      if (!s || !e) return new THREE.Group();
      return buildLineMesh(s.x, s.y, e.x, e.y, mat);
    }

    case "circle": {
      const c = getPointCoords(model, entity.center);
      if (!c) return new THREE.Group();
      return buildCircleMesh(c.x, c.y, entity.radius, mat);
    }

    case "arc": {
      const c = getPointCoords(model, entity.center);
      if (!c) return new THREE.Group();
      return buildArcMesh(
        c.x, c.y, entity.radius,
        entity.startAngle, entity.endAngle, mat
      );
    }

    case "polyline": {
      const coords: { x: number; y: number }[] = [];
      for (const pid of entity.points) {
        const p = getPointCoords(model, pid);
        if (p) coords.push(p);
      }
      return buildPolylineMesh(coords, entity.closed, mat);
    }

    default:
      return new THREE.Group();
  }
}
