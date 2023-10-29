export type WorkspaceId = string;
export type TabId = string;

export type ToDameonMessage = {
    StartWorkspace?: WorkspaceId
    WorkspaceAction?: [WorkspaceId, WorkspaceAction],
}

export type FromDameonMessage = {
    AllWorkspaces?: Workspace[],
    WorkspaceAction?: WorkspaceAction,
    LoadWorkspace?: Workspace,
}

export type Workspace = {
    id: WorkspaceId,
    name: string,
    tabs: Tab[],
}

export type WorkspaceAction = {
    OpenTab?: TabId,
    CloseTab?: TabId,
    ChangeTabUrl?: [TabId, string]
    CreateTab?: Tab,
}

export type Tab = {
    name: string,
    is_open: boolean,
    url: string,
}

export class TabHolder {
    private idMap: Record<TabId, TabId> = {};
    private tabs: Record<TabId, Tab> = {};

    constructor() { }

    setTabId(tabName: string, browserTabId: string) {
        this.idMap[tabName] = browserTabId;
    }

    getTabNameFromBrowserTabId(browserTabId: string) {
        for (const [key, value] of Object.entries(this.idMap)) {
            if (value === browserTabId) {
                return key;
            }
        }
        return null;
    }

    getChromeTabId(modelId: string) {
        return this.idMap[modelId];
    }

    getTabById(tabId: string) {
        return this.tabs[tabId];
    }

    applyAction(action: WorkspaceAction) {
        console.log("Applying action to tab holder", action);
        if (action.OpenTab) {
            this.tabs[action.OpenTab].is_open = true;
        } else if (action.CloseTab) {
            this.tabs[action.CloseTab].is_open = false;
        } else if (action.CreateTab) {
            this.tabs[action.CreateTab.name] = action.CreateTab;
        } else if (action.ChangeTabUrl) {
            const [tabId, url] = action.ChangeTabUrl;
            this.tabs[tabId].url = url;
        }
        console.log("New tab holder", this);
    }

}
