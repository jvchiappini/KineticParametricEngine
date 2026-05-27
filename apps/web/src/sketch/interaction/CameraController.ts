/**
 * Handles viewport navigation — pan, zoom, fit-to-window.
 *
 * Delegates all spatial transformations to the injected `SketchRenderer`.
 *
 * @packageDocumentation
 */

import type { SketchRenderer } from "../renderer/SketchRenderer";
import type { SketchModel } from "../model/SketchModel";

export class CameraController {
  private renderer: SketchRenderer;
  private isPanning: boolean;
  private isSpaceDown: boolean;
  private lastMouseX: number;
  private lastMouseY: number;

  constructor(renderer: SketchRenderer) {
    this.renderer = renderer;
    this.isPanning = false;
    this.isSpaceDown = false;
    this.lastMouseX = 0;
    this.lastMouseY = 0;
  }

  /**
   * Begin a pan gesture at the given screen position.
   *
   * @param screenX - Client X coordinate at pan start.
   * @param screenY - Client Y coordinate at pan start.
   */
  startPan(screenX: number, screenY: number): void {
    this.isPanning = true;
    this.lastMouseX = screenX;
    this.lastMouseY = screenY;
  }

  /**
   * Continue a pan gesture.
   *
   * @param screenX - Current client X coordinate.
   * @param screenY - Current client Y coordinate.
   */
  updatePan(screenX: number, screenY: number): void {
    if (!this.isPanning) return;
    const dx = screenX - this.lastMouseX;
    const dy = screenY - this.lastMouseY;
    this.lastMouseX = screenX;
    this.lastMouseY = screenY;
    this.renderer.pan(dx, dy);
  }

  /** End the current pan gesture. */
  endPan(): void {
    this.isPanning = false;
  }

  /**
   * Apply a zoom factor anchored at a screen position.
   *
   * @param factor  - Scale multiplier (>1 zooms in, <1 zooms out).
   * @param screenX - Anchor client X.
   * @param screenY - Anchor client Y.
   */
  zoom(factor: number, screenX: number, screenY: number): void {
    this.renderer.zoomAtPoint(factor, screenX, screenY);
  }

  /**
   * Adjust the camera so all entities are visible.
   *
   * @param model - The sketch model whose entities should be framed.
   */
  zoomToFit(model: SketchModel): void {
    this.renderer.zoomToFit(model);
  }

  /** Reset to 1:1 scale (100 mm = 100 px). */
  zoomOneToOne(): void {
    this.renderer.zoomOneToOne();
  }
}
