const { chromium } = require('playwright');

const FRONTEND_URL = 'https://epsx-frontend-dev-307278481624.us-central1.run.app';

async function testFrontendFirebase() {
  console.log('🚀 Starting Frontend Firebase Verification Test');
  console.log(`Target URL: ${FRONTEND_URL}`);
  
  const browser = await chromium.launch({ headless: false });
  const context = await browser.newContext();
  const page = await context.newPage();
  
  // Capture console messages
  const consoleMessages = [];
  const errorMessages = [];
  
  page.on('console', (message) => {
    const text = message.text();
    consoleMessages.push(text);
    
    // Look for Firebase errors
    if (text.includes('Firebase: Error (auth/invalid-api-key)') || 
        text.includes('auth/invalid-api-key') ||
        text.includes('FirebaseError')) {
      errorMessages.push(text);
      console.log(`🔥 Firebase Error: ${text}`);
    } else {
      console.log(`📝 Console: ${text}`);
    }
  });
  
  // Capture network failures
  const networkErrors = [];
  page.on('requestfailed', (request) => {
    const failure = `${request.url()} - ${request.failure()?.errorText}`;
    networkErrors.push(failure);
    console.log(`❌ Network Error: ${failure}`);
  });
  
  try {
    console.log('\n=== 1. Loading Frontend Page ===');
    const startTime = Date.now();
    
    const response = await page.goto(FRONTEND_URL, {
      waitUntil: 'networkidle',
      timeout: 60000
    });
    
    const loadTime = Date.now() - startTime;
    console.log(`⏱️  Page loaded in ${loadTime}ms`);
    console.log(`📊 Response status: ${response?.status()}`);
    
    if (response?.status() >= 400) {
      throw new Error(`HTTP Error: ${response?.status()}`);
    }
    
    // Wait for page to fully load
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(10000); // Wait 10 seconds for Firebase initialization
    
    console.log('\n=== 2. Checking for Firebase Errors ===');
    console.log(`Total console messages: ${consoleMessages.length}`);
    console.log(`Firebase error messages: ${errorMessages.length}`);
    
    if (errorMessages.length > 0) {
      console.log('🚨 Firebase errors detected:');
      errorMessages.forEach((error, index) => {
        console.log(`   ${index + 1}. ${error}`);
      });
      throw new Error(`Firebase API key errors detected: ${errorMessages.length} errors`);
    } else {
      console.log('✅ No Firebase API key errors detected!');
    }
    
    console.log('\n=== 3. Checking Page Content ===');
    
    // Check for error indicators
    const errorTexts = [
      'Firebase: Error (auth/invalid-api-key)',
      'Something went wrong',
      'Error loading page',
      'Application Error',
      '500 Internal Server Error',
      '404 Not Found'
    ];
    
    for (const errorText of errorTexts) {
      const hasError = await page.locator(`text=${errorText}`).count() > 0;
      if (hasError) {
        throw new Error(`Error text found on page: "${errorText}"`);
      }
    }
    console.log('✅ No error messages found on page');
    
    // Check for positive indicators
    const htmlExists = await page.locator('html').count() > 0;
    const bodyExists = await page.locator('body').count() > 0;
    
    if (!htmlExists || !bodyExists) {
      throw new Error('Basic HTML structure missing');
    }
    console.log('✅ Basic HTML structure present');
    
    // Check for React/Next.js indicators
    const reactIndicators = [
      'div[id="__next"]',
      'script[src*="/_next/"]',
      'link[href*="/_next/"]'
    ];
    
    let reactFound = false;
    for (const selector of reactIndicators) {
      if (await page.locator(selector).count() > 0) {
        reactFound = true;
        console.log(`✅ React/Next.js indicator found: ${selector}`);
        break;
      }
    }
    
    if (!reactFound) {
      console.log('⚠️  No React/Next.js indicators found (may be SSR or different setup)');
    }
    
    console.log('\n=== 4. Testing Basic Navigation ===');
    
    // Find clickable elements
    const buttons = await page.locator('button').count();
    const links = await page.locator('a[href]').count();
    
    console.log(`🖱️  Found ${buttons} buttons and ${links} links`);
    
    if (buttons > 0 || links > 0) {
      console.log('✅ Interactive elements found');
    } else {
      console.log('⚠️  No interactive elements found');
    }
    
    console.log('\n=== 5. Performance Metrics ===');
    
    const performanceMetrics = await page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0];
      return {
        domContentLoaded: navigation.domContentLoadedEventEnd - navigation.navigationStart,
        loadComplete: navigation.loadEventEnd - navigation.navigationStart,
      };
    });
    
    console.log(`📊 DOM Content Loaded: ${performanceMetrics.domContentLoaded}ms`);
    console.log(`📊 Load Complete: ${performanceMetrics.loadComplete}ms`);
    
    // Take screenshot
    await page.screenshot({ 
      path: 'firebase-verification-result.png',
      fullPage: true 
    });
    console.log('📸 Screenshot saved as firebase-verification-result.png');
    
    console.log('\n=== FINAL RESULTS ===');
    console.log('✅ Frontend service loaded successfully');
    console.log('✅ No Firebase API key errors detected');
    console.log('✅ No error pages displayed');
    console.log('✅ Basic functionality appears to work');
    console.log(`📊 Total console messages: ${consoleMessages.length}`);
    console.log(`📊 Network errors: ${networkErrors.length}`);
    
    if (networkErrors.length > 0) {
      console.log('Network issues detected:');
      networkErrors.forEach((error, index) => {
        console.log(`   ${index + 1}. ${error}`);
      });
    }
    
    console.log('\n🎉 FIREBASE ERROR RESOLUTION CONFIRMED! 🎉');
    console.log('The frontend service is now working properly without Firebase authentication errors.');
    
  } catch (error) {
    console.error('❌ Test failed:', error.message);
    
    // Take error screenshot
    await page.screenshot({ 
      path: 'firebase-verification-error.png',
      fullPage: true 
    });
    console.log('📸 Error screenshot saved as firebase-verification-error.png');
    
    throw error;
  } finally {
    await browser.close();
  }
}

// Run the test
testFrontendFirebase()
  .then(() => {
    console.log('\n✅ Test completed successfully!');
    process.exit(0);
  })
  .catch((error) => {
    console.error('\n❌ Test failed:', error.message);
    process.exit(1);
  });