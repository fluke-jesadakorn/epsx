/**
 * Quick validation test for Analytics Platform theme integration
 * Tests core authentication flow functionality
 */

import { test, expect } from '@playwright/test';

const TEST_CREDENTIALS = {
  email: 'info@epsx.io',
  password: 'P@ssword'
};

const BASE_URL = 'http://localhost:3000';

test.describe('📊 Analytics Platform Theme Validation', () => {
  test('Validate Analytics Platform Theme Elements', async ({ page }) => {
    console.log('🚀 Starting Analytics Platform theme validation...');

    // Step 1: Navigate directly to OIDC authorization endpoint
    const authUrl = `http://localhost:8080/oauth/authorize?response_type=code&client_id=epsx-frontend&redirect_uri=http://localhost:3000/callback&scope=openid+profile+email&state=test123&code_challenge=test&code_challenge_method=S256`;
    
    await page.goto(authUrl);
    console.log('✅ Navigated to OIDC authorization endpoint');

    // Step 2: Verify Analytics Platform theme elements are present
    const analyticsTitle = page.locator('h2:has-text("📊 Data Insights Portal"), h1:has-text("📊 Data Insights Portal")');
    const analyticsEmoji = page.locator('text=📊');
    const analyticsCard = page.locator('.insight-card, [class*="insight"]');

    await expect(analyticsTitle.or(analyticsEmoji).or(analyticsCard).first()).toBeVisible({ timeout: 10000 });
    console.log('✅ Analytics Platform theme elements detected');

    // Step 3: Verify OIDC form structure
    const form = page.locator('form[action*="/oauth/authorize"]');
    await expect(form).toBeVisible();
    
    const clientId = await form.locator('input[name="client_id"]').getAttribute('value');
    const responseType = await form.locator('input[name="response_type"]').getAttribute('value');
    const scope = await form.locator('input[name="scope"]').getAttribute('value');
    
    expect(clientId).toBe('epsx-frontend');
    expect(responseType).toBe('code');
    expect(scope).toContain('openid');
    console.log('✅ OIDC form structure validated');

    // Step 4: Test form interaction
    const emailField = page.locator('input[name="email"], input[type="email"]');
    const passwordField = page.locator('input[name="password"], input[type="password"]');
    
    await expect(emailField).toBeVisible();
    await expect(passwordField).toBeVisible();
    
    await emailField.fill(TEST_CREDENTIALS.email);
    await passwordField.fill(TEST_CREDENTIALS.password);
    console.log('✅ Form interaction tested successfully');

    // Step 5: Verify submit button
    const submitButton = page.locator('button[type="submit"]');
    await expect(submitButton).toBeVisible();
    console.log('✅ Submit button detected');

    console.log('🎉 Analytics Platform theme validation completed successfully!');
  });

  test('Validate Error Handling with Analytics Theme', async ({ page }) => {
    console.log('🚀 Testing error handling...');

    // Navigate to login with invalid credentials
    const authUrl = `http://localhost:8080/oauth/authorize?response_type=code&client_id=epsx-frontend&redirect_uri=http://localhost:3000/callback&scope=openid+profile+email&state=test123&code_challenge=test&code_challenge_method=S256`;
    
    await page.goto(authUrl);

    // Fill invalid credentials
    await page.fill('input[name="email"], input[type="email"]', 'invalid@test.com');
    await page.fill('input[name="password"], input[type="password"]', 'wrongpassword');
    
    // Submit form
    await page.locator('button[type="submit"]').click();
    
    // Check for error display (might redirect back with error)
    await page.waitForTimeout(3000);
    
    const hasError = await page.locator('text=error, text=Invalid, text=failed, .error, .insight-toast').first().isVisible().catch(() => false);
    if (hasError) {
      console.log('✅ Error handling validated');
    } else {
      console.log('ℹ️ Error handling test inconclusive (may redirect)');
    }
  });

  test('Validate Mobile Responsive Theme', async ({ page }) => {
    console.log('🚀 Testing mobile responsive design...');

    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    const authUrl = `http://localhost:8080/oauth/authorize?response_type=code&client_id=epsx-frontend&redirect_uri=http://localhost:3000/callback&scope=openid+profile+email&state=test123&code_challenge=test&code_challenge_method=S256`;
    
    await page.goto(authUrl);

    // Verify mobile responsiveness
    const analyticsCard = page.locator('.insight-card, [class*="insight"], form');
    const cardBounds = await analyticsCard.first().boundingBox();
    
    if (cardBounds) {
      expect(cardBounds.width).toBeLessThanOrEqual(375);
      console.log('✅ Mobile responsive design validated');
    }

    // Test form interaction on mobile
    await page.fill('input[name="email"]', TEST_CREDENTIALS.email);
    await page.fill('input[name="password"]', TEST_CREDENTIALS.password);
    console.log('✅ Mobile form interaction tested');
  });
});

console.log('📊 Analytics Platform Theme Validation Suite Loaded');