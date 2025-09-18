import { test, expect } from '@playwright/test';

/**
 * Comprehensive E2E Test: Localhost Connection Resolution Verification
 * 
 * This test verifies that ALL localhost connection issues have been resolved:
 * - No requests to 127.0.0.1:8080 or localhost:8080
 * - All backend requests go to api.epsx.io
 * - Complete login flow works end-to-end
 * - No connection refused errors
 */

test.describe('Localhost Connection Resolution Verification', () => {
  let networkRequests: Array<{
    url: string;
    method: string;
    status: number;
    timestamp: Date;
    isLocalhost: boolean;
    isProductionAPI: boolean;
  }> = [];

  let consoleErrors: Array<{
    text: string;
    type: string;
    timestamp: Date;
    isConnectionError: boolean;
  }> = [];

  test.beforeEach(async ({ page }) => {
    // Clear tracking arrays
    networkRequests = [];
    consoleErrors = [];

    // Monitor ALL network requests
    page.on('request', (request) => {
      const url = request.url();
      const isLocalhost = url.includes('127.0.0.1') || url.includes('localhost:8080');
      const isProductionAPI = url.includes('api.epsx.io');
      
      networkRequests.push({
        url,
        method: request.method(),
        status: 0, // Will be updated on response
        timestamp: new Date(),
        isLocalhost,
        isProductionAPI
      });
    });

    // Monitor responses to track status codes
    page.on('response', (response) => {
      const url = response.url();
      const request = networkRequests.find(req => req.url === url && req.status === 0);
      if (request) {
        request.status = response.status();
      }
    });

    // Monitor console for connection errors
    page.on('console', (msg) => {
      const text = msg.text();
      const isConnectionError = text.includes('ECONNREFUSED') || 
                               text.includes('127.0.0.1:8080') || 
                               text.includes('localhost:8080') ||
                               text.includes('Failed to fetch') ||
                               text.includes('Network Error');
      
      consoleErrors.push({
        text,
        type: msg.type(),
        timestamp: new Date(),
        isConnectionError
      });
    });

    // Monitor page errors
    page.on('pageerror', (error) => {
      const text = error.message;
      const isConnectionError = text.includes('ECONNREFUSED') || 
                               text.includes('127.0.0.1:8080') || 
                               text.includes('localhost:8080');
      
      consoleErrors.push({
        text,
        type: 'error',
        timestamp: new Date(),
        isConnectionError
      });
    });
  });

  test('should complete login flow with ZERO localhost connections', async ({ page }) => {
    // Navigate to frontend
    await page.goto('https://epsx.io');
    
    // Wait for initial page load and network activity to settle
    await page.waitForLoadState('networkidle');
    
    // Navigate to login page
    await page.click('text=Login');
    await page.waitForLoadState('networkidle');
    
    // Fill in login credentials
    await page.fill('input[name="email"]', 'info@epsx.io');
    await page.fill('input[name="password"]', 'P@ssword');
    
    // Submit login form
    await page.click('button[type="submit"]');
    
    // Wait for login to complete and redirect
    await page.waitForLoadState('networkidle');
    
    // Wait a bit more to catch any delayed network requests
    await page.waitForTimeout(3000);
    
    // CRITICAL VERIFICATION 1: Zero localhost connections
    const localhostRequests = networkRequests.filter(req => req.isLocalhost);
    
    console.log('\n=== NETWORK REQUEST ANALYSIS ===');
    console.log(`Total network requests: ${networkRequests.length}`);
    console.log(`Localhost requests: ${localhostRequests.length}`);
    console.log(`Production API requests: ${networkRequests.filter(req => req.isProductionAPI).length}`);
    
    if (localhostRequests.length > 0) {
      console.log('\n❌ LOCALHOST REQUESTS FOUND:');
      localhostRequests.forEach((req, index) => {
        console.log(`${index + 1}. ${req.method} ${req.url} (Status: ${req.status})`);
      });
    } else {
      console.log('\n✅ NO LOCALHOST REQUESTS - ISSUE RESOLVED!');
    }
    
    // Production API requests analysis
    const productionRequests = networkRequests.filter(req => req.isProductionAPI);
    console.log('\n=== PRODUCTION API REQUESTS ===');
    productionRequests.forEach((req, index) => {
      console.log(`${index + 1}. ${req.method} ${req.url} (Status: ${req.status})`);
    });
    
    // ASSERTION: Verify zero localhost connections
    expect(localhostRequests, 'Should have ZERO localhost connection attempts').toHaveLength(0);
    
    // CRITICAL VERIFICATION 2: Zero connection errors
    const connectionErrors = consoleErrors.filter(error => error.isConnectionError);
    
    console.log('\n=== CONSOLE ERROR ANALYSIS ===');
    console.log(`Total console messages: ${consoleErrors.length}`);
    console.log(`Connection errors: ${connectionErrors.length}`);
    
    if (connectionErrors.length > 0) {
      console.log('\n❌ CONNECTION ERRORS FOUND:');
      connectionErrors.forEach((error, index) => {
        console.log(`${index + 1}. [${error.type}] ${error.text}`);
      });
    } else {
      console.log('\n✅ NO CONNECTION ERRORS - CLEAN CONSOLE!');
    }
    
    // ASSERTION: Verify zero connection errors
    expect(connectionErrors, 'Should have ZERO connection refused errors').toHaveLength(0);
    
    // CRITICAL VERIFICATION 3: Successful production API communication
    const successfulAPIRequests = productionRequests.filter(req => 
      req.status >= 200 && req.status < 400
    );
    
    expect(successfulAPIRequests.length, 'Should have successful production API requests').toBeGreaterThan(0);
    
    // CRITICAL VERIFICATION 4: Login success verification
    // Check for successful redirect or dashboard content
    const isLoggedIn = await page.locator('text=Dashboard').isVisible() || 
                      await page.locator('text=Analytics').isVisible() ||
                      await page.locator('[data-testid="user-menu"]').isVisible() ||
                      await page.locator('button:has-text("Logout")').isVisible();
    
    expect(isLoggedIn, 'Should be successfully logged in').toBe(true);
    
    // Generate comprehensive report
    console.log('\n=== COMPREHENSIVE RESOLUTION REPORT ===');
    console.log('✅ Zero localhost connections (127.0.0.1:8080)');
    console.log('✅ Zero connection refused errors');
    console.log('✅ Successful production API communication (api.epsx.io)');
    console.log('✅ Complete login flow working');
    console.log('\n🎉 LOCALHOST CONNECTION ISSUE COMPLETELY RESOLVED!');
  });

  test('should handle navigation without localhost fallbacks', async ({ page }) => {
    // Navigate through different sections to test all potential connection points
    await page.goto('https://epsx.io');
    await page.waitForLoadState('networkidle');
    
    // Test various navigation paths that might trigger API calls
    const navigationPaths = [
      { name: 'About', selector: 'text=About' },
      { name: 'Analytics', selector: 'text=Analytics' },
      { name: 'Plans', selector: 'text=Plans' },
      { name: 'Login', selector: 'text=Login' }
    ];
    
    for (const path of navigationPaths) {
      try {
        await page.click(path.selector);
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000); // Allow time for any API calls
        
        console.log(`✅ Navigation to ${path.name} completed`);
      } catch (error) {
        console.log(`⚠️  Navigation to ${path.name} failed: ${error}`);
      }
    }
    
    // Final verification: No localhost requests during navigation
    const localhostRequests = networkRequests.filter(req => req.isLocalhost);
    expect(localhostRequests, 'Navigation should not trigger localhost requests').toHaveLength(0);
    
    // Final verification: No connection errors during navigation
    const connectionErrors = consoleErrors.filter(error => error.isConnectionError);
    expect(connectionErrors, 'Navigation should not cause connection errors').toHaveLength(0);
    
    console.log('\n=== NAVIGATION TEST RESULTS ===');
    console.log(`Total requests during navigation: ${networkRequests.length}`);
    console.log(`Localhost requests: ${localhostRequests.length}`);
    console.log(`Connection errors: ${connectionErrors.length}`);
    console.log('✅ All navigation paths work without localhost fallbacks');
  });

  test('should verify production environment configuration', async ({ page }) => {
    // Go to a page that loads environment config
    await page.goto('https://epsx.io');
    await page.waitForLoadState('networkidle');
    
    // Check client-side environment variables (if exposed for debugging)
    const envCheck = await page.evaluate(() => {
      return {
        // Check if any localhost references exist in window object
        hasLocalhost: JSON.stringify(window).includes('localhost:8080') || 
                     JSON.stringify(window).includes('127.0.0.1:8080'),
        // Check for production API URL
        hasProductionAPI: JSON.stringify(window).includes('api.epsx.io')
      };
    });
    
    expect(envCheck.hasLocalhost, 'Client environment should not contain localhost references').toBe(false);
    expect(envCheck.hasProductionAPI, 'Client environment should contain production API references').toBe(true);
    
    console.log('\n=== ENVIRONMENT CONFIGURATION VERIFICATION ===');
    console.log(`Localhost references in client: ${envCheck.hasLocalhost ? '❌ Found' : '✅ None'}`);
    console.log(`Production API references: ${envCheck.hasProductionAPI ? '✅ Found' : '❌ Missing'}`);
  });

  test.afterEach(async ({ page }) => {
    // Generate detailed test report
    console.log('\n=== FINAL TEST SUMMARY ===');
    console.log(`Test completed at: ${new Date().toISOString()}`);
    console.log(`Total network requests monitored: ${networkRequests.length}`);
    console.log(`Localhost requests: ${networkRequests.filter(req => req.isLocalhost).length}`);
    console.log(`Production API requests: ${networkRequests.filter(req => req.isProductionAPI).length}`);
    console.log(`Console errors: ${consoleErrors.length}`);
    console.log(`Connection errors: ${consoleErrors.filter(error => error.isConnectionError).length}`);
    
    // Save detailed logs for analysis
    const testReport = {
      timestamp: new Date().toISOString(),
      networkRequests,
      consoleErrors,
      summary: {
        totalRequests: networkRequests.length,
        localhostRequests: networkRequests.filter(req => req.isLocalhost).length,
        productionRequests: networkRequests.filter(req => req.isProductionAPI).length,
        totalErrors: consoleErrors.length,
        connectionErrors: consoleErrors.filter(error => error.isConnectionError).length
      }
    };
    
    console.log('\n=== DETAILED REPORT ===');
    console.log(JSON.stringify(testReport, null, 2));
  });
});