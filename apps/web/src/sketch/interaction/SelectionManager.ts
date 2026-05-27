/**
 * Manages selected entity and constraint identifiers.
 *
 * Supports single / toggle selection and box (rectangle) selection.
 *
 * @packageDocumentation
 */

import type { EntityId } from "../model/Entity";
import type { ConstraintId } from "../model/Constraint";
import type { SketchModel } from "../model/SketchModel";
import { getPointCoords } from "../model/SketchModel";

export class SelectionManager {
  private selectedEntities: Set<EntityId> = new Set();
  private selectedConstraints: Set<ConstraintId> = new Set();
  private boxSelectStart: { x: number; y: number } | null = null;

  /** Select a single entity, clearing previous selections. */
  select(entityId: EntityId): void {
    this.selectedEntities.clear();
    this.selectedConstraints.clear();
    this.selectedEntities.add(entityId);
  }

  /** Remove an entity from the selection. */
  deselect(entityId: EntityId): void {
    this.selectedEntities.delete(entityId);
  }

  /** Toggle an entity in the current selection. */
  toggle(entityId: EntityId): void {
    if (this.selectedEntities.has(entityId)) {
      this.selectedEntities.delete(entityId);
    } else {
      this.selectedEntities.add(entityId);
    }
  }

  /** Clear all selections. */
  clear(): void {
    this.selectedEntities.clear();
    this.selectedConstraints.clear();
  }

  /** Check whether an entity is currently selected. */
  isSelected(entityId: EntityId): boolean {
    return this.selectedEntities.has(entityId);
  }

  /** Return a copy of the selected entity ID array. */
  getSelectedEntities(): EntityId[] {
    return Array.from(this.selectedEntities);
  }

  /** Return a copy of the selected constraint ID array. */
  getSelectedConstraints(): ConstraintId[] {
    return Array.from(this.selectedConstraints);
  }

  /** Select an individual constraint. */
  selectConstraint(constraintId: ConstraintId): void {
    this.selectedConstraints.clear();
    this.selectedEntities.clear();
    this.selectedConstraints.add(constraintId);
  }

  /**
   * Find all entities whose geometry falls inside the rectangle defined
   * by the two corner points (world coordinates).
   *
   * @param x1 - First corner X.
   * @param y1 - First corner Y.
   * @param x2 - Second corner X.
   * @param y2 - Second corner Y.
   * @param model - The sketch model to search.
   * @returns Array of entity IDs inside the rectangle.
   */
  boxSelect(
    x1: number, y1: number,
    x2: number, y2: number,
    model: SketchModel,
  ): EntityId[] {
    const minX = Math.min(x1, x2);
    const maxX = Math.max(x1, x2);
    const minY = Math.min(y1, y2);
    const maxY = Math.max(y1, y2);
    const inside: EntityId[] = [];

    for (const [id, entity] of model.entities) {
      switch (entity.kind) {
        case "point": {
          if (
            entity.x >= minX && entity.x <= maxX &&
            entity.y >= minY && entity.y <= maxY
          ) {
            inside.push(id);
          }
          break;
        }
        case "line": {
          const s = getPointCoords(model, entity.start);
          const e = getPointCoords(model, entity.end);
          if (!s || !e) break;
          if (
            (s.x >= minX && s.x <= maxX && s.y >= minY && s.y <= maxY) ||
            (e.x >= minX && e.x <= maxX && e.y >= minY && e.y <= maxY)
          ) {
            inside.push(id);
          }
          break;
        }
        case "circle":
        case "arc": {
          const c = getPointCoords(model, entity.center);
          if (!c) break;
          if (
            c.x >= minX && c.x <= maxX &&
            c.y >= minY && c.y <= maxY
          ) {
            inside.push(id);
          }
          break;
        }
        case "polyline": {
          for (const ptId of entity.points) {
            const pt = getPointCoords(model, ptId);
            if (
              pt &&
              pt.x >= minX && pt.x <= maxX &&
              pt.y >= minY && pt.y <= maxY
            ) {
              inside.push(id);
              break;
            }
          }
          break;
        }
      }
    }

    return inside;
  }

  /** Record the start point of a box selection (screen coordinates). */
  startBoxSelect(x: number, y: number): void {
    this.boxSelectStart = { x, y };
  }

  /**
   * Given the current pointer position, return the normalized box
   * rectangle, or `null` if no box selection is in progress.
   *
   * @param x - Current pointer X.
   * @param y - Current pointer Y.
   */
  getBoxRect(
    x: number,
    y: number,
  ): { x1: number; y1: number; x2: number; y2: number } | null {
    if (!this.boxSelectStart) return null;
    return {
      x1: Math.min(this.boxSelectStart.x, x),
      y1: Math.min(this.boxSelectStart.y, y),
      x2: Math.max(this.boxSelectStart.x, x),
      y2: Math.max(this.boxSelectStart.y, y),
    };
  }

  /** The total number of selected items (entities + constraints). */
  get size(): number {
    return this.selectedEntities.size + this.selectedConstraints.size;
  }
}
