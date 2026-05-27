/**
 * Bridges mouse / keyboard events to the active sketch tool.
 *
 * Manages event listeners on the canvas element and handles viewport
 * navigation (pan via space+left-drag or middle-mouse, zoom via wheel).
 *
 * @packageDocumentation
 */

import type { SketchTool, ToolContext } from "../tools/types";

export class InputHandler {
  private canvas: HTMLElement;
  private activeTool: SketchTool | null;
  private context: ToolContext;
  private onSwitchTool: ((name: string) => void) | null;

  /** @internal Track space key state for space+drag pan. */
  private isSpaceDown = false;
  /** @internal Track whether we are currently panning. */
  private isPanning = false;
  /** @internal Last client coordinates while panning. */
  private lastPanX = 0;
  private lastPanY = 0;

  /** Bound event handlers for cleanup. */
  private readonly onPointerDown: (e: PointerEvent) => void;
  private readonly onPointerMove: (e: PointerEvent) => void;
  private readonly onPointerUp: (e: PointerEvent) => void;
  private readonly onKeyDown: (e: KeyboardEvent) => void;
  private readonly onKeyUp: (e: KeyboardEvent) => void;
  private readonly onWheel: (e: WheelEvent) => void;
  private readonly onContextMenu: (e: MouseEvent) => void;

  constructor(canvas: HTMLElement, context: ToolContext, onSwitchTool?: (name: string) => void) {
    this.canvas = canvas;
    this.context = context;
    this.activeTool = null;
    this.onSwitchTool = onSwitchTool ?? null;

    this.onPointerDown = (e: PointerEvent) => {
      if (e.button === 1 || (e.button === 0 && this.isSpaceDown)) {
        this.isPanning = true;
        this.lastPanX = e.clientX;
        this.lastPanY = e.clientY;
        this.canvas.style.cursor = "grabbing";
        return;
      }
      if (e.button !== 0) return;
      if (this.activeTool) {
        this.activeTool.onPointerDown(e, this.context);
      }
    };

    this.onPointerMove = (e: PointerEvent) => {
      if (this.isPanning) {
        const dx = e.clientX - this.lastPanX;
        const dy = e.clientY - this.lastPanY;
        this.lastPanX = e.clientX;
        this.lastPanY = e.clientY;
        this.context.onPan?.(dx, dy);
        return;
      }
      this.context.updateCursor(e.clientX, e.clientY);
      if (this.activeTool) {
        this.activeTool.onPointerMove(e, this.context);
      }
    };

    this.onPointerUp = (e: PointerEvent) => {
      if (this.isPanning) {
        this.isPanning = false;
        this.canvas.style.cursor = this.isSpaceDown ? "grab" : "";
        return;
      }
      if (this.activeTool) {
        this.activeTool.onPointerUp(e, this.context);
      }
    };

    this.onKeyDown = (e: KeyboardEvent) => {
      if (e.key === " " && !e.repeat) {
        this.isSpaceDown = true;
        this.canvas.style.cursor = "grab";
        e.preventDefault();
        return;
      }
      if (e.key === "Escape") {
        this.onSwitchTool?.("select");
        e.preventDefault();
        return;
      }
      if (e.ctrlKey && e.key.toLowerCase() === "z" && !e.shiftKey) {
        e.preventDefault();
        this.context.history.undo(this.context.model);
        return;
      }
      if ((e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "z") || (e.ctrlKey && e.key.toLowerCase() === "y")) {
        e.preventDefault();
        this.context.history.redo(this.context.model);
        return;
      }
      if (e.ctrlKey && e.key.toLowerCase() === "a") {
        e.preventDefault();
        const allIds = Array.from(this.context.model.entities.keys());
        this.context.addToSelection(allIds);
        return;
      }
      if (e.key === "f" && !e.ctrlKey) {
        e.preventDefault();
        this.context.renderer.zoomToFit(this.context.model);
        this.context.renderer.render();
        return;
      }
      const toolShortcuts: Record<string, string> = {
        s: "select", l: "line", r: "rectangle", c: "circle",
        a: "arc", p: "polyline", d: "dimension",
      };
      const toolName = toolShortcuts[e.key.toLowerCase()];
      if (toolName && !e.ctrlKey && !e.metaKey) {
        this.onSwitchTool?.(toolName);
        e.preventDefault();
        return;
      }
      if (this.activeTool) {
        this.activeTool.onKeyDown(e, this.context);
      }
    };

    this.onKeyUp = (e: KeyboardEvent) => {
      if (e.key === " ") {
        this.isSpaceDown = false;
        if (!this.isPanning) {
          this.canvas.style.cursor = "";
        }
      }
    };

    this.onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const factor = e.deltaY < 0 ? 0.9 : 1.1;
      this.context.onZoom?.(factor, e.clientX, e.clientY);
    };

    this.onContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      this.context.onContextMenu?.(e.clientX, e.clientY);
    };
  }

  /**
   * Update the active tool that receives events.
   *
   * @param tool - The new tool, or `null` to disable tool handling.
   */
  setTool(tool: SketchTool | null): void {
    this.activeTool = tool;
    this.canvas.style.cursor = tool?.cursor ?? "";
  }

  /**
   * Replace the tool context (e.g. after the model changes).
   *
   * @param context - The new context.
   */
  setContext(context: ToolContext): void {
    this.context = context;
  }

  /** Attach all event listeners to the canvas. */
  attach(): void {
    this.canvas.addEventListener("pointerdown", this.onPointerDown);
    this.canvas.addEventListener("pointermove", this.onPointerMove);
    this.canvas.addEventListener("pointerup", this.onPointerUp);
    document.addEventListener("keydown", this.onKeyDown);
    document.addEventListener("keyup", this.onKeyUp);
    this.canvas.addEventListener("wheel", this.onWheel, { passive: false });
    this.canvas.addEventListener("contextmenu", this.onContextMenu);
  }

  /** Detach all event listeners and reset pan/space state. */
  detach(): void {
    this.canvas.removeEventListener("pointerdown", this.onPointerDown);
    this.canvas.removeEventListener("pointermove", this.onPointerMove);
    this.canvas.removeEventListener("pointerup", this.onPointerUp);
    document.removeEventListener("keydown", this.onKeyDown);
    document.removeEventListener("keyup", this.onKeyUp);
    this.canvas.removeEventListener("wheel", this.onWheel);
    this.canvas.removeEventListener("contextmenu", this.onContextMenu);
    this.isPanning = false;
    this.isSpaceDown = false;
    this.canvas.style.cursor = "";
  }
}
