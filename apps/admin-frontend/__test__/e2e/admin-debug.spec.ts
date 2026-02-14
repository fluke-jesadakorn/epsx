import { test, expect, capture } from '../utils/screenshot';

test.describe('Admin Debug', () => {
  test('debug auth page shows session info', async ({ authedPage: page }) => {
    await page.goto('/debug/auth');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main, [role="main"]')).toBeVisible();
    await capture(page, 'admin-debug-auth');
  });
});
