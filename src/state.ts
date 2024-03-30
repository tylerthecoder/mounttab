import { getConfig } from "./config";

export type WindowId = string;
export type WorkspaceName = string;
export type TabUrl = string;

export type TabState = {
    workspaces: Record<WorkspaceName, TabUrl[]>;
    openWindows: Record<WindowId, WorkspaceName>;
}

const isTabState = (x: any): x is TabState => {
    if (typeof x !== "object") {
        return false;
    }
    if (typeof x.workspaces !== "object") {
        return false;
    }
    if (typeof x.openWindows !== "object") {
        return false;
    }
    if (Object.keys(x.openWindows).some(key => typeof key !== "string")) {
        return false;
    }
    return true;
}


export const TabService = {
    empty: (): TabState => {
        return {
            workspaces: {},
            openWindows: {}
        }
    },

    getFromFs: async (): Promise<TabState> => {
        const { stateFile } = await getConfig();

        const currentTabState = await (async () => {
            const file = Bun.file(stateFile)
            if (!(await file.exists())) {
                return TabService.empty();
            }
            return await file.json();
        })();

        if (!isTabState(currentTabState)) {
            console.log("Invalid state file");
            return TabService.empty();
        }

        return currentTabState;
    },

    saveToFs: async (state: TabState) => {
        const { stateFile } = await getConfig();
        await Bun.write(stateFile, JSON.stringify(state, null, 2));
    },

    getTabsForWorkspace: async (session: string) => {
        const state = await TabService.getFromFs();
        return state.workspaces[session] ?? [];
    },

    getWorkspaceForWindow: async (windowId: WindowId): Promise<string | null> => {
        const state = await TabService.getFromFs();
        return state.openWindows[windowId] ?? null;
    },

    getTabsForWindow: async (windowId: WindowId) => {
        const state = await TabService.getFromFs();
        const workspace = state.openWindows[windowId];
        if (!workspace) {
            return null;
        }
        return state.workspaces[workspace] ?? [];
    },

    setTabs: async (workspace: WorkspaceName, tabs: TabUrl[]) => {
        const state = await TabService.getFromFs();

        // Calculate the diff
        const existingTabs = state.workspaces[workspace] ?? [];
        const newTabs = tabs.filter(tab => !existingTabs.includes(tab));
        const removedTabs = existingTabs.filter(tab => !tabs.includes(tab));
        if (newTabs.length > 0 || removedTabs.length > 0) {
            console.log(`Found diff, setting tabs for workspace: ${workspace}`);
            console.log("New tabs", newTabs);
            console.log("Removed tabs", removedTabs);
            state.workspaces[workspace] = tabs;
            await TabService.saveToFs(state);
        }

    },

    closeWorkspace: async (workspace: WorkspaceName) => {
        console.log("Closing workspace", workspace);
        const state = await TabService.getFromFs();
        Object.entries(state.openWindows).forEach(([windowId, ws]) => {
            if (ws === workspace) {
                delete state.openWindows[windowId];
            }
        });
        await TabService.saveToFs(state);
    },

    openWorkspaceInWindow: async (workspace: WorkspaceName, windowId: WindowId) => {
        console.log("Opening workspace in window", workspace, windowId);
        const state = await TabService.getFromFs();
        await TabService.closeWorkspace(workspace);
        state.openWindows[windowId] = workspace;
        await TabService.saveToFs(state);
    },

    getAllWorkspaces: async () => {
        const state = await TabService.getFromFs();
        return Object.keys(state.workspaces);
    },

    getActiveWorkspace: async () => {
        const state = await TabService.getFromFs();
        return Object.values(state.openWindows)
    },

    getInactiveWorkspaces: async () => {
        const state = await TabService.getFromFs();
        const activeWindows = new Set(Object.values(state.openWindows));
        return Object.keys(state.workspaces).filter(workspace => !activeWindows.has(workspace));
    }


}
