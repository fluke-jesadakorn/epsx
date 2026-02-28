import { test, expect, capture } from './utils/screenshot';

test.describe('Admin Developer Portal', () => {
  test('developer portal overview', async ({ authedPage: page }) => {
    await page.goto('/developer-portal');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main, [role="main"]')).toBeVisible();
    await capture(page, 'admin-developer-portal');
  });

  test('create API key form', async ({ authedPage: page }) => {
    await page.goto('/developer-portal/api-keys/create');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-api-key-create');
  });

});
