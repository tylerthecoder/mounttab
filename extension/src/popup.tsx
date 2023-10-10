import React from "react";
import { createRoot } from "react-dom/client";

type Workspace = string;

const Popup = () => {
    const [data, setData] = React.useState<Workspace[]>();

    React.useEffect(() => {
        chrome.runtime.sendMessage({ type: "GetAllWorkspaces" }, (response: Workspace[]) => {
            console.log("Got Response", response);
            setData(response)
        });
    }, []);


    const selectWorkspace = (workspace: Workspace) => {
        chrome.runtime.sendMessage({ type: "SelectWorkspace", workspace }, (response: Workspace[]) => {

        });
    }

    return (
        <div className="App">
            <div>
                {data && data.map((workspace) => {
                    return (
                        <div>
                            <p> {workspace} </p>
                            <button onClick={() => selectWorkspace(workspace)}> Launch </button>
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

