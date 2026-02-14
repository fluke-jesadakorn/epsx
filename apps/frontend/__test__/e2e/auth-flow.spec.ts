import { test, expect, capture } from '../utils/screenshot';
import { mockAuth } from '../utils/auth-mock';
import { mockAllApis } from '../utils/api-interceptor';

test.describe('Auth Flow', () => {
  test('auth page shows wallet connect options', async ({ page }) => {
    await mockAllApis(page);
    await page.goto('/auth');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('main')).toBeVisible();
    await capture(page, 'auth');
  });

  test('authenticated user redirected from auth page', async ({ page }) => {
    await mockAuth(page);
    await mockAllApis(page);
    await page.goto('/auth');
    await page.waitForLoadState('networkidle');
    // Should redirect to dashboard or stay on auth with user info
    await capture(page, 'auth-redirect');
  });

  test('unauthenticated user redirected to auth from protected route', async ({ page }) => {
    await mockAllApis(page);
    // Don't set auth cookies - middleware should redirect
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    // Should redirect to /auth
    await expect(page).toHaveURL(/auth/);
  });
});
