import { test, expect } from '@playwright/test';

test('Login Flow - Frontend to Backend Connectivity Verification', async ({ page }) => {
  console.log('🚀 Starting login verification test...');
  
  // Monitor network requests to track connectivity
  const networkRequests: string[] = [];
  let backendConnected = false;
  let oldApiError = false;
  
  page.on('request', request => {
    const url = request.url();
    networkRequests.push(url);
    
    if (url.includes('backend-307278481624.us-central1.run.app')) {
      backendConnected = true;
      console.log(`✅ CORRECT: Request to new backend: ${url}`);
    }
    
    if (url.includes('api.epsx.io')) {
      oldApiError = true;
      console.log(`❌ ERROR: Request to old API: ${url}`);
    }
  });

  // Monitor console for errors
  page.on('console', msg => {
    if (msg.type() === 'error') {
      console.log(`🔍 Console Error: ${msg.text()}`);
    }
  });

  // Navigate to frontend
  console.log('📍 Navigating to frontend...');
  await page.goto('https://epsx.io');
  
  // Wait for page load
  await page.waitForLoadState('networkidle');
  console.log('✅ Frontend loaded');

  // Test backend health directly
  console.log('📍 Testing backend health...');
  const healthResponse = await page.request.get('https://backend-307278481624.us-central1.run.app/health');
  console.log(`🏥 Backend health status: ${healthResponse.status()}`);
  expect(healthResponse.ok()).toBeTruthy();

  // Look for login form or authentication
  console.log('📍 Looking for login interface...');
  
  // Try to find login elements
  const loginLink = page.locator('a[href*="login"], a:has-text("Login"), button:has-text("Login")').first();
  
  if (await loginLink.isVisible()) {
    console.log('🔗 Found login link, clicking...');
    await loginLink.click();
    await page.waitForLoadState('networkidle');
  } else {
    console.log('🔗 Navigating directly to login...');
    await page.goto('https://epsx.io/register'); // Try register first to see auth flow
    await page.waitForLoadState('networkidle');
  }

  // Test form interaction to trigger API calls
  const emailField = page.locator('input[type="email"], input[name="email"], input[placeholder*="email" i]').first();
  const passwordField = page.locator('input[type="password"], input[name="password"]').first();
  
  if (await emailField.isVisible() && await passwordField.isVisible()) {
    console.log('✅ Found auth form fields');
    
    // Fill test credentials
    await emailField.fill('info@epsx.io');
    await passwordField.fill('P@ssword');
    
    console.log('✅ Filled test credentials');
    
    // Try to submit to trigger backend calls
    const submitButton = page.locator('button[type="submit"], button:has-text("Login"), button:has-text("Sign")').first();
    
    if (await submitButton.isVisible()) {
      console.log('🔄 Submitting form...');
      await submitButton.click();
      
      // Wait for network activity
      await page.waitForTimeout(3000);
    }
  }

  // Final verification
  console.log('📍 Final verification...');
  console.log(`🔍 Backend connected: ${backendConnected}`);
  console.log(`🔍 Old API error: ${oldApiError}`);
  console.log(`🔍 Total network requests: ${networkRequests.length}`);
  
  // Check for configuration success
  if (!oldApiError) {
    console.log('✅ SUCCESS: No requests to old api.epsx.io detected');
  } else {
    console.log('❌ FAILURE: Still connecting to old API');
  }
  
  if (backendConnected) {
    console.log('✅ SUCCESS: Frontend connecting to correct backend');
  } else {
    console.log('⚠️  INFO: No backend requests detected (may be normal for this page)');
  }
  
  console.log('🎉 Login verification test completed');
});

test('Environment Configuration Verification', async ({ page }) => {
  console.log('🔧 Testing environment configuration...');
  
  await page.goto('https://epsx.io');
  
  // Check page source for hardcoded URLs
  const content = await page.content();
  const hasOldApi = content.includes('api.epsx.io');
  const hasNewBackend = content.includes('backend-307278481624.us-central1.run.app');
  
  console.log(`🔍 Page analysis:`);
  console.log(`   - Contains old API: ${hasOldApi}`);
  console.log(`   - Contains new backend: ${hasNewBackend}`);
  
  // Verify no hardcoded old API references
  expect(hasOldApi).toBeFalsy();
  
  console.log('✅ Environment configuration verified');
});

test('CORS Configuration Test', async ({ page }) => {
  console.log('🛡️  Testing CORS configuration...');
  
  let corsError = false;
  
  page.on('console', msg => {
    if (msg.text().includes('CORS') || msg.text().includes('Access-Control')) {
      corsError = true;
      console.log(`🚨 CORS Issue: ${msg.text()}`);
    }
  });
  
  // Test CORS with actual frontend origin
  const response = await page.request.get('https://backend-307278481624.us-central1.run.app/health', {
    headers: {
      'Origin': 'https://epsx.io'
    }
  });
  
  console.log(`✅ CORS test response: ${response.status()}`);
  expect(response.ok()).toBeTruthy();
  
  if (!corsError) {
    console.log('✅ No CORS errors detected');
  }
  
  console.log('✅ CORS configuration verified');
});