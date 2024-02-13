

// start a websocket server on port 8080

import type { BrowserToScriptMessage } from "./types";



Bun.serve({
    port: 3149,
    fetch(req, server) {
        // upgrade the request to a WebSocket
        if (server.upgrade(req)) {
            return; // do not return a Response
        }
        return new Response("Upgrade failed :(", { status: 500 });
    },
    websocket: {
        message(ws, message) {
            // got a messages from the client
            const parsed = JSON.parse(typeof message === "string" ? message : message.toString()) as BrowserToScriptMessage;

            if (parsed.tabs) {
                // save tabs to a file
                console.log("Got tabs", parsed.tabs);
            }


        },
        open(ws) {
            console.log("Extension connected");

            ws.send(JSON.stringify({ getTabs: true }));

        }
    }
})



