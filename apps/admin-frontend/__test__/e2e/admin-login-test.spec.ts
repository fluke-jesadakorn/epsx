import { test, expect, Page } from '@playwright/test';

test.describe('Admin Login Flow Testing', () => {
  test.beforeEach(async ({ page }) => {
    // Enable console logging to capture frontend logs
    page.on('console', (msg) => {
      if (msg.type() === 'error' || msg.type() === 'warning') {
        console.log(`🟡 Console ${msg.type()}: ${msg.text()}`);
      } else if (msg.text().includes('Admin') || msg.text().includes('OIDC') || msg.text().includes('OAuth')) {
        console.log(`🔵 Console log: ${msg.text()}`);
      }
    });

    // Enable network request logging
    page.on('request', (request) => {
      const url = request.url();
      if (url.includes('oauth') || url.includes('auth') || url.includes('login') || url.includes('token')) {
        console.log(`🌐 Request: ${request.method()} ${url}`);
      }
    });

    page.on('response', (response) => {
      const url = response.url();
      if (url.includes('oauth') || url.includes('auth') || url.includes('login') || url.includes('token')) {
        console.log(`🌐 Response: ${response.status()} ${url}`);
      }
    });
  });

  test('should navigate to admin login page successfully', async ({ page }) => {
    console.log('🔄 Step 1: Navigating to admin frontend root');
    
    // Navigate to admin frontend
    await page.goto('http://localhost:3001');
    
    // Wait for the page to load
    await page.waitForLoadState('networkidle');
    
    // Capture screenshot
    await page.screenshot({ path: '.debug/step1-admin-root.png', fullPage: true });
    
    // Check if we're on login page or if we get redirected to login
    const currentUrl = page.url();
    console.log('✅ Current URL after navigation:', currentUrl);
    
    // Check for login elements
    const hasLoginButton = await page.locator('button:has-text("Continue with Google")').isVisible().catch(() => false);
    const hasOAuthRedirect = currentUrl.includes('oauth') || currentUrl.includes('login');
    
    console.log('🔍 Login page indicators:');
    console.log('  - Has Google login button:', hasLoginButton);
    console.log('  - URL indicates OAuth/login:', hasOAuthRedirect);
    
    expect(hasLoginButton || hasOAuthRedirect).toBe(true);
  });

  test('should initiate OAuth flow when clicking login', async ({ page }) => {
    console.log('🔄 Step 2: Testing OAuth flow initiation');
    
    // Navigate to admin frontend
    await page.goto('http://localhost:3001');
    await page.waitForLoadState('networkidle');
    
    // Look for login/OAuth related elements
    const googleLoginButton = page.locator('button:has-text("Continue with Google")');
    const loginLink = page.locator('a[href*="login"]');
    const oauthLink = page.locator('a[href*="oauth"]');
    
    let clickTarget = null;
    
    if (await googleLoginButton.isVisible().catch(() => false)) {
      clickTarget = googleLoginButton;
      console.log('📍 Found Google login button');
    } else if (await loginLink.isVisible().catch(() => false)) {
      clickTarget = loginLink;
      console.log('📍 Found login link');
    } else if (await oauthLink.isVisible().catch(() => false)) {
      clickTarget = oauthLink;
      console.log('📍 Found OAuth link');
    } else {
      // Check if already in login flow
      if (page.url().includes('oauth') || page.url().includes('login')) {
        console.log('📍 Already in OAuth/login flow');
        return;
      }
      
      // Look for any button or link that might start the login process
      const allButtons = await page.locator('button, a').all();
      for (const button of allButtons) {
        const text = await button.textContent().catch(() => '');
        if (text && (text.includes('Login') || text.includes('Sign') || text.includes('Auth'))) {
          clickTarget = button;
          console.log('📍 Found potential login trigger:', text);
          break;
        }
      }
    }
    
    if (clickTarget) {
      console.log('🔄 Clicking login trigger...');
      
      // Set up to capture the navigation
      const navigationPromise = page.waitForNavigation({ timeout: 15000 }).catch((error) => {
        console.log('⚠️ Navigation timeout or error:', error.message);
        return null;
      });
      
      await clickTarget.click();
      
      // Wait for navigation to complete
      await navigationPromise;
      
      const newUrl = page.url();
      console.log('✅ URL after login click:', newUrl);
      
      // Capture screenshot
      await page.screenshot({ path: '.debug/step2-after-login-click.png', fullPage: true });
      
      // Check if we're in OAuth flow
      const isInOAuthFlow = newUrl.includes('oauth') || newUrl.includes('authorize') || newUrl.includes('8080');
      console.log('🔍 In OAuth flow:', isInOAuthFlow);
      
      if (isInOAuthFlow) {
        console.log('✅ OAuth flow initiated successfully');
        
        // Wait a bit to see what happens
        await page.waitForTimeout(2000);
        
        const finalUrl = page.url();
        console.log('✅ Final URL after OAuth flow:', finalUrl);
        
        // Capture final screenshot
        await page.screenshot({ path: '.debug/step2-oauth-flow-result.png', fullPage: true });
      }
    } else {
      console.log('❌ No login trigger found');
      await page.screenshot({ path: '.debug/step2-no-login-trigger.png', fullPage: true });
      
      // Log all visible elements for debugging
      const allText = await page.textContent('body');
      console.log('📄 Page content preview:', allText?.slice(0, 500));
    }
  });

  test('should complete OAuth flow and handle redirect', async ({ page }) => {
    console.log('🔄 Step 3: Testing complete OAuth flow with redirect');
    
    // Start at admin root
    await page.goto('http://localhost:3001');
    await page.waitForLoadState('networkidle');
    
    let currentUrl = page.url();
    console.log('🌍 Starting URL:', currentUrl);
    
    // Check if we need to start OAuth flow
    if (!currentUrl.includes('oauth') && !currentUrl.includes('login')) {
      console.log('🔄 Not in OAuth flow, looking for login trigger...');
      
      // Look for any way to start login
      const possibleTriggers = [
        'button:has-text("Continue with Google")',
        'button:has-text("Login")',
        'button:has-text("Sign")',
        'a[href*="login"]',
        'a[href*="oauth"]',
        'button[type="submit"]'
      ];
      
      let triggered = false;
      for (const selector of possibleTriggers) {
        const element = page.locator(selector);
        if (await element.isVisible().catch(() => false)) {
          console.log(`🎯 Found trigger: ${selector}`);
          await element.click();
          triggered = true;
          await page.waitForTimeout(2000);
          break;
        }
      }
      
      if (!triggered) {
        console.log('❌ Could not find OAuth trigger, checking current page state...');
        await page.screenshot({ path: '.debug/step3-no-oauth-trigger.png', fullPage: true });
        
        // Check if we're already authenticated
        const hasAdminContent = await page.locator('text=EPSX Admin').isVisible().catch(() => false);
        if (hasAdminContent) {
          console.log('✅ Already authenticated and on admin dashboard');
          return;
        }
      }
    }
    
    // Monitor for redirects and final destination
    let redirectCount = 0;
    const maxRedirects = 10;
    
    while (redirectCount < maxRedirects) {
      await page.waitForTimeout(2000);
      const newUrl = page.url();
      
      if (newUrl !== currentUrl) {
        redirectCount++;
        console.log(`🔄 Redirect ${redirectCount}: ${currentUrl} → ${newUrl}`);
        currentUrl = newUrl;
        
        // Capture screenshot at each redirect
        await page.screenshot({ 
          path: `.debug/step3-redirect-${redirectCount}.png`, 
          fullPage: true 
        });
        
        // Check if we're back at admin frontend
        if (newUrl.includes('localhost:3001') && !newUrl.includes('login')) {
          console.log('✅ Redirected back to admin frontend');
          break;
        }
        
        // Wait for page to stabilize
        await page.waitForLoadState('networkidle').catch(() => {
          console.log('⚠️ Page did not reach network idle state');
        });
      } else {
        break;
      }
    }
    
    // Final state analysis
    const finalUrl = page.url();
    console.log('🏁 Final URL:', finalUrl);
    
    // Check what's on the final page
    const pageTitle = await page.title();
    const hasAdminContent = await page.locator('text=EPSX Admin').isVisible().catch(() => false);
    const hasErrorContent = await page.locator('text=error, text=Error').isVisible().catch(() => false);
    const hasLoginContent = await page.locator('button:has-text("Login"), button:has-text("Sign")').isVisible().catch(() => false);
    
    console.log('🔍 Final page analysis:');
    console.log('  - Page title:', pageTitle);
    console.log('  - Has admin content:', hasAdminContent);
    console.log('  - Has error content:', hasErrorContent);
    console.log('  - Has login content:', hasLoginContent);
    console.log('  - Total redirects:', redirectCount);
    
    // Capture final screenshot
    await page.screenshot({ path: '.debug/step3-final-state.png', fullPage: true });
    
    // Log final page content
    const bodyText = await page.textContent('body');
    console.log('📄 Final page content preview:', bodyText?.slice(0, 1000));
    
    // Check for specific redirect issues
    if (redirectCount >= maxRedirects) {
      console.log('❌ Too many redirects detected - possible redirect loop');
    } else if (hasErrorContent) {
      console.log('❌ Error content found on final page');
    } else if (hasLoginContent && redirectCount > 0) {
      console.log('⚠️ Redirected back to login page - authentication may have failed');
    } else if (hasAdminContent) {
      console.log('✅ Successfully reached admin dashboard');
    } else {
      console.log('❓ Unknown final state');
    }
  });
});