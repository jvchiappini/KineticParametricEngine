import * as THREE from "three";
import type { SketchModel } from "../model/SketchModel";
import type { Constraint } from "../model/Constraint";
import type { Entity } from "../model/Entity";
import { getPointCoords } from "../model/SketchModel";

const ICON_COLOR = 0xffc107;
const DIM_COLOR = 0xffc107;
const S = 6; // half-size of constraint icons

function line(
  x1: number, y1: number,
  x2: number, y2: number,
  color: number
): THREE.Line {
  const geo = new THREE.BufferGeometry();
  geo.setAttribute(
    "position",
    new THREE.BufferAttribute(new Float32Array([x1, y1, 0, x2, y2, 0]), 3)
  );
  return new THREE.Line(geo, new THREE.LineBasicMaterial({ color }));
}

function makeGroup(children: THREE.Object3D[]): THREE.Group {
  const g = new THREE.Group();
  for (const c of children) g.add(c);
  return g;
}

function getEntityCenter(
  entity: Entity,
  model: SketchModel
): { x: number; y: number } | null {
  switch (entity.kind) {
    case "point":
      return { x: entity.x, y: entity.y };
    case "line": {
      const s = getPointCoords(model, entity.start);
      const e = getPointCoords(model, entity.end);
      return s && e
        ? { x: (s.x + e.x) / 2, y: (s.y + e.y) / 2 }
        : null;
    }
    case "circle":
    case "arc": {
      const c = getPointCoords(model, entity.center);
      return c ?? null;
    }
    case "polyline": {
      if (entity.points.length === 0) return null;
      const p = getPointCoords(model, entity.points[0]);
      return p ?? null;
    }
  }
}

function getEntityCenterById(
  id: string,
  model: SketchModel
): { x: number; y: number } | null {
  const ent = model.entities.get(id);
  return ent ? getEntityCenter(ent, model) : null;
}

function buildHorizontalIcon(cx: number, cy: number): THREE.Group {
  return makeGroup([
    line(cx - S, cy, cx + S, cy, ICON_COLOR),
    line(cx - S - 2, cy - 2, cx - S, cy, ICON_COLOR),
    line(cx - S - 2, cy + 2, cx - S, cy, ICON_COLOR),
  ]);
}

function buildVerticalIcon(cx: number, cy: number): THREE.Group {
  return makeGroup([
    line(cx, cy - S, cx, cy + S, ICON_COLOR),
    line(cx - 2, cy - S - 2, cx, cy - S, ICON_COLOR),
    line(cx + 2, cy - S - 2, cx, cy - S, ICON_COLOR),
  ]);
}

function buildParallelIcon(cx: number, cy: number): THREE.Group {
  return makeGroup([
    line(cx - 3, cy - S + 2, cx + 3, cy - S + 2, ICON_COLOR),
    line(cx - 3, cy + S - 2, cx + 3, cy + S - 2, ICON_COLOR),
  ]);
}

function buildPerpendicularIcon(cx: number, cy: number): THREE.Group {
  return makeGroup([
    line(cx - S, cy - S, cx + S, cy - S, ICON_COLOR),
    line(cx - S, cy - S, cx - S, cy + S, ICON_COLOR),
    line(cx - S + 2, cy - S + 2, cx - S, cy - S, ICON_COLOR),
  ]);
}

function buildTangentIcon(cx: number, cy: number): THREE.Group {
  return makeGroup([
    line(cx - S, cy - S, cx + S, cy - S, ICON_COLOR),
    line(cx + S, cy - S, cx + S, cy + S, ICON_COLOR),
  ]);
}

function buildEqualIcon(cx: number, cy: number): THREE.Group {
  return makeGroup([
    line(cx - S, cy - 2, cx + S, cy - 2, ICON_COLOR),
    line(cx - S, cy + 2, cx + S, cy + 2, ICON_COLOR),
  ]);
}

function buildCoincidentIcon(cx: number, cy: number): THREE.Group {
  const geo = new THREE.CircleGeometry(3, 6);
  const mat = new THREE.MeshBasicMaterial({ color: ICON_COLOR });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.position.set(cx, cy, 0);
  return makeGroup([mesh]);
}

function buildSymmetricIcon(cx: number, cy: number): THREE.Group {
  return makeGroup([
    line(cx, cy - S, cx, cy + S, ICON_COLOR),
    line(cx - 3, cy - S, cx, cy - S + 3, ICON_COLOR),
    line(cx + 3, cy + S, cx, cy + S - 3, ICON_COLOR),
  ]);
}

function buildCollinearIcon(cx: number, cy: number): THREE.Group {
  return makeGroup([
    line(cx - S, cy, cx + S, cy, ICON_COLOR),
    line(cx - S + 2, cy - 2, cx - S, cy, ICON_COLOR),
    line(cx - S + 2, cy + 2, cx - S, cy, ICON_COLOR),
  ]);
}

function buildFixedIcon(cx: number, cy: number): THREE.Group {
  return makeGroup([
    line(cx - 3, cy - S, cx - 3, cy + S, ICON_COLOR),
    line(cx + 3, cy - S, cx + 3, cy + S, ICON_COLOR),
    line(cx - 3, cy + S - 2, cx + 3, cy + S - 2, ICON_COLOR),
  ]);
}

function buildLengthDim(
  model: SketchModel,
  lineId: string
): THREE.Group | null {
  const ent = model.entities.get(lineId);
  if (!ent || ent.kind !== "line") return null;
  const s = getPointCoords(model, ent.start);
  const e = getPointCoords(model, ent.end);
  if (!s || !e) return null;

  const mx = (s.x + e.x) / 2;
  const my = (s.y + e.y) / 2;
  const dx = e.x - s.x;
  const dy = e.y - s.y;
  const len = Math.sqrt(dx * dx + dy * dy) || 1;
  const nx = -dy / len * 15;
  const ny = dx / len * 15;

  const group = new THREE.Group();
  // dimension line offset from the entity
  group.add(
    line(s.x + nx, s.y + ny, e.x + nx, e.y + ny, DIM_COLOR)
  );
  // tick marks at ends
  const t = 4;
  group.add(
    line(
      s.x + nx - t, s.y + ny - t,
      s.x + nx + t, s.y + ny + t,
      DIM_COLOR
    )
  );
  group.add(
    line(
      e.x + nx - t, e.y + ny - t,
      e.x + nx + t, e.y + ny + t,
      DIM_COLOR
    )
  );
  // extension lines
  group.add(line(s.x, s.y, s.x + nx, s.y + ny, DIM_COLOR));
  group.add(line(e.x, e.y, e.x + nx, e.y + ny, DIM_COLOR));
  return group;
}

function buildAngleDim(
  model: SketchModel,
  lineAId: string,
  lineBId: string
): THREE.Group | null {
  const entA = model.entities.get(lineAId);
  const entB = model.entities.get(lineBId);
  if (!entA || !entB || entA.kind !== "line" || entB.kind !== "line") {
    return null;
  }
  const a1 = getPointCoords(model, entA.start);
  const a2 = getPointCoords(model, entA.end);
  const b1 = getPointCoords(model, entB.start);
  const b2 = getPointCoords(model, entB.end);
  if (!a1 || !a2 || !b1 || !b2) return null;

  // Use midpoint of first line as approximate center
  const cx = (a1.x + a2.x) / 2;
  const cy = (a1.y + a2.y) / 2;
  const r = 20;
  // Build a small arc
  const pts: number[] = [];
  const steps = 16;
  for (let i = 0; i <= steps; i++) {
    const theta = (i / steps) * Math.PI * 0.5;
    pts.push(cx + r * Math.cos(theta), cy + r * Math.sin(theta), 0);
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute(
    "position",
    new THREE.BufferAttribute(new Float32Array(pts), 3)
  );
  const arc = new THREE.Line(geo, new THREE.LineBasicMaterial({ color: DIM_COLOR }));
  return makeGroup([arc]);
}

function buildRadiusDim(
  model: SketchModel,
  entityId: string
): THREE.Group | null {
  const ent = model.entities.get(entityId);
  if (!ent || (ent.kind !== "circle" && ent.kind !== "arc")) return null;
  const c = getPointCoords(model, ent.center);
  if (!c) return null;

  const r = "radius" in ent ? ent.radius : 10;
  const endX = c.x + r;
  const endY = c.y;

  const group = new THREE.Group();
  // line from center to edge
  group.add(line(c.x, c.y, endX, endY, DIM_COLOR));
  // tick at edge
  group.add(
    line(endX - 3, endY - 3, endX + 3, endY + 3, DIM_COLOR)
  );
  return group;
}

/**
 * Create a Three.js visual for a constraint.
 *
 * Non-dimensional constraints render as small amber geometric icons
 * near the constrained entity center. Dimension constraints
 * (lengthDim, angleDim, radiusDim) render dimension lines with tick
 * marks in amber (#ffc107).
 *
 * Returns `null` if the constraint references missing entities.
 */
export function createConstraintVisual(
  constraint: Constraint,
  model: SketchModel
): THREE.Object3D | null {
  // --- Dimension constraints (layer 4) ---
  if (constraint.kind === "lengthDim") {
    return buildLengthDim(model, constraint.line);
  }
  if (constraint.kind === "angleDim") {
    return buildAngleDim(model, constraint.lineA, constraint.lineB);
  }
  if (constraint.kind === "radiusDim") {
    return buildRadiusDim(model, constraint.entity);
  }

  // --- Icon constraints (layer 3) ---
  // Determine a placement point from the first referenced entity
  let cx = 0, cy = 0;
  const firstId = constraint.entities[0];
  if (firstId) {
    const center = getEntityCenterById(firstId, model);
    if (center) {
      cx = center.x;
      cy = center.y;
    }
  }

  // Offset slightly so icons don't sit right on top of geometry
  cx += 10;
  cy += 10;

  switch (constraint.kind) {
    case "horizontal":
      return buildHorizontalIcon(cx, cy);
    case "vertical":
      return buildVerticalIcon(cx, cy);
    case "parallel":
      return buildParallelIcon(cx, cy);
    case "perpendicular":
      return buildPerpendicularIcon(cx, cy);
    case "tangent":
      return buildTangentIcon(cx, cy);
    case "equal":
      return buildEqualIcon(cx, cy);
    case "coincident":
      return buildCoincidentIcon(cx, cy);
    case "symmetric":
      return buildSymmetricIcon(cx, cy);
    case "collinear":
      return buildCollinearIcon(cx, cy);
    case "fixed":
      return buildFixedIcon(cx, cy);
    default:
      return null;
  }
}
