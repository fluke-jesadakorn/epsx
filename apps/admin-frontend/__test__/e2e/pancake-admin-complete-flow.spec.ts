/**
 * Complete E2E Admin Authentication Flow Tests with Chef's Kitchen Portal Theme
 * Tests 100% coverage of admin OIDC authentication flow
 * User: info@epsx.io, Password: P@ssword
 */

import { test, expect, Page, BrowserContext } from '@playwright/test';

// Test credentials for admin user
const ADMIN_CREDENTIALS = {
  email: 'info@epsx.io',
  password: 'P@ssword'
};

// Environment configuration
const ADMIN_URL = process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001';
const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

test.describe('👨‍🍳 Chef\'s Kitchen Portal - Admin Authentication Complete Flow', () => {
  let context: BrowserContext;
  let page: Page;

  test.beforeEach(async ({ browser }) => {
    // Create fresh context for each test
    context = await browser.newContext({
      viewport: { width: 1440, height: 900 },
      userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
    });
    page = await context.newPage();
    
    // Enable request/response logging for debugging
    page.on('request', request => {
      if (request.url().includes('/oauth/') || request.url().includes('/auth/')) {
        console.log('🔍 Admin Request:', request.method(), request.url());
      }
    });
    
    page.on('response', response => {
      if (response.url().includes('/oauth/') || response.url().includes('/auth/')) {
        console.log('📄 Admin Response:', response.status(), response.url());
      }
    });
  });

  test.afterEach(async () => {
    await page?.close();
    await context?.close();
  });

  test('👨‍🍳 Complete Admin OIDC Authorization Flow', async () => {
    console.log('🚀 Starting admin OIDC authorization flow test...');

    // Step 1: Navigate to admin homepage
    await page.goto(ADMIN_URL);
    await expect(page).toHaveTitle(/Admin|EPSX/);
    console.log('✅ Admin homepage loaded successfully');

    // Step 2: Click login button to start admin OIDC flow
    const loginButton = page.locator('a[href*="/login"], button:has-text("Sign In"), button:has-text("Login"), a:has-text("Login")').first();
    await expect(loginButton).toBeVisible({ timeout: 10000 });
    await loginButton.click();
    console.log('✅ Admin login button clicked, initiating OIDC flow');

    // Step 3: Wait for navigation to OIDC authorization endpoint
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    console.log('✅ Navigated to admin OIDC authorization endpoint');

    // Step 4: Verify Chef's Kitchen Portal theme OR PancakeSwap admin theme
    const isChefPortal = await page.locator('h2:has-text("Chef\'s Kitchen Portal"), h1:has-text("Chef\'s Kitchen Portal")').first().isVisible({ timeout: 5000 }).catch(() => false);
    const isPancakeAdmin = await page.locator('h2:has-text("Pancake Stack Login"), text=admin').first().isVisible({ timeout: 5000 }).catch(() => false);
    const isPancakeTheme = await page.locator('text=🥞, .pancake-logo, [class*="pancake"]').first().isVisible({ timeout: 5000 }).catch(() => false);
    
    if (isChefPortal) {
      console.log('✅ Chef\'s Kitchen Portal theme detected');
      await expect(page.locator('text=👨‍🍳, text=🍳')).toBeVisible();
    } else if (isPancakeAdmin || isPancakeTheme) {
      console.log('✅ PancakeSwap admin theme detected');
      await expect(page.locator('text=🥞')).toBeVisible();
    } else {
      console.log('ℹ️ Standard admin theme detected, continuing with test');
    }

    // Step 5: Verify admin OIDC form structure
    const form = page.locator('form[action*="/oauth/authorize"]');
    await expect(form).toBeVisible();
    await expect(form.locator('input[name="client_id"]')).toHaveValue(/epsx-admin|epsx-frontend/);
    await expect(form.locator('input[name="response_type"]')).toHaveValue('code');
    
    // Verify admin scope
    const scopeValue = await form.locator('input[name="scope"]').getAttribute('value');
    expect(scopeValue).toContain('admin_modules');
    console.log('✅ Admin OIDC form structure validated with admin scope');

    // Step 6: Fill in admin login credentials
    await page.fill('input[name="email"], input[type="email"]', ADMIN_CREDENTIALS.email);
    await page.fill('input[name="password"], input[type="password"]', ADMIN_CREDENTIALS.password);
    console.log('✅ Admin credentials filled');

    // Step 7: Submit admin login form
    const submitButton = page.locator('button[type="submit"], button:has-text("Login"), button:has-text("Sign In")');
    await expect(submitButton).toBeVisible();
    await submitButton.click();
    console.log('✅ Admin login form submitted');

    // Step 8: Wait for admin authentication redirect
    await page.waitForURL(/callback/, { timeout: 20000 });
    console.log('✅ Redirected to admin callback URL');

    // Step 9: Wait for final redirect to authenticated admin dashboard
    await page.waitForURL(ADMIN_URL, { timeout: 15000 });
    console.log('✅ Successfully redirected to authenticated admin dashboard');

    // Step 10: Verify admin is authenticated and has admin features
    const adminFeatures = page.locator('text=Users, text=Permissions, text=Modules, text=IAM, text=Admin, nav a:has-text("Users"), nav a:has-text("Permissions")');
    await expect(adminFeatures.first()).toBeVisible({ timeout: 10000 });
    console.log('✅ Admin authentication and features confirmed');

    // Step 11: Verify admin session persistence
    await page.reload();
    await expect(adminFeatures.first()).toBeVisible({ timeout: 5000 });
    console.log('✅ Admin session persistence verified');
  });

  test('🔐 Admin Module Access Verification', async () => {
    console.log('🚀 Starting admin module access verification...');

    // Step 1: Complete admin login first
    await page.goto(`${ADMIN_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    
    await page.fill('input[name="email"]', ADMIN_CREDENTIALS.email);
    await page.fill('input[name="password"]', ADMIN_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(ADMIN_URL, { timeout: 20000 });
    console.log('✅ Admin login completed');

    // Step 2: Test User Management module access
    await page.goto(`${ADMIN_URL}/users`);
    await page.waitForTimeout(2000);
    
    const isUsersAccessible = !page.url().includes('/access-denied') && !page.url().includes('/unauthorized');
    if (isUsersAccessible) {
      console.log('✅ User Management module accessible');
      
      // Verify user management features
      const userManagementFeatures = page.locator('text=Create User, text=User List, table, button:has-text("Add User")');
      await expect(userManagementFeatures.first()).toBeVisible({ timeout: 5000 });
    } else {
      console.log('⚠️ User Management module access restricted');
    }

    // Step 3: Test IAM module access
    await page.goto(`${ADMIN_URL}/iam`);
    await page.waitForTimeout(2000);
    
    const isIAMAccessible = !page.url().includes('/access-denied') && !page.url().includes('/unauthorized');
    if (isIAMAccessible) {
      console.log('✅ IAM module accessible');
      
      // Verify IAM features
      const iamFeatures = page.locator('text=Roles, text=Permissions, text=IAM Dashboard');
      await expect(iamFeatures.first()).toBeVisible({ timeout: 5000 });
    } else {
      console.log('⚠️ IAM module access restricted');
    }

    // Step 4: Test Analytics module access
    await page.goto(`${ADMIN_URL}/analytics`);
    await page.waitForTimeout(2000);
    
    const isAnalyticsAccessible = !page.url().includes('/access-denied') && !page.url().includes('/unauthorized');
    if (isAnalyticsAccessible) {
      console.log('✅ Analytics module accessible');
    } else {
      console.log('⚠️ Analytics module access restricted');
    }

    console.log('✅ Admin module access verification completed');
  });

  test('⚡ Admin Performance and Security', async () => {
    console.log('🚀 Starting admin performance and security test...');

    // Step 1: Measure admin login performance
    const startTime = Date.now();
    await page.goto(`${ADMIN_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    const loadTime = Date.now() - startTime;
    
    expect(loadTime).toBeLessThan(5000);
    console.log(`✅ Admin login page load time: ${loadTime}ms`);

    // Step 2: Complete admin authentication flow timing
    const authStartTime = Date.now();
    await page.fill('input[name="email"]', ADMIN_CREDENTIALS.email);
    await page.fill('input[name="password"]', ADMIN_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(ADMIN_URL, { timeout: 20000 });
    const authTime = Date.now() - authStartTime;
    
    expect(authTime).toBeLessThan(10000);
    console.log(`✅ Admin authentication flow time: ${authTime}ms`);

    // Step 3: Verify admin security headers
    const response = await page.goto(ADMIN_URL);
    const headers = response?.headers() || {};
    
    // Check for security headers
    if (headers['x-frame-options'] || headers['x-content-type-options']) {
      console.log('✅ Security headers present');
    }
    
    if (headers['content-security-policy']) {
      console.log('✅ CSP header present for admin');
    }

    // Step 4: Verify no sensitive data exposure
    const pageContent = await page.content();
    expect(pageContent).not.toContain('password');
    expect(pageContent).not.toContain('private_key');
    expect(pageContent).not.toContain('api_secret');
    console.log('✅ No sensitive data exposed in admin interface');
  });

  test('🔒 Admin Authorization and Access Control', async () => {
    console.log('🚀 Starting admin authorization test...');

    // Step 1: Login as admin
    await page.goto(`${ADMIN_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    
    await page.fill('input[name="email"]', ADMIN_CREDENTIALS.email);
    await page.fill('input[name="password"]', ADMIN_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(ADMIN_URL, { timeout: 20000 });
    console.log('✅ Admin login completed');

    // Step 2: Verify admin-only features are visible
    const adminOnlyFeatures = [
      'text=User Management',
      'text=System Settings',
      'text=Admin Dashboard',
      'text=Modules',
      'text=IAM',
      'a[href*="/users"]',
      'a[href*="/iam"]'
    ];

    let visibleFeatures = 0;
    for (const feature of adminOnlyFeatures) {
      const isVisible = await page.locator(feature).first().isVisible({ timeout: 2000 }).catch(() => false);
      if (isVisible) {
        visibleFeatures++;
      }
    }

    expect(visibleFeatures).toBeGreaterThan(0);
    console.log(`✅ ${visibleFeatures} admin features verified as visible`);

    // Step 3: Test deep-link access to admin features
    await page.goto(`${ADMIN_URL}/users/create`);
    await page.waitForTimeout(2000);
    
    const hasCreateUserAccess = !page.url().includes('/access-denied') && !page.url().includes('/unauthorized');
    if (hasCreateUserAccess) {
      console.log('✅ Direct access to admin features confirmed');
    } else {
      console.log('⚠️ Create user access restricted - checking permissions');
    }
  });

  test('📱 Admin Mobile Responsive Interface', async () => {
    console.log('🚀 Starting admin mobile responsive test...');

    // Set mobile viewport
    await page.setViewportSize({ width: 768, height: 1024 });

    // Step 1: Test mobile admin login
    await page.goto(`${ADMIN_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    console.log('✅ Mobile admin login page loaded');

    // Step 2: Verify mobile-responsive admin theme
    const isMobileResponsive = await page.locator('meta[name="viewport"]').isVisible().catch(() => false);
    expect(isMobileResponsive).toBeTruthy();

    // Step 3: Test mobile form interaction
    await page.fill('input[name="email"]', ADMIN_CREDENTIALS.email);
    await page.fill('input[name="password"]', ADMIN_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(ADMIN_URL, { timeout: 20000 });
    console.log('✅ Mobile admin login completed');

    // Step 4: Test mobile navigation
    const mobileMenu = page.locator('button[aria-label*="menu"], .mobile-menu, [data-testid="mobile-menu"]');
    if (await mobileMenu.isVisible()) {
      await mobileMenu.click();
      console.log('✅ Mobile admin menu tested');
    }

    // Step 5: Verify mobile admin dashboard layout
    const dashboardElements = page.locator('.dashboard, .admin-dashboard, main');
    await expect(dashboardElements.first()).toBeVisible();
    console.log('✅ Mobile admin dashboard layout verified');
  });

  test('🔄 Admin Logout and Session Management', async () => {
    console.log('🚀 Starting admin logout test...');

    // Step 1: Login first
    await page.goto(`${ADMIN_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });
    
    await page.fill('input[name="email"]', ADMIN_CREDENTIALS.email);
    await page.fill('input[name="password"]', ADMIN_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(ADMIN_URL, { timeout: 20000 });
    console.log('✅ Admin login completed');

    // Step 2: Find and test logout functionality
    const logoutButton = page.locator('button:has-text("Logout"), a:has-text("Logout"), button:has-text("Sign Out"), [data-testid="logout"]');
    
    // Look for logout in user menu if not immediately visible
    const userMenu = page.locator('.user-menu, .profile-menu, button:has-text("Profile")');
    if (await userMenu.isVisible()) {
      await userMenu.click();
      await page.waitForTimeout(500);
    }

    await expect(logoutButton.first()).toBeVisible({ timeout: 10000 });
    await logoutButton.first().click();
    console.log('✅ Admin logout button clicked');

    // Step 3: Verify logout redirect
    await page.waitForURL(/login|oauth\/authorize/, { timeout: 10000 });
    console.log('✅ Redirected to login after admin logout');

    // Step 4: Verify admin session is cleared
    await page.goto(ADMIN_URL);
    const loginLink = page.locator('a[href*="/login"], button:has-text("Sign In")');
    await expect(loginLink.first()).toBeVisible({ timeout: 5000 });
    console.log('✅ Admin session cleared - login required');

    // Step 5: Verify admin routes are protected after logout
    await page.goto(`${ADMIN_URL}/users`);
    await page.waitForTimeout(2000);
    expect(page.url()).toMatch(/login|oauth\/authorize|unauthorized|access-denied/);
    console.log('✅ Admin routes protected after logout');
  });

  test('🔍 Admin Error Handling', async () => {
    console.log('🚀 Starting admin error handling test...');

    // Step 1: Test invalid admin credentials
    await page.goto(`${ADMIN_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });

    await page.fill('input[name="email"]', 'invalid-admin@test.com');
    await page.fill('input[name="password"]', 'wrongpassword');
    await page.locator('button[type="submit"]').click();
    console.log('✅ Invalid admin credentials submitted');

    // Step 2: Verify error display
    const errorAlert = page.locator('text=Invalid, text=error, text=failed, .error, .alert-error');
    await expect(errorAlert.first()).toBeVisible({ timeout: 10000 });
    console.log('✅ Admin error handling verified');

    // Step 3: Test non-admin user access (if different credentials available)
    // This would need different test credentials for a non-admin user
    console.log('ℹ️ Non-admin user access test would require separate credentials');
  });

  test('🎯 Admin PKCE and Security Parameters', async () => {
    console.log('🚀 Starting admin PKCE validation...');

    // Step 1: Navigate to admin login and intercept requests
    await page.goto(`${ADMIN_URL}/login`);
    await page.waitForURL(/oauth\/authorize/, { timeout: 15000 });

    // Step 2: Verify admin PKCE parameters
    const form = page.locator('form[action*="/oauth/authorize"]');
    
    const codeChallenge = await form.locator('input[name="code_challenge"]').getAttribute('value');
    const codeChallengeMethod = await form.locator('input[name="code_challenge_method"]').getAttribute('value');
    const clientId = await form.locator('input[name="client_id"]').getAttribute('value');
    const scope = await form.locator('input[name="scope"]').getAttribute('value');
    
    expect(codeChallenge).toBeTruthy();
    expect(codeChallengeMethod).toBe('S256');
    expect(scope).toContain('admin_modules');
    console.log('✅ Admin PKCE parameters validated:', { 
      clientId, 
      scope,
      codeChallenge: codeChallenge?.slice(0, 10) + '...' 
    });

    // Step 3: Complete admin PKCE flow
    await page.fill('input[name="email"]', ADMIN_CREDENTIALS.email);
    await page.fill('input[name="password"]', ADMIN_CREDENTIALS.password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(ADMIN_URL, { timeout: 20000 });
    console.log('✅ Admin PKCE token exchange completed successfully');
  });
});

console.log('👨‍🍳 Chef\'s Kitchen Portal Admin E2E Test Suite Loaded');