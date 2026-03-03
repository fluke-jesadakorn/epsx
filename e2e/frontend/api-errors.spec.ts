/**
 * API Error State Tests
 * Verifies the frontend handles API failures gracefully (no crashes, shows fallback UI).
 */
import { test, expect, capture } from './utils/screenshot';
import { mockAuth } from './utils/auth-mock';

test.describe('API Error States', () => {
  test('analytics page renders when rankings API returns 500', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));

    // Return 500 for rankings
    await page.route('**/api/analytics/rankings', route =>
      route.fulfill({ status: 500, contentType: 'application/json', body: JSON.stringify({ error: 'Internal Server Error' }) })
    );
    await page.route('**/api/public/analytics/rankings', route =>
      route.fulfill({ status: 500, contentType: 'application/json', body: JSON.stringify({ error: 'Internal Server Error' }) })
    );

    await page.goto('/analytics');
    await page.waitForLoadState('networkidle');

    await expect(page.locator('body')).toBeVisible();
    expect(errors).toHaveLength(0);
    await capture(page, 'api-error-analytics-500');
  });

  test('home page renders when market overview API returns 500', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));

    await page.route('**/api/analytics/market/**', route =>
      route.fulfill({ status: 500, contentType: 'application/json', body: JSON.stringify({ error: 'Service Unavailable' }) })
    );
    await page.route('**/api/public/**', route =>
      route.fulfill({ status: 500, contentType: 'application/json', body: JSON.stringify({ error: 'Service Unavailable' }) })
    );

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    await expect(page.locator('body')).toBeVisible();
    expect(errors).toHaveLength(0);
    await capture(page, 'api-error-home-500');
  });

  test('plans page renders when plans API returns empty', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));

    await page.route('**/api/public/plans**', route =>
      route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ data: [], success: true }) })
    );

    await page.goto('/plans');
    await page.waitForLoadState('networkidle');

    await expect(page.locator('body')).toBeVisible();
    expect(errors).toHaveLength(0);
    await capture(page, 'api-error-plans-empty');
  });

  test('notifications API 401 does not crash authenticated page', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));

    await mockAuth(page);

    await page.route('**/api/notifications**', route =>
      route.fulfill({ status: 401, contentType: 'application/json', body: JSON.stringify({ error: 'Unauthorized' }) })
    );

    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');

    await expect(page.locator('body')).toBeVisible();
    expect(errors).toHaveLength(0);
    await capture(page, 'api-error-notifications-401');
  });

  test('network error on API does not crash home page', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));

    await page.route('**/api/**', route => route.abort('failed'));

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    await expect(page.locator('body')).toBeVisible();
    expect(errors).toHaveLength(0);
    await capture(page, 'api-error-network-abort');
  });
});
