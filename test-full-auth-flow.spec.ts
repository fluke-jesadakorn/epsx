/**
 * Comprehensive Cross-App Authentication Flow Test
 * Tests authentication across both frontend and admin-frontend apps
 * User: info@epsx.io, Password: P@ssword
 */

import { test, expect, Page, BrowserContext } from '@playwright/test';

// Test credentials
const TEST_CREDENTIALS = {
  email: 'info@epsx.io',
  password: 'P@ssword'
};

// URLs for both apps
const FRONTEND_URL = 'https://epsx.io';
const ADMIN_URL = 'https://admin.epsx.io';
const API_URL = 'https://api.epsx.io';

test.describe('🔐 Full System Authentication Flow', () => {
  let context: BrowserContext;
  let page: Page;

  test.beforeEach(async ({ browser }) => {
    // Create fresh context for each test
    context = await browser.newContext({
      viewport: { width: 1440, height: 900 },
      ignoreHTTPSErrors: true, // Accept self-signed certificates
      userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
    });
    page = await context.newPage();
    
    // Enable request/response logging for debugging
    page.on('request', request => {
      if (request.url().includes('/oauth/') || request.url().includes('/auth/')) {
        console.log('🔍 Request:', request.method(), request.url());
      }
    });
    
    page.on('response', response => {
      if (response.url().includes('/oauth/') || response.url().includes('/auth/')) {
        console.log('📄 Response:', response.status(), response.url());
      }
    });

    // Handle any dialogs
    page.on('dialog', async dialog => {
      console.log('📋 Dialog:', dialog.type(), dialog.message());
      await dialog.accept();
    });
  });

  test.afterEach(async () => {
    await page?.close();
    await context?.close();
  });

  test('🌐 Frontend OAuth Flow Complete Test', async () => {
    console.log('🚀 Starting frontend OAuth flow test...');

    // Step 1: Navigate to frontend homepage
    await page.goto(FRONTEND_URL);
    await page.waitForLoadState('networkidle');
    console.log('✅ Frontend homepage loaded');

    // Step 2: Try to access protected route (should redirect to login)
    await page.goto(`${FRONTEND_URL}/dashboard`);
    await page.waitForURL(/.*login/, { timeout: 10000 });
    console.log('✅ Protected route redirected to login');

    // Step 3: Click login button to start OAuth flow
    const loginButton = page.locator('button:has-text("Continue with EPSX"), button:has-text("Sign In"), a:has-text("Login")').first();
    await expect(loginButton).toBeVisible({ timeout: 10000 });
    await loginButton.click();
    console.log('✅ Login button clicked');

    // Step 4: Wait for OAuth authorization redirect
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    console.log('✅ Redirected to OAuth authorization');

    // Step 5: Fill credentials
    await page.waitForSelector('input[name="email"], input[type="email"]', { timeout: 10000 });
    await page.fill('input[name="email"], input[type="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"], input[type="password"]', TEST_CREDENTIALS.password);
    console.log('✅ Credentials filled');

    // Step 6: Submit login form
    const submitButton = page.locator('button[type="submit"], button:has-text("Login"), button:has-text("Sign In")');
    await submitButton.click();
    console.log('✅ Login form submitted');

    // Step 7: Wait for callback processing
    await page.waitForURL(/callback/, { timeout: 20000 });
    console.log('✅ Callback processing');

    // Step 8: Wait for final redirect to dashboard
    await page.waitForURL(/dashboard/, { timeout: 15000 });
    console.log('✅ Redirected to dashboard');

    // Step 9: Verify authentication successful
    await expect(page.getByText('Dashboard')).toBeVisible({ timeout: 10000 });
    console.log('✅ Frontend authentication successful');
  });

  test('👨‍🍳 Admin Frontend OAuth Flow Complete Test', async () => {
    console.log('🚀 Starting admin frontend OAuth flow test...');

    // Step 1: Navigate to admin homepage
    await page.goto(ADMIN_URL);
    await page.waitForLoadState('networkidle');
    console.log('✅ Admin homepage loaded');

    // Step 2: Navigate to login or find login button
    const loginButton = page.locator('a[href*="/login"], button:has-text("Sign In"), button:has-text("Login")').first();
    
    if (await loginButton.isVisible()) {
      await loginButton.click();
    } else {
      // Try direct login page
      await page.goto(`${ADMIN_URL}/login`);
    }
    console.log('✅ Admin login initiated');

    // Step 3: Wait for OAuth authorization redirect
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    console.log('✅ Redirected to admin OAuth authorization');

    // Step 4: Verify admin-specific scope
    const form = page.locator('form[action*="/oauth/authorize"]');
    if (await form.isVisible()) {
      const scopeValue = await form.locator('input[name="scope"]').getAttribute('value');
      expect(scopeValue).toContain('admin');
      console.log('✅ Admin scope verified:', scopeValue);
    }

    // Step 5: Fill admin credentials
    await page.waitForSelector('input[name="email"], input[type="email"]', { timeout: 10000 });
    await page.fill('input[name="email"], input[type="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"], input[type="password"]', TEST_CREDENTIALS.password);
    console.log('✅ Admin credentials filled');

    // Step 6: Submit admin login form
    const submitButton = page.locator('button[type="submit"], button:has-text("Login"), button:has-text("Sign In")');
    await submitButton.click();
    console.log('✅ Admin login form submitted');

    // Step 7: Wait for admin callback processing
    await page.waitForURL(/callback/, { timeout: 20000 });
    console.log('✅ Admin callback processing');

    // Step 8: Wait for final redirect to admin dashboard
    await page.waitForURL(ADMIN_URL, { timeout: 15000 });
    console.log('✅ Redirected to admin dashboard');

    // Step 9: Verify admin features are accessible
    const adminFeatures = page.locator('text=Users, text=IAM, text=Admin, nav a:has-text("Users")');
    await expect(adminFeatures.first()).toBeVisible({ timeout: 10000 });
    console.log('✅ Admin authentication successful');
  });

  test('🔄 Cross-App Session Verification', async () => {
    console.log('🚀 Starting cross-app session verification...');

    // Step 1: Login to frontend first
    await page.goto(`${FRONTEND_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    
    await page.fill('input[name="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"]', TEST_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(/dashboard/, { timeout: 20000 });
    console.log('✅ Frontend login completed');

    // Step 2: Check if session works on admin (should require separate login)
    await page.goto(ADMIN_URL);
    await page.waitForTimeout(3000);
    
    // Step 3: Verify session behavior
    const isAdminAccessible = !page.url().includes('/login') && !page.url().includes('/oauth/authorize');
    if (isAdminAccessible) {
      console.log('✅ Cross-app session sharing detected');
    } else {
      console.log('ℹ️ Apps require separate authentication (expected behavior)');
    }
  });

  test('🔐 Session API Endpoints Test', async () => {
    console.log('🚀 Testing session API endpoints...');

    // Test frontend session endpoint
    const frontendResponse = await page.request.get(`${FRONTEND_URL}/api/auth/session`);
    expect(frontendResponse.ok()).toBeTruthy();
    console.log('✅ Frontend session API accessible');

    // Test admin session endpoint
    const adminResponse = await page.request.get(`${ADMIN_URL}/api/auth/session`);
    expect(adminResponse.ok()).toBeTruthy();
    console.log('✅ Admin session API accessible');

    // Test backend health/auth endpoints
    try {
      const backendResponse = await page.request.get(`${API_URL}/health`);
      if (backendResponse.ok()) {
        console.log('✅ Backend health endpoint accessible');
      }
    } catch (error) {
      console.log('⚠️ Backend health endpoint not accessible:', error);
    }
  });

  test('🚪 Logout Flow Test', async () => {
    console.log('🚀 Testing logout flows...');

    // Step 1: Login to frontend
    await page.goto(`${FRONTEND_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    
    await page.fill('input[name="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"]', TEST_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(/dashboard/, { timeout: 20000 });
    console.log('✅ Frontend login for logout test completed');

    // Step 2: Find and test logout
    const logoutButton = page.locator('button:has-text("Logout"), button:has-text("Sign Out"), a:has-text("Logout")');
    
    if (await logoutButton.first().isVisible()) {
      await logoutButton.first().click();
      await page.waitForURL(/login|oauth/, { timeout: 10000 });
      console.log('✅ Frontend logout successful');
    } else {
      console.log('⚠️ Logout button not found in frontend');
    }

    // Step 3: Verify session is cleared
    await page.goto(`${FRONTEND_URL}/dashboard`);
    await page.waitForURL(/login/, { timeout: 10000 });
    console.log('✅ Session cleared - protected route redirects to login');
  });

  test('⚡ Performance and Error Handling', async () => {
    console.log('🚀 Testing performance and error handling...');

    // Step 1: Test invalid credentials
    await page.goto(`${FRONTEND_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });

    await page.fill('input[name="email"]', 'invalid@test.com');
    await page.fill('input[name="password"]', 'wrongpassword');
    await page.locator('button[type="submit"]').click();

    // Step 2: Check for error handling
    const errorMessage = page.locator('text=Invalid, text=Error, text=Failed, .error');
    await expect(errorMessage.first()).toBeVisible({ timeout: 10000 });
    console.log('✅ Error handling working for invalid credentials');

    // Step 3: Test valid credentials after error
    await page.fill('input[name="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"]', TEST_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(/dashboard/, { timeout: 20000 });
    console.log('✅ Recovery from error successful');
  });

  test('🎯 Backend API Integration Test', async () => {
    console.log('🚀 Testing backend API integration...');

    // Step 1: Complete authentication flow
    await page.goto(`${FRONTEND_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    
    await page.fill('input[name="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"]', TEST_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(/dashboard/, { timeout: 20000 });
    console.log('✅ Authentication completed for API test');

    // Step 2: Test authenticated API calls
    try {
      const apiResponse = await page.request.get(`${API_URL}/api/v1/user/profile`);
      if (apiResponse.ok()) {
        console.log('✅ Authenticated API call successful');
      } else {
        console.log('⚠️ API call returned:', apiResponse.status());
      }
    } catch (error) {
      console.log('⚠️ API call failed:', error);
    }
  });
});

console.log('🔐 Full System Auth Test Suite Loaded');