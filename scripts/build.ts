import { mkdirSync, copyFileSync } from "node:fs";

export async function buildWebAssets(): Promise<string> {
  mkdirSync("dist", { recursive: true });
  const result = await Bun.build({
    entrypoints: ["src/web/main.tsx"],
    outdir: "dist",
    target: "browser",
    minify: process.env.NODE_ENV === "production",
  });

  if (!result.success) {
    const logs = result.logs.map((l) => l.message).join("\n");
    throw new Error(`Frontend build failed:\n${logs}`);
  }

  copyFileSync("src/web/styles.css", "dist/styles.css");
  return "dist/main.js";
}

if (import.meta.main) {
  await buildWebAssets();
  console.log("✓ Built web frontend assets to dist/");
}
