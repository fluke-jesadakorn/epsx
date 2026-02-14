import { test, expect, capture } from '../utils/screenshot';
import { mockAllApis } from '../utils/api-interceptor';

test.describe('Analytics', () => {
  test('analytics page loads with public data', async ({ page }) => {
    await mockAllApis(page);
    await page.goto('/analytics');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'analytics');
  });

  test('analytics with auth shows full data', async ({ authedPage: page }) => {
    await page.goto('/analytics');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'analytics-auth');
  });

  test('analytics filters interaction', async ({ authedPage: page }) => {
    await page.goto('/analytics');
    await page.waitForLoadState('networkidle');
    // Try to click filter elements if they exist
    const filterBtn = page.locator('[data-testid="filter-btn"], button:has-text("Filter"), button:has-text("filter")').first();
    if (await filterBtn.isVisible({ timeout: 3000 }).catch(() => false)) {
      await filterBtn.click();
      await page.waitForTimeout(500);
    }
    await capture(page, 'analytics-filters');
  });
});
