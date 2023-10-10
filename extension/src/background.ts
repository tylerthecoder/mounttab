// Global state
let ALL_WORKSPACES: string[] = [];
let socket: WebSocket;

const connectToSocket = () => {
    socket = new WebSocket("ws://localhost:3030/chat");
    socket.onopen = () => {
        console.log("Connected to socket");
    }
    socket.onclose = () => {
        console.log("Disconnected from socket");
    }
    socket.onmessage = (event) => {
        console.log("Message received", event.data);
        const strData = event.data.toString();
        const message = JSON.parse(strData);
        handleSocketMessage(message);
    }
    chrome.runtime.onMessage.addListener((message, _sender, _sendResponse) => {
        console.log("Chrome message received", message);
    });
}

type SocketMessage = {
    CloseTab: string,
    AllWorkspaces: string[]
}

let tabs: any[] = []
const getAllTabs = async () => {
    tabs = await chrome.tabs.query({});
    console.log("Tabs", tabs);
};
getAllTabs();

chrome.runtime.onMessage.addListener((request, _sender, sendResponse) => {
    if (request.type === "GetAllWorkspaces") {
        console.log("Sending all workspaces", ALL_WORKSPACES);
        sendResponse(ALL_WORKSPACES);
    }
    if (request.type === "SelectWorkspace") {
        console.log("Selecting workspace request, sending socket message", request.workspace);
        socket.send(JSON.stringify({
            "OpenWorkspace": request.workspace
        }));
    }
    return true
});

const handleSocketMessage = (message: SocketMessage) => {
    if (message.AllWorkspaces) {
        console.log("Setting all workspaces because of socket message", message.AllWorkspaces);
        ALL_WORKSPACES = message.AllWorkspaces;
    }
    if (message.CloseTab) {
        chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
            if (tabs.length > 0) {
                chrome.tabs.remove(tabs[0].id!);
            }
        });
    }
}

connectToSocket();

