import type { SnapCandidate } from "./SnapTypes";
import type { SketchModel } from "../model/SketchModel";
import { getPointCoords } from "../model/SketchModel";

/**
 * Computes the foot of the perpendicular from point (px, py) onto the
 * line segment defined by (x1, y1)-(x2, y2).
 *
 * The result is clamped to the segment extents so the foot always lies
 * on the segment.
 *
 * @returns The perpendicular foot coordinates on the segment.
 */
function footOfPerpendicular(
  px: number,
  py: number,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): { x: number; y: number } {
  const dx = x2 - x1;
  const dy = y2 - y1;
  const lenSq = dx * dx + dy * dy;

  if (lenSq < 1e-12) return { x: x1, y: y1 };

  let t = ((px - x1) * dx + (py - y1) * dy) / lenSq;
  t = Math.max(0, Math.min(1, t));

  return {
    x: x1 + t * dx,
    y: y1 + t * dy,
  };
}

/**
 * Snaps to the perpendicular foot from the cursor onto the nearest line
 * entity within the threshold.
 *
 * @param x - Cursor X coordinate in mm.
 * @param y - Cursor Y coordinate in mm.
 * @param model - The current sketch model.
 * @param threshold - Maximum distance in mm to consider a snap (default 10).
 * @returns A perpendicular snap candidate, or null if no perpendicular foot
 *   is within range.
 */
export function perpendicularSnap(
  x: number,
  y: number,
  model: SketchModel,
  threshold = 10,
): SnapCandidate | null {
  let best: SnapCandidate | null = null;

  for (const [id, entity] of model.entities) {
    if (entity.kind !== "line") continue;

    const start = getPointCoords(model, entity.start);
    const end = getPointCoords(model, entity.end);
    if (!start || !end) continue;

    const foot = footOfPerpendicular(x, y, start.x, start.y, end.x, end.y);
    const dx = x - foot.x;
    const dy = y - foot.y;
    const distance = Math.sqrt(dx * dx + dy * dy);

    if (distance > threshold) continue;
    if (!best || distance < best.distance) {
      best = {
        kind: "perpendicular",
        x: foot.x,
        y: foot.y,
        distance,
        entityId: id,
      };
    }
  }

  return best;
}
