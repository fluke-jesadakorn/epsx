import { defineConfig } from 'tsup'

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    session: 'src/session.ts',
    context: 'src/context.ts',
    hooks: 'src/hooks.ts',
    actions: 'src/actions.ts',
    service: 'src/service.ts',
  },
  format: ['cjs', 'esm'],
  dts: true,
  sourcemap: true,
  clean: true,
  splitting: false,
  external: ['react', 'react-dom', 'next', 'firebase', 'firebase-admin'],
})
