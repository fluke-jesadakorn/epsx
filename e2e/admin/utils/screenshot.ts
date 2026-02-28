/**
 * Screenshot utility + extended test fixture for Admin Frontend E2E Tests.
 * Saves screenshots to public/screenshots/ for E2E visual regression.
 */
import { test as base, type Page } from '@playwright/test';
import { mockAuth } from './auth-mock';
import { mockAllApis } from './api-interceptor';
import path from 'node:path';
import fs from 'node:fs';

const SCREENSHOT_DIR = path.resolve(__dirname, '../../public/screenshots');

// Ensure directory exists
if (!fs.existsSync(SCREENSHOT_DIR)) {
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
}

/**
 * Capture a named screenshot and save to public/screenshots/{name}.png
 */
export async function capture(page: Page, name: string, opts?: { fullPage?: boolean }) {
  const filePath = path.join(SCREENSHOT_DIR, `${name}.png`);
  await page.screenshot({ path: filePath, fullPage: opts?.fullPage ?? false });
  return filePath;
}

/**
 * Extended test fixture with pre-authenticated page + API mocks.
 */
export const test = base.extend<{ authedPage: Page }>({
  authedPage: async ({ page }, use) => {
    await mockAuth(page);
    await mockAllApis(page);
    await use(page);
  },
});

export { expect } from '@playwright/test';
