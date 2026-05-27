import type { SnapCandidate } from "./SnapTypes";

/**
 * Computes the nearest grid intersection point.
 *
 * @param x - Cursor X coordinate in mm.
 * @param y - Cursor Y coordinate in mm.
 * @param gridSpacing - Spacing between grid lines in mm. Must be > 0.
 * @returns A grid snap candidate, or null if gridSpacing <= 0.
 */
export function gridSnap(
  x: number,
  y: number,
  gridSpacing: number,
): SnapCandidate | null {
  if (gridSpacing <= 0) return null;

  const gx = Math.round(x / gridSpacing) * gridSpacing;
  const gy = Math.round(y / gridSpacing) * gridSpacing;
  const dx = x - gx;
  const dy = y - gy;
  const distance = Math.sqrt(dx * dx + dy * dy);

  return {
    kind: "grid",
    x: gx,
    y: gy,
    distance,
  };
}
