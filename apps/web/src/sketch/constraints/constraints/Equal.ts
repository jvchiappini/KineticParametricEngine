import type { SketchModel } from "../../model/SketchModel";
import type { EqualConstraint } from "../../model/Constraint";

function readVar(variables: Map<string, number>, key: string, fallback: number): number {
  const v = variables.get(key);
  return v !== undefined ? v : fallback;
}

function getCoords(
  model: SketchModel,
  variables: Map<string, number>,
  pointId: string
): { x: number; y: number } {
  const pt = model.points.get(pointId);
  if (!pt) return { x: 0, y: 0 };
  return {
    x: readVar(variables, `${pointId}.x`, pt.x),
    y: readVar(variables, `${pointId}.y`, pt.y)
  };
}

function lineLength(
  model: SketchModel,
  variables: Map<string, number>,
  lineId: string
): number {
  const ent = model.entities.get(lineId);
  if (!ent || ent.kind !== "line") return 0;
  const s = getCoords(model, variables, ent.start);
  const e = getCoords(model, variables, ent.end);
  const dx = e.x - s.x;
  const dy = e.y - s.y;
  return Math.sqrt(dx * dx + dy * dy + 1e-10);
}

function getRadius(
  model: SketchModel,
  variables: Map<string, number>,
  entityId: string
): number {
  const ent = model.entities.get(entityId);
  if (!ent) return 0;
  const key = `${entityId}.radius`;
  if (ent.kind === "circle" || ent.kind === "arc") {
    return readVar(variables, key, "radius" in ent ? (ent as any).radius : 0);
  }
  return 0;
}

/**
 * Evaluates an equal constraint between two entities.
 * For lines: error = len1 - len2.
 * For circles/arcs: error = radius1 - radius2.
 */
export function evaluateConstraint(
  constraint: EqualConstraint,
  model: SketchModel,
  variables: Map<string, number>,
  gradients: Map<string, number>
): number {
  const eA = model.entities.get(constraint.entityA);
  const eB = model.entities.get(constraint.entityB);
  if (!eA || !eB) return 0;

  if (eA.kind === "line" && eB.kind === "line") {
    const len1 = lineLength(model, variables, constraint.entityA);
    const len2 = lineLength(model, variables, constraint.entityB);
    const error = len1 - len2;

    const a1 = getCoords(model, variables, eA.start);
    const a2 = getCoords(model, variables, eA.end);
    const b1 = getCoords(model, variables, eB.start);
    const b2 = getCoords(model, variables, eB.end);

    const dx1 = a2.x - a1.x;
    const dy1 = a2.y - a1.y;
    const dx2 = b2.x - b1.x;
    const dy2 = b2.y - b1.y;

    gradients.set(`${eA.start}.x`, (gradients.get(`${eA.start}.x`) ?? 0) - dx1 / len1);
    gradients.set(`${eA.start}.y`, (gradients.get(`${eA.start}.y`) ?? 0) - dy1 / len1);
    gradients.set(`${eA.end}.x`, (gradients.get(`${eA.end}.x`) ?? 0) + dx1 / len1);
    gradients.set(`${eA.end}.y`, (gradients.get(`${eA.end}.y`) ?? 0) + dy1 / len1);

    gradients.set(`${eB.start}.x`, (gradients.get(`${eB.start}.x`) ?? 0) + dx2 / len2);
    gradients.set(`${eB.start}.y`, (gradients.get(`${eB.start}.y`) ?? 0) + dy2 / len2);
    gradients.set(`${eB.end}.x`, (gradients.get(`${eB.end}.x`) ?? 0) - dx2 / len2);
    gradients.set(`${eB.end}.y`, (gradients.get(`${eB.end}.y`) ?? 0) - dy2 / len2);

    return error;
  }

  if ((eA.kind === "circle" || eA.kind === "arc") && (eB.kind === "circle" || eB.kind === "arc")) {
    const r1 = getRadius(model, variables, constraint.entityA);
    const r2 = getRadius(model, variables, constraint.entityB);
    const error = r1 - r2;

    gradients.set(`${constraint.entityA}.radius`, (gradients.get(`${constraint.entityA}.radius`) ?? 0) + 1);
    gradients.set(`${constraint.entityB}.radius`, (gradients.get(`${constraint.entityB}.radius`) ?? 0) - 1);

    return error;
  }

  return 0;
}
