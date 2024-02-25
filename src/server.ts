import { spawnSync } from "bun";
import { type BrowserToScriptMessage } from "./types";
import { getTabStateFromFs, saveTabStateToFs } from "./state";
import { Config } from "./config";

const currentState = await getTabStateFromFs();

let currentlyStartingWorkspace: string | null = null;
let chromeWindowIdToWorkspace: Record<string, string> = {};
const notConnectedWindowIds = new Set<string>();

export const startServer = () => {
    Bun.serve({
        port: Config.serverPort,
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

                if (!currentState.tabs[workspace]) {
                    console.log("No tabs for workspace", workspace);
                }

                const tabs = currentState.tabs[workspace] ?? [];

                currentlyStartingWorkspace = workspace;

                const command = ["chromium", "--new-window", ...tabs];

                spawnSync(command);
            }

            if (url.pathname === "/get-workspace-for-windowid") {
                const windowId = url.searchParams.get("windowId");

                if (!windowId) {
                    return new Response("No windowId specified", { status: 400 });
                }

                const workspace = chromeWindowIdToWorkspace[windowId];

                const res = new Response("Ok", { status: 200 });
                res.headers.set('Access-Control-Allow-Origin', '*');
                res.headers.set('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
                return res

            }

            if (url.pathname === "/assign-window-to-workspace") {
                const workspace = url.searchParams.get("workspace");
                const windowId = url.searchParams.get("windowId");

                if (!workspace) {
                    return new Response("No workspace specified", { status: 400 });
                }
                if (!windowId) {
                    return new Response("No windowId specified", { status: 400 });
                }

                console.log("Assigning window to workspace", windowId, workspace);

                notConnectedWindowIds.delete(windowId);
                chromeWindowIdToWorkspace[windowId] = workspace;

                const res = new Response("Ok", { status: 200 });
                res.headers.set('Access-Control-Allow-Origin', '*');
                res.headers.set('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
                return res
            }

            if (url.pathname === "/get-workspaces") {
                const inactive = url.searchParams.get("inactive") === "true";

                let workspaces = Object.keys(currentState.tabs);

                if (inactive) {
                    const activeWindows = new Set(Object.values(chromeWindowIdToWorkspace));
                    workspaces = workspaces.filter(workspace => !activeWindows.has(workspace));
                }

                const res = new Response(JSON.stringify(workspaces), { status: 200 });
                res.headers.set('Access-Control-Allow-Origin', '*');
                res.headers.set('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
                return res
            }

        },
        websocket: {
            async message(ws, message) {
                const parsed = JSON.parse(typeof message === "string" ? message : message.toString()) as BrowserToScriptMessage;

                if (parsed.tabs) {
                    for (const windowId in parsed.tabs) {
                        const workspace = chromeWindowIdToWorkspace[windowId];
                        if (workspace) {
                            currentState.tabs[workspace] = parsed.tabs[windowId];
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

                    await saveTabStateToFs(currentState);
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

