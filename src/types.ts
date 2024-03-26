import type { TabUrl, WindowId } from "./state";

export type ScriptToBrowserMessage = {
  getTabs: boolean;
};

export type BrowserToScriptMessage = {
  tabs: Record<WindowId, TabUrl[]>;
};
