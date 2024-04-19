import { createRoot } from "react-dom/client";
import type { TabState } from "../state";
import { STATIC_CONFIG } from "../static-config";

const { serverPort } = STATIC_CONFIG;

chrome.runtime.sendMessage({
    message: "ping"
})

const currentWindow = await chrome.windows.getCurrent();
console.log(currentWindow);

const stateRes = await fetch(`http://localhost:${serverPort}/state`);
const state = await stateRes.json() as TabState;
const workspaces = Object.keys(state.workspaces);
const openWindows = state.openWindows;

const myWorkspace = currentWindow.id ? openWindows[currentWindow.id] : undefined;

const assignWindowToWorkspace = async (workspace: string) => {
    if (currentWindow.id === undefined) {
        alert("Can not find current window id");
        throw new Error("Can not find current window id");
    }

    const params = new URLSearchParams(window.location.search);
    params.append("workspace", workspace);
    params.append("windowId", currentWindow.id.toString());

    await fetch(`http://localhost:${serverPort}/assign-window-to-workspace?${params.toString()}`);

    // This is the ideal way to refresh state
    location.reload();
}

const closeWorkspace = async (workspace: string) => {
    await fetch(`http://localhost:${serverPort}/close-workspace?workspace=${workspace}`);
    // This is the ideal way to refresh state
    location.reload();
}

const Popup = () => {
    return (
        <div className="App" style={{ width: "300px" }}>
            <div>
                <p> My Workspace: {myWorkspace} </p>
                <b> Available Workspaces </b>
                <div>
                    {workspaces.map((workspace) => {
                        const windowId = Object.keys(openWindows).find(windowId => openWindows[windowId] === workspace);


                        return <div key={workspace}>
                            <p> {workspace}: {windowId && <> Assigned to window: {windowId} </>} </p>

                            <button onClick={() => assignWindowToWorkspace(workspace)}> Assign current tabs to workspace </button>
                            {windowId && <button onClick={() => closeWorkspace(workspace)}> Close workspace </button>}
                        </div>
                    })}
                </div>

                <div>
                    <b> Make new workspace </b>
                    <input type="text" id="new-workspace" />
                    <button onClick={() => {
                        const workspace = (document.getElementById("new-workspace") as HTMLInputElement).value;
                        if (workspace) {
                            assignWindowToWorkspace(workspace);
                        }
                    }}> Make new workspace </button>
                </div>


            </div>
        </div>
    );
};


const appContainer = document.querySelector("#app-container");
if (!appContainer) {
    throw new Error("Can not find #app-container");
}
const root = createRoot(appContainer);

root.render(<Popup />);
