
export type ScriptToBrowserMessage = {
    getTabs: boolean;
}

export type WindowId = string;

export type TabUrl = string;

export type BrowserToScriptMessage = {
    tabs: Record<WindowId, TabUrl[]>
}