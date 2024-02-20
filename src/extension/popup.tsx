import { createRoot } from "react-dom/client";
import { Config } from "../config";

const getAvailableWorkspaces = async (): Promise<string[]> => {
    const res = await fetch(`http://localhost:${Config.serverPort}/get-workspaces?inactive=true`)
    const data = await res.json();
    console.log(data);
    return data as string[];
}

const Popup = (props: { workspaces: string[] }) => {
    return (
        <div className="App">
            <div>
                <p> Hello </p>
                <h2> Available Workspaces </h2>
                <ul>
                    {props.workspaces.map((workspace) => (
                        <li key={workspace}>{workspace}</li>
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

const data = await getAvailableWorkspaces();
root.render(<Popup workspaces={data} />);


export default Popup;
