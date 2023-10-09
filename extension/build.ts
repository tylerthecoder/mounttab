import fs from "node:fs/promises"

if (!(await fs.exists("./pkg"))) {
    await fs.mkdir("./pkg");
} else {
    await fs.rm("./pkg", {recursive: true});
    await fs.mkdir("./pkg");
}


await Bun.write("pkg/manifest.json", Bun.file("manifest.json"));
await Bun.write("pkg/popup.html", Bun.file("src/popup.html"));

const buildOutput = await Bun.build({
    outdir: "pkg", 
    entrypoints: [
        "src/background.ts",
        "src/popup.tsx",
    ],
})

console.log(buildOutput);
