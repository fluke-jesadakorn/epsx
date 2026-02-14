import { test, expect, capture } from '../utils/screenshot';

test.describe('Admin Profile & Settings', () => {
  test('profile page', async ({ authedPage: page }) => {
    await page.goto('/profile');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main, [role="main"]')).toBeVisible();
    await capture(page, 'admin-profile');
  });

  test('settings page', async ({ authedPage: page }) => {
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-settings');
  });
});
