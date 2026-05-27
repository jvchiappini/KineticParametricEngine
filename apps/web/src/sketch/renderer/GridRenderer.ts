import * as THREE from "three";

const VALID_SPACINGS = [1, 2, 5, 10, 25, 50, 100, 250, 500];

function buildGridLines(
  spacing: number,
  visibleSize: number
): Float32Array {
  const half = Math.ceil(visibleSize / spacing / 2) * spacing;
  const verts: number[] = [];
  for (let v = -half; v <= half; v += spacing) {
    // vertical line
    verts.push(v, -half, 0, v, half, 0);
    // horizontal line
    verts.push(-half, v, 0, half, v, 0);
  }
  return new Float32Array(verts);
}

function makeLine(
  verts: Float32Array,
  color: string,
  opacity: number
): THREE.LineSegments {
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.BufferAttribute(verts, 3));
  const mat = new THREE.LineBasicMaterial({
    color,
    transparent: true,
    opacity,
    depthWrite: false,
  });
  return new THREE.LineSegments(geo, mat);
}

/**
 * Renders an adaptive grid for the sketch plane.
 *
 * - Minor lines: white at 3 % opacity.
 * - Major lines (every 5 intervals): white at 8 % opacity.
 * - Origin: distinct crosshair.
 *
 * Grid spacing is chosen from `[1, 2, 5, 10, 25, 50, 100, 250, 500]`
 * so that 8-20 intervals are visible in the current view.
 */
export class GridRenderer {
  readonly group: THREE.Group;

  private currentSpacing = 0;
  private minorMesh: THREE.LineSegments | null = null;
  private majorMesh: THREE.LineSegments | null = null;
  private originMesh: THREE.LineSegments | null = null;

  constructor() {
    this.group = new THREE.Group();
    this.group.renderOrder = 0;
  }

  /** Rebuild grid lines when spacing changes */
  private rebuild(spacing: number, visibleSize: number): void {
    // Remove old meshes
    if (this.minorMesh) this.group.remove(this.minorMesh);
    if (this.majorMesh) this.group.remove(this.majorMesh);
    if (this.originMesh) this.group.remove(this.originMesh);

    this.currentSpacing = spacing;
    const majorEvery = 5;
    const half = Math.ceil(visibleSize / spacing / 2) * spacing;

    // Collect minor / major vertices separately
    const minorVerts: number[] = [];
    const majorVerts: number[] = [];

    for (let v = -half; v <= half; v += spacing) {
      const isMajor = Math.round(v / spacing) % majorEvery === 0;
      const target = isMajor ? majorVerts : minorVerts;
      target.push(v, -half, 0, v, half, 0);
      target.push(-half, v, 0, half, v, 0);
    }

    this.minorMesh = makeLine(
      new Float32Array(minorVerts),
      "#ffffff", 0.03
    );
    this.majorMesh = makeLine(
      new Float32Array(majorVerts),
      "#ffffff", 0.08
    );
    this.group.add(this.minorMesh);
    this.group.add(this.majorMesh);

    // Origin crosshair
    const o = 10;
    const ov = new Float32Array([
      -o, 0, 0, o, 0, 0,
      0, -o, 0, 0, o, 0,
    ]);
    this.originMesh = makeLine(ov, "#ffffff", 0.15);
    this.group.add(this.originMesh);
  }

  /**
   * Update grid based on camera zoom level.
   * Should be called once per frame before rendering.
   */
  update(camera: THREE.OrthographicCamera): void {
    const visibleSize = camera.top * 2;
    let spacing = VALID_SPACINGS[0];
    for (const s of VALID_SPACINGS) {
      const count = visibleSize / s;
      if (count >= 8 && count <= 20) {
        spacing = s;
        break;
      }
      if (count < 8) {
        spacing = s;
        break;
      }
    }

    if (spacing !== this.currentSpacing) {
      this.rebuild(spacing, visibleSize);
    }
  }

  /** Return the current grid spacing in mm */
  getGridSpacing(): number {
    return this.currentSpacing;
  }

  /** Snap a point to the nearest grid intersection */
  snapToGrid(x: number, y: number): { x: number; y: number } {
    if (this.currentSpacing === 0) return { x, y };
    const s = this.currentSpacing;
    return {
      x: Math.round(x / s) * s,
      y: Math.round(y / s) * s,
    };
  }

  dispose(): void {
    this.group.clear();
    this.minorMesh = null;
    this.majorMesh = null;
    this.originMesh = null;
  }
}
