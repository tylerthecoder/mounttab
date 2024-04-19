import type { TabUrl, WindowId } from "../state";
import { STATIC_CONFIG } from "../static-config";
import type { BrowserToScriptMessage, ScriptToBrowserMessage } from "../types";

const wsURL = `ws://localhost:${STATIC_CONFIG.serverPort}/ws`

let socket: WebSocket;

const connectToSocket = (): Promise<void> => {
    return new Promise((resolve, _) => {
        console.log("Connecting to socket");

        try {
            socket = new WebSocket(wsURL);
        } catch (e) {
            console.error("Error creating socket", e);
            resolve();
            return;
        }

        socket.onopen = () => {
            console.log("Connected to socket");
        }
        socket.onmessage = (event) => {
            console.log("Got message", event.data);
            const message = JSON.parse(event.data) as ScriptToBrowserMessage;
            handleSocketMessage(message);
        }
        socket.onclose = async () => {
            console.log("Disconnected from socket");
            resolve();
        }
        socket.onerror = (event) => {
            console.error("Error in socket", event);
            resolve();
        }
    });
}

const connectOnLoop = async () => {
    while (true) {
        console.log("Connecting to socket");
        await connectToSocket();
        console.log("Socket disconnected, reconnecting in 1 second...")
        await wait(1000);
    }
}


const sendSocketMessage = (message: BrowserToScriptMessage) => {
    socket.send(JSON.stringify(message));
}

const handleSocketMessage = async (message: ScriptToBrowserMessage) => {
    if (message.getTabs) {
        const windows = await chrome.windows.getAll({ populate: true })

        console.log("Got windows", windows);

        const tabs = windows.reduce((acc, window) => {
            const windowId = window.id?.toString();
            const urls = window.tabs?.map(tab => tab.url ?? "");
            if (!windowId || !urls) {
                return acc;
            }
            acc[windowId] = urls;
            return acc;
        }, {} as Record<WindowId, TabUrl[]>);

        sendSocketMessage({ tabs });
    }
}

connectOnLoop();

const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));


// Listen for the popup script
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    console.log("Got message", message);
    sendResponse({ message: "Got message" });
});
