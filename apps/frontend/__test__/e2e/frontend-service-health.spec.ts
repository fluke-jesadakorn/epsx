import { test, expect } from '@playwright/test';

test.describe('Frontend Service Health Check', () => {
  const frontendUrl = 'https://epsx-frontend-dev-307278481624.us-central1.run.app';
  const backendUrl = 'https://epsx-backend-dev-307278481624.us-central1.run.app';

  test('Service loads without 500 errors', async ({ page }) => {
    const consoleErrors = [];
    const networkErrors = [];

    // Monitor console and network
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    page.on('response', response => {
      if (response.status() >= 500) {
        networkErrors.push({
          url: response.url(),
          status: response.status()
        });
      }
    });

    // Navigate to frontend
    const response = await page.goto(frontendUrl, { 
      waitUntil: 'domcontentloaded',
      timeout: 30000 
    });

    expect(response.status()).toBeLessThan(500);
    console.log(`✅ Frontend loaded with status: ${response.status()}`);

    // Check for critical errors
    const criticalErrors = consoleErrors.filter(error => 
      error.includes('Connection refused') || 
      error.includes('127.0.0.1:8080') ||
      error.includes('localhost:8080')
    );

    if (criticalErrors.length > 0) {
      console.log('❌ Critical connection errors:', criticalErrors);
    } else {
      console.log('✅ No connection refused errors found');
    }

    expect(criticalErrors.length).toBe(0);
  });

  test('Backend connectivity check', async ({ page }) => {
    const requests = {
      backend: [],
      localhost: [],
      firebase: []
    };

    page.on('request', request => {
      const url = request.url();
      if (url.includes('epsx-backend-dev-307278481624.us-central1.run.app')) {
        requests.backend.push(url);
      }
      if (url.includes('127.0.0.1:8080') || url.includes('localhost:8080')) {
        requests.localhost.push(url);
      }
      if (url.includes('firebase') || url.includes('googleapis.com')) {
        requests.firebase.push(url);
      }
    });

    await page.goto(frontendUrl, { waitUntil: 'networkidle', timeout: 30000 });
    await page.waitForTimeout(5000);

    console.log('Backend requests:', requests.backend.length);
    console.log('Localhost requests:', requests.localhost.length);
    console.log('Firebase requests:', requests.firebase.length);

    // Verify no localhost requests
    expect(requests.localhost.length).toBe(0);
    
    if (requests.backend.length > 0) {
      console.log('✅ Successfully connecting to dev backend');
    } else {
      console.log('ℹ️ No backend API calls detected in initial load');
    }
  });

  test('Firebase configuration health', async ({ page }) => {
    const firebaseErrors = [];
    
    page.on('console', msg => {
      if (msg.type() === 'error' && 
          (msg.text().includes('Firebase') || 
           msg.text().includes('invalid-api-key') ||
           msg.text().includes('API key not valid'))) {
        firebaseErrors.push(msg.text());
      }
    });

    await page.goto(frontendUrl, { waitUntil: 'networkidle', timeout: 30000 });
    await page.waitForTimeout(3000);

    console.log('Firebase errors found:', firebaseErrors.length);
    
    if (firebaseErrors.length > 0) {
      console.log('❌ Firebase errors:', firebaseErrors);
    } else {
      console.log('✅ No Firebase configuration errors');
    }

    expect(firebaseErrors.length).toBe(0);
  });

  test('Overall page functionality', async ({ page }) => {
    await page.goto(frontendUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
    
    // Wait for React to hydrate
    await page.waitForTimeout(3000);

    const bodyText = await page.textContent('body');
    
    // Check if it's not an error page
    const isErrorPage = bodyText.includes('500') || 
                       bodyText.includes('Internal Server Error') ||
                       bodyText.includes('Something went wrong');

    if (isErrorPage) {
      console.log('❌ Error page detected');
    } else {
      console.log('✅ Page loaded successfully - no error page detected');
    }

    expect(isErrorPage).toBe(false);

    // Look for typical app content
    const hasContent = bodyText.length > 1000; // Should have substantial content
    console.log(`Content length: ${bodyText.length} characters`);
    
    expect(hasContent).toBe(true);

    // Take screenshot for manual verification
    await page.screenshot({ 
      path: 'test-results/frontend-health-check.png', 
      fullPage: true 
    });
    
    console.log('✅ Screenshot saved for manual verification');
  });
});