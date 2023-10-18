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

        setTimeout(() => {
            const worksapceAction = {
                "WorkspaceAction": [
                    "/home/tylord/dev/tabfs-rs/test/",
                    {
                        CreateTab: "gandalf"
                    }
                ]
            }
            console.log("Sending message", worksapceAction);
            socket.send(JSON.stringify(worksapceAction));

            const worksapceAction2 = {
                "WorkspaceAction": [
                    "/home/tylord/dev/tabfs-rs/test/",
                    {
                        OpenTab: "gandalf"
                    }
                ]
            }
            console.log("Sending message", worksapceAction2);
            socket.send(JSON.stringify(worksapceAction2));

            const worksapceAction3 = {
                "WorkspaceAction": [
                    "/home/tylord/dev/tabfs-rs/test/",
                    {
                        ChangeTabUrl: ["gandalf", "https://tylertracy.com"]
                    }
                ]
            }
            console.log("Sending message", worksapceAction3);
            socket.send(JSON.stringify(worksapceAction3));

        }, 1000);
    }, 1000)
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




