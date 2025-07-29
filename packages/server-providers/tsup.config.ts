import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    server: 'src/server.ts',
    client: 'src/client.ts'
  },
  format: ['cjs', 'esm'],
  dts: true,
  clean: true,
  external: ['react', '@epsx/server-actions'],
  target: 'es2020',
});