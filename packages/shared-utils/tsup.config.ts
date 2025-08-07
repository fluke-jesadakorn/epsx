import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    formatting: 'src/formatting/index.ts',
    dom: 'src/dom/index.ts',
    validation: 'src/validation/index.ts',
  },
  format: ['esm', 'cjs'],
  dts: true,
  sourcemap: true,
  clean: true,
  splitting: false,
  treeshake: true,
  minify: false,
  external: ['react'],
  target: 'es2022',
  platform: 'neutral',
});