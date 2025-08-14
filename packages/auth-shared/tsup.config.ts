import { tsupPresets, createTsupConfig } from '@epsx/config/tsup';
import { defineConfig } from 'tsup';

export default defineConfig([
  // Server and base builds only
  createTsupConfig({
    entry: {
      index: 'src/index.ts',
      server: 'src/server/index.ts',
    },
    external: ['react', 'next'],
  })
]);