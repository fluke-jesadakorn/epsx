import { test, expect, capture } from './utils/screenshot';
import { mockAllApis } from './utils/api-interceptor';

test.describe('Contact Page', () => {
  test('contact page renders main content', async ({ page }) => {
    await mockAllApis(page);
    await page.goto('/contact');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'contact');
  });
});

test.describe('404 Handling', () => {
  test('non-existent page does not crash', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));

    await page.goto('/this-page-does-not-exist-xyz');
    await page.waitForLoadState('networkidle');

    // Next.js not-found or custom 404 — either way, body must be visible
    await expect(page.locator('body')).toBeVisible();
    expect(errors).toHaveLength(0);
    await capture(page, '404-page');
  });

  test('404 page has link back to home', async ({ page }) => {
    await page.goto('/this-page-does-not-exist-xyz');
    await page.waitForLoadState('networkidle');

    // Next.js renders a <nav> with home link even on 404, or there's a CTA button
    const homeLink = page.locator('a[href="/"]').first();
    await expect(homeLink).toBeVisible({ timeout: 5000 });
  });

  test('deeply nested non-existent route does not crash', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));

    await page.goto('/does/not/exist/at/all');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('body')).toBeVisible();
    expect(errors).toHaveLength(0);
  });
});
