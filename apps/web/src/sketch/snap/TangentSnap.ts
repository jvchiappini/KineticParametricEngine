import type { SnapCandidate } from "./SnapTypes";
import type { SketchModel } from "../model/SketchModel";
import { getPointCoords } from "../model/SketchModel";

/**
 * Computes the two tangent points from an external point (px, py) to a
 * circle centered at (cx, cy) with the given radius.
 *
 * Returns an array of 0, 1, or 2 tangent points depending on the geometry:
 *   - 0 when the point lies inside the circle.
 *   - 1 when the point lies exactly on the circle.
 *   - 2 when the point is outside the circle.
 */
function tangentPointsFromExternalPoint(
  px: number,
  py: number,
  cx: number,
  cy: number,
  radius: number,
): { x: number; y: number }[] {
  const dx = px - cx;
  const dy = py - cy;
  const d = Math.sqrt(dx * dx + dy * dy);

  if (d < radius - 1e-10) return [];
  if (Math.abs(d - radius) < 1e-10) return [{ x: px, y: py }];

  const baseAngle = Math.atan2(dy, dx);
  const theta = Math.acos(radius / d);

  return [
    {
      x: cx + radius * Math.cos(baseAngle + theta),
      y: cy + radius * Math.sin(baseAngle + theta),
    },
    {
      x: cx + radius * Math.cos(baseAngle - theta),
      y: cy + radius * Math.sin(baseAngle - theta),
    },
  ];
}

/**
 * Snaps to the nearest tangent point from the cursor onto a circle or
 * arc entity within the threshold.
 *
 * @param x - Cursor X coordinate in mm.
 * @param y - Cursor Y coordinate in mm.
 * @param model - The current sketch model.
 * @param threshold - Maximum distance in mm to consider a snap (default 10).
 * @returns A tangent snap candidate, or null if no tangent point is within
 *   range.
 */
export function tangentSnap(
  x: number,
  y: number,
  model: SketchModel,
  threshold = 10,
): SnapCandidate | null {
  let best: SnapCandidate | null = null;

  for (const [id, entity] of model.entities) {
    if (entity.kind !== "circle" && entity.kind !== "arc") continue;

    const center = getPointCoords(model, entity.center);
    if (!center) continue;

    const tangents = tangentPointsFromExternalPoint(
      x,
      y,
      center.x,
      center.y,
      entity.radius,
    );

    for (const pt of tangents) {
      const dx = x - pt.x;
      const dy = y - pt.y;
      const distance = Math.sqrt(dx * dx + dy * dy);

      if (distance > threshold) continue;
      if (!best || distance < best.distance) {
        best = {
          kind: "tangent",
          x: pt.x,
          y: pt.y,
          distance,
          entityId: id,
        };
      }
    }
  }

  return best;
}
