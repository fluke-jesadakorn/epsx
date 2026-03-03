import { defineConfig, devices } from '@playwright/test'

const ENV = (process.env.TEST_ENV ?? 'dev') as 'dev' | 'devtunnel' | 'staging' | 'prod'

const URLS = {
  dev:       { frontend: 'http://localhost:3000', admin: 'http://localhost:3001' },
  devtunnel: { frontend: 'https://dev.epsx.io',  admin: 'https://dev-admin.epsx.io' },
  staging:   { frontend: 'https://staging.epsx.io', admin: 'https://staging-admin.epsx.io' },
  prod:      { frontend: 'https://epsx.io',       admin: 'https://admin.epsx.io' },
}

export default defineConfig({
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { outputFolder: './playwright-report' }],
    ['list'],
    ['json', { outputFile: './test-results/results.json' }],
  ],
  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  projects: [
    {
      name: 'frontend',
      testDir: './e2e/frontend',
      use: { ...devices['Desktop Chrome'], baseURL: URLS[ENV].frontend },
    },
    {
      name: 'admin',
      testDir: './e2e/admin',
      use: { ...devices['Desktop Chrome'], baseURL: URLS[ENV].admin },
    },
  ],
})
