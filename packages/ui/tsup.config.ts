import { defineConfig } from "tsup";

export default defineConfig({
  entry: ["src/index.ts"],
  format: ["cjs", "esm"],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  external: [
    "react",
    "react-dom",
    "react-hook-form",
    "@radix-ui/react-label",
    "@radix-ui/react-slot",
  ],
  esbuildOptions(options) {
    options.banner = {
      js: '"use client";',
    };
  },
  treeshake: true,
});
