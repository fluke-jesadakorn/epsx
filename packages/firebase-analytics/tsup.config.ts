import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts', 'src/server.ts', 'src/client.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  clean: true,
  splitting: false,
  sourcemap: true,
  external: ['react', 'react-dom', 'next', 'next/navigation', 'firebase/app', 'firebase/analytics'],
});