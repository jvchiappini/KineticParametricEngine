import type { SnapCandidate } from "./SnapTypes";
import type { SketchModel } from "../model/SketchModel";
import { getPointCoords } from "../model/SketchModel";

interface LineSegment {
  id: string;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

/**
 * Computes the intersection point of two line segments using the
 * standard parametric line intersection formula.
 *
 * Returns null if the segments are parallel or do not intersect within
 * their extents.
 */
function intersectLines(
  a: LineSegment,
  b: LineSegment,
): { x: number; y: number } | null {
  const denom = (a.x1 - a.x2) * (b.y1 - b.y2) - (a.y1 - a.y2) * (b.x1 - b.x2);

  if (Math.abs(denom) < 1e-12) return null;

  const t =
    ((a.x1 - b.x1) * (b.y1 - b.y2) - (a.y1 - b.y1) * (b.x1 - b.x2)) / denom;
  const u =
    -((a.x1 - a.x2) * (a.y1 - b.y1) - (a.y1 - a.y2) * (a.x1 - b.x1)) / denom;

  if (t < 0 || t > 1 || u < 0 || u > 1) return null;

  return {
    x: a.x1 + t * (a.x2 - a.x1),
    y: a.y1 + t * (a.y2 - a.y1),
  };
}

/**
 * Snaps to the intersection of any two line entities within the threshold.
 *
 * @param x - Cursor X coordinate in mm.
 * @param y - Cursor Y coordinate in mm.
 * @param model - The current sketch model.
 * @param threshold - Maximum distance in mm to consider a snap (default 10).
 * @returns An intersection snap candidate, or null if no intersection is within range.
 */
export function intersectionSnap(
  x: number,
  y: number,
  model: SketchModel,
  threshold = 10,
): SnapCandidate | null {
  const segments: LineSegment[] = [];

  for (const [id, entity] of model.entities) {
    if (entity.kind !== "line") continue;

    const start = getPointCoords(model, entity.start);
    const end = getPointCoords(model, entity.end);
    if (!start || !end) continue;

    segments.push({
      id,
      x1: start.x,
      y1: start.y,
      x2: end.x,
      y2: end.y,
    });
  }

  let best: SnapCandidate | null = null;

  for (let i = 0; i < segments.length; i++) {
    for (let j = i + 1; j < segments.length; j++) {
      const pt = intersectLines(segments[i], segments[j]);
      if (!pt) continue;

      const dx = x - pt.x;
      const dy = y - pt.y;
      const distance = Math.sqrt(dx * dx + dy * dy);

      if (distance > threshold) continue;
      if (!best || distance < best.distance) {
        best = {
          kind: "intersection",
          x: pt.x,
          y: pt.y,
          distance,
        };
      }
    }
  }

  return best;
}
