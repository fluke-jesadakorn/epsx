import { test, expect, capture } from '../utils/screenshot';

test.describe('Portfolio & Permissions', () => {
  test('portfolio page shows holdings', async ({ authedPage: page }) => {
    await page.goto('/portfolio');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'portfolio');
  });

  test('permissions page shows access list', async ({ authedPage: page }) => {
    await page.goto('/permissions');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'permissions');
  });
});
