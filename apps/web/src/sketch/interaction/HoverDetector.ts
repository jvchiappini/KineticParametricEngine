/**
 * Detects the entity or constraint closest to the cursor position.
 *
 * @packageDocumentation
 */

import type { SketchModel } from "../model/SketchModel";
import type { EntityId } from "../model/Entity";
import type { ConstraintId } from "../model/Constraint";
import { getPointCoords } from "../model/SketchModel";

export interface HoverResult {
  entityId?: EntityId;
  constraintId?: ConstraintId;
  pointId?: EntityId;
  distance: number;
  x: number;
  y: number;
}

/**
 * Returns the squared distance between `(px, py)` and the line segment
 * `(x1, y1) — (x2, y2)`, along with the closest point on the segment.
 */
function closestOnSegment(
  px: number,
  py: number,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): { distSq: number; x: number; y: number } {
  const dx = x2 - x1;
  const dy = y2 - y1;
  const lenSq = dx * dx + dy * dy;
  if (lenSq === 0) {
    const cx = px - x1;
    const cy = py - y1;
    return { distSq: cx * cx + cy * cy, x: x1, y: y1 };
  }
  let t = ((px - x1) * dx + (py - y1) * dy) / lenSq;
  t = Math.max(0, Math.min(1, t));
  const nx = x1 + t * dx;
  const ny = y1 + t * dy;
  const cx = px - nx;
  const cy = py - ny;
  return { distSq: cx * cx + cy * cy, x: nx, y: ny };
}

/**
 * Iterate all entities in the model and return the one whose geometry
 * is nearest to the cursor, provided it is within `threshold` mm.
 *
 * @param model     - The sketch model to search.
 * @param cursorX   - Cursor X in world coordinates.
 * @param cursorY   - Cursor Y in world coordinates.
 * @param threshold - Maximum distance in mm for a hit.
 * @returns The closest hover result, or `null` if nothing is close enough.
 */
export function detectHover(
  model: SketchModel,
  cursorX: number,
  cursorY: number,
  threshold: number,
): HoverResult | null {
  let best: HoverResult | null = null;
  const thresholdSq = threshold * threshold;

  for (const [id, entity] of model.entities) {
    switch (entity.kind) {
      case "point": {
        const dx = cursorX - entity.x;
        const dy = cursorY - entity.y;
        const dSq = dx * dx + dy * dy;
        if (dSq <= thresholdSq && (!best || dSq < best.distance)) {
          best = {
            entityId: id,
            pointId: id,
            distance: dSq,
            x: entity.x,
            y: entity.y,
          };
        }
        break;
      }

      case "line": {
        const s = getPointCoords(model, entity.start);
        const e = getPointCoords(model, entity.end);
        if (!s || !e) break;
        const { distSq, x, y } = closestOnSegment(
          cursorX, cursorY,
          s.x, s.y,
          e.x, e.y,
        );
        if (distSq <= thresholdSq && (!best || distSq < best.distance)) {
          best = { entityId: id, distance: distSq, x, y };
        }
        break;
      }

      case "circle": {
        const c = getPointCoords(model, entity.center);
        if (!c) break;
        const dx = cursorX - c.x;
        const dy = cursorY - c.y;
        const distToCenter = Math.sqrt(dx * dx + dy * dy);
        const radialDist = Math.abs(distToCenter - entity.radius);
        const dSq = radialDist * radialDist;
        if (dSq <= thresholdSq && (!best || dSq < best.distance)) {
          const angle = Math.atan2(dy, dx);
          const px = c.x + entity.radius * Math.cos(angle);
          const py = c.y + entity.radius * Math.sin(angle);
          best = { entityId: id, distance: dSq, x: px, y: py };
        }
        break;
      }

      case "arc": {
        const c = getPointCoords(model, entity.center);
        if (!c) break;
        const dx = cursorX - c.x;
        const dy = cursorY - c.y;
        const distToCenter = Math.sqrt(dx * dx + dy * dy);
        const angle = Math.atan2(dy, dx);

        let a = angle % (2 * Math.PI);
        if (a < 0) a += 2 * Math.PI;
        let sa = entity.startAngle % (2 * Math.PI);
        if (sa < 0) sa += 2 * Math.PI;
        let ea = entity.endAngle % (2 * Math.PI);
        if (ea < 0) ea += 2 * Math.PI;

        const onArc = sa <= ea ? (a >= sa && a <= ea) : (a >= sa || a <= ea);
        if (onArc) {
          const radialDist = Math.abs(distToCenter - entity.radius);
          const dSq = radialDist * radialDist;
          if (dSq <= thresholdSq && (!best || dSq < best.distance)) {
            const px = c.x + entity.radius * Math.cos(angle);
            const py = c.y + entity.radius * Math.sin(angle);
            best = { entityId: id, distance: dSq, x: px, y: py };
          }
        }

        for (const ptId of [entity.startPoint, entity.endPoint]) {
          const pt = getPointCoords(model, ptId);
          if (!pt) continue;
          const ex = cursorX - pt.x;
          const ey = cursorY - pt.y;
          const dSq = ex * ex + ey * ey;
          if (dSq <= thresholdSq && (!best || dSq < best.distance)) {
            best = {
              entityId: id,
              pointId: ptId,
              distance: dSq,
              x: pt.x,
              y: pt.y,
            };
          }
        }
        break;
      }

      case "polyline": {
        const coords: { x: number; y: number }[] = [];
        for (const ptId of entity.points) {
          const pt = getPointCoords(model, ptId);
          if (pt) coords.push(pt);
        }
        if (coords.length < 2) break;

        for (let i = 0; i < coords.length - 1; i++) {
          const { distSq, x, y } = closestOnSegment(
            cursorX, cursorY,
            coords[i].x, coords[i].y,
            coords[i + 1].x, coords[i + 1].y,
          );
          if (distSq <= thresholdSq && (!best || distSq < best.distance)) {
            best = { entityId: id, distance: distSq, x, y };
          }
        }

        if (entity.closed && coords.length > 2) {
          const last = coords[coords.length - 1];
          const first = coords[0];
          const { distSq, x, y } = closestOnSegment(
            cursorX, cursorY,
            last.x, last.y,
            first.x, first.y,
          );
          if (distSq <= thresholdSq && (!best || distSq < best.distance)) {
            best = { entityId: id, distance: distSq, x, y };
          }
        }
        break;
      }
    }
  }

  if (best) {
    best.distance = Math.sqrt(best.distance);
  }
  return best;
}
