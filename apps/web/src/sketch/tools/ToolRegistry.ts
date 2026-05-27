import type { SketchTool, ToolContext } from "./types";

/**
 * Registry for managing sketch tool lifecycle. Tools are registered by name
 * and can be activated/deactivated. The active tool receives all pointer and
 * keyboard events.
 */
export class ToolRegistry {
  private tools = new Map<string, SketchTool>();
  private activeTool: SketchTool | null = null;

  /**
   * Register a tool. The tool's name is used as the lookup key.
   */
  register(tool: SketchTool): void {
    this.tools.set(tool.name, tool);
  }

  /**
   * Activate a tool by name. Deactivates the previously active tool
   * before activating the new one.
   */
  activate(name: string, context: ToolContext): void {
    if (this.activeTool) {
      this.activeTool.onDeactivate(context);
    }
    const tool = this.tools.get(name);
    if (tool) {
      this.activeTool = tool;
      tool.onActivate(context);
    }
  }

  /**
   * Returns the currently active tool, or null if none is active.
   */
  getActive(): SketchTool | null {
    return this.activeTool;
  }

  /**
   * Look up a registered tool by name.
   */
  get(name: string): SketchTool | undefined {
    return this.tools.get(name);
  }
}
