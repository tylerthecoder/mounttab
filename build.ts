import fs from "node:fs/promises"
import { watch } from "fs/promises";
import path from "path"
import { $ } from "bun"

const dev = process.argv.includes("--dev");

const buildExtention = async () => {
    if (!(await fs.exists("./pkg"))) {
        await fs.mkdir("./pkg");
    } else {
        await fs.rm("./pkg", { recursive: true });
        await fs.mkdir("./pkg");
    }

    await Bun.write("pkg/manifest.json", Bun.file("./src/manifest.json"));
    const buildOutput = await Bun.build({
        outdir: "pkg",
        entrypoints: [
            "src/background.ts",
        ],
    })
    console.log("Extension built", buildOutput);
}

const buildCli = async () => {
    if (!(await fs.exists("./bin"))) {
        await fs.mkdir("./bin");
    } else {
        await fs.rm("./bin", { recursive: true });
        await fs.mkdir("./bin");
    }

    await $`bun build ./src/cli.ts --compile --outfile ./bin/mt`
}

const installCli = async () => {
    const bin_dir = `${process.env.HOME}/.local/bin`;
    await $`chmod +x ./bin/mt`
    await $`rm -f ${bin_dir}/mt`
    await $`cp ./bin/mt ${bin_dir}/mt`
}

await buildExtention();
await buildCli();
await installCli();

if (dev) {
    const src_dir = path.join(import.meta.dir, "src");
    console.log("Watching for changes \n", src_dir, "\n")
    const watcher = watch(src_dir, { recursive: true })
    for await (const event of watcher) {
        console.log("File changed, rebuilding...");
        await buildExtention();
        await buildCli();
        await installCli();
    }
}
