import type { SketchModel } from "../../model/SketchModel";
import type { SymmetricConstraint } from "../../model/Constraint";

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

/**
 * Evaluates a symmetric constraint.
 * Two entities A and B are symmetric about a symmetry line.
 * Error = signedDist(A) + signedDist(B) (equal but opposite distances).
 */
export function evaluateConstraint(
  constraint: SymmetricConstraint,
  model: SketchModel,
  variables: Map<string, number>,
  gradients: Map<string, number>
): number {
  const lineEnt = model.entities.get(constraint.symmetryLine);
  if (!lineEnt || lineEnt.kind !== "line") return 0;

  const sPt = model.points.get(lineEnt.start);
  const ePt = model.points.get(lineEnt.end);
  if (!sPt || !ePt) return 0;

  const sx = readVar(variables, `${lineEnt.start}.x`, sPt.x);
  const sy = readVar(variables, `${lineEnt.start}.y`, sPt.y);
  const ex = readVar(variables, `${lineEnt.end}.x`, ePt.x);
  const ey = readVar(variables, `${lineEnt.end}.y`, ePt.y);

  const dxL = ex - sx;
  const dyL = ey - sy;
  const lenL = Math.sqrt(dxL * dxL + dyL * dyL + 1e-10);

  const eA = model.entities.get(constraint.entityA);
  const eB = model.entities.get(constraint.entityB);
  if (!eA || !eB) return 0;
  if (eA.kind !== "point" || eB.kind !== "point") return 0;

  const aPt = model.points.get(constraint.entityA);
  const bPt = model.points.get(constraint.entityB);
  if (!aPt || !bPt) return 0;

  const ax = readVar(variables, `${constraint.entityA}.x`, aPt.x);
  const ay = readVar(variables, `${constraint.entityA}.y`, aPt.y);
  const bx = readVar(variables, `${constraint.entityB}.x`, bPt.x);
  const by = readVar(variables, `${constraint.entityB}.y`, bPt.y);

  const sda = ((ax - sx) * dyL - (ay - sy) * dxL) / lenL;
  const sdb = ((bx - sx) * dyL - (by - sy) * dxL) / lenL;

  const error = sda + sdb;

  const N = (ax - sx) * dyL - (ay - sy) * dxL + (bx - sx) * dyL - (by - sy) * dxL;
  const D = lenL;

  gradients.set(`${constraint.entityA}.x`, (gradients.get(`${constraint.entityA}.x`) ?? 0) + dyL / D);
  gradients.set(`${constraint.entityA}.y`, (gradients.get(`${constraint.entityA}.y`) ?? 0) - dxL / D);
  gradients.set(`${constraint.entityB}.x`, (gradients.get(`${constraint.entityB}.x`) ?? 0) + dyL / D);
  gradients.set(`${constraint.entityB}.y`, (gradients.get(`${constraint.entityB}.y`) ?? 0) - dxL / D);

  const dN_dSx = -2 * dyL + (dxL) + (dxL) + (ay - sy) + (by - sy);
  const dD_dSx = -dxL / D;

  const dN_dSy = (ax - sx) + (bx - sx) + 2 * dxL;
  const dD_dSy = -dyL / D;

  const dN_dEx = (ax - sx) - (ay - sy) + (bx - sx) - (by - sy);
  const dD_dEx = dxL / D;

  const dN_dEy = (ax - sx) + (bx - sx);
  const dD_dEy = dyL / D;

  gradients.set(`${lineEnt.start}.x`, (gradients.get(`${lineEnt.start}.x`) ?? 0) + (dN_dSx * D - N * dD_dSx) / (D * D));
  gradients.set(`${lineEnt.start}.y`, (gradients.get(`${lineEnt.start}.y`) ?? 0) + (dN_dSy * D - N * dD_dSy) / (D * D));
  gradients.set(`${lineEnt.end}.x`, (gradients.get(`${lineEnt.end}.x`) ?? 0) + (dN_dEx * D - N * dD_dEx) / (D * D));
  gradients.set(`${lineEnt.end}.y`, (gradients.get(`${lineEnt.end}.y`) ?? 0) + (dN_dEy * D - N * dD_dEy) / (D * D));

  return error;
}
