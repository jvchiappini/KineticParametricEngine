import type { SnapCandidate } from "./SnapTypes";
import type { SketchModel } from "../model/SketchModel";
import { getPointCoords } from "../model/SketchModel";

/**
 * Snaps to the midpoint of line and arc entities within the threshold.
 *
 * For lines the midpoint is the average of start/end coordinates.
 * For arcs the midpoint is the point at the average of startAngle and endAngle
 * along the arc radius.
 *
 * @param x - Cursor X coordinate in mm.
 * @param y - Cursor Y coordinate in mm.
 * @param model - The current sketch model.
 * @param threshold - Maximum distance in mm to consider a snap (default 8).
 * @returns A midpoint snap candidate, or null if no midpoint is within range.
 */
export function midpointSnap(
  x: number,
  y: number,
  model: SketchModel,
  threshold = 8,
): SnapCandidate | null {
  let best: SnapCandidate | null = null;

  for (const [id, entity] of model.entities) {
    if (entity.kind === "line") {
      const start = getPointCoords(model, entity.start);
      const end = getPointCoords(model, entity.end);
      if (!start || !end) continue;

      const mx = (start.x + end.x) / 2;
      const my = (start.y + end.y) / 2;
      const dx = x - mx;
      const dy = y - my;
      const distance = Math.sqrt(dx * dx + dy * dy);

      if (distance > threshold) continue;
      if (!best || distance < best.distance) {
        best = {
          kind: "midpoint",
          x: mx,
          y: my,
          distance,
          entityId: id,
        };
      }
    } else if (entity.kind === "arc") {
      const center = getPointCoords(model, entity.center);
      if (!center) continue;

      const midAngle = (entity.startAngle + entity.endAngle) / 2;
      const mx = center.x + entity.radius * Math.cos(midAngle);
      const my = center.y + entity.radius * Math.sin(midAngle);
      const dx = x - mx;
      const dy = y - my;
      const distance = Math.sqrt(dx * dx + dy * dy);

      if (distance > threshold) continue;
      if (!best || distance < best.distance) {
        best = {
          kind: "midpoint",
          x: mx,
          y: my,
          distance,
          entityId: id,
        };
      }
    }
  }

  return best;
}
