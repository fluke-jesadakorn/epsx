import { test, expect, capture } from '../utils/screenshot';

test.describe('Notifications', () => {
  test('notifications page shows list', async ({ authedPage: page }) => {
    await page.goto('/notifications');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'notifications');
  });

  test('notifications empty state', async ({ page, authedPage }) => {
    // Override notifications to return empty
    await authedPage.route('**/api/notifications', route =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ data: { items: [], pagination: { page: 1, limit: 20, total: 0, totalPages: 0 } }, success: true }),
      })
    );
    await authedPage.goto('/notifications');
    await authedPage.waitForLoadState('networkidle');
    await capture(authedPage, 'notifications-empty');
  });
});
