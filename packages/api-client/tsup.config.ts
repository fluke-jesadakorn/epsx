import { createTsupConfig } from '@epsx/config/tsup';

export default createTsupConfig({
  external: ['next', 'react'],
  tsconfig: './tsconfig.build.json',
});