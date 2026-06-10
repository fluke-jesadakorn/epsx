import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:3000',
    screenshot: 'off',
    trace: 'off',
    headless: true,
    viewport: { width: 1280, height: 800 },
  },
  outputDir: './test-results',
  timeout: 30000,
});
