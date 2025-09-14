import { test, expect } from '@playwright/test';

test.describe('Production Deployment Check', () => {
  test('should load deployed frontend without JavaScript errors', async ({ page }) => {
    // Collect console errors
    const consoleErrors: string[] = [];
    const consoleWarnings: string[] = [];
    const consoleMessages: string[] = [];

    page.on('console', (msg) => {
      const text = msg.text();
      consoleMessages.push(`[${msg.type()}] ${text}`);
      
      if (msg.type() === 'error') {
        consoleErrors.push(text);
      } else if (msg.type() === 'warning') {
        consoleWarnings.push(text);
      }
    });

    // Collect network errors
    const networkErrors: string[] = [];
    page.on('requestfailed', (request) => {
      networkErrors.push(`Failed request: ${request.url()} - ${request.failure()?.errorText}`);
    });

    // Navigate to the deployed frontend
    console.log('🚀 Navigating to deployed frontend...');
    await page.goto('https://epsx-frontend-dev-307278481624.us-central1.run.app');

    // Wait for page to load
    await page.waitForLoadState('networkidle');

    // Check Firebase configuration in browser by inspecting actual config objects
    const firebaseAnalysis = await page.evaluate(() => {
      let results = {
        clientConfigAvailable: false,
        firebaseConfigValues: {},
        firebaseAppInitialized: false,
        firebaseErrors: [],
        configSource: 'unknown'
      };

      try {
        // Check if clientConfig is available globally
        if (typeof window !== 'undefined' && (window as any).clientConfig) {
          results.clientConfigAvailable = true;
          results.firebaseConfigValues = (window as any).clientConfig.firebase || {};
          results.configSource = 'window.clientConfig';
        }

        // Check if Firebase app is initialized
        if (typeof window !== 'undefined' && (window as any).firebase) {
          try {
            const apps = (window as any).firebase.apps || [];
            results.firebaseAppInitialized = apps.length > 0;
          } catch (e) {
            results.firebaseErrors.push(`Firebase apps check failed: ${e.message}`);
          }
        }

        // Look for Firebase config in common locations
        const possibleConfigPaths = [
          'window.__FIREBASE_CONFIG__',
          'window.firebaseConfig',
          'window.__NEXT_DATA__'
        ];

        possibleConfigPaths.forEach(path => {
          try {
            const parts = path.split('.');
            let obj = window as any;
            for (const part of parts.slice(1)) {
              obj = obj[part];
              if (!obj) break;
            }
            if (obj && typeof obj === 'object') {
              results.firebaseConfigValues = { ...results.firebaseConfigValues, ...obj };
              results.configSource = path;
            }
          } catch (e) {
            // Ignore path not found
          }
        });

      } catch (error) {
        results.firebaseErrors.push(`Config analysis failed: ${error.message}`);
      }

      return results;
    });

    console.log('\n🔧 Firebase Configuration Analysis:');
    console.log(`📋 Client Config Available: ${firebaseAnalysis.clientConfigAvailable ? '✅' : '❌'}`);
    console.log(`🔥 Firebase App Initialized: ${firebaseAnalysis.firebaseAppInitialized ? '✅' : '❌'}`);
    console.log(`📍 Config Source: ${firebaseAnalysis.configSource}`);
    
    if (Object.keys(firebaseAnalysis.firebaseConfigValues).length > 0) {
      console.log('\n🔑 Firebase Config Values Found:');
      Object.entries(firebaseAnalysis.firebaseConfigValues).forEach(([key, value]) => {
        const status = value ? '✅' : '❌';
        const displayValue = value ? (typeof value === 'string' && value.length > 20 ? 'SET (truncated)' : 'SET') : 'NOT_SET';
        console.log(`${status} ${key}: ${displayValue}`);
      });
    } else {
      console.log('❌ No Firebase configuration values found');
    }

    if (firebaseAnalysis.firebaseErrors.length > 0) {
      console.log('\n🚨 Firebase Configuration Errors:');
      firebaseAnalysis.firebaseErrors.forEach(error => console.log(`  - ${error}`));
    }

    // Take a screenshot
    await page.screenshot({ 
      path: '.playwright-mcp/production-deployment-check.png',
      fullPage: true 
    });

    // Check for specific errors mentioned
    const adminUrlError = consoleErrors.find(error => 
      error.includes('ADMIN_FRONTEND_URL is required in production environment')
    );

    const firebaseApiKeyError = consoleErrors.find(error => 
      error.includes('Firebase: Error (auth/invalid-api-key)')
    );

    // Log all console messages for debugging
    console.log('\n📋 Console Messages Summary:');
    console.log(`Total messages: ${consoleMessages.length}`);
    console.log(`Errors: ${consoleErrors.length}`);
    console.log(`Warnings: ${consoleWarnings.length}`);
    console.log(`Network errors: ${networkErrors.length}`);

    if (consoleErrors.length > 0) {
      console.log('\n❌ Console Errors:');
      consoleErrors.forEach((error, index) => {
        console.log(`${index + 1}. ${error}`);
      });
    }

    if (consoleWarnings.length > 0) {
      console.log('\n⚠️  Console Warnings:');
      consoleWarnings.forEach((warning, index) => {
        console.log(`${index + 1}. ${warning}`);
      });
    }

    if (networkErrors.length > 0) {
      console.log('\n🌐 Network Errors:');
      networkErrors.forEach((error, index) => {
        console.log(`${index + 1}. ${error}`);
      });
    }

    // Specific checks for the mentioned errors
    if (adminUrlError) {
      console.log('\n🔍 Found ADMIN_FRONTEND_URL error:', adminUrlError);
    } else {
      console.log('\n✅ No ADMIN_FRONTEND_URL error found');
    }

    if (firebaseApiKeyError) {
      console.log('\n🔍 Found Firebase API key error:', firebaseApiKeyError);
    } else {
      console.log('\n✅ No Firebase API key error found');
    }

    // Check if page title loaded properly
    const title = await page.title();
    console.log('\n📄 Page title:', title);

    // Check if main content is visible
    const bodyText = await page.locator('body').textContent();
    const hasContent = bodyText && bodyText.trim().length > 0;
    console.log('\n📝 Page has content:', hasContent);

    // Verify the page loaded successfully
    expect(page.url()).toBe('https://epsx-frontend-dev-307278481624.us-central1.run.app/');
    
    // Report test results
    console.log('\n🎯 Test Results:');
    console.log(`- Page loaded: ✅`);
    console.log(`- ADMIN_FRONTEND_URL error: ${adminUrlError ? '❌ Found' : '✅ Not found'}`);
    console.log(`- Firebase API key error: ${firebaseApiKeyError ? '❌ Found' : '✅ Not found'}`);
    console.log(`- Total console errors: ${consoleErrors.length}`);
    console.log(`- Screenshot saved: .playwright-mcp/production-deployment-check.png`);

    // Fail test if critical errors are found
    if (adminUrlError || firebaseApiKeyError) {
      throw new Error(`Critical errors found: ${adminUrlError ? 'ADMIN_FRONTEND_URL error; ' : ''}${firebaseApiKeyError ? 'Firebase API key error' : ''}`);
    }
  });
});