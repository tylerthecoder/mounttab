import { Config } from "./config";
import { startServer } from "./server";
import { TabService } from "./state";

const command = process.argv[2];

if (command === "start") {
    const workspace = process.argv[3];
    await fetch(`http://localhost:${Config.serverPort}/start?workspace=${workspace}`)
} else if (command == "serve") {
    startServer();
} else if (command == "list-workspaces") {
    const worksapces = await TabService.getAllWorkspaces();
    console.log(worksapces.join("\n"));
} else {
    console.log("Command not valid");
}


