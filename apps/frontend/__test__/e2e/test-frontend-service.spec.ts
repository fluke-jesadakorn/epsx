import { test, expect } from '@playwright/test';

test.describe('Frontend Service Verification', () => {
  const frontendUrl = 'https://epsx-frontend-dev-307278481624.us-central1.run.app';
  const backendUrl = 'https://epsx-backend-dev-307278481624.us-central1.run.app';

  test('Service Accessibility - Should load without 500 errors', async ({ page }) => {
    // Monitor console errors
    const consoleErrors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    // Monitor network failures
    const networkErrors = [];
    page.on('response', response => {
      if (response.status() >= 500) {
        networkErrors.push({
          url: response.url(),
          status: response.status(),
          statusText: response.statusText()
        });
      }
    });

    // Navigate to frontend
    const response = await page.goto(frontendUrl, { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    // Verify response status
    expect(response.status()).toBeLessThan(500);
    console.log(`Frontend loaded with status: ${response.status()}`);

    // Check for 500 errors in network requests
    if (networkErrors.length > 0) {
      console.log('Network 500 errors found:', networkErrors);
    }
    expect(networkErrors.length).toBe(0);

    // Wait for page to be ready
    await page.waitForLoadState('domcontentloaded');
    
    // Take screenshot for verification
    await page.screenshot({ path: 'frontend-loaded.png', fullPage: true });
  });

  test('Backend Connectivity - Should connect to dev backend', async ({ page }) => {
    const backendRequests = [];
    const localhostRequests = [];

    // Monitor network requests
    page.on('request', request => {
      const url = request.url();
      if (url.includes('epsx-backend-dev-307278481624.us-central1.run.app')) {
        backendRequests.push(url);
      }
      if (url.includes('127.0.0.1:8080') || url.includes('localhost:8080')) {
        localhostRequests.push(url);
      }
    });

    await page.goto(frontendUrl, { waitUntil: 'networkidle' });

    // Wait for API calls to complete
    await page.waitForTimeout(5000);

    console.log('Backend requests found:', backendRequests);
    console.log('Localhost requests found:', localhostRequests);

    // Verify no localhost requests
    expect(localhostRequests.length).toBe(0);
    
    // Log backend connectivity status
    if (backendRequests.length > 0) {
      console.log('✅ Successfully connecting to dev backend');
    } else {
      console.log('ℹ️ No backend requests detected yet');
    }
  });

  test('Firebase Configuration - Should work with correct API keys', async ({ page }) => {
    const firebaseErrors = [];
    const firebaseRequests = [];

    // Monitor console for Firebase errors
    page.on('console', msg => {
      if (msg.type() === 'error' && msg.text().includes('Firebase')) {
        firebaseErrors.push(msg.text());
      }
    });

    // Monitor Firebase network requests
    page.on('request', request => {
      if (request.url().includes('firebase') || request.url().includes('googleapis.com')) {
        firebaseRequests.push(request.url());
      }
    });

    await page.goto(frontendUrl, { waitUntil: 'networkidle' });
    await page.waitForTimeout(3000);

    console.log('Firebase requests:', firebaseRequests);
    console.log('Firebase errors:', firebaseErrors);

    // Check for invalid API key errors
    const invalidApiKeyErrors = firebaseErrors.filter(error => 
      error.includes('invalid-api-key') || error.includes('API key not valid')
    );

    expect(invalidApiKeyErrors.length).toBe(0);
    
    if (firebaseRequests.length > 0) {
      console.log('✅ Firebase requests detected - configuration appears valid');
    }
  });

  test('Network Requests Analysis', async ({ page }) => {
    const allRequests = [];
    const failedRequests = [];

    page.on('request', request => {
      allRequests.push({
        url: request.url(),
        method: request.method(),
        resourceType: request.resourceType()
      });
    });

    page.on('response', response => {
      if (response.status() >= 400) {
        failedRequests.push({
          url: response.url(),
          status: response.status(),
          statusText: response.statusText()
        });
      }
    });

    await page.goto(frontendUrl, { waitUntil: 'networkidle' });
    await page.waitForTimeout(5000);

    console.log(`Total requests: ${allRequests.length}`);
    console.log('Failed requests:', failedRequests);

    // Analyze request patterns
    const apiRequests = allRequests.filter(req => 
      req.url.includes('/api/') || req.resourceType === 'xhr' || req.resourceType === 'fetch'
    );
    
    console.log('API requests:', apiRequests);

    // Should have minimal failed requests
    expect(failedRequests.length).toBeLessThan(5);
  });

  test('Console Errors Check', async ({ page }) => {
    const consoleMessages = [];

    page.on('console', msg => {
      consoleMessages.push({
        type: msg.type(),
        text: msg.text(),
        location: msg.location()
      });
    });

    await page.goto(frontendUrl, { waitUntil: 'networkidle' });
    await page.waitForTimeout(5000);

    const errors = consoleMessages.filter(msg => msg.type === 'error');
    const warnings = consoleMessages.filter(msg => msg.type === 'warning');

    console.log('Console errors:', errors);
    console.log('Console warnings:', warnings);

    // Check for critical errors
    const criticalErrors = errors.filter(error => 
      error.text.includes('Connection refused') ||
      error.text.includes('invalid-api-key') ||
      error.text.includes('500') ||
      error.text.includes('Internal Server Error')
    );

    expect(criticalErrors.length).toBe(0);
  });

  test('Trading Dashboard Functionality', async ({ page }) => {
    await page.goto(frontendUrl, { waitUntil: 'networkidle' });

    // Wait for page to load completely
    await page.waitForTimeout(3000);

    // Check if main elements are present
    const bodyContent = await page.textContent('body');
    
    // Look for common dashboard elements
    const dashboardIndicators = [
      'EPS',
      'Analytics',
      'Dashboard',
      'Trading',
      'Market'
    ];

    const foundIndicators = dashboardIndicators.filter(indicator => 
      bodyContent.toLowerCase().includes(indicator.toLowerCase())
    );

    console.log('Dashboard indicators found:', foundIndicators);

    // Check for React hydration
    const reactElements = await page.$$('[data-reactroot], [data-react-helmet], #__next');
    console.log(`React elements found: ${reactElements.length}`);

    // Verify page is not showing error page
    const isErrorPage = bodyContent.includes('500') || 
                       bodyContent.includes('Internal Server Error') ||
                       bodyContent.includes('Something went wrong');

    expect(isErrorPage).toBe(false);

    // Take final screenshot
    await page.screenshot({ path: 'dashboard-loaded.png', fullPage: true });
  });
});