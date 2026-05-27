import type { SketchModel } from "../../model/SketchModel";
import type { CoincidentConstraint } from "../../model/Constraint";

function readVar(variables: Map<string, number>, key: string, fallback: number): number {
  const v = variables.get(key);
  return v !== undefined ? v : fallback;
}

/**
 * Evaluates a coincident constraint between two points.
 * Error is the Euclidean distance between the points.
 * Gradient moves each point toward the other.
 */
export function evaluateConstraint(
  constraint: CoincidentConstraint,
  model: SketchModel,
  variables: Map<string, number>,
  gradients: Map<string, number>
): number {
  const p1 = model.points.get(constraint.pointA);
  const p2 = model.points.get(constraint.pointB);
  if (!p1 || !p2) return 0;

  const x1 = readVar(variables, `${constraint.pointA}.x`, p1.x);
  const y1 = readVar(variables, `${constraint.pointA}.y`, p1.y);
  const x2 = readVar(variables, `${constraint.pointB}.x`, p2.x);
  const y2 = readVar(variables, `${constraint.pointB}.y`, p2.y);

  const dx = x1 - x2;
  const dy = y1 - y2;
  const dist = Math.sqrt(dx * dx + dy * dy + 1e-10);

  const kAx = `${constraint.pointA}.x`;
  const kAy = `${constraint.pointA}.y`;
  const kBx = `${constraint.pointB}.x`;
  const kBy = `${constraint.pointB}.y`;

  const gx = dx / dist;
  const gy = dy / dist;

  gradients.set(kAx, (gradients.get(kAx) ?? 0) + gx);
  gradients.set(kAy, (gradients.get(kAy) ?? 0) + gy);
  gradients.set(kBx, (gradients.get(kBx) ?? 0) - gx);
  gradients.set(kBy, (gradients.get(kBy) ?? 0) - gy);

  return dist;
}
