import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    server: 'src/server.ts',
    client: 'src/client.ts',
  },
  format: ['cjs', 'esm'],
  dts: true,
  clean: true,
  external: ['react', '@epsx/server-actions'],
  target: 'es2020',
  splitting: false,
  sourcemap: true,
  treeshake: true,
  esbuildOptions(options) {
    options.keepNames = true;
    options.ignoreAnnotations = false;
  },
  // Add client directive only to client files
  async onSuccess() {
    const fs = await import('fs');
    const path = await import('path');
    
    // Add 'use client' to client build files
    const clientFiles = ['dist/client.js', 'dist/client.mjs'];
    for (const file of clientFiles) {
      if (fs.existsSync(file)) {
        const content = fs.readFileSync(file, 'utf8');
        if (!content.startsWith('"use client"')) {
          fs.writeFileSync(file, `"use client";\n${content}`);
        }
      }
    }
  },
});