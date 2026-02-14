import { test, expect, capture } from '../utils/screenshot';
import { mockAllApis } from '../utils/api-interceptor';

test.describe('Plans & Payment', () => {
  test('plans page shows available tiers', async ({ page }) => {
    await mockAllApis(page);
    await page.goto('/plans');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'plans');
  });

  test('payment page shows checkout form', async ({ authedPage: page }) => {
    await page.goto('/payment');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'payment');
  });

  test('payment with type and id renders details', async ({ authedPage: page }) => {
    await page.goto('/payment/crypto/pro');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'payment-detail');
  });
});
