import resolve from "@rollup/plugin-node-resolve";
import terser from "@rollup/plugin-terser";

export default [
  {
    input: "./dist/inject.js",
    output: {
      dir: "./dist",
      entryFileNames: "[name].min.js",
      format: "iife",
    },
    plugins: [resolve(), terser({ format: { comments: false } })],
  },
];
