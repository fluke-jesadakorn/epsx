import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './.debug',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { outputFolder: './.debug/playwright-report' }],
    ['list'],
    ['json', { outputFile: './.debug/test-results.json' }]
  ],
  use: {
    baseURL: 'https://epsx.io',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});