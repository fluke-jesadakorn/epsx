import { test, expect, capture } from './utils/screenshot';

test.describe('Admin Wallet Management', () => {
  test('wallet management hub', async ({ authedPage: page }) => {
    await page.goto('/wallet-management');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main, [role="main"]')).toBeVisible();
    await capture(page, 'admin-wallet-management');
  });

  test('wallet detail page', async ({ authedPage: page }) => {
    await page.goto('/wallet-management/0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-wallet-detail');
  });

  test('wallets list', async ({ authedPage: page }) => {
    await page.goto('/wallet-management/wallets');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-wallets-list');
  });

  test('wallet disable page', async ({ authedPage: page }) => {
    await page.goto('/wallet-management/wallets/0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68/disable');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-wallet-disable');
  });

  test('wallet activity log', async ({ authedPage: page }) => {
    await page.goto('/wallet-management/activity');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-wallet-activity');
  });

  test('wallet credits', async ({ authedPage: page }) => {
    await page.goto('/wallet-management/credits');
    await page.waitForLoadState('networkidle');
    await capture(page, 'admin-wallet-credits');
  });
});
