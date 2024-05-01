import { getConfig } from "./config";

export type WindowId = string;
export type WorkspaceName = string;
export type TabUrl = string;
export type ComputerName = string;

export type TabState = {
    workspaces: Record<WorkspaceName, TabUrl[]>;
    openWindows: Record<WindowId, WorkspaceName>;
    // Stores the name of the computer that has control of this window
    windowOwners?: Record<WindowId, ComputerName>;
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
            openWindows: {},
            windowOwners: {}
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

    getWindowOwner: async (windowId: WindowId) => {
        const state = await TabService.getFromFs();
        return state.windowOwners?.[windowId] ?? null;
    },

    getWorkspaceOwner: async (workspace: WorkspaceName) => {
        const state = await TabService.getFromFs();
        const windowIds = Object.entries(state.openWindows).filter(([windowId, ws]) => ws === workspace).map(([windowId, ws]) => windowId);
        if (windowIds.length === 0) {
            return null;
        }
        if (windowIds.length > 1) {
            console.error("More than one window for workspace", workspace, windowIds);

            return null;
        }

        return TabService.getWindowOwner(windowIds[0]);
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

    closeWorkspace: async (workspace: WorkspaceName, computer: ComputerName) => {
        console.log("Closing workspace", workspace);
        const state = await TabService.getFromFs();

        const owner = await TabService.getWorkspaceOwner(workspace);

        if (owner !== computer) {
            console.log("Not closing workspace, not owner");
            return;
        }

        Object.entries(state.openWindows).forEach(([windowId, ws]) => {
            if (ws === workspace) {
                delete state.openWindows[windowId];
            }
        });
        await TabService.saveToFs(state);
    },

    openWorkspaceInWindow: async (workspace: WorkspaceName, windowId: WindowId, computer: ComputerName) => {
        console.log(`Opening workspace (${workspace}) in window (${windowId}) for computer (${computer})`);
        const state = await TabService.getFromFs();

        await TabService.closeWorkspace(workspace, computer)

        // remove all windows that point to this workspace
        state.openWindows = Object.fromEntries(Object.entries(state.openWindows).filter(([wid, ws]) => ws !== workspace));
        state.openWindows[windowId] = workspace;

        if (!state.windowOwners) {
            state.windowOwners = {};
        }
        state.windowOwners[windowId] = computer;

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
