import { test, expect, Page } from '@playwright/test';

test.describe('Admin Login Flow Testing', () => {
  test.beforeEach(async ({ page }) => {
    // Enable console logging to capture frontend logs
    page.on('console', (msg) => {
      if (msg.type() === 'error' || msg.type() === 'warning') {
      } else if (msg.text().includes('Admin') || msg.text().includes('OIDC') || msg.text().includes('OAuth')) {
      }
    });

    // Enable network request logging
    page.on('request', (request) => {
      const url = request.url();
      if (url.includes('oauth') || url.includes('auth') || url.includes('login') || url.includes('token')) {
      }
    });

    page.on('response', (response) => {
      const url = response.url();
      if (url.includes('oauth') || url.includes('auth') || url.includes('login') || url.includes('token')) {
      }
    });
  });

  test('should navigate to admin login page successfully', async ({ page }) => {
    
    // Navigate to admin frontend
    await page.goto('http://localhost:3001');
    
    // Wait for the page to load
    await page.waitForLoadState('networkidle');
    
    // Capture screenshot
    await page.screenshot({ path: '.debug/step1-admin-root.png', fullPage: true });
    
    // Check if we're on login page or if we get redirected to login
    const currentUrl = page.url();
    
    // Check for login elements
    const hasLoginButton = await page.locator('button:has-text("Continue with Google")').isVisible().catch(() => false);
    const hasOAuthRedirect = currentUrl.includes('oauth') || currentUrl.includes('login');
    
    
    expect(hasLoginButton || hasOAuthRedirect).toBe(true);
  });

  test('should initiate OAuth flow when clicking login', async ({ page }) => {
    
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
    } else if (await loginLink.isVisible().catch(() => false)) {
      clickTarget = loginLink;
    } else if (await oauthLink.isVisible().catch(() => false)) {
      clickTarget = oauthLink;
    } else {
      // Check if already in login flow
      if (page.url().includes('oauth') || page.url().includes('login')) {
        return;
      }
      
      // Look for any button or link that might start the login process
      const allButtons = await page.locator('button, a').all();
      for (const button of allButtons) {
        const text = await button.textContent().catch(() => '');
        if (text && (text.includes('Login') || text.includes('Sign') || text.includes('Auth'))) {
          clickTarget = button;
          break;
        }
      }
    }
    
    if (clickTarget) {
      
      // Set up to capture the navigation
      const navigationPromise = page.waitForNavigation({ timeout: 15000 }).catch((error) => {
        return null;
      });
      
      await clickTarget.click();
      
      // Wait for navigation to complete
      await navigationPromise;
      
      const newUrl = page.url();
      
      // Capture screenshot
      await page.screenshot({ path: '.debug/step2-after-login-click.png', fullPage: true });
      
      // Check if we're in OAuth flow
      const isInOAuthFlow = newUrl.includes('oauth') || newUrl.includes('authorize') || newUrl.includes('8080');
      
      if (isInOAuthFlow) {
        
        // Wait a bit to see what happens
        await page.waitForTimeout(2000);
        
        const finalUrl = page.url();
        
        // Capture final screenshot
        await page.screenshot({ path: '.debug/step2-oauth-flow-result.png', fullPage: true });
      }
    } else {
      await page.screenshot({ path: '.debug/step2-no-login-trigger.png', fullPage: true });
      
      // Log all visible elements for debugging
      const allText = await page.textContent('body');
    }
  });

  test('should complete OAuth flow and handle redirect', async ({ page }) => {
    
    // Start at admin root
    await page.goto('http://localhost:3001');
    await page.waitForLoadState('networkidle');
    
    let currentUrl = page.url();
    
    // Check if we need to start OAuth flow
    if (!currentUrl.includes('oauth') && !currentUrl.includes('login')) {
      
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
          await element.click();
          triggered = true;
          await page.waitForTimeout(2000);
          break;
        }
      }
      
      if (!triggered) {
        await page.screenshot({ path: '.debug/step3-no-oauth-trigger.png', fullPage: true });
        
        // Check if we're already authenticated
        const hasAdminContent = await page.locator('text=EPSX Admin').isVisible().catch(() => false);
        if (hasAdminContent) {
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
        currentUrl = newUrl;
        
        // Capture screenshot at each redirect
        await page.screenshot({ 
          path: `.debug/step3-redirect-${redirectCount}.png`, 
          fullPage: true 
        });
        
        // Check if we're back at admin frontend
        if (newUrl.includes('localhost:3001') && !newUrl.includes('login')) {
          break;
        }
        
        // Wait for page to stabilize
        await page.waitForLoadState('networkidle').catch(() => {
        });
      } else {
        break;
      }
    }
    
    // Final state analysis
    const finalUrl = page.url();
    
    // Check what's on the final page
    const pageTitle = await page.title();
    const hasAdminContent = await page.locator('text=EPSX Admin').isVisible().catch(() => false);
    const hasErrorContent = await page.locator('text=error, text=Error').isVisible().catch(() => false);
    const hasLoginContent = await page.locator('button:has-text("Login"), button:has-text("Sign")').isVisible().catch(() => false);
    
    
    // Capture final screenshot
    await page.screenshot({ path: '.debug/step3-final-state.png', fullPage: true });
    
    // Log final page content
    const bodyText = await page.textContent('body');
    
    // Check for specific redirect issues
    if (redirectCount >= maxRedirects) {
    } else if (hasErrorContent) {
    } else if (hasLoginContent && redirectCount > 0) {
    } else if (hasAdminContent) {
    } else {
    }
  });
});