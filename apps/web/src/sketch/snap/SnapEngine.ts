import type { SnapCandidate, SnapResult, SketchGeometry } from "./SnapTypes";
import type { SketchModel } from "../model/SketchModel";
import { endpointSnap } from "./EndpointSnap";
import { centerSnap } from "./CenterSnap";
import { midpointSnap } from "./MidpointSnap";
import { intersectionSnap } from "./IntersectionSnap";
import { perpendicularSnap } from "./PerpendicularSnap";
import { tangentSnap } from "./TangentSnap";
import { gridSnap } from "./GridSnap";

/**
 * Runs all registered snap strategies in priority order and returns the
 * best (closest distance) snap candidate for the given cursor position.
 *
 * Priority order (highest first):
 *   1. Endpoint
 *   2. Center
 *   3. Midpoint
 *   4. Intersection
 *   5. Perpendicular
 *   6. Tangent
 *   7. Grid
 *
 * @param x - Cursor X coordinate in mm.
 * @param y - Cursor Y coordinate in mm.
 * @param model - The current sketch model containing points, entities, and
 *   constraints.
 * @param gridSpacing - Spacing between grid lines in mm (0 or negative to
 *   disable grid snap).
 * @param activeTool - Name of the currently active tool (may influence
 *   strategy selection or thresholds in future use).
 * @param _geometry - Optional array of lightweight geometry descriptors used
 *   for raycasting (reserved for future use).
 * @returns A SnapResult containing the best candidate found and metadata.
 */
export function resolveSnap(
  x: number,
  y: number,
  model: SketchModel,
  gridSpacing: number,
  activeTool: string,
  _geometry: SketchGeometry[] = [],
): SnapResult {
  let best: SnapCandidate | null = null;

  const candidates: (SnapCandidate | null)[] = [
    endpointSnap(x, y, model),
    centerSnap(x, y, model),
    midpointSnap(x, y, model),
    intersectionSnap(x, y, model),
    perpendicularSnap(x, y, model),
    tangentSnap(x, y, model),
    gridSnap(x, y, gridSpacing),
  ];

  for (const candidate of candidates) {
    if (candidate && (!best || candidate.distance < best.distance)) {
      best = candidate;
    }
  }

  return {
    candidate: best ?? { kind: "none", x, y, distance: 0 },
    isSnapped: best !== null,
    rawX: x,
    rawY: y,
  };
}
