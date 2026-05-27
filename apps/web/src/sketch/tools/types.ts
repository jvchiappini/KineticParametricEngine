import type { SketchModel } from "../model/SketchModel";
import type { SketchHistory } from "../model/SketchHistory";
import type { SketchRenderer } from "../renderer/SketchRenderer";
import type { SnapResult } from "../snap/SnapTypes";

/**
 * Context passed to all tool lifecycle and interaction methods.
 */
export interface ToolContext {
  model: SketchModel;
  setModel: (model: SketchModel) => void;
  snapResult: SnapResult;
  selectedIds: string[];
  addToSelection: (ids: string[]) => void;
  clearSelection: () => void;
  history: SketchHistory;
  renderer: SketchRenderer;
  onRequestRender: () => void;
  getCursorInSketch: () => { x: number; y: number };
  getSnapAwareCursor: () => { x: number; y: number };
  setPreview: (mesh: import("three").Object3D | null) => void;
  screenToWorld: (clientX: number, clientY: number) => { x: number; y: number };
  updateCursor: (clientX: number, clientY: number) => void;
  onPan: (dx: number, dy: number) => void;
  onZoom: (factor: number, clientX: number, clientY: number) => void;
  onContextMenu: (clientX: number, clientY: number) => void;
}

/**
 * Interface all sketch tools must implement.
 */
export interface SketchTool {
  readonly name: string;
  readonly cursor: string;
  onActivate(context: ToolContext): void;
  onDeactivate(context: ToolContext): void;
  onPointerDown(event: PointerEvent, context: ToolContext): void;
  onPointerMove(event: PointerEvent, context: ToolContext): void;
  onPointerUp(event: PointerEvent, context: ToolContext): void;
  onKeyDown(event: KeyboardEvent, context: ToolContext): void;
  getHint(): string;
}
