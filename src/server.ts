import { $, spawnSync } from "bun";
import { CONFIG, type BrowserToScriptMessage, type TabState } from "./types";



let currentTabState = await (async () => {
    const file = Bun.file(CONFIG.stateFile);
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

console.log("Current state", currentTabState);


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
                console.log("currentlyStartingWorkspace", currentlyStartingWorkspace);

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


                await Bun.write(CONFIG.stateFile, JSON.stringify(currentTabState, null, 2));
            }

            setTimeout(() => {
                ws.send(JSON.stringify({ getTabs: true }));
            }, 500);
        },
        open(ws) {
            console.log("Extension connected");

            ws.send(JSON.stringify({ getTabs: true }));

        }
    }
})



