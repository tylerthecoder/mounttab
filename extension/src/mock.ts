const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const socket = new WebSocket("ws://127.0.0.1:3030/chat");
socket.onopen = async () => {
  console.log("Connected to socket");

  await wait(1000);

  socket.send(
    JSON.stringify({
      OpenTab: "tylertracy.com?n=" + Math.random(),
    }),
  );
};
socket.onclose = () => {
  console.log("Disconnected from socket");
};
socket.onmessage = (event) => {
  console.log("Message received", event.data);
};
