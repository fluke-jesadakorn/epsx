import { test, expect, capture } from '../utils/screenshot';

test.describe('Admin Notifications', () => {
  test('notifications list', async ({ authedPage: page }) => {
    await page.goto('/notifications');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main, [role="main"]')).toBeVisible();
    await capture(page, 'admin-notifications');
  });

  test('create notification', async ({ authedPage: page }) => {
    await page.goto('/notifications/create');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-notification-create');
  });

  test('manage notifications', async ({ authedPage: page }) => {
    await page.goto('/notifications/manage');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-notification-manage');
  });
});
