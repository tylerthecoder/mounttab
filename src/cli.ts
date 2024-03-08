import { getConfig } from "./config";
import { startServer } from "./server";
import { TabService } from "./state";
import { STATIC_CONFIG } from "./static-config";

const command = process.argv[2];


function printHelp() {
    console.log("Commands:");
    console.log("  start <workspace>");
    console.log("  serve");
    console.log("  list-workspaces");
    console.log("  read-state");
    console.log("  read-config");
}

if (command === "start") {
    const { serverPort } = STATIC_CONFIG;
    const workspace = process.argv[3];
    await fetch(`http://localhost:${serverPort}/start?workspace=${workspace}`)
} else if (command == "serve") {
    startServer();
} else if (command == "list-workspaces") {
    const worksapces = await TabService.getAllWorkspaces();
    console.log(worksapces.join("\n"));
} else if (command == "read-state") {
    const state = await TabService.getFromFs();
    console.log(JSON.stringify(state, null, 2));
} else if (command == "read-config") {
    const config = await getConfig();
    console.log(JSON.stringify(config, null, 2));
} else {
    printHelp();
}

