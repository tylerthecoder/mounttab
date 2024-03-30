import { spawnSync } from "bun";
import { type BrowserToScriptMessage } from "./types";
import { TabService } from "./state";
import { STATIC_CONFIG } from "./static-config";

let currentlyStartingWorkspace: string | null = null;
const notConnectedWindowIds = new Set<string>();

let getTabsTimer: Timer | null = null;

const { serverPort } = STATIC_CONFIG;

export const startServer = () => {
    console.log("Starting server on port", serverPort);

    Bun.serve({
        port: serverPort,
        async fetch(req, server) {
            console.log("Incoming request", req.url);
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

                currentlyStartingWorkspace = workspace;

                const tabs = await TabService.getTabsForWorkspace(workspace);

                const command = ["chromium", "--new-window", ...tabs];

                spawnSync(command);
            }

            if (url.pathname === "/state") {
                const state = await TabService.getFromFs();

                const res = new Response(JSON.stringify(state), { status: 200 });
                res.headers.set("Access-Control-Allow-Origin", "*");
                res.headers.set(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, OPTIONS",
                );
                return res;
            }

            if (url.pathname === "/close-workspace") {
                const workspace = url.searchParams.get("workspace");

                console.log("API Closing workspace", workspace);

                if (!workspace) {
                    return new Response("No workspace specified", { status: 400 });
                }

                await TabService.closeWorkspace(workspace);

                const res = new Response("Ok", { status: 200 });
                res.headers.set("Access-Control-Allow-Origin", "*");
                res.headers.set(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, OPTIONS",
                );
                return res;
            }

            if (url.pathname === "/get-workspace-for-windowid") {
                const windowId = url.searchParams.get("windowId");

                if (!windowId) {
                    return new Response("No windowId specified", { status: 400 });
                }

                const workspace = await TabService.getWorkspaceForWindow(windowId);

                const res = new Response(workspace, { status: 200 });
                res.headers.set("Access-Control-Allow-Origin", "*");
                res.headers.set(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, OPTIONS",
                );
                return res;
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

                await TabService.openWorkspaceInWindow(workspace, windowId);

                notConnectedWindowIds.delete(windowId);

                const res = new Response("Ok", { status: 200 });
                res.headers.set("Access-Control-Allow-Origin", "*");
                res.headers.set(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, OPTIONS",
                );
                return res;
            }

            if (url.pathname === "/get-workspaces") {
                const inactive = url.searchParams.get("inactive") === "true";

                const workspaces = inactive
                    ? await TabService.getInactiveWorkspaces()
                    : TabService.getAllWorkspaces();

                const res = new Response(JSON.stringify(workspaces), { status: 200 });
                res.headers.set("Access-Control-Allow-Origin", "*");
                res.headers.set(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, OPTIONS",
                );
                return res;
            }
        },
        websocket: {
            async message(ws, message) {
                const parsed = JSON.parse(
                    typeof message === "string" ? message : message.toString(),
                ) as BrowserToScriptMessage;

                if (parsed.tabs) {
                    const { tabs } = parsed;

                    const state = await TabService.getFromFs();
                    const stateOpenWindows = Object.keys(state.openWindows);

                    const newWindows = Object.keys(tabs)
                        .filter((windowId) => !notConnectedWindowIds.has(windowId))
                        .filter((windowId) => !state.openWindows[windowId]);

                    if (newWindows.length > 1 && currentlyStartingWorkspace) {
                        console.log(
                            "Too many windows to determine which window assigns to ",
                            currentlyStartingWorkspace,
                            "New windows",
                            newWindows,
                        );
                        console.log("Open windows", Object.keys(state.openWindows));
                        currentlyStartingWorkspace = null;
                    }

                    const closedWindows = stateOpenWindows.filter(
                        (windowId) => !tabs[windowId],
                    );
                    for (const windowId of closedWindows) {
                        console.log("Closing workspace for window", windowId);
                        await TabService.closeWorkspace(state.openWindows[windowId]);
                    }

                    for (const windowId in tabs) {
                        const workspace = await TabService.getWorkspaceForWindow(windowId);
                        if (workspace) {
                            await TabService.setTabs(workspace, parsed.tabs[windowId]);
                        } else if (
                            currentlyStartingWorkspace &&
                            !notConnectedWindowIds.has(windowId)
                        ) {
                            console.log(
                                "Setting workspace",
                                windowId,
                                currentlyStartingWorkspace,
                            );
                            await TabService.openWorkspaceInWindow(
                                currentlyStartingWorkspace,
                                windowId,
                            );
                            currentlyStartingWorkspace = null;
                        } else {
                            if (!notConnectedWindowIds.has(windowId)) {
                                console.log("Adding not connected", windowId);
                            }
                            notConnectedWindowIds.add(windowId);
                        }
                    }
                }
            },
            open(ws) {
                console.log("Extension connected");

                if (getTabsTimer) {
                    console.log("Clearing existing timer");
                    clearInterval(getTabsTimer);
                }

                getTabsTimer = setInterval(() => {
                    ws.send(JSON.stringify({ getTabs: true }));
                }, 500);
            },
            close(_ws) {
                console.log("Extension disconnected");

                if (getTabsTimer) {
                    console.log("Clearing timer");
                    clearInterval(getTabsTimer);
                }
            },
        },
    });
};
