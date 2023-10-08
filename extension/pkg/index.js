// src/index.ts
var connectToSocket = () => {
  const socket = new WebSocket("ws://localhost:8080");
  socket.onopen = () => {
    console.log("Connected to socket");
  };
  chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    console.log("Message received", message);
  });
};
connectToSocket();
