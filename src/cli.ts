import { Config } from "./config";
import { startServer } from "./server";
import { getTabStateFromFs } from "./state";


const command = process.argv[2];
if (command === "start") {
    const workspace = process.argv[3];
    await fetch(`http://localhost:${Config.serverPort}/start?workspace=${workspace}`)
} else if (command == "serve") {
    startServer();
} else if (command == "list-workspaces") {
    const currentTabState = await getTabStateFromFs();
    console.log(Object.keys(currentTabState.tabs).join("\n"));
} else {
    console.log("Not a valid command");
}


