import * as THREE from "three";
import { SketchEngine } from "./sketch-engine";
import type { ToolType } from "./sketch-engine";

export class SketchUI {
  private engine: SketchEngine;
  private scene: THREE.Scene;
  private renderer: THREE.WebGLRenderer;
  private overlay = new THREE.Group();
  private preview = new THREE.Group();
  private orthoCam: THREE.OrthographicCamera | null = null;
  private isActive = false;
  private canvas: HTMLCanvasElement;

  public plane: "XY" | "XZ" | "YZ" = "XZ";
  public offset: number = 0;

  private mouseWorld = new THREE.Vector3();
  private mouseDown = false;
  private dragStart: { x: number; y: number } | null = null;
  private drawPreview: THREE.Line | null = null;

  onExtrudeJSON: ((json: string) => void) | null = null;

  private gridGroup = new THREE.Group();
  private constraintLabels: THREE.Sprite[] = [];

  constructor(scene: THREE.Scene, renderer: THREE.WebGLRenderer) {
    this.engine = new SketchEngine();
    this.scene = scene;
    this.renderer = renderer;
    this.canvas = renderer.domElement;
    this.setupEvents();
  }

  get engineInstance(): SketchEngine {
    return this.engine;
  }

  get active(): boolean {
    return this.isActive;
  }

  get activeCamera(): THREE.OrthographicCamera | null {
    return this.orthoCam;
  }

  setTool(tool: ToolType) {
    this.engine.setTool(tool);
    this.clearDrawPreview();
  }

  activate() {
    this.isActive = true;
    const size = 8;
    this.orthoCam = new THREE.OrthographicCamera(-size, size, size, -size, 0.1, 100);
    this.updateCamera();
    this.scene.add(this.overlay);
    this.scene.add(this.preview);
    this.buildGrid();
    this.rebuild();
  }

  updateCamera() {
    if (!this.orthoCam) return;
    if (this.plane === "XZ") {
      this.orthoCam.position.set(0, 10 + this.offset, 0);
      this.orthoCam.up.set(0, 0, -1);
      this.orthoCam.lookAt(0, this.offset, 0);
    } else if (this.plane === "XY") {
      this.orthoCam.position.set(0, 0, 10 + this.offset);
      this.orthoCam.up.set(0, 1, 0);
      this.orthoCam.lookAt(0, 0, this.offset);
    } else if (this.plane === "YZ") {
      this.orthoCam.position.set(10 + this.offset, 0, 0);
      this.orthoCam.up.set(0, 1, 0);
      this.orthoCam.lookAt(this.offset, 0, 0);
    }
    this.orthoCam.updateProjectionMatrix();
  }

  deactivate() {
    this.isActive = false;
    this.scene.remove(this.overlay);
    this.scene.remove(this.preview);
    this.scene.remove(this.gridGroup);
    this.overlay.clear();
    this.preview.clear();
    this.gridGroup.clear();
    this.constraintLabels = [];
  }

  private screenToWorld(clientX: number, clientY: number): { x: number; y: number } {
    const rect = this.canvas.getBoundingClientRect();
    const ndcX = ((clientX - rect.left) / rect.width) * 2 - 1;
    const ndcY = -((clientY - rect.top) / rect.height) * 2 + 1;
    if (this.orthoCam) {
      this.mouseWorld.set(ndcX, ndcY, 0);
      this.mouseWorld.unproject(this.orthoCam);
    }
    if (this.plane === "XZ") return { x: this.mouseWorld.x, y: this.mouseWorld.z };
    if (this.plane === "XY") return { x: this.mouseWorld.x, y: this.mouseWorld.y };
    return { x: this.mouseWorld.z, y: this.mouseWorld.y }; // YZ plane
  }

  private setupEvents() {
    this.canvas.addEventListener("pointerdown", (e) => this.onDown(e));
    this.canvas.addEventListener("pointermove", (e) => this.onMove(e));
    this.canvas.addEventListener("pointerup", (e) => this.onUp(e));
    this.canvas.addEventListener("dblclick", () => {
      if (this.engine.state.tool === "polyline" && this.engine.state.mode === "drawing") {
        this.engine.state.mode = "idle";
        this.engine.state.drawStart = null;
      }
    });
    document.addEventListener("keydown", (e) => {
      if (!this.isActive) return;
      if (e.key === "Escape") this.clearDrawPreview();
      if (e.key === "Delete" || e.key === "Backspace") {
        if (this.engine.state.selectedIds.length > 0) {
          this.engine.state.selectedIds = [];
          this.rebuild();
        }
      }
    });
  }

  private onDown(e: PointerEvent) {
    if (!this.isActive) return;
    this.mouseDown = true;
    const pos = this.screenToWorld(e.clientX, e.clientY);
    const snap = this.engine.snap(pos.x, pos.y);
    this.dragStart = { x: snap.x, y: snap.y };

    const tool = this.engine.state.tool;

    if (tool === "line" || tool === "polyline") {
      if (this.engine.state.mode === "idle") {
        this.engine.state.mode = "drawing";
        this.engine.state.drawStart = { x: snap.x, y: snap.y };
      } else {
        const start = this.engine.state.drawStart!;
        this.engine.addLine(start.x, start.y, snap.x, snap.y);
        this.engine.state.drawStart = { x: snap.x, y: snap.y };
        this.rebuild();
      }
    }

    if (tool === "select") {
      const doc = this.engine.getDoc();
      if (!doc) return;
      const pt = this.pickPoint(pos.x, pos.y, doc.points || []);
      if (pt) {
        const idx = this.engine.state.selectedIds.indexOf(pt.id);
        if (idx >= 0) this.engine.state.selectedIds.splice(idx, 1);
        else this.engine.state.selectedIds.push(pt.id);
        this.rebuild();
      }
    }
  }

  private onMove(e: PointerEvent) {
    if (!this.isActive) return;
    const pos = this.screenToWorld(e.clientX, e.clientY);
    this.engine.state.hoverSnap = this.engine.snap(pos.x, pos.y);
    const sp = this.engine.state.hoverSnap;

    if (this.engine.state.mode === "drawing") {
      const tool = this.engine.state.tool;
      if (tool === "rect" && this.dragStart) {
        const x = Math.min(this.dragStart.x, sp.x);
        const y = Math.min(this.dragStart.y, sp.y);
        const w = Math.abs(sp.x - this.dragStart.x);
        const h = Math.abs(sp.y - this.dragStart.y);
        this.showDrawPreviewRect(x, y, w, h);
      } else if (this.engine.state.drawStart) {
        this.showDrawPreviewLine(this.engine.state.drawStart.x, this.engine.state.drawStart.y, sp.x, sp.y);
      }
    }

    this.rebuildSnapIndicator(sp);
  }

  private onUp(e: PointerEvent) {
    if (!this.isActive || !this.mouseDown) return;
    this.mouseDown = false;
    const pos = this.screenToWorld(e.clientX, e.clientY);
    const snap = this.engine.snap(pos.x, pos.y);

    const tool = this.engine.state.tool;

    if (tool === "rect") {
      if (this.dragStart) {
        const x = Math.min(this.dragStart.x, snap.x);
        const y = Math.min(this.dragStart.y, snap.y);
        const w = Math.abs(snap.x - this.dragStart.x);
        const h = Math.abs(snap.y - this.dragStart.y);
        if (w > 0.05 && h > 0.05) {
          this.engine.addRect(x, y, w, h);
          this.rebuild();
        }
      }
      this.dragStart = null;
    }

    if (tool === "line" && this.engine.state.mode === "drawing") {
      const start = this.engine.state.drawStart!;
      if (Math.abs(start.x - snap.x) > 0.01 || Math.abs(start.y - snap.y) > 0.01) {
        this.engine.addLine(start.x, start.y, snap.x, snap.y);
        this.engine.state.mode = "idle";
        this.engine.state.drawStart = null;
        this.rebuild();
      }
    }

    this.clearDrawPreview();
  }

  private pickPoint(x: number, y: number, points: { id: number; x: number; y: number }[]): { id: number; x: number; y: number } | null {
    let best: { id: number; x: number; y: number; d: number } | null = null;
    for (const p of points) {
      const d = Math.sqrt((p.x - x) ** 2 + (p.y - y) ** 2);
      if (d < 0.2 && (!best || d < best.d)) best = { ...p, d };
    }
    return best;
  }

  private get3DPos(x: number, y: number): THREE.Vector3 {
    if (this.plane === "XZ") return new THREE.Vector3(x, this.offset + 0.01, y);
    if (this.plane === "XY") return new THREE.Vector3(x, y, this.offset + 0.01);
    return new THREE.Vector3(this.offset + 0.01, y, x); // YZ
  }

  private showDrawPreviewLine(x1: number, y1: number, x2: number, y2: number) {
    this.clearDrawPreview();
    const geo = new THREE.BufferGeometry().setFromPoints([
      this.get3DPos(x1, y1),
      this.get3DPos(x2, y2),
    ]);
    this.drawPreview = new THREE.Line(geo, new THREE.LineBasicMaterial({
      color: 0x88ddff, transparent: true, opacity: 0.6, linewidth: 2,
    }));
    this.preview.add(this.drawPreview);
  }

  private showDrawPreviewRect(x: number, y: number, w: number, h: number) {
    this.clearDrawPreview();
    const pts = [
      this.get3DPos(x, y),
      this.get3DPos(x + w, y),
      this.get3DPos(x + w, y + h),
      this.get3DPos(x, y + h),
      this.get3DPos(x, y),
    ];
    const geo = new THREE.BufferGeometry().setFromPoints(pts);
    this.drawPreview = new THREE.Line(geo, new THREE.LineBasicMaterial({
      color: 0x88ddff, transparent: true, opacity: 0.6,
    }));
    this.preview.add(this.drawPreview);
  }

  private clearDrawPreview() {
    this.preview.clear();
    this.drawPreview = null;
  }

  public buildGrid() {
    this.scene.remove(this.gridGroup);
    this.gridGroup = new THREE.Group();
    const gridHelper = new THREE.GridHelper(16, 32, 0x4488ff, 0x224488);
    const subGrid = new THREE.GridHelper(16, 160, 0x224466, 0x112233);

    this.gridGroup.add(gridHelper);
    this.gridGroup.add(subGrid);

    if (this.plane === "XZ") {
      gridHelper.position.y = this.offset - 0.01;
      subGrid.position.y = this.offset - 0.005;
    } else if (this.plane === "XY") {
      gridHelper.rotation.x = Math.PI / 2;
      subGrid.rotation.x = Math.PI / 2;
      gridHelper.position.z = this.offset - 0.01;
      subGrid.position.z = this.offset - 0.005;
    } else if (this.plane === "YZ") {
      gridHelper.rotation.z = Math.PI / 2;
      subGrid.rotation.z = Math.PI / 2;
      gridHelper.position.x = this.offset - 0.01;
      subGrid.position.x = this.offset - 0.005;
    }

    this.scene.add(this.gridGroup);
  }

  private rebuildSnapIndicator(snap: { x: number; y: number; kind: string; target_id: number | null }) {
    const existing = this.overlay.getObjectByName("snap");
    if (existing) this.overlay.remove(existing);

    if (snap.kind === "none" || snap.kind === "grid") return;

    const color = snap.kind === "endpoint" ? 0xffaa00 :
      snap.kind === "midpoint" ? 0x00ffaa : 0xaaffaa;

    const ring = new THREE.RingGeometry(0.06, 0.1, 12);
    const mat = new THREE.MeshBasicMaterial({
      color, side: THREE.DoubleSide, transparent: true, opacity: 0.8, depthWrite: false,
    });
    const mesh = new THREE.Mesh(ring, mat);
    mesh.name = "snap";
    const pos3D = this.get3DPos(snap.x, snap.y);
    mesh.position.copy(pos3D);
    if (this.plane === "XZ") mesh.rotation.x = -Math.PI / 2;
    else if (this.plane === "YZ") mesh.rotation.y = Math.PI / 2;
    this.overlay.add(mesh);
  }

  private rebuild() {
    this.overlay.clear();
    if (!this.isActive) return;

    const doc = this.engine.getDoc();
    if (!doc) return;

    const ptMap = new Map<number, { x: number; y: number }>();
    for (const p of doc.points || []) ptMap.set(p.id, p);

    const lineMat = (selected: boolean) => new THREE.LineBasicMaterial({
      color: selected ? 0xff8844 : 0x44aaff,
      linewidth: selected ? 2 : 1,
    });

    const selectedSet = new Set(this.engine.state.selectedIds);

    for (const line of doc.lines || []) {
      const s = ptMap.get(line.start);
      const e = ptMap.get(line.end);
      if (!s || !e) continue;
      const selected = selectedSet.has(line.start) || selectedSet.has(line.end);
      const pts = [this.get3DPos(s.x, s.y), this.get3DPos(e.x, e.y)];
      const geo = new THREE.BufferGeometry().setFromPoints(pts);
      const mesh = new THREE.Line(geo, lineMat(selected));
      this.overlay.add(mesh);
    }

    for (const c of doc.circles || []) {
      const center = ptMap.get(c.center);
      if (!center) continue;
      const pts: THREE.Vector3[] = [];
      for (let i = 0; i <= 32; i++) {
        const a = (i / 32) * Math.PI * 2;
        pts.push(this.get3DPos(center.x + c.radius * Math.cos(a), center.y + c.radius * Math.sin(a)));
      }
      const geo = new THREE.BufferGeometry().setFromPoints(pts);
      const mesh = new THREE.Line(geo, new THREE.LineBasicMaterial({ color: 0x44aaff }));
      this.overlay.add(mesh);
    }

    for (const p of doc.points || []) {
      const selected = selectedSet.has(p.id);
      const geo = new THREE.SphereGeometry(selected ? 0.06 : 0.04, 8, 8);
      const mat = new THREE.MeshBasicMaterial({
        color: selected ? 0xff8844 : 0x88ccff,
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.copy(this.get3DPos(p.x, p.y));
      this.overlay.add(mesh);
    }

    this.rebuildSnapIndicator(this.engine.state.hoverSnap || {
      x: 0, y: 0, kind: "none", target_id: null,
    });
  }
}
