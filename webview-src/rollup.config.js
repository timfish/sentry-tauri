import resolve from "@rollup/plugin-node-resolve";
import { terser } from "rollup-plugin-terser";
import typescript from "@rollup/plugin-typescript";

export default [
  {
    input: "./webview-src/inject.ts",
    output: {
      dir: "./webview-dist",
      entryFileNames: "[name].min.js",
      format: "iife",
    },
    plugins: [
      resolve(),
      terser({ format: { comments: false } }),
      typescript({
        tsconfig: "./webview-src/tsconfig.json",
        outDir: "./webview-dist",
      }),
    ],
  },
];
