import fs from "node:fs/promises"
import { watch } from "fs/promises";
import { $ } from "bun"

const dev = process.argv.includes("--dev");

const outDir = "./out"
const extensionOutDir = `${outDir}/pkg`
const extensionSourceDir = "./src/extension"

console.log("Options", { dev, outDir, extensionOutDir, extensionSourceDir })

const assertDir = async (dir: string) => {
    const dirExists = await fs.exists(dir);

    if (!dirExists) {
        await fs.mkdir(dir, { recursive: true })
    } else {
        await fs.rm(dir, { recursive: true });
        await fs.mkdir(dir);
    }
}

const buildExtention = async () => {
    await assertDir(extensionOutDir);

    await Bun.write(`${extensionOutDir}/manifest.json`, Bun.file(`${extensionSourceDir}/manifest.json`));
    await Bun.write(`${extensionOutDir}/popup.html`, Bun.file(`${extensionSourceDir}/popup.html`));
    await Bun.build({
        outdir: extensionOutDir,
        entrypoints: [
            "src/extension/background.ts",
            "src/extension/popup.tsx",
        ],
    })

    console.log("Extension built");
}

const buildCli = async () => {
    await assertDir(outDir);
    await $`bun build ./src/cli.ts --compile --outfile ./${outDir}/mt`
    console.log("Cli built");
}

const installCli = async () => {
    const bin_dir = `${process.env.HOME}/.local/bin`;
    await $`chmod +x ./${outDir}/mt`
    await $`rm -f ${bin_dir}/mt`
    await $`cp ./${outDir}/mt ${bin_dir}/mt`
}

const installSystemService = async () => {
    const serviceFile = Bun.file("./build/MountTab.service")
    const res = await Bun.write(`${process.env.HOME}/.config/systemd/user/MountTab.service`, serviceFile);

    console.log("Installed service file", res);

    await $`systemctl --user daemon-reload`
    await $`systemctl --user enable MountTab`
    await $`systemctl --user start MountTab`

    console.log("Started service");
}

const updateSystemdService = async () => {
    const serviceFile = Bun.file("./build/MountTab.service")
    const res = await Bun.write(`${process.env.HOME}/.config/systemd/user/MountTab.service`, serviceFile);
    console.log("Installed service file", res);
    await $`systemctl --user daemon-reload`
    const restartRes = await $`systemctl --user restart MountTab`
    console.log("Restarted service", restartRes);
}

await buildCli();
await buildExtention();
await installCli();
await installSystemService();

if (dev) {
    console.log("Watching for changes in ./src")
    const watcher = watch("./src", { recursive: true })
    for await (const _ of watcher) {
        console.log("File changed, rebuilding...");
        await buildCli();
        await buildExtention();
        await updateSystemdService();
    }
}



