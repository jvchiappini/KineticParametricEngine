import type { SnapCandidate } from "./SnapTypes";
import type { SketchModel } from "../model/SketchModel";

/**
 * Snaps to the nearest point entity in the sketch model within the threshold.
 *
 * @param x - Cursor X coordinate in mm.
 * @param y - Cursor Y coordinate in mm.
 * @param model - The current sketch model.
 * @param threshold - Maximum distance in mm to consider a snap (default 8).
 * @returns An endpoint snap candidate, or null if no point is within range.
 */
export function endpointSnap(
  x: number,
  y: number,
  model: SketchModel,
  threshold = 8,
): SnapCandidate | null {
  let best: SnapCandidate | null = null;

  for (const [id, pt] of model.points) {
    const dx = x - pt.x;
    const dy = y - pt.y;
    const distance = Math.sqrt(dx * dx + dy * dy);

    if (distance > threshold) continue;
    if (!best || distance < best.distance) {
      best = {
        kind: "endpoint",
        x: pt.x,
        y: pt.y,
        distance,
        entityId: id,
        pointId: id,
      };
    }
  }

  return best;
}
