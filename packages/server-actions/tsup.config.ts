import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm', 'cjs'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  external: ['next', 'react', 'next/cache', 'next/headers'],
  target: 'node18',
  minify: false,
  treeshake: false,
});