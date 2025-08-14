/**
 * Frontend Authentication E2E Tests
 * Tests the complete custom OAuth 2.0 authentication flow
 */
import { test, expect } from '@playwright/test';

test.describe('Frontend Authentication Flow', () => {
  const TEST_EMAIL = 'jesadakorn.kirtnu@gmail.com';
  const TEST_PASSWORD = 'Aa_12345678';
  const BACKEND_URL = 'http://localhost:8080';

  test.beforeEach(async ({ page }) => {
    // Clear any existing session/cookies
    await page.context().clearCookies();
    await page.context().clearPermissions();
  });

  test('should redirect unauthenticated user to login', async ({ page }) => {
    // Try to access protected dashboard route
    await page.goto('/dashboard');
    
    // Should redirect to login page
    await expect(page).toHaveURL(/\/login/);
    
    // Should display login form
    await expect(page.getByRole('button', { name: 'Continue with EPSX' })).toBeVisible();
  });

  test('should complete OAuth 2.0 sign-in flow successfully', async ({ page }) => {
    console.log('🧪 Starting frontend OAuth 2.0 authentication test');

    // Step 1: Navigate to login page
    await page.goto('/login');
    await expect(page.getByRole('button', { name: 'Continue with EPSX' })).toBeVisible();
    
    console.log('✅ Login page loaded successfully');

    // Step 2: Click sign-in button to start OAuth flow
    const signInButton = page.getByRole('button', { name: 'Continue with EPSX' });
    await signInButton.click();

    // Step 3: Should redirect to backend authorization server
    await page.waitForURL(new RegExp(`${BACKEND_URL}/oauth/authorize`), { timeout: 10000 });
    
    console.log('✅ Redirected to backend authorization server');

    // Step 4: Fill in test credentials on backend login form
    await page.waitForSelector('input[name="email"]', { timeout: 10000 });
    await page.fill('input[name="email"]', TEST_EMAIL);
    await page.fill('input[name="password"]', TEST_PASSWORD);
    
    console.log('✅ Test credentials entered');

    // Step 5: Submit backend login form
    const loginButton = page.locator('button[type="submit"]').first();
    await loginButton.click();

    // Step 6: Should redirect back to frontend callback
    await page.waitForURL(/\/api\/auth\/callback/, { timeout: 15000 });
    
    console.log('✅ Redirected to frontend callback');

    // Step 7: Should complete callback processing and redirect to dashboard
    await page.waitForURL('/dashboard', { timeout: 15000 });
    
    console.log('✅ Successfully redirected to dashboard');

    // Step 8: Verify user is authenticated and dashboard loads
    await expect(page.getByText('Dashboard')).toBeVisible();
    
    // Should display user information
    await expect(page.getByText(TEST_EMAIL.split('@')[0])).toBeVisible();
    
    console.log('✅ User authenticated successfully on dashboard');
  });

  test('should persist session across page reloads', async ({ page }) => {
    // First complete the login flow
    await page.goto('/login');
    const signInButton = page.getByRole('button', { name: 'Continue with EPSX' });
    await signInButton.click();

    await page.waitForURL(new RegExp(`${BACKEND_URL}/oauth/authorize`), { timeout: 10000 });
    await page.waitForSelector('input[name="email"]', { timeout: 10000 });
    await page.fill('input[name="email"]', TEST_EMAIL);
    await page.fill('input[name="password"]', TEST_PASSWORD);
    
    const loginButton = page.locator('button[type="submit"]').first();
    await loginButton.click();
    
    await page.waitForURL('/dashboard', { timeout: 15000 });

    // Verify session persists after page reload
    await page.reload();
    await expect(page).toHaveURL('/dashboard');
    await expect(page.getByText('Dashboard')).toBeVisible();
    
    console.log('✅ Session persisted across page reload');
  });

  test('should allow user to sign out successfully', async ({ page }) => {
    // First complete the login flow
    await page.goto('/login');
    const signInButton = page.getByRole('button', { name: 'Continue with EPSX' });
    await signInButton.click();

    await page.waitForURL(new RegExp(`${BACKEND_URL}/oauth/authorize`), { timeout: 10000 });
    await page.waitForSelector('input[name="email"]', { timeout: 10000 });
    await page.fill('input[name="email"]', TEST_EMAIL);
    await page.fill('input[name="password"]', TEST_PASSWORD);
    
    const loginButton = page.locator('button[type="submit"]').first();
    await loginButton.click();
    
    await page.waitForURL('/dashboard', { timeout: 15000 });

    // Find and click sign out button
    const signOutButton = page.getByText('Sign Out');
    await signOutButton.click();

    // Should redirect to login page
    await page.waitForURL('/login', { timeout: 10000 });
    await expect(page.getByRole('button', { name: 'Continue with EPSX' })).toBeVisible();
    
    console.log('✅ User signed out successfully');

    // Verify session is cleared - accessing protected route should redirect to login
    await page.goto('/dashboard');
    await expect(page).toHaveURL(/\/login/);
  });

  test('should handle OAuth callback errors gracefully', async ({ page }) => {
    // Test callback with error parameter
    await page.goto('/api/auth/callback/epsx-backend?error=access_denied&error_description=User+denied+access');
    
    // Should redirect to login with error
    await page.waitForURL(/\/login.*error=access_denied/, { timeout: 10000 });
    
    console.log('✅ OAuth error handled gracefully');
  });

  test('should protect routes and redirect to login', async ({ page }) => {
    const protectedRoutes = ['/dashboard', '/settings', '/my-data', '/analytics'];
    
    for (const route of protectedRoutes) {
      await page.goto(route);
      await expect(page).toHaveURL(/\/login.*callbackUrl/);
      console.log(`✅ Protected route ${route} redirects to login`);
    }
  });

  test('should allow access to public routes', async ({ page }) => {
    const publicRoutes = ['/', '/analytics', '/privacy', '/terms'];
    
    for (const route of publicRoutes) {
      await page.goto(route);
      await expect(page).not.toHaveURL(/\/login/);
      console.log(`✅ Public route ${route} accessible without authentication`);
    }
  });

  test('should validate session API endpoint', async ({ page }) => {
    // Test session endpoint when not authenticated
    const response = await page.request.get('/api/auth/session');
    expect(response.ok()).toBeTruthy();
    
    const sessionData = await response.json();
    expect(sessionData.isLoggedIn).toBeFalsy();
    expect(sessionData.user).toBeNull();
    
    console.log('✅ Session API returns correct unauthenticated state');
  });
});

test.describe('Frontend Authentication Integration', () => {
  const TEST_EMAIL = 'jesadakorn.kirtnu@gmail.com';
  
  test('should display correct user information after login', async ({ page }) => {
    // Complete login flow
    await page.goto('/login');
    const signInButton = page.getByRole('button', { name: 'Continue with EPSX' });
    await signInButton.click();

    await page.waitForURL(new RegExp('http://localhost:8080/oauth/authorize'), { timeout: 10000 });
    await page.waitForSelector('input[name="email"]', { timeout: 10000 });
    await page.fill('input[name="email"]', TEST_EMAIL);
    await page.fill('input[name="password"]', 'Aa_12345678');
    
    const loginButton = page.locator('button[type="submit"]').first();
    await loginButton.click();
    
    await page.waitForURL('/dashboard', { timeout: 15000 });

    // Check user information is displayed correctly
    await expect(page.getByText(TEST_EMAIL.split('@')[0])).toBeVisible();
    
    // Navigate to settings to check user details
    await page.goto('/settings');
    await expect(page.getByText(TEST_EMAIL)).toBeVisible();
    
    console.log('✅ User information displayed correctly after authentication');
  });
});