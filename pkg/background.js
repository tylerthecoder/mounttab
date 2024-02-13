// src/background.ts
var CONFIG = {
  wsURL: "ws://localhost:3149"
};
var socket;
var connectToSocket = () => {
  return new Promise((resolve, reject) => {
    console.log("Connecting to socket");
    socket = new WebSocket(CONFIG.wsURL);
    socket.onopen = () => {
      console.log("Connected to socket");
      resolve(true);
    };
    socket.onclose = async () => {
      console.log("Disconnected from socket");
      await wait(1000);
    };
    socket.onerror = (event) => {
      console.error("Error in socket", event);
      resolve(false);
    };
    socket.onmessage = (event) => {
      console.log("Got message", event.data);
      const message = JSON.parse(event.data);
      handleSocketMessage(message);
    };
  });
};
var connectOnLoop = async () => {
  let connected = false;
  do {
    connected = await connectToSocket();
    await wait(1000);
  } while (!connected);
};
var sendSocketMessage = (message) => {
  socket.send(JSON.stringify(message));
};
var handleSocketMessage = async (message) => {
  if (message.getTabs) {
    const windows = await chrome.windows.getAll({ populate: true });
    console.log("Got windows", windows);
    const tabs = windows.reduce((acc, window) => {
      const windowId = window.id?.toString();
      const urls = window.tabs?.map((tab) => tab.url ?? "");
      if (!windowId || !urls) {
        return acc;
      }
      acc[windowId] = urls;
      return acc;
    }, {});
    sendSocketMessage({ tabs });
  }
};
connectOnLoop();
var wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
