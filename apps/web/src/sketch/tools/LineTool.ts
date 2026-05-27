import * as THREE from "three";
import type { SketchTool, ToolContext } from "./types";
import type { Command } from "../model/SketchHistory";
import type { SketchModel } from "../model/SketchModel";
import { addEntity, removeEntity } from "../model/SketchModel";
import { makePoint, makeLine } from "../model/Entity";
import { compoundCommand } from "../model/SketchHistory";

export class LineTool implements SketchTool {
  readonly name = "line";
  readonly cursor = "crosshair";

  private state: "idle" | "placing" = "idle";
  private startCoords: { x: number; y: number } | null = null;

  onActivate(_context: ToolContext): void { this.reset(); }
  onDeactivate(context: ToolContext): void { context.setPreview(null); this.reset(); }

  onPointerDown(_event: PointerEvent, context: ToolContext): void {
    if (this.state === "idle") {
      this.startCoords = context.getSnapAwareCursor();
      this.state = "placing";
    } else {
      this.complete(context);
    }
  }

  onPointerMove(_event: PointerEvent, context: ToolContext): void {
    if (this.state === "placing" && this.startCoords) {
      const end = context.getSnapAwareCursor();
      const pts = new Float32Array([
        this.startCoords.x, this.startCoords.y, 0,
        end.x, end.y, 0,
      ]);
      const geo = new THREE.BufferGeometry();
      geo.setAttribute("position", new THREE.BufferAttribute(pts, 3));
      const mat = new THREE.LineDashedMaterial({
        color: 0xffffff, dashSize: 4, gapSize: 3,
        transparent: true, opacity: 0.4,
      });
      const line = new THREE.Line(geo, mat);
      line.computeLineDistances();
      context.setPreview(line);
    }
  }

  onPointerUp(_event: PointerEvent, _context: ToolContext): void {}

  onKeyDown(event: KeyboardEvent, context: ToolContext): void {
    if (event.key === "Escape") {
      context.setPreview(null);
      this.reset();
    }
    if (event.key === "Enter" && this.state === "placing") {
      this.complete(context);
    }
  }

  getHint(): string {
    return this.state === "idle"
      ? "Click to place line start point"
      : "Click or Enter to place end point, Escape to cancel";
  }

  private complete(context: ToolContext): void {
    const startPt = makePoint(this.startCoords!.x, this.startCoords!.y);
    const endCursor = context.getSnapAwareCursor();
    const endPt = makePoint(endCursor.x, endCursor.y);
    const line = makeLine(startPt.id, endPt.id);

    const cmd = compoundCommand("Create line", [
      { description: "Add start point", execute: (m) => addEntity(m, startPt), undo: (m) => removeEntity(m, startPt.id) as SketchModel },
      { description: "Add end point", execute: (m) => addEntity(m, endPt), undo: (m) => removeEntity(m, endPt.id) as SketchModel },
      { description: "Add line", execute: (m) => addEntity(m, line), undo: (m) => removeEntity(m, line.id) as SketchModel },
    ]);

    context.history.push(cmd);
    context.setModel(cmd.execute(context.model));
    context.setPreview(null);
    this.reset();
  }

  private reset(): void {
    this.state = "idle";
    this.startCoords = null;
  }
}
