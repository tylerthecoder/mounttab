import fs from "node:fs/promises"
import { watch } from "fs/promises";
import path from "path"
import { $ } from "bun"

const dev = process.argv.includes("--dev");

const outDir = "./out"
const extensionDir = `${outDir}/pkg`


console.log("Options", { dev, outDir, extensionDir })

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
    const extensionSource = "./src/extension"

    await Bun.write(`${extensionDir}/manifest.json`, Bun.file(`${extensionSource}/manifest.json`));
    await Bun.write(`${extensionDir}/popup.html`, Bun.file(`${extensionSource}/popup.html`));
    const buildOutput = await Bun.build({
        outdir: extensionDir,
        entrypoints: [
            "src/extension/background.ts",
            "src/extension/popup.tsx",
        ],
    })

    console.log("Extension built", buildOutput);
}

const buildCli = async () => {
    await $`bun build ./src/cli.ts --compile --outfile ./${outDir}/mt`
}

const build = async () => {
    await assertDir(outDir);
    await assertDir(extensionDir);
    await buildExtention();
    await buildCli();
}

const installCli = async () => {
    const bin_dir = `${process.env.HOME}/.local/bin`;
    await $`chmod +x ./${outDir}/mt`
    await $`rm -f ${bin_dir}/mt`
    await $`cp ./${outDir}/mt ${bin_dir}/mt`
}

await build();
await installCli();

if (dev) {
    const src_dir = path.join(import.meta.dir, "src");
    console.log("Watching for changes \n", src_dir, "\n")
    const watcher = watch(src_dir, { recursive: true })
    for await (const event of watcher) {
        console.log("File changed, rebuilding...");
        await build();
        // await installCli();
    }
}



