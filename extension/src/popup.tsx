import React from "react";
import { createRoot } from "react-dom/client";
import { Workspace } from "./model";


const Popup = () => {
    const [data, setData] = React.useState<Workspace[]>();

    React.useEffect(() => {
        chrome.tabs.getCurrent((tab) => {
            console.log("Current Tab", tab);
        });


        chrome.runtime.sendMessage({ type: "GetAllWorkspaces" }, (response: Workspace[]) => {
            console.log("Got Response", response);
            setData(response)
        });
    }, []);


    const selectWorkspace = (workspace_id: string) => {
        chrome.runtime.sendMessage({ type: "SelectWorkspace", workspace: workspace_id }, (response: Workspace[]) => {

        });
    }

    return (
        <div className="App">
            <div>
                {data && data.map((workspace) => {
                    return (
                        <div>
                            <p> {workspace.name} </p>
                            <button onClick={() => selectWorkspace(workspace.id)}> Launch </button>
                        </div>
                    );
                })}
            </div>
        </div>
    );
};


function init() {
    const appContainer = document.querySelector("#app-container");
    if (!appContainer) {
        throw new Error("Can not find #app-container");
    }
    const root = createRoot(appContainer);
    root.render(<Popup />);
}

init();

export default Popup;

