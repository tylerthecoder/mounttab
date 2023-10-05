
const connectToSocket = () => {
    const socket = new WebSocket("ws://localhost:3030/chat");
    socket.onopen = () => {
        console.log("Connected to socket");
        socket.send(JSON.stringify({
            "OpenWorkspace": "test"
        }));
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
    CloseTab: string;
}

const getAllTabs = async () => {
    const tabs = await chrome.tabs.query({});
};



const handleSocketMessage = (message: SocketMessage) => {
    if (message.CloseTab) {
        chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
            if (tabs.length > 0) {
                chrome.tabs.remove(tabs[0].id!);
            }
        });
    }
}

connectToSocket();

