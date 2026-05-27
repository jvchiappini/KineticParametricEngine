import * as THREE from "three";

/**
 * Available snap kinds for visual feedback.
 * Defined here until the snap module is created.
 */
export type SnapKind =
  | "endpoint"
  | "center"
  | "midpoint"
  | "intersection"
  | "tangent"
  | "perpendicular"
  | "grid"
  | "none";

const SNAP_COLOR = 0xffd700;
const S = 4;

function line(
  x1: number, y1: number,
  x2: number, y2: number,
  color: number
): THREE.Line {
  const geo = new THREE.BufferGeometry();
  geo.setAttribute(
    "position",
    new THREE.BufferAttribute(new Float32Array([x1, y1, 0, x2, y2, 0]), 3)
  );
  return new THREE.Line(geo, new THREE.LineBasicMaterial({ color }));
}

/**
 * Visual feedback for active snapping.
 *
 * Each snap kind shows a distinct yellow (#ffd700) indicator:
 * - endpoint: filled 8×8 px square
 * - center:   circle outline, 10 px diameter
 * - midpoint: upward-pointing triangle
 * - intersection: X mark
 * - tangent:  circle with tangent line hint
 * - perpendicular: right-angle mark
 * - grid/none: no indicator
 *
 * Inference lines are managed separately via `showInferenceLines` /
 * `clearInferenceLines`.
 */
export class SnapIndicator {
  readonly group: THREE.Group;
  private current: THREE.Object3D | null = null;

  constructor() {
    this.group = new THREE.Group();
    this.group.renderOrder = 7;
  }

  /** Show indicator at position for given snap kind */
  show(kind: SnapKind, x: number, y: number): void {
    this.hide();
    if (kind === "none" || kind === "grid") return;

    let obj: THREE.Object3D | null = null;

    switch (kind) {
      case "endpoint": {
        const geo = new THREE.PlaneGeometry(S * 2, S * 2);
        const mat = new THREE.MeshBasicMaterial({
          color: SNAP_COLOR,
          depthTest: false,
        });
        const mesh = new THREE.Mesh(geo, mat);
        mesh.position.set(x, y, 0);
        obj = mesh;
        break;
      }
      case "center": {
        const geo = new THREE.RingGeometry(4, 5, 16);
        const mat = new THREE.MeshBasicMaterial({
          color: SNAP_COLOR,
          side: THREE.DoubleSide,
          depthTest: false,
        });
        const mesh = new THREE.Mesh(geo, mat);
        mesh.position.set(x, y, 0);
        obj = mesh;
        break;
      }
      case "midpoint": {
        const pts: number[] = [];
        const r = S;
        // upward triangle
        pts.push(x, y + r, 0, x + r, y - r, 0);
        pts.push(x + r, y - r, 0, x - r, y - r, 0);
        pts.push(x - r, y - r, 0, x, y + r, 0);
        const geo = new THREE.BufferGeometry();
        geo.setAttribute(
          "position",
          new THREE.BufferAttribute(new Float32Array(pts), 3)
        );
        obj = new THREE.Line(
          geo,
          new THREE.LineBasicMaterial({ color: SNAP_COLOR, depthTest: false })
        );
        break;
      }
      case "intersection": {
        obj = new THREE.Group();
        (obj as THREE.Group).add(line(x - S, y - S, x + S, y + S, SNAP_COLOR));
        (obj as THREE.Group).add(line(x + S, y - S, x - S, y + S, SNAP_COLOR));
        break;
      }
      case "tangent": {
        const g = new THREE.Group();
        // circle outline
        const circle = new THREE.RingGeometry(4, 5, 16);
        const cm = new THREE.MeshBasicMaterial({
          color: SNAP_COLOR,
          side: THREE.DoubleSide,
          depthTest: false,
        });
        const mesh = new THREE.Mesh(circle, cm);
        mesh.position.set(x, y, 0);
        g.add(mesh);
        // tangent line hint
        g.add(line(x + 5, y - 5, x + 12, y - 5, SNAP_COLOR));
        obj = g;
        break;
      }
      case "perpendicular": {
        obj = new THREE.Group();
        const g = obj as THREE.Group;
        g.add(line(x - S, y - S, x + S, y - S, SNAP_COLOR));
        g.add(line(x - S, y - S, x - S, y + S, SNAP_COLOR));
        break;
      }
    }

    if (obj) {
      this.current = obj;
      this.group.add(obj);
    }
  }

  /** Hide the current indicator */
  hide(): void {
    if (this.current) {
      this.group.remove(this.current);
      this.current = null;
    }
  }

  dispose(): void {
    this.hide();
    this.group.clear();
  }
}

const INFERENCE_COLORS: Record<string, number> = {
  horizontal: 0xff4444,
  vertical: 0x44ff44,
  aligned: 0x00e5ff,
  angle: 0xff44ff,
};

/**
 * Draw dashed inference lines between two points.
 *
 * - `alignment` determines the dash color:
 *   - horizontal → red
 *   - vertical   → green
 *   - aligned    → cyan
 *   - angle      → magenta
 *
 * Call `clearInferenceLines(group)` to remove them.
 */
export function showInferenceLines(
  group: THREE.Group,
  fromX: number, fromY: number,
  toX: number, toY: number,
  alignment: "horizontal" | "vertical" | "aligned" | "angle"
): void {
  const color = INFERENCE_COLORS[alignment] ?? 0xffffff;
  const geo = new THREE.BufferGeometry();
  geo.setAttribute(
    "position",
    new THREE.BufferAttribute(
      new Float32Array([fromX, fromY, 0, toX, toY, 0]),
      3
    )
  );
  const mat = new THREE.LineDashedMaterial({
    color,
    dashSize: 3,
    gapSize: 2,
    transparent: true,
    opacity: 0.6,
    depthTest: false,
  });
  const line = new THREE.Line(geo, mat);
  line.computeLineDistances();
  group.add(line);
}

/** Remove all inference lines from the given group */
export function clearInferenceLines(group: THREE.Group): void {
  group.clear();
}
