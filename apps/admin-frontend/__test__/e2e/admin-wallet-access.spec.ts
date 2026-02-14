import { test, expect, capture } from '../utils/screenshot';

test.describe('Admin Wallet Access Control', () => {
  test('access overview', async ({ authedPage: page }) => {
    await page.goto('/wallet-management/access');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main, [role="main"]')).toBeVisible();
    await capture(page, 'admin-access-overview');
  });

  test('access permissions page', async ({ authedPage: page }) => {
    await page.goto('/wallet-management/access/permissions');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-access-permissions');
  });

  test('access plans page', async ({ authedPage: page }) => {
    await page.goto('/wallet-management/access/plans');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-access-plans');
  });

  test('access plan detail', async ({ authedPage: page }) => {
    await page.goto('/wallet-management/access/plans/plan-1');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-access-plan-detail');
  });
});
