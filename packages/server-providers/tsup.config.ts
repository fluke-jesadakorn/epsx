import { defineConfig } from 'tsup';

export default defineConfig([
  // Server and base builds
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
  // Client build with "use client" directive
  {
    entry: {
      client: 'src/client.ts',
    },
    format: ['cjs', 'esm'],
    dts: true,
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