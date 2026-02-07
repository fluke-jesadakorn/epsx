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

    // Check Web3 configuration and app initialization
    const web3Analysis = await page.evaluate(() => {
      const results = {
        clientConfigAvailable: false,
        web3ConfigValues: {},
        web3Initialized: false,
        web3Errors: [],
        configSource: 'unknown'
      };

      try {
        // Check if clientConfig is available globally
        if (typeof window !== 'undefined' && (window as any).clientConfig) {
          results.clientConfigAvailable = true;
          results.web3ConfigValues = (window as any).clientConfig.web3 || {};
          results.configSource = 'window.clientConfig';
        }

        // Check if Web3 providers are available
        if (typeof window !== 'undefined') {
          try {
            results.web3Initialized = !!(window as any).ethereum || !!(window as any).web3;
          } catch (e) {
            results.web3Errors.push(`Web3 provider check failed: ${e.message}`);
          }
        }

        // Look for environment variables in Next.js data
        try {
          const nextData = (window as any).__NEXT_DATA__;
          if (nextData?.runtimeConfig || nextData?.buildId) {
            results.configSource = 'window.__NEXT_DATA__';
          }
        } catch (e) {
          // Ignore if not found
        }

      } catch (error) {
        results.web3Errors.push(`Config analysis failed: ${error.message}`);
      }

      return results;
    });

    console.log('\n🔧 Web3 Configuration Analysis:');
    console.log(`📋 Client Config Available: ${web3Analysis.clientConfigAvailable ? '✅' : '❌'}`);
    console.log(`⛓️ Web3 Provider Available: ${web3Analysis.web3Initialized ? '✅' : '❌'}`);
    console.log(`📍 Config Source: ${web3Analysis.configSource}`);
    
    if (Object.keys(web3Analysis.web3ConfigValues).length > 0) {
      console.log('\n🔑 Web3 Config Values Found:');
      Object.entries(web3Analysis.web3ConfigValues).forEach(([key, value]) => {
        const status = value ? '✅' : '❌';
        const displayValue = value ? (typeof value === 'string' && value.length > 20 ? 'SET (truncated)' : 'SET') : 'NOT_SET';
        console.log(`${status} ${key}: ${displayValue}`);
      });
    } else {
      console.log('❌ No Web3 configuration values found');
    }

    if (web3Analysis.web3Errors.length > 0) {
      console.log('\n🚨 Web3 Configuration Errors:');
      web3Analysis.web3Errors.forEach(error => console.log(`  - ${error}`));
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

    const web3ConnectionError = consoleErrors.find(error => 
      error.includes('Web3') || error.includes('MetaMask') || error.includes('wallet')
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

    if (web3ConnectionError) {
      console.log('\n🔍 Found Web3 connection error:', web3ConnectionError);
    } else {
      console.log('\n✅ No Web3 connection errors found');
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
    console.log(`- Web3 connection error: ${web3ConnectionError ? '❌ Found' : '✅ Not found'}`);
    console.log(`- Total console errors: ${consoleErrors.length}`);
    console.log(`- Screenshot saved: .playwright-mcp/production-deployment-check.png`);

    // Fail test if critical errors are found
    if (adminUrlError || web3ConnectionError) {
      throw new Error(`Critical errors found: ${adminUrlError ? 'ADMIN_FRONTEND_URL error; ' : ''}${web3ConnectionError ? 'Web3 connection error' : ''}`);
    }
  });
});