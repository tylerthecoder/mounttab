import type { BrowserToScriptMessage, ScriptToBrowserMessage, TabUrl, WindowId } from "./types";

const CONFIG = {
    wsURL: "ws://localhost:3149"
}

let socket: WebSocket;

const connectToSocket = (): Promise<boolean> => {
    return new Promise((resolve, reject) => {
        console.log("Connecting to socket");
        socket = new WebSocket(CONFIG.wsURL);
        socket.onopen = () => {
            console.log("Connected to socket");
            resolve(true);
        }
        socket.onclose = async () => {
            console.log("Disconnected from socket");
            await wait(1000);
        }
        socket.onerror = (event) => {
            console.error("Error in socket", event);
            resolve(false);
        }

        socket.onmessage = (event) => {
            console.log("Got message", event.data);
            const message = JSON.parse(event.data) as ScriptToBrowserMessage;
            handleSocketMessage(message);
        }
    });
}

const connectOnLoop = async () => {
    // keep trying to connect to the socket
    let connected = false;
    do {
        connected = await connectToSocket();
        await wait(1000);

    } while (!connected);
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
