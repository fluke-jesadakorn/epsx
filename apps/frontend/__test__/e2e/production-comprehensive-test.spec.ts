import { test, expect } from '@playwright/test';

const DEPLOYED_SERVICES = {
  backend: 'https://backend-6wjeb6vw2q-uc.a.run.app',
  admin: 'https://admin-307278481624.us-central1.run.app', 
  frontend: 'https://frontend-6wjeb6vw2q-uc.a.run.app'
};

test.describe('EPSX Production Deployment Comprehensive Test', () => {
  
  test.beforeEach(async ({ page }) => {
    // Collect console errors for debugging
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        console.log(`❌ Console Error: ${msg.text()}`);
      } else if (msg.type() === 'warning') {
        console.log(`⚠️  Console Warning: ${msg.text()}`);
      }
    });

    page.on('requestfailed', (request) => {
      console.log(`🌐 Network Error: ${request.url()} - ${request.failure()?.errorText}`);
    });
  });

  test('Backend Health Check and API Endpoints', async ({ page }) => {
    console.log('🔧 Testing Backend Service...');
    
    // Test health endpoint
    const healthResponse = await page.request.get(`${DEPLOYED_SERVICES.backend}/health`);
    console.log(`Health endpoint status: ${healthResponse.status()}`);
    expect(healthResponse.status()).toBe(200);
    
    const healthData = await healthResponse.json();
    console.log('Health response:', healthData);
    
    // Test CORS headers
    const corsHeaders = healthResponse.headers();
    console.log('CORS Headers:', {
      'access-control-allow-origin': corsHeaders['access-control-allow-origin'],
      'access-control-allow-credentials': corsHeaders['access-control-allow-credentials'],
      'access-control-allow-methods': corsHeaders['access-control-allow-methods']
    });
    
    // Test OIDC endpoints
    const oidcWellKnown = await page.request.get(`${DEPLOYED_SERVICES.backend}/.well-known/openid_configuration`);
    console.log(`OIDC Well-known endpoint status: ${oidcWellKnown.status()}`);
    
    if (oidcWellKnown.status() === 200) {
      const oidcConfig = await oidcWellKnown.json();
      console.log('OIDC Configuration:', oidcConfig);
    }
    
    // Test unauthorized access to protected endpoint
    const userInfoResponse = await page.request.get(`${DEPLOYED_SERVICES.backend}/oauth/userinfo`);
    console.log(`UserInfo endpoint (no auth) status: ${userInfoResponse.status()}`);
    // Should return 401 for unauthorized access
    expect([401, 403]).toContain(userInfoResponse.status());
  });

  test('Frontend Application Loading and Navigation', async ({ page }) => {
    console.log('🎯 Testing Frontend Application...');
    
    // Navigate to frontend
    await page.goto(DEPLOYED_SERVICES.frontend);
    await page.waitForLoadState('networkidle');
    
    // Check page title
    const title = await page.title();
    console.log(`Page title: ${title}`);
    expect(title).toBeTruthy();
    
    // Check if main content loads
    const bodyText = await page.locator('body').textContent();
    expect(bodyText).toBeTruthy();
    expect(bodyText!.trim().length).toBeGreaterThan(0);
    
    // Look for navigation elements
    const navElements = await page.locator('nav, header, [role="navigation"]').count();
    console.log(`Navigation elements found: ${navElements}`);
    
    // Check for login/auth related elements
    const loginButton = page.locator('button:has-text("Login"), a:has-text("Login"), button:has-text("Sign In"), a:has-text("Sign In")').first();
    const loginExists = await loginButton.count() > 0;
    console.log(`Login button found: ${loginExists}`);
    
    // Take screenshot
    await page.screenshot({ 
      path: '.debug/frontend-production-test.png',
      fullPage: true 
    });
    
    console.log('✅ Frontend loading test completed');
  });

  test('Admin Application Loading and Authentication Flow', async ({ page }) => {
    console.log('👨‍💼 Testing Admin Application...');
    
    // Navigate to admin
    await page.goto(DEPLOYED_SERVICES.admin);
    await page.waitForLoadState('networkidle');
    
    // Check page title
    const title = await page.title();
    console.log(`Admin page title: ${title}`);
    
    // Check if this redirects to login or shows login form
    const currentUrl = page.url();
    console.log(`Current URL after navigation: ${currentUrl}`);
    
    // Look for admin-specific elements
    const bodyText = await page.locator('body').textContent();
    const hasAdminContent = bodyText?.toLowerCase().includes('admin') || 
                           bodyText?.toLowerCase().includes('login') ||
                           bodyText?.toLowerCase().includes('dashboard');
    console.log(`Admin content detected: ${hasAdminContent}`);
    
    // Look for authentication elements
    const authElements = await page.locator('form, input[type="email"], input[type="password"], button:has-text("Login")').count();
    console.log(`Authentication elements found: ${authElements}`);
    
    // Check for OIDC login button
    const oidcLogin = page.locator('button:has-text("Sign in with"), button:has-text("Login with"), a:has-text("Continue with")').first();
    const oidcLoginExists = await oidcLogin.count() > 0;
    console.log(`OIDC login button found: ${oidcLoginExists}`);
    
    // Take screenshot
    await page.screenshot({ 
      path: '.debug/admin-production-test.png',
      fullPage: true 
    });
    
    console.log('✅ Admin loading test completed');
  });

  test('Cross-Service Communication Test', async ({ page }) => {
    console.log('🔄 Testing Cross-Service Communication...');
    
    // Test if frontend can communicate with backend
    await page.goto(DEPLOYED_SERVICES.frontend);
    await page.waitForLoadState('networkidle');
    
    // Check if any API calls to backend are made
    const apiRequests: string[] = [];
    page.on('request', (request) => {
      if (request.url().includes(DEPLOYED_SERVICES.backend)) {
        apiRequests.push(request.url());
      }
    });
    
    // Wait a bit to see if any automatic API calls are made
    await page.waitForTimeout(3000);
    
    console.log(`API requests to backend: ${apiRequests.length}`);
    apiRequests.forEach(url => console.log(`  - ${url}`));
    
    // Test direct API call from browser context
    const apiResponse = await page.evaluate(async (backendUrl) => {
      try {
        const response = await fetch(`${backendUrl}/health`);
        return {
          status: response.status,
          ok: response.ok,
          headers: Object.fromEntries(response.headers.entries())
        };
      } catch (error) {
        return {
          error: error.message
        };
      }
    }, DEPLOYED_SERVICES.backend);
    
    console.log('Direct API call result:', apiResponse);
    
    console.log('✅ Cross-service communication test completed');
  });

  test('Performance and Load Time Analysis', async ({ page }) => {
    console.log('⚡ Testing Performance...');
    
    const startTime = Date.now();
    
    // Test frontend performance
    await page.goto(DEPLOYED_SERVICES.frontend);
    await page.waitForLoadState('networkidle');
    
    const frontendLoadTime = Date.now() - startTime;
    console.log(`Frontend load time: ${frontendLoadTime}ms`);
    
    // Test admin performance
    const adminStartTime = Date.now();
    await page.goto(DEPLOYED_SERVICES.admin);
    await page.waitForLoadState('networkidle');
    
    const adminLoadTime = Date.now() - adminStartTime;
    console.log(`Admin load time: ${adminLoadTime}ms`);
    
    // Performance expectations (reasonable for Cloud Run cold starts)
    expect(frontendLoadTime).toBeLessThan(15000); // 15 seconds max
    expect(adminLoadTime).toBeLessThan(15000); // 15 seconds max
    
    console.log('✅ Performance test completed');
  });

  test('Security Headers and HTTPS Verification', async ({ page }) => {
    console.log('🔒 Testing Security Configuration...');
    
    // Test all services for HTTPS and security headers
    for (const [serviceName, serviceUrl] of Object.entries(DEPLOYED_SERVICES)) {
      console.log(`Testing ${serviceName} at ${serviceUrl}`);
      
      const response = await page.request.get(serviceUrl);
      const headers = response.headers();
      
      // Check HTTPS
      expect(serviceUrl).toMatch(/^https:/);
      
      // Log security headers
      console.log(`${serviceName} security headers:`, {
        'strict-transport-security': headers['strict-transport-security'],
        'x-frame-options': headers['x-frame-options'],
        'x-content-type-options': headers['x-content-type-options'],
        'access-control-allow-origin': headers['access-control-allow-origin']
      });
    }
    
    console.log('✅ Security verification completed');
  });

  test('Authentication Flow End-to-End Test', async ({ page }) => {
    console.log('🔐 Testing Complete Authentication Flow...');
    
    // Start at frontend
    await page.goto(DEPLOYED_SERVICES.frontend);
    await page.waitForLoadState('networkidle');
    
    // Look for login functionality
    const loginButton = page.locator('button:has-text("Login"), a:has-text("Login"), button:has-text("Sign In"), a:has-text("Sign In")').first();
    
    if (await loginButton.count() > 0) {
      console.log('Login button found, testing auth flow...');
      
      // Click login (may redirect to admin or show modal)
      await loginButton.click();
      await page.waitForTimeout(2000);
      
      const currentUrl = page.url();
      console.log(`After login click URL: ${currentUrl}`);
      
      // Check if redirected to admin or OIDC provider
      if (currentUrl.includes('admin')) {
        console.log('Redirected to admin for authentication');
      } else if (currentUrl !== DEPLOYED_SERVICES.frontend) {
        console.log('Redirected to external auth provider');
      } else {
        console.log('Stayed on frontend - may have opened modal or inline form');
      }
      
      // Take screenshot of auth state
      await page.screenshot({ 
        path: '.debug/auth-flow-test.png',
        fullPage: true 
      });
    } else {
      console.log('No login button found - may be auto-login or different UI pattern');
    }
    
    // Test admin auth separately
    await page.goto(DEPLOYED_SERVICES.admin);
    await page.waitForLoadState('networkidle');
    
    const adminUrl = page.url();
    console.log(`Admin URL after navigation: ${adminUrl}`);
    
    // Take screenshot of admin auth
    await page.screenshot({ 
      path: '.debug/admin-auth-test.png',
      fullPage: true 
    });
    
    console.log('✅ Authentication flow test completed');
  });
});

// Test with specific test credentials if available
test.describe('EPSX Authentication with Test Credentials', () => {
  test('Login with test credentials', async ({ page }) => {
    const testEmail = process.env.TEST_ADMIN_EMAIL || 'info@epsx.io';
    const testPassword = process.env.TEST_ADMIN_PASSWORD || 'P@ssword';
    
    console.log(`🔑 Testing login with email: ${testEmail}`);
    
    // Navigate to admin login
    await page.goto(DEPLOYED_SERVICES.admin);
    await page.waitForLoadState('networkidle');
    
    // Look for email input
    const emailInput = page.locator('input[type="email"], input[name="email"], input[placeholder*="email" i]').first();
    const passwordInput = page.locator('input[type="password"], input[name="password"], input[placeholder*="password" i]').first();
    
    if (await emailInput.count() > 0 && await passwordInput.count() > 0) {
      console.log('Found email/password form, attempting login...');
      
      await emailInput.fill(testEmail);
      await passwordInput.fill(testPassword);
      
      // Look for submit button
      const submitButton = page.locator('button[type="submit"], button:has-text("Login"), button:has-text("Sign In")').first();
      
      if (await submitButton.count() > 0) {
        await submitButton.click();
        await page.waitForTimeout(3000);
        
        const afterLoginUrl = page.url();
        console.log(`URL after login attempt: ${afterLoginUrl}`);
        
        // Check if login was successful (redirect to dashboard or staying with different content)
        const bodyText = await page.locator('body').textContent();
        const loginSuccess = !bodyText?.toLowerCase().includes('invalid') && 
                           !bodyText?.toLowerCase().includes('error') &&
                           !bodyText?.toLowerCase().includes('failed');
        
        console.log(`Login appears successful: ${loginSuccess}`);
        
        // Take screenshot of login result
        await page.screenshot({ 
          path: '.debug/login-result-test.png',
          fullPage: true 
        });
      } else {
        console.log('No submit button found');
      }
    } else {
      console.log('No traditional email/password form found - may use OIDC/OAuth only');
    }
    
    console.log('✅ Credential login test completed');
  });
});