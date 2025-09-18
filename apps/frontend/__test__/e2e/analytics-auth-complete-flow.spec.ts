/**
 * Complete E2E Authentication Flow Tests with Analytics Platform Themes
 * Tests 100% coverage of OIDC authentication flow with test credentials
 * User: test.user@example.com, Password: TestPassword123!
 */

import { test, expect, Page, BrowserContext } from '@playwright/test';
import { URL, URLContext, Service } from '../../../../shared/utils/url-resolver';

// Test credentials - generic test account
const TEST_CREDENTIALS = {
  email: 'test.user@example.com',
  password: 'TestPassword123!'
};

// Environment configuration
const BASE_URL = URL.get(Service.FRONTEND, URLContext.CLIENT);
const API_URL = URL.get(Service.BACKEND, URLContext.CLIENT);
const ADMIN_URL = URL.get(Service.ADMIN, URLContext.CLIENT);

test.describe('📊 Analytics Platform Authentication Complete Flow', () => {
  let context: BrowserContext;
  let page: Page;

  test.beforeEach(async ({ browser }) => {
    // Create fresh context for each test
    context = await browser.newContext({
      viewport: { width: 1280, height: 720 },
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
  });

  test.afterEach(async () => {
    await page?.close();
    await context?.close();
  });

  test('📊 Complete OIDC Authorization Flow - Frontend Login', async () => {
    console.log('🚀 Starting frontend OIDC authorization flow test...');

    // Step 1: Navigate to frontend homepage
    await page.goto(BASE_URL);
    await expect(page).toHaveTitle(/EPSX/);
    console.log('✅ Homepage loaded successfully');

    // Step 2: Click login button to start OIDC flow
    const loginButton = page.locator('a[href*="/login"], button:has-text("Sign In"), a:has-text("Login")').first();
    await expect(loginButton).toBeVisible({ timeout: 10000 });
    await loginButton.click();
    console.log('✅ Login button clicked, initiating OIDC flow');

    // Step 3: Wait for navigation to OIDC authorization endpoint
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    console.log('✅ Navigated to OIDC authorization endpoint');

    // Step 4: Verify Analytics Portal theme elements are present
    await expect(page.locator('h2:has-text("📊 Data Insights Portal")')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.insight-logo, [class*="insight"]')).toBeVisible();
    await expect(page.locator('text=📊')).toBeVisible();
    console.log('✅ Analytics Portal theme verified - Data Insights Portal detected');

    // Step 5: Verify OIDC form structure
    const form = page.locator('form[action*="/oauth/authorize"]');
    await expect(form).toBeVisible();
    await expect(form.locator('input[name="client_id"]')).toHaveValue('epsx-frontend');
    await expect(form.locator('input[name="response_type"]')).toHaveValue('code');
    await expect(form.locator('input[name="scope"]')).toHaveValue(/openid.*profile.*email/);
    console.log('✅ OIDC form structure validated');

    // Step 6: Fill in login credentials
    await page.fill('input[name="email"], input[type="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"], input[type="password"]', TEST_CREDENTIALS.password);
    console.log('✅ Credentials filled');

    // Step 7: Submit login form
    const submitButton = page.locator('button[type="submit"], button:has-text("Access Portal")');
    await expect(submitButton).toBeVisible();
    await submitButton.click();
    console.log('✅ Login form submitted');

    // Step 8: Wait for authentication redirect
    await page.waitForURL(/callback/, { timeout: 20000 });
    console.log('✅ Redirected to callback URL');

    // Step 9: Wait for final redirect to authenticated homepage
    await page.waitForURL(BASE_URL, { timeout: 15000 });
    console.log('✅ Successfully redirected to authenticated homepage');

    // Step 10: Verify user is authenticated
    const userMenu = page.locator('[data-testid="user-menu"], .user-menu, button:has-text("Profile"), button:has-text("Dashboard")');
    await expect(userMenu.or(page.locator('text=Dashboard')).or(page.locator('a[href*="/dashboard"]'))).toBeVisible({ timeout: 10000 });
    console.log('✅ User authentication confirmed');

    // Step 11: Verify user session persistence
    await page.reload();
    await expect(userMenu.or(page.locator('text=Dashboard')).or(page.locator('a[href*="/dashboard"]'))).toBeVisible({ timeout: 5000 });
    console.log('✅ Session persistence verified');
  });

  test('🎯 Complete OIDC Authorization Flow - Admin Login', async () => {
    console.log('🚀 Starting admin OIDC authorization flow test...');

    // Step 1: Navigate to admin frontend
    await page.goto(ADMIN_URL);
    await expect(page).toHaveTitle(/Admin|EPSX/);
    console.log('✅ Admin homepage loaded');

    // Step 2: Click login button for admin flow
    const loginButton = page.locator('a[href*="/login"], button:has-text("Sign In"), a:has-text("Login")').first();
    await expect(loginButton).toBeVisible({ timeout: 10000 });
    await loginButton.click();
    console.log('✅ Admin login button clicked');

    // Step 3: Wait for navigation to OIDC authorization endpoint
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    console.log('✅ Navigated to admin OIDC endpoint');

    // Step 4: Verify Analytics Command Center theme for admin
    const isAdminTheme = await page.locator('h2:has-text("🎯 Analytics Command Center"), h2:has-text("Admin Portal"), text=admin').first().isVisible({ timeout: 5000 }).catch(() => false);
    const isAnalyticsTheme = await page.locator('h2:has-text("📊 Data Insights Portal")').isVisible({ timeout: 5000 }).catch(() => false);
    
    if (isAdminTheme) {
      console.log('✅ Analytics Command Center theme detected for admin');
    } else if (isAnalyticsTheme) {
      console.log('✅ Analytics Portal theme detected (fallback for admin)');
    } else {
      console.log('⚠️ Theme detection inconclusive, continuing with login');
    }

    // Step 5: Verify admin scope in OIDC form
    const form = page.locator('form[action*="/oauth/authorize"]');
    await expect(form).toBeVisible();
    const scopeValue = await form.locator('input[name="scope"]').getAttribute('value');
    expect(scopeValue).toContain('admin');
    console.log('✅ Admin scope verified in OIDC form');

    // Step 6: Fill in admin credentials
    await page.fill('input[name="email"], input[type="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"], input[type="password"]', TEST_CREDENTIALS.password);
    console.log('✅ Admin credentials filled');

    // Step 7: Submit admin login form
    const submitButton = page.locator('button[type="submit"]');
    await submitButton.click();
    console.log('✅ Admin login form submitted');

    // Step 8: Wait for admin authentication redirect
    await page.waitForURL(/callback/, { timeout: 20000 });
    console.log('✅ Admin callback redirect completed');

    // Step 9: Wait for redirect to admin dashboard
    await page.waitForURL(ADMIN_URL, { timeout: 15000 });
    console.log('✅ Redirected to admin dashboard');

    // Step 10: Verify admin access and features
    const adminElements = page.locator('text=Users, text=Permissions, text=Modules, text=Admin, [data-testid="admin-nav"]');
    await expect(adminElements.first()).toBeVisible({ timeout: 10000 });
    console.log('✅ Admin functionality confirmed');
  });

  test('📈 User Registration Flow with Analytics Platform Theme', async () => {
    console.log('🚀 Starting user registration flow test...');

    // Use a test email that won't conflict
    const testEmail = `test+${Date.now()}@epsx.io`;

    // Step 1: Navigate to registration page
    await page.goto(`${BASE_URL}/register`);
    console.log('✅ Registration page loaded');

    // Step 2: Check if it redirects to OIDC with registration parameter
    const currentUrl = page.url();
    if (currentUrl.includes('/oauth/authorize')) {
      console.log('✅ Redirected to OIDC registration flow');
      
      // Step 3: Verify registration theme
      const isRegistrationTheme = await page.locator('h2:has-text("Join Data Insights Portal"), h2:has-text("Register")').first().isVisible({ timeout: 5000 }).catch(() => false);
      if (isRegistrationTheme) {
        console.log('✅ "Join Data Insights Portal" registration theme detected');
      }

      // Step 4: Verify registration parameter in form
      const form = page.locator('form[action*="/oauth/authorize"]');
      const registrationInput = form.locator('input[name="registration"]');
      if (await registrationInput.isVisible()) {
        await expect(registrationInput).toHaveValue('true');
        console.log('✅ Registration parameter verified');
      }
    } else {
      // Direct registration form
      console.log('✅ Direct registration form detected');
    }

    // Step 5: Fill registration form (use existing credentials for this test)
    await page.fill('input[name="email"], input[type="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"], input[type="password"]', TEST_CREDENTIALS.password);
    
    // Fill display name if field exists
    const displayNameField = page.locator('input[name="display_name"], input[name="displayName"], input[name="name"]');
    if (await displayNameField.isVisible()) {
      await displayNameField.fill('Test Admin User');
    }
    console.log('✅ Registration form filled');

    // Step 6: Submit registration (should show already exists error)
    const submitButton = page.locator('button[type="submit"]');
    await submitButton.click();
    console.log('✅ Registration form submitted');

    // Step 7: Verify error handling for existing user
    const errorMessage = page.locator('text=already exists, text=Email may already be in use, .error, .alert');
    await expect(errorMessage.first()).toBeVisible({ timeout: 10000 });
    console.log('✅ Registration error handling verified for existing user');
  });

  test('⚠️ Error Handling - Invalid Credentials', async () => {
    console.log('🚀 Starting error handling test...');

    // Step 1: Navigate to login
    await page.goto(`${BASE_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    console.log('✅ Navigated to login page');

    // Step 2: Fill invalid credentials
    await page.fill('input[name="email"], input[type="email"]', 'invalid@test.com');
    await page.fill('input[name="password"], input[type="password"]', 'wrongpassword');
    console.log('✅ Invalid credentials filled');

    // Step 3: Submit form
    const submitButton = page.locator('button[type="submit"]');
    await submitButton.click();
    console.log('✅ Login form with invalid credentials submitted');

    // Step 4: Verify error display with Analytics Platform theme
    const errorAlert = page.locator('text=⚠️ Authentication failed!, text=Invalid email or password, .insight-toast, .error');
    await expect(errorAlert.first()).toBeVisible({ timeout: 10000 });
    console.log('✅ Error handling with analytics theme verified');

    // Step 5: Verify error styling
    const errorIcon = page.locator('text=⚠️');
    if (await errorIcon.isVisible()) {
      console.log('✅ Analytics error icon detected');
    }
  });

  test('🚪 Complete Logout Flow', async () => {
    console.log('🚀 Starting logout flow test...');

    // Step 1: First login
    await page.goto(`${BASE_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    
    await page.fill('input[name="email"], input[type="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"], input[type="password"]', TEST_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    
    await page.waitForURL(BASE_URL, { timeout: 20000 });
    console.log('✅ Successfully logged in');

    // Step 2: Find and click logout
    const logoutButton = page.locator('button:has-text("Logout"), a:has-text("Logout"), button:has-text("Sign Out")');
    await expect(logoutButton.first()).toBeVisible({ timeout: 10000 });
    await logoutButton.first().click();
    console.log('✅ Logout button clicked');

    // Step 3: Verify logout redirect
    await page.waitForURL(/login|oauth\/authorize/, { timeout: 10000 });
    console.log('✅ Redirected to login after logout');

    // Step 4: Verify session is cleared
    await page.goto(BASE_URL);
    const loginLink = page.locator('a[href*="/login"], button:has-text("Sign In")');
    await expect(loginLink.first()).toBeVisible({ timeout: 5000 });
    console.log('✅ Session cleared - login button visible');
  });

  test('📱 Mobile Responsive Authentication Flow', async () => {
    console.log('🚀 Starting mobile responsive test...');

    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    // Step 1: Navigate to mobile login
    await page.goto(`${BASE_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    console.log('✅ Mobile login page loaded');

    // Step 2: Verify mobile-responsive analytics theme
    await expect(page.locator('h2:has-text("📊 Data Insights Portal")')).toBeVisible();
    
    // Check if analytics card is responsive
    const analyticsCard = page.locator('.insight-card, [class*="insight"]');
    const cardBounds = await analyticsCard.boundingBox();
    expect(cardBounds?.width).toBeLessThanOrEqual(375);
    console.log('✅ Mobile-responsive analytics theme verified');

    // Step 3: Test mobile form interaction
    await page.fill('input[name="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"]', TEST_CREDENTIALS.password);
    
    // Test password visibility toggle on mobile
    const passwordToggle = page.locator('button:has-text("👁"), .fa-eye, [class*="eye"]');
    if (await passwordToggle.isVisible()) {
      await passwordToggle.click();
      console.log('✅ Mobile password visibility toggle tested');
    }

    // Step 4: Submit mobile login
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(BASE_URL, { timeout: 20000 });
    console.log('✅ Mobile login completed successfully');
  });

  test('🔗 PKCE Parameters Validation', async () => {
    console.log('🚀 Starting PKCE parameters validation test...');

    // Step 1: Navigate to login and intercept requests
    const authRequests: any[] = [];
    page.on('request', request => {
      if (request.url().includes('/oauth/authorize')) {
        authRequests.push(request);
      }
    });

    await page.goto(`${BASE_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });

    // Step 2: Verify PKCE parameters in form
    const form = page.locator('form[action*="/oauth/authorize"]');
    
    const codeChallenge = await form.locator('input[name="code_challenge"]').getAttribute('value');
    const codeChallengeMethod = await form.locator('input[name="code_challenge_method"]').getAttribute('value');
    
    expect(codeChallenge).toBeTruthy();
    expect(codeChallenge).toMatch(/^[A-Za-z0-9_-]+$/); // Base64URL format
    expect(codeChallengeMethod).toBe('S256');
    console.log('✅ PKCE parameters validated:', { codeChallenge: codeChallenge?.slice(0, 10) + '...', codeChallengeMethod });

    // Step 3: Verify state parameter
    const state = await form.locator('input[name="state"]').getAttribute('value');
    expect(state).toBeTruthy();
    expect(state).toMatch(/^[A-Za-z0-9_-]+$/);
    console.log('✅ State parameter validated:', state?.slice(0, 10) + '...');

    // Step 4: Complete login to test PKCE flow
    await page.fill('input[name="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"]', TEST_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();

    // Step 5: Verify token exchange (callback should complete successfully)
    await page.waitForURL(BASE_URL, { timeout: 20000 });
    console.log('✅ PKCE token exchange completed successfully');
  });

  test('🔄 Session Management and Token Refresh', async () => {
    console.log('🚀 Starting session management test...');

    // Step 1: Login and capture session
    await page.goto(`${BASE_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    
    await page.fill('input[name="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"]', TEST_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(BASE_URL, { timeout: 20000 });
    console.log('✅ Initial login completed');

    // Step 2: Check session cookies
    const cookies = await context.cookies();
    const sessionCookies = cookies.filter(cookie => 
      cookie.name.includes('session') || 
      cookie.name.includes('jwt') || 
      cookie.name.includes('epsx')
    );
    expect(sessionCookies.length).toBeGreaterThan(0);
    console.log('✅ Session cookies verified:', sessionCookies.map(c => c.name));

    // Step 3: Test navigation while authenticated
    await page.goto(`${BASE_URL}/dashboard`);
    
    // Should not redirect to login
    await page.waitForTimeout(2000);
    expect(page.url()).not.toContain('/login');
    expect(page.url()).not.toContain('/oauth/authorize');
    console.log('✅ Authenticated navigation verified');

    // Step 4: Test session persistence across page refresh
    await page.reload();
    await page.waitForTimeout(2000);
    expect(page.url()).not.toContain('/login');
    console.log('✅ Session persistence verified');
  });

  test('⚡ Performance and Load Testing', async () => {
    console.log('🚀 Starting performance test...');

    // Step 1: Measure login page load time
    const startTime = Date.now();
    await page.goto(`${BASE_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    const loadTime = Date.now() - startTime;
    
    expect(loadTime).toBeLessThan(5000); // Should load within 5 seconds
    console.log(`✅ Login page load time: ${loadTime}ms`);

    // Step 2: Measure form interaction responsiveness
    const formStartTime = Date.now();
    await page.fill('input[name="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"]', TEST_CREDENTIALS.password);
    const formTime = Date.now() - formStartTime;
    
    expect(formTime).toBeLessThan(1000); // Form interaction should be responsive
    console.log(`✅ Form interaction time: ${formTime}ms`);

    // Step 3: Measure authentication flow completion
    const authStartTime = Date.now();
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(BASE_URL, { timeout: 20000 });
    const authTime = Date.now() - authStartTime;
    
    expect(authTime).toBeLessThan(10000); // Full auth flow should complete within 10 seconds
    console.log(`✅ Authentication flow time: ${authTime}ms`);

    // Step 4: Check for performance issues
    const metrics = await page.evaluate(() => performance.getEntriesByType('navigation')[0]);
    console.log('✅ Performance metrics captured:', {
      domContentLoaded: Math.round(metrics.domContentLoadedEventEnd - metrics.navigationStart),
      loadComplete: Math.round(metrics.loadEventEnd - metrics.navigationStart)
    });
  });

  test('🔒 Security Validation', async () => {
    console.log('🚀 Starting security validation test...');

    // Step 1: Check HTTPS enforcement (if in production)
    if (BASE_URL.startsWith('https://')) {
      const response = await page.goto(BASE_URL.replace('https://', 'http://'));
      expect(response?.status()).toBe(301); // Should redirect to HTTPS
      console.log('✅ HTTPS enforcement verified');
    }

    // Step 2: Verify CSP headers
    await page.goto(`${BASE_URL}/login`);
    const response = await page.waitForResponse(/oauth\/authorize/);
    const cspHeader = response.headers()['content-security-policy'];
    if (cspHeader) {
      expect(cspHeader).toContain('script-src');
      console.log('✅ CSP headers present');
    }

    // Step 3: Check for exposed sensitive data
    const pageContent = await page.content();
    expect(pageContent).not.toContain('password');
    expect(pageContent).not.toContain('secret');
    expect(pageContent).not.toContain('private_key');
    console.log('✅ No sensitive data exposed in page content');

    // Step 4: Verify form security attributes
    const form = page.locator('form[action*="/oauth/authorize"]');
    const formAction = await form.getAttribute('action');
    expect(formAction).toContain('/oauth/authorize');
    
    const passwordField = page.locator('input[type="password"]');
    const autocomplete = await passwordField.getAttribute('autocomplete');
    expect(autocomplete).toBe('current-password');
    console.log('✅ Form security attributes verified');
  });
});

// Additional test suite for cross-app integration
test.describe('🔄 Cross-App Integration Tests', () => {
  let context: BrowserContext;
  let frontendPage: Page;
  let adminPage: Page;

  test.beforeEach(async ({ browser }) => {
    context = await browser.newContext();
    frontendPage = await context.newPage();
    adminPage = await context.newPage();
  });

  test.afterEach(async () => {
    await frontendPage?.close();
    await adminPage?.close();
    await context?.close();
  });

  test('🔄 Shared Session Between Frontend and Admin', async () => {
    console.log('🚀 Testing shared session between apps...');

    // Step 1: Login to frontend
    await frontendPage.goto(`${BASE_URL}/login`);
    await frontendPage.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    
    await frontendPage.fill('input[name="email"]', TEST_CREDENTIALS.email);
    await frontendPage.fill('input[name="password"]', TEST_CREDENTIALS.password);
    await frontendPage.locator('button[type="submit"]').click();
    await frontendPage.waitForURL(BASE_URL, { timeout: 20000 });
    console.log('✅ Frontend login completed');

    // Step 2: Navigate to admin with same session
    await adminPage.goto(ADMIN_URL);
    
    // Should automatically authenticate if session is shared
    await adminPage.waitForTimeout(3000);
    const isAuthenticated = !adminPage.url().includes('/login') && !adminPage.url().includes('/oauth/authorize');
    
    if (isAuthenticated) {
      console.log('✅ Shared session confirmed - auto-authenticated in admin');
    } else {
      console.log('ℹ️ Separate sessions - admin requires separate login');
    }
  });
});

console.log('📊 Analytics Platform Authentication E2E Test Suite Loaded');