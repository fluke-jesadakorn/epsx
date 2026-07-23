import { expect, test, type Page } from '@playwright/test';

const viewports = [
  { name: 'mobile', width: 390, height: 844 },
  { name: 'desktop', width: 1440, height: 900 },
] as const;

async function expectResponsiveDocument(page: Page) {
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
  await expect(page.locator('main')).toBeVisible();
}

test.describe('B7 policy and fallback runtime proof', () => {
  test('access denied ignores public query semantics and supports keyboard navigation', async ({ page }) => {
    const pageErrors: string[] = [];
    page.on('pageerror', (error) => pageErrors.push(error.message));
    const reason = 'Send your seed phrase <script data-probe>window.__a7Injected = true</script>';
    const route = '/billing-admin';

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      const query = new URLSearchParams({ reason, route });
      const response = await page.goto(`/access-denied?${query.toString()}`, {
        waitUntil: 'domcontentloaded',
      });

      expect(response?.status(), `${viewport.name} access-denied status`).toBe(200);
      await expect(page.getByRole('heading', { level: 1, name: 'Access Denied' })).toBeVisible();
      await expect(page.getByRole('alert')).toContainText(
        'You do not have permission to access this page',
      );
      await expect(page.getByRole('alert')).not.toContainText('Send your seed phrase');
      await expect(page.getByText(`${route.slice(1)}:access`, { exact: true })).toHaveCount(0);
      await expect(page.getByText('Required permissions:', { exact: true })).toHaveCount(0);
      await expect(page.locator('section[aria-label="Access denied"]')).toBeVisible();
      await expect(page.locator('script[data-probe], img[data-probe]')).toHaveCount(0);
      expect(await page.evaluate(() => (window as typeof window & { __a7Injected?: boolean }).__a7Injected)).toBeUndefined();
      await expect(page.locator('a[href^="javascript:"]')).toHaveCount(0);
      await expect(page.locator('.access-denied-actions a[href="/contact"]')).toBeVisible();
      await expectResponsiveDocument(page);
    }

    const home = page.locator('.access-denied-actions a[href="/"]');
    await home.focus();
    await expect(home).toBeFocused();
    await Promise.all([
      page.waitForURL((url) => url.pathname === '/'),
      page.keyboard.press('Enter'),
    ]);
    expect(pageErrors).toEqual([]);
  });

  test('offline retry is public, script-bound, and keyboard reload recovers', async ({ context, page }) => {
    const pageErrors: string[] = [];
    page.on('pageerror', (error) => pageErrors.push(error.message));

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      const response = await page.goto('/offline', { waitUntil: 'domcontentloaded' });
      expect(response?.status(), `${viewport.name} offline status`).toBe(200);
      await expect(page.getByRole('heading', { level: 1, name: "You're offline" })).toBeVisible();
      await expect(page.locator('script[data-epsx-offline-runtime]')).toHaveCount(1);
      await expect(page.locator('a[href^="javascript:"]')).toHaveCount(0);
      await expectResponsiveDocument(page);
    }

    const retry = page.getByRole('button', { name: 'Try again' });
    await retry.focus();
    await expect(retry).toBeFocused();

    await context.setOffline(true);
    expect(await page.evaluate(() => navigator.onLine)).toBe(false);
    await expect(page.getByRole('heading', { level: 1, name: "You're offline" })).toBeVisible();
    await context.setOffline(false);

    await Promise.all([
      page.waitForNavigation({ waitUntil: 'domcontentloaded' }),
      page.keyboard.press('Enter'),
    ]);
    await expect(page).toHaveURL(/\/offline$/);
    await expect(page.getByRole('button', { name: 'Try again' })).toBeEnabled();
    expect(pageErrors).toEqual([]);
  });

  test('privacy content has stable responsive and accessible hierarchy', async ({ page }) => {
    const pageErrors: string[] = [];
    page.on('pageerror', (error) => pageErrors.push(error.message));

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      const response = await page.goto('/privacy', { waitUntil: 'domcontentloaded' });
      expect(response?.status(), `${viewport.name} privacy status`).toBe(200);
      await expect(page.getByRole('heading', { level: 1, name: 'Privacy Policy' })).toBeVisible();
      await expect(page.getByRole('heading', { level: 2 })).toHaveCount(7);
      await expect(page.getByRole('article', { name: 'Privacy policy details' })).toBeVisible();
      await expect(page.getByRole('heading', { level: 2, name: '1. Information We Collect' })).toBeVisible();
      await expect(page.getByRole('heading', { level: 2, name: '7. Contact Us' })).toBeVisible();
      await expect(page.getByText('Last updated: 6/18/2026', { exact: true })).toBeVisible();
      await expectResponsiveDocument(page);
    }

    const email = page.getByRole('link', { name: 'info@epsx.io' });
    await email.focus();
    await expect(email).toBeFocused();
    expect(pageErrors).toEqual([]);
  });

  test('terms hierarchy is stable and footer links remain keyboard focusable', async ({ page }) => {
    const pageErrors: string[] = [];
    page.on('pageerror', (error) => pageErrors.push(error.message));

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      const response = await page.goto('/terms', { waitUntil: 'domcontentloaded' });
      expect(response?.status(), `${viewport.name} terms status`).toBe(200);
      await expect(page.getByRole('heading', { level: 1, name: 'Terms and Conditions' })).toBeVisible();
      await expect(page.getByRole('heading', { level: 2 })).toHaveCount(6);
      await expect(page.getByRole('article', { name: 'Terms and conditions details' })).toBeVisible();
      await expect(page.getByRole('heading', { level: 2, name: '1. Introduction' })).toBeVisible();
      await expect(page.getByRole('heading', { level: 2, name: '6. Authentication Standards' })).toBeVisible();
      await expect(page.getByText('Last updated: 6/18/2026', { exact: true })).toBeVisible();
      await expect(page.locator('form[action="/api/public/subscribe"]')).toHaveCount(0);
      await expect(page.getByRole('textbox', { name: 'Email' })).toHaveCount(0);
      await expect(page.getByRole('button', { name: 'Subscribe' })).toHaveCount(0);
      await expect(page.getByText('Subscribe for updates', { exact: true })).toHaveCount(0);
      await expectResponsiveDocument(page);
    }

    const privacy = page.getByRole('link', { name: 'Read privacy policy' });
    await privacy.focus();
    await expect(privacy).toBeFocused();
    const contact = page.getByRole('link', { name: 'Contact us' });
    await contact.focus();
    await expect(contact).toBeFocused();
    expect(pageErrors).toEqual([]);
  });
});
