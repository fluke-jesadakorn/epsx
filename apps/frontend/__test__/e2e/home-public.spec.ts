import { test, expect, capture } from '../utils/screenshot';

test.describe('Public Pages', () => {
  test('home page renders hero and navigation', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveTitle(/EPSX/i);
    await expect(page.locator('nav')).toBeVisible();
    await capture(page, 'home', { fullPage: true });
  });

  test('about page displays content', async ({ page }) => {
    await page.goto('/about');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'about');
  });

  test('terms page displays legal content', async ({ page }) => {
    await page.goto('/terms');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'terms');
  });

  test('privacy page displays policy', async ({ page }) => {
    await page.goto('/privacy');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'privacy');
  });

  test('offline page shows fallback', async ({ page }) => {
    await page.goto('/offline');
    await page.waitForLoadState('networkidle');
    await capture(page, 'offline');
  });

  test('access-denied page shows error state', async ({ page }) => {
    await page.goto('/access-denied');
    await page.waitForLoadState('networkidle');
    await capture(page, 'access-denied');
  });
});
