import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  testMatch: 'capture-screenshots.ts',
  fullyParallel: true,
  workers: 2,
  timeout: 60000,
  use: {
    screenshot: 'off',
    video: 'off',
    trace: 'off',
  },
});
