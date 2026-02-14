import { test, expect, capture } from '../utils/screenshot';

test.describe('Admin Audit, Payments & Affiliates', () => {
  test('audit log page with table', async ({ authedPage: page }) => {
    await page.goto('/audit-log');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main, [role="main"]')).toBeVisible();
    await capture(page, 'admin-audit-log');
  });

  test('payments page', async ({ authedPage: page }) => {
    await page.goto('/payments');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-payments');
  });

  test('affiliates page', async ({ authedPage: page }) => {
    await page.goto('/affiliates');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-affiliates');
  });
});
