import { defineConfig } from '@playwright/test';
import { resolve } from 'node:path';

const artifactRoot =
  process.env.E2E_ARTIFACT_ROOT ??
  resolve(process.cwd(), 'e2e/migration/artifacts');

export default defineConfig({
  testDir: './specs',
  outputDir: `${artifactRoot}/playwright`,
  fullyParallel: false,
  forbidOnly: true,
  retries: 0,
  workers: 1,
  timeout: 180_000,
  expect: {
    timeout: 10_000,
  },
  reporter: [
    ['list'],
    [
      'json',
      {
        outputFile: `${artifactRoot}/playwright-results.json`,
      },
    ],
    [
      'html',
      {
        outputFolder: `${artifactRoot}/playwright-report`,
        open: 'never',
      },
    ],
  ],
  use: {
    browserName: 'chromium',
    headless: true,
    actionTimeout: 15_000,
    navigationTimeout: 60_000,
  },
  projects: [
    {
      name: 'migration-chromium',
      use: {
        browserName: 'chromium',
      },
    },
  ],
});
