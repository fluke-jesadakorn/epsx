import { test, expect, capture } from './utils/screenshot';
import { mockAllApis } from './utils/api-interceptor';

test.describe('Watchlist - Unauthenticated', () => {
  test('portfolio page renders without crashing', async ({ page }) => {
    await mockAllApis(page);
    await page.goto('/portfolio');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'watchlist-unauthed');
  });
});

test.describe('Watchlist - Authenticated', () => {
  test('portfolio page renders with search input', async ({ authedPage: page }) => {
    await page.goto('/portfolio');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    const searchInput = page.locator('input[placeholder*="watchlist"]');
    await expect(searchInput).toBeVisible({ timeout: 5000 });
    await capture(page, 'watchlist-authed');
  });

  test('search input accepts text', async ({ authedPage: page }) => {
    await page.goto('/portfolio');
    await page.waitForLoadState('networkidle');
    const searchInput = page.locator('input[placeholder*="watchlist"]');
    await expect(searchInput).toBeVisible({ timeout: 5000 });
    await searchInput.fill('AAPL');
    const val = await searchInput.inputValue();
    expect(val).toBe('AAPL');
    await capture(page, 'watchlist-search-typed');
  });

  test('portfolio heading is visible', async ({ authedPage: page }) => {
    await page.goto('/portfolio');
    await page.waitForLoadState('networkidle');
    await expect(page.getByRole('heading', { name: /portfolio/i }).first()).toBeVisible({ timeout: 5000 });
  });

  test('portfolio content renders without JS errors', async ({ authedPage: page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));
    await page.goto('/portfolio');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    expect(errors).toHaveLength(0);
  });
});
