import { test, expect, capture } from './utils/screenshot';

test.describe('Developer', () => {
  test('developer overview page', async ({ authedPage: page }) => {
    await page.goto('/developer');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'developer');
  });

  test('developer docs page shows API documentation', async ({ authedPage: page }) => {
    await page.goto('/developer/docs');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'developer-docs');
  });

  test('developer usage page shows stats', async ({ authedPage: page }) => {
    await page.goto('/developer/usage');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'developer-usage');
  });
});
