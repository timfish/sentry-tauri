import { defineConfig } from "tsup";

export default defineConfig({
    entry: ["webview-src/index.ts"],
    format: ["esm"],
    outDir: "webview-dist",
    clean: true,
    dts: true
});