/**
 * Snap kind discriminator for all supported snap types.
 */
export type SnapKind = "endpoint" | "center" | "midpoint" | "intersection" | "tangent" | "perpendicular" | "grid" | "none";

/**
 * A candidate snap point returned by a snap strategy.
 */
export interface SnapCandidate {
  kind: SnapKind;
  x: number;
  y: number;
  distance: number;
  entityId?: string;
  pointId?: string;
}

/**
 * The final output of the snap engine after resolving the best candidate.
 */
export interface SnapResult {
  candidate: SnapCandidate;
  isSnapped: boolean;
  rawX: number;
  rawY: number;
}

/**
 * Lightweight geometry descriptor used for raycasting during snapping.
 */
export interface SketchGeometry {
  kind: "line" | "circle" | "arc";
  id: string;
  startX?: number;
  startY?: number;
  endX?: number;
  endY?: number;
  cx?: number;
  cy?: number;
  radius?: number;
}
