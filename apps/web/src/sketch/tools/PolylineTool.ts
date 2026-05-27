import * as THREE from "three";
import type { SketchTool, ToolContext } from "./types";
import type { PointEntity } from "../model/Entity";
import type { Command } from "../model/SketchHistory";
import type { SketchModel } from "../model/SketchModel";
import { addEntity, removeEntity } from "../model/SketchModel";
import { makePoint, makePolyline } from "../model/Entity";
import { compoundCommand } from "../model/SketchHistory";

const CLOSE_THRESHOLD = 5;

/**
 * Polyline tool. Click to add vertices, double-click or Enter to close.
 * When the cursor is near the first vertex, a close indicator appears.
 */
export class PolylineTool implements SketchTool {
  readonly name = "polyline";
  readonly cursor = "crosshair";

  private state: "idle" | "placing" = "idle";
  private points: PointEntity[] = [];
  private startCoords: { x: number; y: number } | null = null;
  private lastClickTime = 0;

  onActivate(_context: ToolContext): void { this.reset(); }
  onDeactivate(_context: ToolContext): void { this.reset(); }

  onPointerDown(_event: PointerEvent, context: ToolContext): void {
    const cursor = context.getSnapAwareCursor();
    const now = Date.now();
    const isDoubleClick = now - this.lastClickTime < 300;
    this.lastClickTime = now;

    if (this.state === "idle") {
      const pt = makePoint(cursor.x, cursor.y);
      this.points = [pt];
      this.startCoords = { x: cursor.x, y: cursor.y };
      this.state = "placing";
      context.onRequestRender();
      return;
    }

    if (isDoubleClick) {
      this.closePolyline(context);
      return;
    }

    const dx = cursor.x - this.startCoords!.x;
    const dy = cursor.y - this.startCoords!.y;
    const dist = Math.hypot(dx, dy);

    if (dist < CLOSE_THRESHOLD) {
      this.closePolyline(context);
      return;
    }

    const pt = makePoint(cursor.x, cursor.y);
    this.points.push(pt);
    context.onRequestRender();
  }

  onPointerMove(_event: PointerEvent, context: ToolContext): void {
    if (this.state === "placing" && this.points.length > 0) {
      const last = this.points[this.points.length - 1];
      const cursor = context.getSnapAwareCursor();
      context.setPreview(buildLinePreview(last.x, last.y, cursor.x, cursor.y));
    }
  }

  onPointerUp(_event: PointerEvent, _context: ToolContext): void {
  }

  onKeyDown(event: KeyboardEvent, context: ToolContext): void {
    if (event.key === "Escape") {
      this.reset();
      context.onRequestRender();
    }
    if (event.key === "Enter" && this.state === "placing") {
      this.closePolyline(context);
    }
  }

  getHint(): string {
    if (this.state === "idle") return "Click to start polyline";
    const cursor = this.getLastCursor();
    if (cursor && this.startCoords) {
      const dx = cursor.x - this.startCoords.x;
      const dy = cursor.y - this.startCoords.y;
      if (Math.hypot(dx, dy) < CLOSE_THRESHOLD) {
        return "Click or Enter to close polyline, double-click to finish";
      }
    }
    return `Click to add point (${this.points.length} placed), Enter or double-click to close`;
  }

  private closePolyline(context: ToolContext): void {
    if (this.points.length < 2) {
      this.reset();
      return;
    }

    const isClosed = this.detectClose(context);
    const polyline = makePolyline(
      this.points.map((p) => p.id),
      isClosed
    );

    const addCmds = this.points.map((pt) => ({
      description: `Add polyline vertex`,
      execute: (m: SketchModel) => addEntity(m, pt),
      undo: (m: SketchModel) => removeEntity(m, pt.id),
    }));

    const cmd = compoundCommand("Create polyline", [
      ...addCmds,
      {
        description: "Add polyline",
        execute: (m: SketchModel) => addEntity(m, polyline),
        undo: (m: SketchModel) => removeEntity(m, polyline.id),
      },
    ]);

    context.history.push(cmd);
    context.setModel(cmd.execute(context.model));
    this.reset();
  }

  private detectClose(context: ToolContext): boolean {
    const cursor = context.getSnapAwareCursor();
    const dx = cursor.x - this.startCoords!.x;
    const dy = cursor.y - this.startCoords!.y;
    return Math.hypot(dx, dy) < CLOSE_THRESHOLD;
  }

  private getLastCursor(): { x: number; y: number } | null {
    if (this.points.length === 0) return null;
    const last = this.points[this.points.length - 1];
    return { x: last.x, y: last.y };
  }

  private reset(): void {
    this.state = "idle";
    this.points = [];
    this.startCoords = null;
  }
}

function buildLinePreview(x1: number, y1: number, x2: number, y2: number): THREE.Line {
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.BufferAttribute(new Float32Array([x1, y1, 0, x2, y2, 0]), 3));
  const mat = new THREE.LineDashedMaterial({ color: 0xffffff, dashSize: 4, gapSize: 3, transparent: true, opacity: 0.4 });
  const line = new THREE.Line(geo, mat);
  line.computeLineDistances();
  return line;
}
