import { defineConfig } from 'tsup';

export default defineConfig([
  // Main builds (index, server) - clean first
  {
    entry: {
      index: 'src/index.ts',
      server: 'src/server.ts',
    },
    format: ['cjs', 'esm'],
    dts: true,
    clean: true,
    external: ['react', '@epsx/server-actions'],
    target: 'es2020',
    splitting: false,
    sourcemap: true,
    treeshake: true,
  },
  // Client build - no clean to avoid race condition
  {
    entry: {
      client: 'src/client.ts',
    },
    format: ['cjs', 'esm'],
    dts: true,
    clean: false,
    external: ['react', '@epsx/server-actions'],
    target: 'es2020',
    splitting: false,
    sourcemap: true,
    treeshake: false,
    bundle: true,
    esbuildOptions(options) {
      options.keepNames = true;
      options.ignoreAnnotations = false;
    },
    banner: {
      js: '"use client";',
    },
  }
]);