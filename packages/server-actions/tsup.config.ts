import { createTsupConfig } from '@epsx/config/tsup';

export default createTsupConfig({
  external: ['next', 'react', 'next/cache', 'next/headers'],
  override: {
    target: 'node18',
    treeshake: false,
  },
});