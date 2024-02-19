import { spawnSync } from "bun";
import { type BrowserToScriptMessage, type TabState } from "./types";
import { $ } from "bun";

export const CONFIG = {
    port: 3149,
}

const STATE_FILE = `${process.env.HOME}/.config/mt/state.json`;
await $`mkdir -p ${process.env.HOME}/.config/mt`;

let currentTabState = await (async () => {
    const file = Bun.file(STATE_FILE)
    if (!(await file.exists())) {
        return {
            tabs: {}
        } as TabState;
    }

    return await file.json() as TabState;
})();

let currentlyStartingWorkspace: string | null = null;
let chromeWindowIdToWorkspace: Record<string, string> = {};
const notConnectedWindowIds = new Set<string>();

const startServer = () => {
    Bun.serve({
        port: CONFIG.port,
        async fetch(req, server) {
            console.log(req);
            const url = new URL(req.url);

            if (url.pathname === "/ws") {
                if (server.upgrade(req)) {
                    return; // do not return a Response
                }
                return new Response("Upgrade failed :(", { status: 500 });
            }

            if (url.pathname === "/start") {
                const workspace = url.searchParams.get("workspace");

                if (!workspace) {
                    return new Response("No workspace specified", { status: 400 });
                }

                console.log("Starting workspace", workspace);

                if (!currentTabState.tabs[workspace]) {
                    console.log("No tabs for workspace", workspace);
                }

                const tabs = currentTabState.tabs[workspace] ?? [];

                currentlyStartingWorkspace = workspace;

                const command = ["chromium", "--new-window", ...tabs];

                spawnSync(command);

            }
        },
        websocket: {
            async message(ws, message) {
                // got a messages from the client
                const parsed = JSON.parse(typeof message === "string" ? message : message.toString()) as BrowserToScriptMessage;

                if (parsed.tabs) {
                    for (const windowId in parsed.tabs) {
                        const workspace = chromeWindowIdToWorkspace[windowId];
                        if (workspace) {
                            currentTabState.tabs[workspace] = parsed.tabs[windowId];
                        } else if (currentlyStartingWorkspace && !notConnectedWindowIds.has(windowId)) {
                            console.log("Setting workspace", windowId, currentlyStartingWorkspace);
                            chromeWindowIdToWorkspace[windowId] = currentlyStartingWorkspace;
                            currentlyStartingWorkspace = null;
                        } else {
                            if (!notConnectedWindowIds.has(windowId)) {
                                console.log("Adding Not connected", windowId);
                            }
                            notConnectedWindowIds.add(windowId);
                        }
                    }

                    await Bun.write(STATE_FILE, JSON.stringify(currentTabState, null, 2));
                }

                setTimeout(() => {
                    ws.send(JSON.stringify({ getTabs: true }));
                }, 500);
            },
            open(ws) {
                console.log("Extension connected");

                ws.send(JSON.stringify({ getTabs: true }));

            },
            close(ws) {
                console.log("Extension disconnected");
            }
        }
    })
}





const command = process.argv[2];
if (command === "start") {
    const workspace = process.argv[3];
    await fetch(`http://localhost:${CONFIG.port}/start?workspace=${workspace}`)
} else if (command == "serve") {
    startServer();
} else if (command == "list-workspaces") {
    console.log(Object.keys(currentTabState.tabs).join("\n"));
} else {
    console.log("Not a valid command");
}


