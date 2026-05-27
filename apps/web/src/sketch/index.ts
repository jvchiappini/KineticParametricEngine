export { SketchEditor } from "./SketchEditor";

export type { SketchModel } from "./model/SketchModel";
export { createSketchModel } from "./model/SketchModel";
export type { Entity, EntityId, PointEntity, LineEntity, CircleEntity, ArcEntity, PolylineEntity } from "./model/Entity";
export { makePoint, makeLine, makeCircle, makeArc, makePolyline } from "./model/Entity";
export type { Constraint, ConstraintId, CoincidentConstraint, HorizontalConstraint, VerticalConstraint, ParallelConstraint, PerpendicularConstraint, EqualConstraint, SymmetricConstraint, TangentConstraint, CollinearConstraint, FixedConstraint, LengthDimConstraint, AngleDimConstraint, RadiusDimConstraint } from "./model/Constraint";
export { SketchHistory, compoundCommand } from "./model/SketchHistory";
export { exportSketchProfile, exportRevolveProfile } from "./model/SketchExport";
export type { ProfileNode, RevolveProfile } from "./model/SketchExport";

export { SketchRenderer } from "./renderer/SketchRenderer";
export { GridRenderer } from "./renderer/GridRenderer";
export { SnapIndicator } from "./renderer/SnapIndicator";
export { createEntityMesh } from "./renderer/EntityMesh";
export { createConstraintVisual } from "./renderer/ConstraintVisual";

export { resolveSnap } from "./snap/SnapEngine";
export type { SnapKind, SnapCandidate, SnapResult } from "./snap/SnapTypes";

export { solveConstraints } from "./constraints/ConstraintSolver";
export { ConstraintGraph } from "./constraints/ConstraintGraph";
export { countDOF, dofBreakdown } from "./constraints/DOFTracker";

export { ToolRegistry } from "./tools/ToolRegistry";
export { LineTool } from "./tools/LineTool";
export { RectangleTool } from "./tools/RectangleTool";
export { CircleTool } from "./tools/CircleTool";
export { ArcTool } from "./tools/ArcTool";
export { PolylineTool } from "./tools/PolylineTool";
export { SelectTool } from "./tools/SelectTool";
export { DimensionTool } from "./tools/DimensionTool";
export type { SketchTool, ToolContext } from "./tools/types";

export { InputHandler } from "./interaction/InputHandler";
export { detectHover } from "./interaction/HoverDetector";
export type { HoverResult } from "./interaction/HoverDetector";
export { SelectionManager } from "./interaction/SelectionManager";
export { CameraController } from "./interaction/CameraController";

export { SketchToolbar } from "./ui/SketchToolbar";
export { SketchPropertiesPanel } from "./ui/SketchPropertiesPanel";
export { ConstraintPanel } from "./ui/ConstraintPanel";
export { StatusBar } from "./ui/StatusBar";
export { DimensionInput } from "./ui/DimensionInput";
export { ConstraintMenu } from "./ui/ConstraintMenu";
export { Viewport3D } from "./viewport3d/Viewport3D";
