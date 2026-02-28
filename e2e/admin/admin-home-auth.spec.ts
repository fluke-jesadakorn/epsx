import { test, expect, capture } from './utils/screenshot';
import { mockAllApis } from './utils/api-interceptor';

test.describe('Admin Home & Auth', () => {
  test('admin dashboard shows stats', async ({ authedPage: page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main, [role="main"], #main')).toBeVisible();
    await capture(page, 'admin-dashboard');
  });

  test('auth page shows wallet connect', async ({ page }) => {
    await mockAllApis(page);
    await page.goto('/auth');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-auth');
  });

  test('access-denied page', async ({ page }) => {
    await mockAllApis(page);
    await page.goto('/access-denied');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-access-denied');
  });

  test('unauthorized page', async ({ page }) => {
    await mockAllApis(page);
    await page.goto('/unauthorized');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-unauthorized');
  });

});
