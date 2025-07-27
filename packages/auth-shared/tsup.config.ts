import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    server: 'src/server/index.ts',
    client: 'src/client/index.ts',
    middleware: 'src/middleware/index.ts',
  },
  format: ['esm', 'cjs'],
  dts: true,
  clean: true,
  external: ['react', 'next'],
  splitting: false,
  sourcemap: true,
  treeshake: true,
});