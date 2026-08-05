import { defineConfig } from '@playwright/test';
import { resolve } from 'node:path';

const artifactRoot =
  process.env.E2E_ARTIFACT_ROOT ??
  resolve(process.cwd(), 'e2e/migration/artifacts');
const groupId = Number(process.env.E2E_GROUP ?? '0');
const projects =
  groupId === 9
    ? [
        { name: 'migration-chromium', use: { browserName: 'chromium' as const } },
        { name: 'migration-firefox', use: { browserName: 'firefox' as const } },
        { name: 'migration-webkit', use: { browserName: 'webkit' as const } },
      ]
    : [
        {
          name: 'migration-chromium',
          use: { browserName: 'chromium' as const },
        },
      ];

export default defineConfig({
  testDir: './specs',
  outputDir: `${artifactRoot}/playwright`,
  fullyParallel: false,
  forbidOnly: true,
  retries: 0,
  workers: 1,
  timeout: 300_000,
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
  projects,
});
