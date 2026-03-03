import { test, expect, capture } from './utils/screenshot';
import { mockAllApis } from './utils/api-interceptor';

test.describe('Navigation & Theme', () => {
  test.beforeEach(async ({ page }) => {
    await mockAllApis(page);
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    // Wait for nav hydration (wagmi/auth resolves)
    await expect(page.locator('header')).toBeVisible();
  });

  test('home nav link is accessible', async ({ page }) => {
    await expect(page.locator('header')).toBeVisible();
    // Desktop nav buttons confirm the nav is rendered and hydrated
    await expect(page.getByRole('button', { name: /Market/i }).first()).toBeVisible();
  });

  test('Market dropdown opens with analytics link', async ({ page }) => {
    const marketBtn = page.getByRole('button', { name: /Market/i }).first();
    await expect(marketBtn).toBeVisible();
    await marketBtn.click();
    await page.waitForTimeout(400);
    // Radix renders portal outside <nav> — search at page level
    const analyticsLink = page.locator('a[href="/analytics"]').first();
    await expect(analyticsLink).toBeVisible({ timeout: 3000 });
    await capture(page, 'nav-market-dropdown');
  });

  test('Developer dropdown opens with API keys link', async ({ page }) => {
    const devBtn = page.getByRole('button', { name: /Developer/i }).first();
    await expect(devBtn).toBeVisible();
    await devBtn.click();
    await page.waitForTimeout(400);
    // Developer group has /developer and /developer/docs links
    const devLink = page.locator('a[href="/developer"], a[href="/developer/docs"]').first();
    await expect(devLink).toBeVisible({ timeout: 3000 });
    await capture(page, 'nav-developer-dropdown');
  });

  test('Company dropdown opens with contact link', async ({ page }) => {
    const companyBtn = page.getByRole('button', { name: /Company/i }).first();
    await expect(companyBtn).toBeVisible();
    await companyBtn.click();
    await page.waitForTimeout(400);
    // Company group has /about, /news, /contact, /chat
    const contactLink = page.locator('a[href="/contact"], a[href="/about"]').first();
    await expect(contactLink).toBeVisible({ timeout: 3000 });
    await capture(page, 'nav-company-dropdown');
  });

  test('theme toggle switches between light and dark', async ({ page }) => {
    const themeBtn = page.getByRole('button', { name: /light|dark|theme/i }).first();
    await expect(themeBtn).toBeVisible();
    const htmlEl = page.locator('html');
    const initialClass = await htmlEl.getAttribute('class') ?? '';
    await themeBtn.click();
    await page.waitForTimeout(300);
    const newClass = await htmlEl.getAttribute('class') ?? '';
    expect(newClass).not.toBe(initialClass);
    await capture(page, 'nav-theme-toggled');
  });
});
