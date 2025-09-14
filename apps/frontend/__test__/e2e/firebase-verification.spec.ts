import { test, expect } from '@playwright/test';

test.describe('EPSX Frontend Firebase Error Resolution Verification', () => {
  const frontendUrl = 'https://epsx-frontend-dev-307278481624.us-central1.run.app';

  test('should load frontend without Firebase authentication errors', async ({ page }) => {
    // Capture console messages to check for Firebase errors
    const consoleMessages: Array<{type: string, text: string, timestamp: string}> = [];
    const errors: Array<{message: string, stack?: string, timestamp: string}> = [];

    page.on('console', msg => {
      consoleMessages.push({
        type: msg.type(),
        text: msg.text(),
        timestamp: new Date().toISOString()
      });
    });

    page.on('pageerror', error => {
      errors.push({
        message: error.message,
        stack: error.stack,
        timestamp: new Date().toISOString()
      });
    });

    // Navigate to the frontend URL
    console.log('Navigating to:', frontendUrl);
    const response = await page.goto(frontendUrl, { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    // Verify the page loads successfully
    expect(response?.status()).toBeLessThan(400);
    console.log('Response status:', response?.status());

    // Wait for page to be fully loaded
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(3000); // Allow Firebase to initialize

    // Check for Firebase-specific errors
    const firebaseErrors = consoleMessages.filter(msg => 
      msg.text.toLowerCase().includes('firebase') && 
      (msg.text.toLowerCase().includes('error') || msg.text.toLowerCase().includes('invalid-api-key'))
    );

    const firebaseInit = consoleMessages.filter(msg => 
      msg.text.toLowerCase().includes('firebase') && 
      msg.text.toLowerCase().includes('initialized')
    );

    // Log all console messages for analysis
    console.log('=== CONSOLE MESSAGES ===');
    consoleMessages.forEach(msg => {
      console.log(`[${msg.type.toUpperCase()}] ${msg.text}`);
    });

    console.log('=== PAGE ERRORS ===');
    errors.forEach(error => {
      console.log(`ERROR: ${error.message}`);
    });

    console.log('=== FIREBASE-SPECIFIC ANALYSIS ===');
    console.log('Firebase errors found:', firebaseErrors.length);
    firebaseErrors.forEach(error => {
      console.log(`FIREBASE ERROR: ${error.text}`);
    });

    console.log('Firebase initialization messages:', firebaseInit.length);
    firebaseInit.forEach(init => {
      console.log(`FIREBASE INIT: ${init.text}`);
    });

    // Verify no Firebase authentication errors
    expect(firebaseErrors.length).toBe(0);
    
    // Verify page title and basic content
    const title = await page.title();
    console.log('Page title:', title);
    
    // Check if the page displays content instead of error
    const bodyText = await page.textContent('body');
    const hasErrorContent = bodyText?.toLowerCase().includes('firebase: error') || 
                           bodyText?.toLowerCase().includes('invalid-api-key');
    
    expect(hasErrorContent).toBe(false);
    
    // Verify the page has some meaningful content
    const hasContent = (bodyText?.length || 0) > 100; // Reasonable content length
    expect(hasContent).toBe(true);
    
    console.log('Page loaded successfully with content length:', bodyText?.length);
  });

  test('should have working navigation and basic functionality', async ({ page }) => {
    await page.goto(frontendUrl, { waitUntil: 'networkidle' });
    
    // Wait for page to be fully loaded
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(2000);
    
    // Check for navigation elements
    const navigation = await page.locator('nav').count();
    console.log('Navigation elements found:', navigation);
    
    // Check for interactive elements
    const buttons = await page.locator('button').count();
    const links = await page.locator('a').count();
    
    console.log('Interactive elements - Buttons:', buttons, 'Links:', links);
    
    // Verify the page is interactive (has some buttons or links)
    expect(buttons + links).toBeGreaterThan(0);
    
    // Take a screenshot for visual verification
    await page.screenshot({ 
      path: 'test-results/frontend-verification-screenshot.png', 
      fullPage: true 
    });
    
    console.log('Screenshot saved for visual verification');
  });

  test('should load required assets without errors', async ({ page }) => {
    const failedRequests: Array<{url: string, status: number, statusText: string}> = [];
    
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
    await page.waitForTimeout(3000);

    console.log('=== FAILED REQUESTS ===');
    failedRequests.forEach(req => {
      console.log(`FAILED: ${req.status} ${req.statusText} - ${req.url}`);
    });

    // Allow for some non-critical failures but verify critical assets load
    const criticalFailures = failedRequests.filter(req => 
      req.url.includes('.js') || req.url.includes('.css') || req.status >= 500
    );

    expect(criticalFailures.length).toBe(0);
    console.log('All critical assets loaded successfully');
  });
});