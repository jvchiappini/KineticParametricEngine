import { useRef, useState, useEffect, useCallback, type JSX } from "react";
const btn = { padding: "4px 12px", border: "1px solid #444", borderRadius: "4px", background: "#2a2a4a", color: "#ccc", cursor: "pointer", fontSize: "12px", fontFamily: "inherit", whiteSpace: "nowrap" } as const;
import { createSketchModel, cloneModel } from "./model/SketchModel";
import type { SketchModel } from "./model/SketchModel";
import { SketchHistory, addConstraintCommand } from "./model/SketchHistory";
import type { Command } from "./model/SketchHistory";
import { solveWithWasm } from "../wasm";
import { solveConstraints } from "./constraints/ConstraintSolver";
import { countDOF } from "./constraints/DOFTracker";
import { serializeSketches, deserializeSketches } from "./model/SketchSerializer";
import { generateConstraintId } from "./model/Constraint";
import type { Constraint } from "./model/Constraint";
import { SketchRenderer } from "./renderer/SketchRenderer";
import { resolveSnap } from "./snap/SnapEngine";
import type { SnapResult } from "./snap/SnapTypes";
import { ToolRegistry } from "./tools/ToolRegistry";
import { LineTool } from "./tools/LineTool";
import { RectangleTool } from "./tools/RectangleTool";
import { CircleTool } from "./tools/CircleTool";
import { ArcTool } from "./tools/ArcTool";
import { PolylineTool } from "./tools/PolylineTool";
import { SelectTool } from "./tools/SelectTool";
import { DimensionTool } from "./tools/DimensionTool";
import { InputHandler } from "./interaction/InputHandler";
import { SketchToolbar } from "./ui/SketchToolbar";
import { SketchPropertiesPanel } from "./ui/SketchPropertiesPanel";
import { ConstraintPanel } from "./ui/ConstraintPanel";
import { StatusBar } from "./ui/StatusBar";
import { Viewport3D } from "./viewport3d/Viewport3D";
import type { SketchData } from "./viewport3d/ExtrudeBuilder";
import type { ToolContext } from "./tools/types";
import { initWasm, hello, wasmCountDOF, wasmCountDOFSync, wasmGetContours } from "../wasm";

type SketchPlane = "XY" | "XZ" | "YZ";

interface SketchDoc {
  id: string; name: string;
  model: SketchModel; history: SketchHistory;
  depth: number; bevel: boolean; bevelSize: number;
}

let nextSketchId = 2;

function newDoc(plane: SketchPlane = "XZ", offset = 0): SketchDoc {
  const m = createSketchModel(); m.plane = plane; m.planeOffset = offset;
  return { id: `sketch_${nextSketchId++}`, name: `Sketch ${nextSketchId - 1}`, model: m, history: new SketchHistory(), depth: 10, bevel: true, bevelSize: 0.5 };
}

export function SketchEditor(): JSX.Element {
  const [sketches, setSketches] = useState<SketchDoc[]>([newDoc("XZ")]);
  const [activeSketchId, setActiveSketchId] = useState(sketches[0].id);

  const containerRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const rendererRef = useRef<SketchRenderer | null>(null);
  const modelRef = useRef<SketchModel>(sketches[0].model);
  const historyRef = useRef<SketchHistory>(sketches[0].history);
  const toolRegistryRef = useRef<ToolRegistry | null>(null);
  const inputHandlerRef = useRef<InputHandler | null>(null);

  const cursorCoordsRef = useRef({ sketchX: 0, sketchY: 0, rawX: 0, rawY: 0 });
  const snapResultRef = useRef<SnapResult>({
    candidate: { kind: "none", x: 0, y: 0, distance: 0 }, isSnapped: false, rawX: 0, rawY: 0,
  });

  const [activeTool, setActiveTool] = useState("select");
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [hint, setHint] = useState("Ready");
  const [dof, setDof] = useState(0);
  const [cursorX, setCursorX] = useState(0);
  const [cursorY, setCursorY] = useState(0);
  const [zoomLevel, setZoomLevel] = useState(100);
  const [snapKind, setSnapKind] = useState("none");
  const [gridVisible, setGridVisible] = useState(true);
  const [constructionMode, setConstructionMode] = useState(false);
  const [show3D, setShow3D] = useState(false);
  const [, forceUpdate] = useState(0);
  const [wasmReady, setWasmReady] = useState(false);

  const requestRender = useCallback(() => rendererRef.current?.render(), []);
  const rebuildScene = useCallback(() => {
    const m = modelRef.current;
    const r = rendererRef.current;
    if (!r) return;
    setDof(wasmReady ? wasmCountDOFSync(m) : countDOF(m));
    r.rebuild(m);
    r.render();
  }, [wasmReady]);

  const setModel = useCallback((model: SketchModel) => {
    modelRef.current = model;
    setSketches((prev) => prev.map((s) => (s.id === activeSketchId ? { ...s, model } : s)));
    rebuildScene();
  }, [activeSketchId, rebuildScene]);

  const activeDoc = sketches.find((s) => s.id === activeSketchId) ?? sketches[0];
  const activeModel = activeDoc.model;
  const activeHistory = activeDoc.history;

  useEffect(() => {
    modelRef.current = activeModel;
    historyRef.current = activeHistory;
    setSelectedIds([]);
    rebuildScene();
  }, [activeModel, activeHistory]);

  const handleNewSketch = useCallback((plane: SketchPlane) => {
    const doc = newDoc(plane);
    setSketches((prev) => [...prev, doc]);
    setActiveSketchId(doc.id);
    setConstructionMode(false);
  }, []);

  const handleActivateSketch = useCallback((id: string) => {
    if (id === activeSketchId) return;
    setSketches((prev) => prev.map((s) => (s.id === activeSketchId ? { ...s, model: modelRef.current } : s)));
    setActiveSketchId(id);
  }, [activeSketchId]);

  const handleDeleteSketch = useCallback((id: string) => {
    if (sketches.length <= 1) return;
    setSketches((prev) => {
      const next = prev.filter((s) => s.id !== id);
      if (id === activeSketchId) setActiveSketchId(next[0].id);
      return next;
    });
  }, [sketches.length, activeSketchId]);

  const handleToolChange = useCallback((toolName: string) => {
    const reg = toolRegistryRef.current;
    if (!reg) return;
    const ctx = buildContextRef.current();
    reg.getActive()?.onDeactivate(ctx);
    reg.activate(toolName, ctx);
    setActiveTool(toolName);
    setHint(reg.getActive()?.getHint() ?? "Ready");
    inputHandlerRef.current?.setTool(reg.getActive());
  }, []);

  const buildContextRef = useRef<() => ToolContext>(() => { throw new Error("not initialized"); });
  const updateCursor = useCallback((clientX: number, clientY: number) => {
    const r = rendererRef.current;
    if (!r) return;
    const w = r.screenToSketch(clientX, clientY);
    const res = resolveSnap(w.x, w.y, modelRef.current, 10, activeTool);
    cursorCoordsRef.current = {
      sketchX: res.isSnapped ? res.candidate.x : w.x, sketchY: res.isSnapped ? res.candidate.y : w.y,
      rawX: w.x, rawY: w.y,
    };
    snapResultRef.current = res;
    setCursorX(w.x); setCursorY(w.y);
    setSnapKind(res.isSnapped ? res.candidate.kind : "none");
  }, [activeTool]);
  const getCursorInSketch = useCallback(() => ({ x: cursorCoordsRef.current.rawX, y: cursorCoordsRef.current.rawY }), []);
  const getSnapAwareCursor = useCallback(() => ({ x: cursorCoordsRef.current.sketchX, y: cursorCoordsRef.current.sketchY }), []);

  const buildContext = useCallback((): ToolContext => {
    const addSel = (ids: string[]) => setSelectedIds((prev) => [...new Set([...prev, ...ids])]);
    const ctx: ToolContext = {
      model: null as unknown as SketchModel, setModel, selectedIds, addToSelection: addSel, clearSelection: () => setSelectedIds([]),
      snapResult: snapResultRef.current, history: historyRef.current, renderer: rendererRef.current!,
      onRequestRender: requestRender, getCursorInSketch, getSnapAwareCursor,
      setPreview: (mesh) => rendererRef.current?.setPreview(mesh), updateCursor,
      screenToWorld: (cX, cY) => rendererRef.current?.screenToSketch(cX, cY) ?? { x: 0, y: 0 },
      onPan: (dx, dy) => { rendererRef.current?.pan(dx, dy); requestRender(); },
      onZoom: (factor, cX, cY) => { rendererRef.current?.zoom(factor, cX, cY); setZoomLevel((p) => p * factor); requestRender(); },
      onContextMenu: () => {},
    };
    Object.defineProperty(ctx, "model", { get: () => modelRef.current, enumerable: true });
    return ctx;
  }, [selectedIds, setModel, requestRender, getCursorInSketch, getSnapAwareCursor, updateCursor]);

  buildContextRef.current = buildContext;

  const handleAddConstraint = useCallback((kind: string, entityIds: string[]) => {
    let m = modelRef.current;
    const addOne = (extra: Record<string, unknown>) => {
      const constraint = { id: generateConstraintId(), kind, entities: entityIds, satisfied: true, ...extra } as unknown as Constraint;
      const cmd = addConstraintCommand(constraint);
      historyRef.current.push(cmd); m = cmd.execute(m);
    };
    const kinds: Record<string, [number, Record<string, unknown>]> = {
      horizontal: [1, { line: entityIds[0] }], vertical: [1, { line: entityIds[0] }], fixed: [1, { entity: entityIds[0] }],
      coincident: [2, { pointA: entityIds[0], pointB: entityIds[1] }], parallel: [2, { lineA: entityIds[0], lineB: entityIds[1] }],
      perpendicular: [2, { lineA: entityIds[0], lineB: entityIds[1] }], equal: [2, { entityA: entityIds[0], entityB: entityIds[1] }],
      tangent: [2, { entityA: entityIds[0], entityB: entityIds[1] }],
    };
    const entry = kinds[kind];
    if (!entry || entityIds.length !== entry[0]) return;
    addOne(entry[1]);
    if (wasmReady) {
      solveWithWasm(m).then(() => { modelRef.current = m; setModel(m); });
    } else {
      setModel(solveConstraints(m, { maxIterations: 500 }).model);
    }
  }, [setModel, wasmReady]);

  const handlePlaneChange = useCallback((plane: SketchPlane) => { const n = cloneModel(modelRef.current); n.plane = plane; setModel(n); }, [setModel]);
  const handlePlaneOffsetChange = useCallback((offset: number) => { const n = cloneModel(modelRef.current); n.planeOffset = offset; setModel(n); }, [setModel]);
  const updateSketch = useCallback((fn: (d: SketchDoc) => SketchDoc) => setSketches((prev) => prev.map((s) => (s.id === activeSketchId ? fn(s) : s))), [activeSketchId]);
  const handleDepthChange = useCallback((d: number) => updateSketch((s) => ({ ...s, depth: d })), [updateSketch]);
  const handleBevelToggle = useCallback((v: boolean) => updateSketch((s) => ({ ...s, bevel: v })), [updateSketch]);
  const handleBevelSizeChange = useCallback((v: number) => updateSketch((s) => ({ ...s, bevelSize: v })), [updateSketch]);
  const handleRemoveConstraint = useCallback((constraintId: string) => {
    let m = modelRef.current;
    const cmd: Command = {
      description: "Remove constraint",
      execute: (x) => { const n = { ...x, constraints: new Map(x.constraints) }; n.constraints.delete(constraintId); return n; },
      undo: (x) => { const c = modelRef.current.constraints.get(constraintId); if (!c) return x; const n = { ...x, constraints: new Map(x.constraints) }; n.constraints.set(constraintId, c); return n; },
    };
    historyRef.current.push(cmd); setModel(cmd.execute(m));
  }, [setModel]);

  useEffect(() => {
    const c = containerRef.current;
    if (!c) return;
    const renderer = new SketchRenderer(c);
    renderer.resize(c.clientWidth, c.clientHeight);
    rendererRef.current = renderer;
    const registry = new ToolRegistry();
    for (const T of [SelectTool, LineTool, RectangleTool, CircleTool, ArcTool, PolylineTool, DimensionTool]) registry.register(new T());
    toolRegistryRef.current = registry;
    const context = buildContext();
    registry.activate("select", context);
    const inputHandler = new InputHandler(c, context, handleToolChange);
    inputHandler.setTool(registry.getActive());
    inputHandler.attach();
    inputHandlerRef.current = inputHandler;
    rebuildScene();
    const ro = new ResizeObserver((entries) => {
      for (const e of entries) { renderer.resize(e.contentRect.width, e.contentRect.height); renderer.render(); }
    });
    ro.observe(c);
    initWasm().then(() => { setWasmReady(true); console.log("WASM:", hello()); });
    return () => { ro.disconnect(); inputHandler.detach(); renderer.dispose(); };
  }, []);

  const model = modelRef.current;
  const totalDOF = sketches.reduce((sum, s) => sum + (wasmReady ? wasmCountDOFSync(s.model) : countDOF(s.model)), 0);

  const handleFinish = useCallback(() => {
    const json = serializeSketches(sketches);
    const blob = new Blob([JSON.stringify(json, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob); const a = document.createElement("a"); a.href = url; a.download = "sketch-export.kpe"; a.click(); URL.revokeObjectURL(url);
  }, [sketches]);

  const handleOpenKpe = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]; if (!file) return;
    const reader = new FileReader();
    reader.onload = () => { try {
      const docs = deserializeSketches(reader.result as string);
      setSketches(docs.map((d) => ({ ...d, history: new SketchHistory() })));
      setActiveSketchId(docs[0].id);
    } catch { alert("Error loading .kpe file"); } };
    reader.readAsText(file); e.target.value = "";
  }, []);

  const sketchDatas: SketchData[] = sketches.map((s, i) => ({
    model: s.id === activeSketchId ? modelRef.current : s.model,
    depth: s.depth, index: i, bevel: s.bevel, bevelSize: s.bevelSize,
  }));

  return (
    <div style={{ display: "flex", flexDirection: "column", width: "100%", height: "100%", background: "#111", color: "#ddd", fontFamily: "'Segoe UI', system-ui, sans-serif", overflow: "hidden" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "4px", padding: "0 10px", background: "#0d0d1a", borderBottom: "1px solid #2a2a3e", flexShrink: 0, height: "34px" }}>
        <span style={{ fontWeight: 700, color: "#8af", fontSize: "12px", marginRight: "4px" }}>KPE</span>
        <div style={{ display: "flex", gap: "1px", alignSelf: "stretch", marginTop: "2px" }}>
          {sketches.map((s) => (
            <div key={s.id} style={{ display: "flex", alignItems: "center", gap: "2px" }}>
              <button style={{
                padding: "3px 10px", border: "none", borderRadius: "4px 4px 0 0",
                background: s.id === activeSketchId ? "#2a2a4a" : "transparent",
                color: s.id === activeSketchId ? "#8af" : "#777",
                cursor: "pointer", fontSize: "11px", fontFamily: "inherit",
              }} onClick={() => handleActivateSketch(s.id)}>
                {s.name}
              </button>
              {sketches.length > 1 && (
                <button style={{
                  background: "none", border: "none", color: "#555", cursor: "pointer",
                  fontSize: "10px", padding: "0 2px", fontFamily: "inherit",
                }} onClick={() => handleDeleteSketch(s.id)} title="Delete sketch">x</button>
              )}
            </div>
          ))}
        </div>
        <button style={{ ...btn, padding: "2px 6px", fontSize: "11px", background: "#1a1a3a" }}
          onClick={() => handleNewSketch("XZ")} title="New sketch on XZ plane">+</button>
        <input ref={fileInputRef} type="file" accept=".kpe,.json" onChange={handleOpenKpe} style={{ display: "none" }} />
        <span style={{ width: "1px", height: "16px", background: "#333", margin: "0 6px" }} />
        <span style={{ fontSize: "11px", color: "#888" }}>
          {model.plane} plane {model.planeOffset !== 0 ? `@ ${model.planeOffset}mm` : ""}
        </span>
        <span style={{ flex: 1 }} />
        <button style={{ ...btn, padding: "2px 6px", fontSize: "11px" }} onClick={() => fileInputRef.current?.click()} title="Open .kpe file">Open</button>
        <button style={{ ...btn, background: totalDOF > 0 ? "#5a3030" : "#2a2a4a" }} onClick={handleFinish}>
          {totalDOF > 0 ? `Finish (${totalDOF} DOF)` : "Finish"}
        </button>
        <button style={{ ...btn, background: show3D ? "#3a3a6a" : "#2a2a4a" }} onClick={() => setShow3D((v) => !v)}>{show3D ? "Sketch" : "3D View"}</button>
        <span style={{ fontSize: "10px", color: wasmReady ? "#4a4" : "#644", marginLeft: "4px" }}>{wasmReady ? "⚡WASM" : "⏳WASM"}</span>
      </div>
      {totalDOF > 0 && <div style={{ background: "#3a2020", color: "#e88", fontSize: "11px", padding: "1px 10px", borderBottom: "1px solid #5a3030", flexShrink: 0 }}>{totalDOF} DOF — {sketches.filter(s => (wasmReady ? wasmCountDOFSync(s.model) : countDOF(s.model)) > 0).length} underconstrained</div>}
      <div style={{ display: "flex", flex: show3D ? "0 0 60%" : 1, overflow: "hidden", minHeight: show3D ? 200 : 0, borderBottom: show3D ? "3px solid #333" : "none" }}>
        <SketchPropertiesPanel plane={model.plane} planeOffset={model.planeOffset} gridVisible={gridVisible} constructionMode={constructionMode} entityCount={model.entities.size} constraintCount={model.constraints.size} dof={dof} depth={activeDoc.depth} bevel={activeDoc.bevel} bevelSize={activeDoc.bevelSize} onPlaneChange={handlePlaneChange} onPlaneOffsetChange={handlePlaneOffsetChange} onGridToggle={() => { setGridVisible((v) => !v); const g = rendererRef.current?.getLayer(0); if (g) g.visible = !gridVisible; requestRender(); }} onConstructionModeToggle={() => setConstructionMode((v) => !v)} onDepthChange={handleDepthChange} onBevelToggle={handleBevelToggle} onBevelSizeChange={handleBevelSizeChange} />
        <SketchToolbar activeTool={activeTool} onToolChange={handleToolChange} />
        <div ref={containerRef} style={{ flex: 1, position: "relative" }} />
        <ConstraintPanel model={model} selectedIds={selectedIds} onAddConstraint={handleAddConstraint} onRemoveConstraint={handleRemoveConstraint} />
      </div>
      {show3D && <div style={{ flex: "0 0 40%", overflow: "hidden" }}><Viewport3D sketches={sketchDatas} /></div>}
      <StatusBar hint={hint} dof={dof} cursorX={cursorX} cursorY={cursorY} zoomLevel={zoomLevel} snapKind={snapKind} />
    </div>
  );
}
