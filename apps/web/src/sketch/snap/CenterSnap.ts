import type { SnapCandidate } from "./SnapTypes";
import type { SketchModel } from "../model/SketchModel";
import { getPointCoords } from "../model/SketchModel";

/**
 * Snaps to the center point of circle and arc entities within the threshold.
 *
 * @param x - Cursor X coordinate in mm.
 * @param y - Cursor Y coordinate in mm.
 * @param model - The current sketch model.
 * @param threshold - Maximum distance in mm to consider a snap (default 8).
 * @returns A center snap candidate, or null if no center is within range.
 */
export function centerSnap(
  x: number,
  y: number,
  model: SketchModel,
  threshold = 8,
): SnapCandidate | null {
  let best: SnapCandidate | null = null;

  for (const [id, entity] of model.entities) {
    if (entity.kind !== "circle" && entity.kind !== "arc") continue;

    const center = getPointCoords(model, entity.center);
    if (!center) continue;

    const dx = x - center.x;
    const dy = y - center.y;
    const distance = Math.sqrt(dx * dx + dy * dy);

    if (distance > threshold) continue;
    if (!best || distance < best.distance) {
      best = {
        kind: "center",
        x: center.x,
        y: center.y,
        distance,
        entityId: id,
        pointId: entity.center,
      };
    }
  }

  return best;
}
