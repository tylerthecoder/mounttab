import { Config } from "./config";
import { startServer } from "./server";
import { TabService } from "./state";

const command = process.argv[2];

function printHelp() {
    console.log("Commands:");
    console.log("  start <workspace>");
    console.log("  serve");
    console.log("  list-workspaces");
    console.log("  read-state");
}

if (command === "start") {
    const workspace = process.argv[3];
    await fetch(`http://localhost:${Config.serverPort}/start?workspace=${workspace}`)
} else if (command == "serve") {
    startServer();
} else if (command == "list-workspaces") {
    const worksapces = await TabService.getAllWorkspaces();
    console.log(worksapces.join("\n"));
} else if (command == "read-state") {
    const state = await TabService.getFromFs();
    console.log(JSON.stringify(state, null, 2));
} else {
    printHelp();
}

