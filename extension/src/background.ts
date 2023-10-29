import { FromDameonMessage, TabHolder, ToDameonMessage, Workspace, WorkspaceAction } from "./model";

// Global state
let ALL_WORKSPACES: Workspace[] = [];
let socket: WebSocket;
const tabHolder = new TabHolder();

const connectToSocket = () => {
    socket = new WebSocket("ws://localhost:3030/chat");
    socket.onopen = () => {
        onSocketConnected();
    }
    socket.onclose = () => {
        console.log("Disconnected from socket");
    }
    socket.onmessage = (event) => {
        console.log("Message received", event.data);
        const strData = event.data.toString();
        const message = JSON.parse(strData);
        handleDameonMessage(message);
    }
}

const onSocketConnected = async () => {
    console.log("Connected to socket");
    const tabs = await chrome.tabs.query({});

    console.log("Tabs", tabs);

    // const theTab = tabs.find(tab => tab.id == 1164657125);
    //
    // const workspaceId = "/home/tylord/dev/tabfs-rs/test/";
    //
    // if (!theTab) {
    //     throw new Error("Couldn't find tab");
    // }
    //
    // const action1 = {
    //     StartWorkspace: workspaceId
    // }
    // sendMessageToDaemon(action1);
    //
    // await new Promise((resolve) => setTimeout(resolve, 1000));
    //
    // {
    //     const worksapceAction: WorkspaceAction = {
    //         CreateTab: String(theTab.id),
    //     }
    //     sendMessageToDaemon({
    //         WorkspaceAction: [workspaceId, worksapceAction]
    //     });
    //     applyActionToTabHolder(tabHolder, worksapceAction);
    // }
    // {
    //     const worksapceAction: WorkspaceAction = {
    //         ChangeTabUrl: [String(theTab.id), theTab.url ?? ""],
    //     }
    //     sendMessageToDaemon({
    //         WorkspaceAction: [workspaceId, worksapceAction]
    //     });
    //     applyActionToTabHolder(tabHolder, worksapceAction);
    // }

}

const sendMessageToDaemon = (message: ToDameonMessage) => {
    console.log("Sending message to daemon", message);
    socket.send(JSON.stringify(message));
}


const handleDameonMessage = async (message: FromDameonMessage) => {
    console.log("Handling dameon message", message);
    if (message.AllWorkspaces) {
        console.log("Setting all workspaces because of socket message", message.AllWorkspaces);
        ALL_WORKSPACES = message.AllWorkspaces;
    }

    if (message.WorkspaceAction) {
        tabHolder.applyAction(message.WorkspaceAction);
        if (message.WorkspaceAction.ChangeTabUrl) {
            const [tabId, url] = message.WorkspaceAction.ChangeTabUrl;
            const chromeTabId = tabHolder.getChromeTabId(tabId);
            await chrome.tabs.update(parseInt(chromeTabId), { url });
        } else if (message.WorkspaceAction.CloseTab) {
            const tabId = message.WorkspaceAction.CloseTab;
            const realTabId = tabHolder.getChromeTabId(tabId);
            await chrome.tabs.remove(parseInt(realTabId));
        } else if (message.WorkspaceAction.OpenTab) {
            const modelTabId = message.WorkspaceAction.OpenTab;
            const tab = tabHolder.getTabById(modelTabId);
            const newTab = await chrome.tabs.create({
                url: tab.url,
            });
            tabHolder.setTabId(tab.name, String(newTab.id));
        }
    }

    if (message.LoadWorkspace) {
        const workspaceId = message.LoadWorkspace.id;
        const tabs = message.LoadWorkspace.tabs;
        const window = await chrome.windows.create({});

        chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
            const tabName = tabHolder.getTabNameFromBrowserTabId(String(tabId));

            if (!tabName) {
                console.log("Unknown tab")
                return;
            }

            console.log("Tab event", tab, changeInfo);

            if (changeInfo.url) {
                const worksapceAction: WorkspaceAction = {
                    ChangeTabUrl: [tabName, tab.url ?? ""],
                }
                sendMessageToDaemon({
                    WorkspaceAction: [workspaceId, worksapceAction]
                });
            }
        });

        for (const tab of tabs) {
            const chromeTab = await chrome.tabs.create({
                windowId: window.id,
                url: tab.url,
            });
            if (!chromeTab.id) {
                throw new Error("Chrome tab id was null");
            }


            tabHolder.setTabId(tab.name, chromeTab.id.toString());

            tabHolder.applyAction({
                CreateTab: tab,
            });
        }
    }
}

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    console.log("Message from popup", request, sender);
    if (request.type === "GetAllWorkspaces") {
        console.log("Sending all workspaces", ALL_WORKSPACES);
        sendResponse(ALL_WORKSPACES);
    }
    if (request.type === "SelectWorkspace") {
        sendMessageToDaemon({
            StartWorkspace: request.workspace,
        })
    }
    return true
});

connectToSocket();

