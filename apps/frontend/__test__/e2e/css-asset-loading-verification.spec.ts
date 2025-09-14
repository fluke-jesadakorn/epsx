import { test, expect } from '@playwright/test';

test.describe('CSS and Asset Loading Verification', () => {
  const FRONTEND_URL = 'https://epsx-frontend-dev-307278481624.us-central1.run.app';
  const ADMIN_URL = 'https://epsx-admin-dev-307278481624.us-central1.run.app';

  test('Frontend: CSS and assets loading verification', async ({ page }) => {
    console.log('🚀 Testing Frontend CSS and Asset Loading...');
    console.log(`URL: ${FRONTEND_URL}`);

    // Track all network requests
    const allRequests: Array<{url: string, method: string, resourceType: string}> = [];
    const allResponses: Array<{url: string, status: number, statusText: string}> = [];
    const failedRequests: Array<{url: string, status: number, failure?: string}> = [];
    const consoleErrors: string[] = [];

    // Set up request/response tracking
    page.on('request', request => {
      allRequests.push({
        url: request.url(),
        method: request.method(),
        resourceType: request.resourceType()
      });
    });

    page.on('response', response => {
      allResponses.push({
        url: response.url(),
        status: response.status(),
        statusText: response.statusText()
      });

      // Track failed responses
      if (response.status() >= 400) {
        failedRequests.push({
          url: response.url(),
          status: response.status()
        });
      }
    });

    page.on('requestfailed', request => {
      failedRequests.push({
        url: request.url(),
        status: 0,
        failure: request.failure()?.errorText
      });
    });

    // Track console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    const startTime = Date.now();

    // Navigate to frontend
    await page.goto(FRONTEND_URL, { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    const loadTime = Date.now() - startTime;

    // Take screenshot
    await page.screenshot({ 
      path: '.playwright-mcp/frontend-css-verification.png',
      fullPage: true 
    });

    // Analyze CSS files
    const cssResponses = allResponses.filter(r => 
      r.url.includes('.css') || r.url.includes('_next/static/css')
    );
    
    const jsResponses = allResponses.filter(r => 
      r.url.includes('.js') && r.url.includes('_next/static')
    );

    const staticAssetResponses = allResponses.filter(r => 
      r.url.includes('_next/static/') || r.url.includes('/static/')
    );

    // Check for 404 errors specifically on assets
    const failed404Assets = failedRequests.filter(r => 
      r.status === 404 && (
        r.url.includes('_next/static') || 
        r.url.includes('.css') || 
        r.url.includes('.js')
      )
    );

    // Check styling by examining computed styles
    const stylingAnalysis = await page.evaluate(() => {
      const body = document.body;
      const computedStyle = window.getComputedStyle(body);
      
      return {
        backgroundColor: computedStyle.backgroundColor,
        color: computedStyle.color,
        fontFamily: computedStyle.fontFamily,
        hasClasses: document.querySelectorAll('[class]').length > 0,
        totalElements: document.querySelectorAll('*').length,
        styledElements: document.querySelectorAll('[class], [style]').length,
        hasVisibleContent: document.body.innerText.length > 100,
        tailwindClasses: document.querySelectorAll('[class*="bg-"], [class*="text-"], [class*="font-"]').length
      };
    });

    // Log detailed results
    console.log('\n📊 Frontend CSS and Asset Loading Results');
    console.log('='.repeat(50));
    console.log(`⏱️  Page Load Time: ${loadTime}ms`);
    console.log(`📁 Total Requests: ${allRequests.length}`);
    console.log(`📁 Total Responses: ${allResponses.length}`);
    console.log(`🎨 CSS Files: ${cssResponses.length}`);
    console.log(`📜 JS Files: ${jsResponses.length}`);
    console.log(`🏗️  Static Assets: ${staticAssetResponses.length}`);
    console.log(`❌ Failed Requests: ${failedRequests.length}`);
    console.log(`🔥 404 Asset Errors: ${failed404Assets.length}`);
    console.log(`🚨 Console Errors: ${consoleErrors.length}`);

    // Detail CSS file status
    if (cssResponses.length > 0) {
      console.log('\n🎨 CSS File Status Details:');
      cssResponses.forEach(css => {
        const status = css.status === 200 ? '✅' : '❌';
        const fileName = css.url.split('/').pop() || 'unknown';
        console.log(`  ${status} ${css.status} - ${fileName}`);
      });
    } else {
      console.log('\n❌ No CSS files detected!');
    }

    // Detail JS file status (first 5)
    if (jsResponses.length > 0) {
      console.log('\n📜 JavaScript File Status (Top 5):');
      jsResponses.slice(0, 5).forEach(js => {
        const status = js.status === 200 ? '✅' : '❌';
        const fileName = js.url.split('/').pop() || 'unknown';
        console.log(`  ${status} ${js.status} - ${fileName}`);
      });
    }

    // Show 404 errors
    if (failed404Assets.length > 0) {
      console.log('\n🚨 404 Asset Errors:');
      failed404Assets.forEach(asset => {
        console.log(`  ❌ ${asset.url}`);
      });
    }

    // Show console errors
    if (consoleErrors.length > 0) {
      console.log('\n🚨 Console Errors:');
      consoleErrors.slice(0, 3).forEach(error => {
        console.log(`  ❌ ${error}`);
      });
    }

    // Styling analysis
    console.log('\n🎨 Visual Styling Analysis:');
    console.log(`  Background Color: ${stylingAnalysis.backgroundColor}`);
    console.log(`  Text Color: ${stylingAnalysis.color}`);
    console.log(`  Font Family: ${stylingAnalysis.fontFamily}`);
    console.log(`  Has Classes: ${stylingAnalysis.hasClasses ? '✅' : '❌'}`);
    console.log(`  Styled Elements: ${stylingAnalysis.styledElements}/${stylingAnalysis.totalElements}`);
    console.log(`  Has Content: ${stylingAnalysis.hasVisibleContent ? '✅' : '❌'}`);
    console.log(`  Tailwind Classes: ${stylingAnalysis.tailwindClasses}`);

    // Determine overall success
    const cssSuccess = cssResponses.length > 0 && cssResponses.some(r => r.status === 200);
    const noAsset404s = failed404Assets.length === 0;
    const hasGoodStyling = stylingAnalysis.hasClasses && stylingAnalysis.styledElements > 0;
    
    const overallSuccess = cssSuccess && noAsset404s && hasGoodStyling;

    console.log(`\n🎯 Frontend Result: ${overallSuccess ? '✅ SUCCESS' : '❌ FAILED'}`);
    
    if (overallSuccess) {
      console.log('  - CSS files are loading correctly');
      console.log('  - No 404 errors for static assets');  
      console.log('  - Page has proper styling');
    } else {
      if (!cssSuccess) console.log('  - ❌ CSS files not loading properly');
      if (!noAsset404s) console.log('  - ❌ 404 errors found for assets');
      if (!hasGoodStyling) console.log('  - ❌ Page styling issues detected');
    }

    // Assertions
    expect(cssResponses.length).toBeGreaterThan(0);
    expect(cssResponses.some(r => r.status === 200)).toBe(true);
    expect(failed404Assets.length).toBe(0);
    expect(stylingAnalysis.hasClasses).toBe(true);
  });

  test('Admin: CSS and assets loading verification', async ({ page }) => {
    console.log('\n🚀 Testing Admin CSS and Asset Loading...');
    console.log(`URL: ${ADMIN_URL}`);

    // Track all network requests
    const allRequests: Array<{url: string, method: string, resourceType: string}> = [];
    const allResponses: Array<{url: string, status: number, statusText: string}> = [];
    const failedRequests: Array<{url: string, status: number, failure?: string}> = [];
    const consoleErrors: string[] = [];

    // Set up request/response tracking
    page.on('request', request => {
      allRequests.push({
        url: request.url(),
        method: request.method(),
        resourceType: request.resourceType()
      });
    });

    page.on('response', response => {
      allResponses.push({
        url: response.url(),
        status: response.status(),
        statusText: response.statusText()
      });

      // Track failed responses
      if (response.status() >= 400) {
        failedRequests.push({
          url: response.url(),
          status: response.status()
        });
      }
    });

    page.on('requestfailed', request => {
      failedRequests.push({
        url: request.url(),
        status: 0,
        failure: request.failure()?.errorText
      });
    });

    // Track console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    const startTime = Date.now();

    // Navigate to admin
    await page.goto(ADMIN_URL, { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    const loadTime = Date.now() - startTime;

    // Take screenshot
    await page.screenshot({ 
      path: '.playwright-mcp/admin-css-verification.png',
      fullPage: true 
    });

    // Analyze CSS files
    const cssResponses = allResponses.filter(r => 
      r.url.includes('.css') || r.url.includes('_next/static/css')
    );
    
    const jsResponses = allResponses.filter(r => 
      r.url.includes('.js') && r.url.includes('_next/static')
    );

    const staticAssetResponses = allResponses.filter(r => 
      r.url.includes('_next/static/') || r.url.includes('/static/')
    );

    // Check for 404 errors specifically on assets
    const failed404Assets = failedRequests.filter(r => 
      r.status === 404 && (
        r.url.includes('_next/static') || 
        r.url.includes('.css') || 
        r.url.includes('.js')
      )
    );

    // Check styling by examining computed styles
    const stylingAnalysis = await page.evaluate(() => {
      const body = document.body;
      const computedStyle = window.getComputedStyle(body);
      
      return {
        backgroundColor: computedStyle.backgroundColor,
        color: computedStyle.color,
        fontFamily: computedStyle.fontFamily,
        hasClasses: document.querySelectorAll('[class]').length > 0,
        totalElements: document.querySelectorAll('*').length,
        styledElements: document.querySelectorAll('[class], [style]').length,
        hasVisibleContent: document.body.innerText.length > 100,
        tailwindClasses: document.querySelectorAll('[class*="bg-"], [class*="text-"], [class*="font-"]').length
      };
    });

    // Log detailed results
    console.log('\n📊 Admin CSS and Asset Loading Results');
    console.log('='.repeat(50));
    console.log(`⏱️  Page Load Time: ${loadTime}ms`);
    console.log(`📁 Total Requests: ${allRequests.length}`);
    console.log(`📁 Total Responses: ${allResponses.length}`);
    console.log(`🎨 CSS Files: ${cssResponses.length}`);
    console.log(`📜 JS Files: ${jsResponses.length}`);
    console.log(`🏗️  Static Assets: ${staticAssetResponses.length}`);
    console.log(`❌ Failed Requests: ${failedRequests.length}`);
    console.log(`🔥 404 Asset Errors: ${failed404Assets.length}`);
    console.log(`🚨 Console Errors: ${consoleErrors.length}`);

    // Detail CSS file status
    if (cssResponses.length > 0) {
      console.log('\n🎨 CSS File Status Details:');
      cssResponses.forEach(css => {
        const status = css.status === 200 ? '✅' : '❌';
        const fileName = css.url.split('/').pop() || 'unknown';
        console.log(`  ${status} ${css.status} - ${fileName}`);
      });
    } else {
      console.log('\n❌ No CSS files detected!');
    }

    // Detail JS file status (first 5)
    if (jsResponses.length > 0) {
      console.log('\n📜 JavaScript File Status (Top 5):');
      jsResponses.slice(0, 5).forEach(js => {
        const status = js.status === 200 ? '✅' : '❌';
        const fileName = js.url.split('/').pop() || 'unknown';
        console.log(`  ${status} ${js.status} - ${fileName}`);
      });
    }

    // Show 404 errors
    if (failed404Assets.length > 0) {
      console.log('\n🚨 404 Asset Errors:');
      failed404Assets.forEach(asset => {
        console.log(`  ❌ ${asset.url}`);
      });
    }

    // Show console errors (first 3)
    if (consoleErrors.length > 0) {
      console.log('\n🚨 Console Errors:');
      consoleErrors.slice(0, 3).forEach(error => {
        console.log(`  ❌ ${error}`);
      });
    }

    // Styling analysis
    console.log('\n🎨 Visual Styling Analysis:');
    console.log(`  Background Color: ${stylingAnalysis.backgroundColor}`);
    console.log(`  Text Color: ${stylingAnalysis.color}`);
    console.log(`  Font Family: ${stylingAnalysis.fontFamily}`);
    console.log(`  Has Classes: ${stylingAnalysis.hasClasses ? '✅' : '❌'}`);
    console.log(`  Styled Elements: ${stylingAnalysis.styledElements}/${stylingAnalysis.totalElements}`);
    console.log(`  Has Content: ${stylingAnalysis.hasVisibleContent ? '✅' : '❌'}`);
    console.log(`  Tailwind Classes: ${stylingAnalysis.tailwindClasses}`);

    // Determine overall success
    const cssSuccess = cssResponses.length > 0 && cssResponses.some(r => r.status === 200);
    const noAsset404s = failed404Assets.length === 0;
    const hasGoodStyling = stylingAnalysis.hasClasses && stylingAnalysis.styledElements > 0;
    
    const overallSuccess = cssSuccess && noAsset404s && hasGoodStyling;

    console.log(`\n🎯 Admin Result: ${overallSuccess ? '✅ SUCCESS' : '❌ FAILED'}`);
    
    if (overallSuccess) {
      console.log('  - CSS files are loading correctly');
      console.log('  - No 404 errors for static assets');  
      console.log('  - Page has proper styling');
    } else {
      if (!cssSuccess) console.log('  - ❌ CSS files not loading properly');
      if (!noAsset404s) console.log('  - ❌ 404 errors found for assets');
      if (!hasGoodStyling) console.log('  - ❌ Page styling issues detected');
    }

    // Assertions
    expect(cssResponses.length).toBeGreaterThan(0);
    expect(cssResponses.some(r => r.status === 200)).toBe(true);
    expect(failed404Assets.length).toBe(0);
    expect(stylingAnalysis.hasClasses).toBe(true);
  });

  test('Performance comparison analysis', async ({ page }) => {
    console.log('\n🏁 Running Performance Comparison Analysis...');

    const results = {
      frontend: { loadTime: 0, cssFiles: 0, totalRequests: 0 },
      admin: { loadTime: 0, cssFiles: 0, totalRequests: 0 }
    };

    // Quick test of both services
    for (const service of [
      { name: 'frontend', url: FRONTEND_URL },
      { name: 'admin', url: ADMIN_URL }
    ]) {
      const requests: any[] = [];
      const responses: any[] = [];

      page.on('request', r => requests.push(r));
      page.on('response', r => responses.push(r));

      const startTime = Date.now();
      await page.goto(service.url, { waitUntil: 'networkidle', timeout: 20000 });
      const loadTime = Date.now() - startTime;

      const cssCount = responses.filter(r => r.url().includes('.css')).length;

      results[service.name as keyof typeof results] = {
        loadTime,
        cssFiles: cssCount,
        totalRequests: requests.length
      };

      // Clear listeners for next iteration
      page.removeAllListeners('request');
      page.removeAllListeners('response');
    }

    console.log('\n📈 Performance Comparison Results');
    console.log('='.repeat(40));
    console.log(`Frontend Load Time: ${results.frontend.loadTime}ms`);
    console.log(`Admin Load Time:    ${results.admin.loadTime}ms`);
    console.log(`Frontend CSS Files: ${results.frontend.cssFiles}`);
    console.log(`Admin CSS Files:    ${results.admin.cssFiles}`);
    console.log(`Frontend Requests:  ${results.frontend.totalRequests}`);
    console.log(`Admin Requests:     ${results.admin.totalRequests}`);

    const avgLoadTime = (results.frontend.loadTime + results.admin.loadTime) / 2;
    const totalCssFiles = results.frontend.cssFiles + results.admin.cssFiles;

    console.log(`\n🎯 Summary:`);
    console.log(`  Average Load Time: ${avgLoadTime.toFixed(0)}ms`);
    console.log(`  Total CSS Files Loading: ${totalCssFiles}`);
    console.log(`  Both Services Status: ${totalCssFiles > 0 ? '✅ CSS Loading' : '❌ CSS Issues'}`);

    // Success if both services have reasonable load times and CSS files
    expect(results.frontend.loadTime).toBeLessThan(15000);
    expect(results.admin.loadTime).toBeLessThan(15000);
    expect(results.frontend.cssFiles).toBeGreaterThan(0);
    expect(results.admin.cssFiles).toBeGreaterThan(0);
  });
});