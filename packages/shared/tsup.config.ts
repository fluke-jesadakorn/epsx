import { defineConfig } from 'tsup';

export default defineConfig({
  entry: [
    'src/index.ts',
    'src/decorators/index.ts',
    'src/types/index.ts',
    'src/utils/index.ts'
  ],
  format: ['esm', 'cjs'],
  dts: false,
  splitting: true,
  sourcemap: true,
  clean: true,
  minify: process.env.NODE_ENV === 'production',
  watch: process.env.NODE_ENV === 'development',
  external: [
    '@nestjs/common',
    '@nestjs/core',
    '@nestjs/microservices',
    '@nestjs/mongoose',
    'mongoose',
    'class-transformer',
    'class-validator',
    'firebase',
    'zod',
    'axios'
  ],
  noExternal: ['@epsx/*'],
  esbuildOptions(options) {
    options.target = 'es2020'
  }
})
