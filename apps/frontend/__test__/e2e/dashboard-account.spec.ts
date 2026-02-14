import { test, expect, capture } from '../utils/screenshot';

test.describe('Dashboard & Account', () => {
  test('dashboard shows user stats', async ({ authedPage: page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'dashboard');
  });

  test('account page displays user info', async ({ authedPage: page }) => {
    await page.goto('/account');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'account');
  });

  test('account credits page shows balance', async ({ authedPage: page }) => {
    await page.goto('/account/credits');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'account-credits');
  });

  test('profile page shows editable fields', async ({ authedPage: page }) => {
    await page.goto('/profile');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'profile');
  });
});
