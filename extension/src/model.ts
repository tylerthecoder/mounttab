export type WorkspaceId = string;
export type TabId = string;

export type ToDameonMessage = {
    StartWorkspace?: WorkspaceId
    WorkspaceAction?: [WorkspaceId, WorkspaceAction],
}

export type FromDameonMessage = {
    AllWorkspaces?: WorkspaceId[],
    WorkspaceAction?: WorkspaceAction,
}

export type WorkspaceAction = {
    OpenTab?: TabId,
    CloseTab?: TabId,
    ChangeTabUrl?: [TabId, string]
    CreateTab?: TabId,
}


export type Tab = {
    id: string,
    is_open: boolean,
    url: string,
}


export type TabHolder = {
    // new id to old id
    idMap: Record<TabId, TabId>,
    tabs: Record<TabId, Tab>,
}

export const updateTabId = (holder: TabHolder, old_id: string, new_id: string) => {
    holder.idMap[old_id] = new_id;
    console.log("updated Id map tab id", holder.idMap);
}

export const getNewIdFromId = (holder: TabHolder, id: string) => {
    if (id in holder.idMap) {
        return holder.idMap[id];
    }
    return id;
}

export const applyActionToTabHolder = (holder: TabHolder, action: WorkspaceAction) => {
    console.log("Applying action to tab holder", action);
    if (action.OpenTab) {
        holder.tabs[action.OpenTab].is_open = true;
    } else if (action.CloseTab) {
        holder.tabs[action.CloseTab].is_open = false;
    } else if (action.CreateTab) {
        holder.tabs[action.CreateTab] = { id: action.CreateTab, is_open: false, url: "" };
    } else if (action.ChangeTabUrl) {
        const [tabId, url] = action.ChangeTabUrl;
        holder.tabs[tabId].url = url;
    }
    console.log("New tab holder", holder);
}
