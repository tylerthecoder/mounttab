type Message = {
    "AllMessages"?: string[];
}

const socket = new WebSocket("ws://127.0.0.1:3030/chat");
socket.onopen = () => {
    console.log("Connected to socket");


    setTimeout(() => {
        const startWorkspaceMessage = {
            "StartWorkspace": "/home/tylord/dev/tabfs-rs/test/"
        }
        console.log("Sending message", startWorkspaceMessage);
        socket.send(JSON.stringify(startWorkspaceMessage));
    }, 2000)



}
socket.onclose = () => {
    console.log("Disconnected from socket");
}
socket.onmessage = (event) => {
    console.log("Message received", event.data);

    try {
        const data = JSON.parse(event.data) as Message;


        if ("AllMessages" in data && data.AllMessages) {
            const workspace = data.AllMessages[0];
            socket.send(JSON.stringify({
                "OpenWorkspace": workspace
            }));
        }
    } catch (err) {


    }
}




