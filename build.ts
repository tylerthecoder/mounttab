import fs from "node:fs/promises"
import { watch } from "fs/promises";
import path from "path"

const watch_flag = true;

const buildExtention = async () => {
    if (!(await fs.exists("./pkg"))) {
        await fs.mkdir("./pkg");
    } else {
        await fs.rm("./pkg", { recursive: true });
        await fs.mkdir("./pkg");
    }

    await Bun.write("pkg/manifest.json", Bun.file("manifest.json"));
    const buildOutput = await Bun.build({
        outdir: "pkg",
        entrypoints: [
            "src/background.ts",
        ],
    })
    console.log("Extension built", buildOutput);
}

await buildExtention();

if (watch_flag) {
    const src_dir = path.join(import.meta.dir, "src");
    console.log("Watching for changes \n", src_dir, "\n")
    const watcher = watch(src_dir, { recursive: true })
    for await (const event of watcher) {
        console.log("File changed, rebuilding...");
        await buildExtention();
    }
}
