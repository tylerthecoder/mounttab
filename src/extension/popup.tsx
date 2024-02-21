import { createRoot } from "react-dom/client";
import { Config } from "../config";

const currentWindow = await chrome.windows.getCurrent();
console.log(currentWindow);

const myWorkspaceRes = await fetch(`http://localhost:${Config.serverPort}/get-workspace-for-windowid?windowId=${currentWindow.id}`);
const myWorkspace = await myWorkspaceRes.text();
console.log(myWorkspace);

const workspaceRes = await fetch(`http://localhost:${Config.serverPort}/get-workspaces?inactive=true`)
const workspaces = await workspaceRes.json() as string[];
console.log(workspaces)


const assignWindowToWorkspace = async (workspace: string) => {
    if (currentWindow.id === undefined) {
        alert("Can not find current window id");
        throw new Error("Can not find current window id");
    }

    const params = new URLSearchParams(window.location.search);
    params.append("workspace", workspace);
    params.append("windowId", currentWindow.id.toString());

    await fetch(`http://localhost:${Config.serverPort}/assign-window-to-workspace?${params.toString()}`);

    // This is the ideal way to refresh state
    location.reload();
}

const Popup = () => {
    return (
        <div className="App" style={{ width: "300px" }}>
            <div>
                <p> My Workspace: {myWorkspace} </p>
                <b> Available Workspaces </b>
                <ul>
                    {workspaces.map((workspace) => (
                        <li key={workspace}>
                            <p> {workspace} </p>
                            <button onClick={() => assignWindowToWorkspace(workspace)}> Assign current tabs to workspace </button>
                        </li>
                    ))}
                </ul>
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
