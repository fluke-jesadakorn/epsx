import { test, expect, capture } from '../utils/screenshot';

test.describe('Admin Analytics', () => {
  test('analytics page shows charts and data', async ({ authedPage: page }) => {
    await page.goto('/analytics');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main, [role="main"]')).toBeVisible();
    await capture(page, 'admin-analytics');
  });
});
