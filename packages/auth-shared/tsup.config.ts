import { tsupPresets, createTsupConfig } from '@epsx/config/tsup';
import { defineConfig } from 'tsup';

export default defineConfig([
  // Server and base builds
  createTsupConfig({
    entry: {
      index: 'src/index.ts',
      server: 'src/server/index.ts',
      middleware: 'src/middleware.ts',
    },
    external: ['react', 'next'],
  }),
  // Client build with "use client" directive
  createTsupConfig({
    entry: {
      client: 'src/client/index.ts',
    },
    external: ['react', 'next'],
    useClient: true,
    override: {
      treeshake: false,
      bundle: true,
      esbuildOptions(options: any) {
        options.keepNames = true;
        options.ignoreAnnotations = false;
        options.banner = { js: '"use client";' };
      },
    },
  })
]);