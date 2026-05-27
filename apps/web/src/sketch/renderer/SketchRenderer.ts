import * as THREE from "three";
import type { SketchModel } from "../model/SketchModel";
import { GridRenderer } from "./GridRenderer";
import { createEntityMesh } from "./EntityMesh";
import { createConstraintVisual } from "./ConstraintVisual";
import { SnapIndicator } from "./SnapIndicator";

const LAYER_GRID = 0;
const LAYER_CONSTRUCTION = 1;
const LAYER_ENTITIES = 2;
const LAYER_CONSTRAINTS = 3;
const LAYER_DIMENSIONS = 4;
const LAYER_PREVIEW = 6;
const LAYER_SNAP = 7;
const LAYER_INFERENCE = 8;

/**
 * Main renderer for the 2D sketch overlay.
 * Manages a Three.js scene, orthographic camera, and WebGL renderer
 * with ordered layers for different sketch element types.
 */
export class SketchRenderer {
  readonly scene: THREE.Scene;
  readonly camera: THREE.OrthographicCamera;
  readonly renderer: THREE.WebGLRenderer;

  private layers: Map<number, THREE.Group>;
  private grid: GridRenderer;
  readonly snapIndicator: SnapIndicator;
  readonly inferenceGroup: THREE.Group;
  readonly previewGroup: THREE.Group;
  private baseFrustumSize: number;

  constructor(container: HTMLElement) {
    this.scene = new THREE.Scene();

    this.camera = new THREE.OrthographicCamera(-1, 1, 1, -1, -1000, 1000);
    this.camera.position.set(0, 0, 1000);
    this.camera.lookAt(0, 0, 0);
    this.baseFrustumSize = 100;

    this.renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.setClearColor(0x000000, 0);
    container.appendChild(this.renderer.domElement);

    this.layers = new Map();
    this.grid = new GridRenderer();
    this.snapIndicator = new SnapIndicator();
    this.inferenceGroup = new THREE.Group();
    this.inferenceGroup.renderOrder = LAYER_INFERENCE;
    this.previewGroup = new THREE.Group();
    this.previewGroup.renderOrder = LAYER_PREVIEW;

    this.getLayer(LAYER_GRID).add(this.grid.group);
    this.getLayer(LAYER_SNAP).add(this.snapIndicator.group);
    this.getLayer(LAYER_INFERENCE).add(this.inferenceGroup);
    this.getLayer(LAYER_PREVIEW).add(this.previewGroup);
  }

  /** Rebuild the entire scene from the model */
  rebuild(model: SketchModel): void {
    for (const [order, group] of this.layers) {
      if (order === LAYER_GRID || order === LAYER_SNAP || order === LAYER_INFERENCE) continue;
      group.clear();
    }

    const constrained = new Set<string>();
    for (const [, c] of model.constraints) for (const eId of c.entities) constrained.add(eId);

    const constructionLayer = this.getLayer(LAYER_CONSTRUCTION);
    const entityLayer = this.getLayer(LAYER_ENTITIES);
    const constraintLayer = this.getLayer(LAYER_CONSTRAINTS);
    const dimensionLayer = this.getLayer(LAYER_DIMENSIONS);

    for (const [, entity] of model.entities) {
      const mesh = createEntityMesh(entity, model, false, constrained.has(entity.id));
      if (entity.kind === "point") {
        mesh.renderOrder = LAYER_ENTITIES;
        entityLayer.add(mesh);
      } else if (entity.construction) {
        mesh.renderOrder = LAYER_CONSTRUCTION;
        constructionLayer.add(mesh);
      } else {
        mesh.renderOrder = LAYER_ENTITIES;
        entityLayer.add(mesh);
      }
    }

    for (const [, constraint] of model.constraints) {
      const visual = createConstraintVisual(constraint, model);
      if (visual) {
        const isDim =
          constraint.kind === "lengthDim" ||
          constraint.kind === "angleDim" ||
          constraint.kind === "radiusDim";
        visual.renderOrder = isDim ? LAYER_DIMENSIONS : LAYER_CONSTRAINTS;
        (isDim ? dimensionLayer : constraintLayer).add(visual);
      }
    }

    this.grid.update(this.camera);
  }

  /** Render a single frame */
  render(): void {
    this.grid.update(this.camera);
    this.renderer.render(this.scene, this.camera);
  }

  /** Resize the renderer to match container */
  resize(width: number, height: number): void {
    this.renderer.setSize(width, height);
    const aspect = width / height;
    this.camera.left = -this.baseFrustumSize * aspect;
    this.camera.right = this.baseFrustumSize * aspect;
    this.camera.top = this.baseFrustumSize;
    this.camera.bottom = -this.baseFrustumSize;
    this.camera.updateProjectionMatrix();
  }

  /** Get layer group by render order */
  getLayer(order: number): THREE.Group {
    let group = this.layers.get(order);
    if (!group) {
      group = new THREE.Group();
      group.renderOrder = order;
      this.scene.add(group);
      this.layers.set(order, group);
    }
    return group;
  }

  /** Pan the camera by screen-space delta */
  pan(dx: number, dy: number): void {
    this.camera.position.x -= dx;
    this.camera.position.y += dy;
    this.camera.lookAt(this.camera.position.x, this.camera.position.y, 0);
  }

  /**
   * Zoom centered on a screen point.
   * @param factor - multiply frustum size (< 1 zooms in, > 1 zooms out)
   */
  zoom(factor: number, centerX: number, centerY: number): void {
    const before = this.screenToSketch(centerX, centerY);
    this.baseFrustumSize *= factor;
    const aspect =
      this.renderer.domElement.width / this.renderer.domElement.height;
    this.camera.top = this.baseFrustumSize;
    this.camera.bottom = -this.baseFrustumSize;
    this.camera.left = -this.baseFrustumSize * aspect;
    this.camera.right = this.baseFrustumSize * aspect;
    this.camera.updateProjectionMatrix();
    const after = this.screenToSketch(centerX, centerY);
    this.pan(after.x - before.x, after.y - before.y);
  }

  /** Zoom to fit all entities with 20 % padding */
  zoomToFit(model: SketchModel): void {
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    const seen = new Set<string>();
    for (const [id, entity] of model.entities) {
      const visit = (pid: string) => {
        if (seen.has(pid)) return;
        const p = model.points.get(pid);
        if (p) {
          seen.add(pid);
          minX = Math.min(minX, p.x);
          maxX = Math.max(maxX, p.x);
          minY = Math.min(minY, p.y);
          maxY = Math.max(maxY, p.y);
        }
      };
      if (entity.kind === "point") {
        visit(id);
      } else if (entity.kind === "line") {
        visit(entity.start);
        visit(entity.end);
      } else if (entity.kind === "circle" || entity.kind === "arc") {
        visit(entity.center);
      } else if (entity.kind === "polyline") {
        for (const pid of entity.points) visit(pid);
      }
    }
    if (minX === Infinity) return;

    const midX = (minX + maxX) / 2;
    const midY = (minY + maxY) / 2;
    const extent = Math.max(maxX - minX, maxY - minY, 1) * 1.2;
    const aspect =
      this.renderer.domElement.width / this.renderer.domElement.height;

    this.camera.position.set(midX, midY, 1000);
    this.camera.lookAt(midX, midY, 0);
    this.baseFrustumSize = extent / 2;
    this.camera.top = this.baseFrustumSize;
    this.camera.bottom = -this.baseFrustumSize;
    this.camera.left = -this.baseFrustumSize * aspect;
    this.camera.right = this.baseFrustumSize * aspect;
    this.camera.updateProjectionMatrix();
  }

  /**
   * Convert screen coordinates (relative to viewport) to sketch coordinates.
   * Returns `{x, y}` in mm.
   */
  screenToSketch(
    screenX: number,
    screenY: number
  ): { x: number; y: number } {
    const rect = this.renderer.domElement.getBoundingClientRect();
    const nx = ((screenX - rect.left) / rect.width) * 2 - 1;
    const ny = -((screenY - rect.top) / rect.height) * 2 + 1;
    return {
      x: this.camera.position.x + nx * this.camera.right,
      y: this.camera.position.y + ny * this.camera.top,
    };
  }

  /** Convert sketch coordinates to screen coordinates */
  sketchToScreen(x: number, y: number): { x: number; y: number } {
    const rect = this.renderer.domElement.getBoundingClientRect();
    const nx = (x - this.camera.position.x) / this.camera.right;
    const ny = (y - this.camera.position.y) / this.camera.top;
    return {
      x: ((nx + 1) / 2) * rect.width + rect.left,
      y: ((-ny + 1) / 2) * rect.height + rect.top,
    };
  }

  /** Set preview geometry (tool ghost). Clears on every rebuild. */
  setPreview(mesh: THREE.Object3D | null): void {
    this.previewGroup.clear();
    if (mesh) {
      this.previewGroup.add(mesh);
    }
  }

  /** Alias for zoom - used by CameraController */
  zoomAtPoint(factor: number, centerX: number, centerY: number): void {
    this.zoom(factor, centerX, centerY);
  }

  /** Reset to 1:1 scale (100mm = 100px on screen) */
  zoomOneToOne(): void {
    this.baseFrustumSize = 50;
    const aspect =
      this.renderer.domElement.width / this.renderer.domElement.height;
    this.camera.top = this.baseFrustumSize;
    this.camera.bottom = -this.baseFrustumSize;
    this.camera.left = -this.baseFrustumSize * aspect;
    this.camera.right = this.baseFrustumSize * aspect;
    this.camera.updateProjectionMatrix();
  }

  /** Clean up all GPU resources */
  dispose(): void {
    this.renderer.dispose();
    this.grid.dispose();
    this.snapIndicator.dispose();
    this.scene.clear();
    this.layers.clear();
  }
}
