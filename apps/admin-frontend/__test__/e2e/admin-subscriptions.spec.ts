import { test, expect, capture } from '../utils/screenshot';

test.describe('Admin Subscriptions', () => {
  test('subscription detail page', async ({ authedPage: page }) => {
    await page.goto('/subscriptions/sub-001');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main, [role="main"]')).toBeVisible();
    await capture(page, 'admin-subscription-detail');
  });

  test('create new subscription', async ({ authedPage: page }) => {
    await page.goto('/subscriptions/new');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-subscription-new');
  });

  test('create new plan', async ({ authedPage: page }) => {
    await page.goto('/subscriptions/plans/new');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-plan-new');
  });

  test('edit plan', async ({ authedPage: page }) => {
    await page.goto('/subscriptions/plans/plan-1/edit');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-plan-edit');
  });
});
