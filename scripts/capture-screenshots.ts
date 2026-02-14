/**
 * Deep interactive screenshot capture for both frontends.
 * Run: npx playwright test --config=scripts/playwright-screenshots.config.ts
 */
import { test } from '@playwright/test';
import path from 'path';
import { setupAuth, capture, MOCK_USER, MOCK_ADMIN, USER_PERMS, ADMIN_PERMS, waitForAuth } from './screenshots/helpers';
import { setupMocks, type MockOverrides } from './screenshots/mocks';
import { frontendObjectives } from './screenshots/interactions/frontend';
import { adminObjectives } from './screenshots/interactions/admin';
import type { Objective } from './screenshots/types';

const FRONTEND_URL = 'http://localhost:3000';
const ADMIN_URL = 'http://localhost:3001';
const FRONTEND_DIR = path.resolve(__dirname, '../apps/frontend');
const ADMIN_DIR = path.resolve(__dirname, '../apps/admin-frontend');

function runObjectives(
  objectives: Objective[],
  baseUrl: string,
  outDir: string,
  user: typeof MOCK_USER,
  perms: string[],
  group: string,
) {
  test.describe(group, () => {
    for (const obj of objectives) {
      test(`${obj.id}`, async ({ page }) => {
        // Setup mocks
        const overrides: MockOverrides = {};
        if (obj.mockOverrides) {
          if (obj.mockOverrides.emptyNotifications) overrides.emptyNotifications = true;
        }
        await setupMocks(page, overrides);

        // Setup auth if needed
        if (obj.auth) {
          await setupAuth(page, user, perms);
        }

        // Navigate
        await page.goto(`${baseUrl}${obj.route}`, { waitUntil: 'load', timeout: 15000 });

        // Wait for auth to resolve on authenticated pages
        if (obj.auth) {
          await waitForAuth(page);
        } else {
          await page.waitForTimeout(1000);
        }

        // Execute each step and capture
        for (const step of obj.steps) {
          await step.action(page);
          await capture(page, outDir, step.name);
        }
      });
    }
  });
}

// Frontend
runObjectives(frontendObjectives, FRONTEND_URL, FRONTEND_DIR, MOCK_USER, USER_PERMS, 'Frontend');

// Admin
runObjectives(adminObjectives, ADMIN_URL, ADMIN_DIR, MOCK_ADMIN, ADMIN_PERMS, 'Admin');
