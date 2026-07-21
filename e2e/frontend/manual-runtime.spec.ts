import { expect, test, type Page } from '@playwright/test';

const viewports = [
  { name: 'mobile', width: 390, height: 844 },
  { name: 'desktop', width: 1440, height: 900 },
] as const;

const categories = [
  'Public',
  'Auth',
  'Dashboard',
  'Analytics',
  'Plans',
  'Portfolio',
  'Notifications',
  'Developer',
] as const;

async function expectNoHorizontalOverflow(page: Page) {
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
}

test.describe('A7 /manual accepted-content and runtime proof', () => {
  test('catalog is responsive, semantic, keyboard usable, and error free', async ({ page, request }) => {
    const pageErrors: string[] = [];
    const consoleErrors: string[] = [];
    page.on('pageerror', (error) => pageErrors.push(error.message));
    page.on('console', (message) => {
      if (message.type() === 'error') consoleErrors.push(message.text());
    });

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      const response = await page.goto('/manual', { waitUntil: 'domcontentloaded' });

      expect(response?.status(), `${viewport.name} /manual status`).toBe(200);
      await expect(page).toHaveTitle('EPSX Manual - Feature Guide');
      await expect(page.getByRole('heading', { level: 1, name: 'EPSX Feature Manual' })).toBeVisible();
      await expect(page.getByRole('navigation', { name: 'Manual categories' })).toBeVisible();
      await expect(page.locator('.manual-prod-category > h2')).toHaveText([...categories]);
      await expect(page.locator('article.manual-prod-feature')).toHaveCount(35);
      await expect(page.locator('[data-manual-screenshot="true"]')).toHaveCount(35);
      await expect(page.locator('a.manual-prod-feature-link')).toHaveCount(35);
      await expect(page.getByText('Complete guide to all platform features. Screenshots auto-generated from E2E tests.')).toBeVisible();
      await expect(page.getByRole('heading', { level: 3, name: 'Stock Rankings' })).toBeVisible();
      await expect(page.getByText('/payment/[type]/[id]', { exact: true })).toBeVisible();
      await expect(page.locator('section[aria-labelledby="public-heading"]')).toBeVisible();
      await expect(page.locator('script[data-epsx-manual-runtime]')).toHaveCount(1);
      await expect(page.locator('a[href^="javascript:"]')).toHaveCount(0);
      await expectNoHorizontalOverflow(page);
    }

    const screenshotPaths = await page.locator('[data-manual-screenshot="true"]').evaluateAll(
      (buttons) => buttons.map((button) => button.getAttribute('data-screenshot-src')),
    );
    expect(screenshotPaths).toHaveLength(35);
    expect(new Set(screenshotPaths).size).toBe(35);
    for (const path of screenshotPaths) {
      expect(path).not.toBeNull();
      const asset = await request.get(path!);
      expect(asset.status(), `${path} status`).toBe(200);
      expect(asset.headers()['content-type'], `${path} content type`).toContain('image/webp');
    }

    const firstCategory = page.getByRole('link', { name: 'Public', exact: true });
    await firstCategory.focus();
    await expect(firstCategory).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(page).toHaveURL(/\/manual#public$/);
    await expect(page.locator('#public')).toBeVisible();

    const screenshotButton = page.getByRole('button', { name: 'Open Home screenshot' });
    await screenshotButton.focus();
    await expect(screenshotButton).toBeFocused();
    await page.keyboard.press('Enter');
    const dialog = page.locator('[data-manual-dialog="true"]');
    await expect(dialog).toBeVisible();
    await expect(dialog).toHaveAttribute('aria-modal', 'true');
    await expect(dialog.locator('[data-manual-dialog-title="true"]')).toHaveText('Home');
    await expect(dialog.locator('[data-manual-dialog-image="true"]')).toHaveAttribute(
      'src',
      '/public/screenshots/home.webp',
    );
    const close = page.getByRole('button', { name: 'Close screenshot' });
    await expect(close).toBeFocused();
    await page.keyboard.press('Tab');
    await expect(close).toBeFocused();
    await page.keyboard.press('Escape');
    await expect(dialog).toBeHidden();
    await expect(screenshotButton).toBeFocused();

    const dynamicRoute = page.locator('a[data-route-template="true"]');
    await expect(dynamicRoute).toHaveCount(1);
    await dynamicRoute.focus();
    await expect(dynamicRoute).toBeFocused();
    const beforeDynamic = page.url();
    await page.keyboard.press('Enter');
    expect(page.url()).toBe(beforeDynamic);

    const homeLink = page.locator('article[aria-labelledby="manual-feature-home"] a[href="/"]');
    await homeLink.focus();
    await expect(homeLink).toBeFocused();
    await Promise.all([
      page.waitForURL((url) => url.pathname === '/'),
      page.keyboard.press('Enter'),
    ]);
    await expect(page.getByRole('main')).toBeVisible();
    expect(pageErrors).toEqual([]);
    expect(consoleErrors).toEqual([]);
  });

  test('missing screenshot becomes a non-interactive fallback', async ({ page }) => {
    await page.route('**/public/screenshots/home.webp', (route) => route.abort());
    const response = await page.goto('/manual', { waitUntil: 'domcontentloaded' });
    expect(response?.status()).toBe(200);

    const homeScreenshot = page.getByRole('button', { name: 'Home screenshot unavailable' });
    await expect(homeScreenshot).toBeDisabled();
    await expect(homeScreenshot.locator('.manual-prod-screenshot-fallback')).toBeVisible();
    await expect(page.locator('[data-manual-dialog="true"]')).toBeHidden();
  });
});
