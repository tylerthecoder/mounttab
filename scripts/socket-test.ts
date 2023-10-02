    const socket = new WebSocket("ws://127.0.0.1:3030/chat");
    socket.onopen = () => {
        console.log("Connected to socket");
        socket.send(JSON.stringify({
            "OpenWorkspace": "/home/tylord/dev/tabfs-rs/test"
        }));
    }
    socket.onclose = () => {
        console.log("Disconnected from socket");
    }
    socket.onmessage = (event) => {
        console.log("Message received", event.data);
    }
