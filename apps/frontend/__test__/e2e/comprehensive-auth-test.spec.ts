import { test, expect, Page } from '@playwright/test';

// Test credentials - generic test account
const TEST_CREDENTIALS = {
  email: 'test.user@example.com',
  password: 'TestPassword123!'
};

// Test URLs
const FRONTEND_URL = 'http://localhost:3000';
const ADMIN_URL = 'http://localhost:3001';

test.describe('Comprehensive Authentication Tests', () => {
  
  test.describe('Frontend App Authentication', () => {
    
    test('should login and logout successfully', async ({ page }) => {
      // Navigate to frontend
      await page.goto(FRONTEND_URL);
      
      // Check if already logged in
      const signOutButton = page.getByRole('button', { name: 'Sign Out' });
      if (await signOutButton.isVisible()) {
        await signOutButton.click();
        await expect(page).toHaveURL(/.*\/login.*/);
      }
      
      // Wait for login page and perform login
      await expect(page.getByRole('button', { name: 'Continue with EPSX' })).toBeVisible();
      await page.getByRole('button', { name: 'Continue with EPSX' }).click();
      
      // Wait for OAuth redirect to backend
      await expect(page).toHaveURL(/.*localhost:8080.*oauth\/authorize.*/);
      await expect(page.locator('h1')).toContainText('EPSX Login');
      
      // Fill login form
      await page.getByRole('textbox', { name: 'Email' }).fill(TEST_CREDENTIALS.email);
      await page.getByRole('textbox', { name: 'Password' }).fill(TEST_CREDENTIALS.password);
      await page.getByRole('button', { name: 'Sign In' }).click();
      
      // Should redirect back to frontend and be logged in
      await expect(page).toHaveURL(FRONTEND_URL);
      await expect(page.getByRole('button', { name: 'Sign Out' })).toBeVisible();
      await expect(page.getByText('Level 0')).toBeVisible();
      
      // Test logout
      await page.getByRole('button', { name: 'Sign Out' }).click();
      await expect(page).toHaveURL(/.*\/login.*/);
      await expect(page.getByRole('button', { name: 'Continue with EPSX' })).toBeVisible();
    });
    
    test('should show proper navigation for authenticated user', async ({ page }) => {
      // Login first
      await loginToFrontend(page);
      
      // Check navigation elements
      await expect(page.getByRole('link', { name: 'EPSX' })).toBeVisible();
      await expect(page.getByRole('link', { name: 'Settings' })).toBeVisible();
      await expect(page.getByRole('button', { name: 'Sign Out' })).toBeVisible();
      
      // Check analytics and portfolio links
      const analyticsLink = page.locator('a[href="/analytics"]').first();
      const portfolioLink = page.locator('a[href="/portfolio"]').first();
      await expect(analyticsLink).toBeVisible();
      await expect(portfolioLink).toBeVisible();
    });
    
  });
  
  test.describe('Admin App Authentication', () => {
    
    test('should login and logout successfully', async ({ page }) => {
      // Navigate to admin frontend
      await page.goto(ADMIN_URL);
      
      // Should redirect to login if not authenticated
      await expect(page).toHaveURL(/.*login.*/);
      await expect(page.getByText('Admin Login')).toBeVisible();
      
      // Click signin
      await page.getByRole('link', { name: 'Sign in with EPSX Backend' }).click();
      
      // Wait for OAuth redirect to backend
      await expect(page).toHaveURL(/.*localhost:8080.*oauth\/authorize.*/);
      await expect(page.locator('h1')).toContainText('EPSX Login');
      
      // Verify admin scope
      await expect(page.getByText('epsx-admin-frontend')).toBeVisible();
      await expect(page.getByText('permissions')).toBeVisible();
      
      // Fill login form
      await page.getByRole('textbox', { name: 'Email' }).fill(TEST_CREDENTIALS.email);
      await page.getByRole('textbox', { name: 'Password' }).fill(TEST_CREDENTIALS.password);
      await page.getByRole('button', { name: 'Sign In' }).click();
      
      // Should redirect back to admin and be logged in
      await expect(page).toHaveURL(ADMIN_URL);
      await expect(page.getByText('test.user@example.com')).toBeVisible();
      await expect(page.getByText('10 modules')).toBeVisible();
      await expect(page.getByText('Online')).toBeVisible();
      
      // Test admin dashboard elements
      await expect(page.getByRole('heading', { name: 'Admin Dashboard' })).toBeVisible();
      await expect(page.getByText('Total Users')).toBeVisible();
      await expect(page.getByText('Active Sessions')).toBeVisible();
      
      // Test logout
      const signOutButton = page.getByRole('button', { name: 'Sign out' });
      await signOutButton.scrollIntoViewIfNeeded();
      await signOutButton.click();
      await expect(page).toHaveURL(/.*login.*/);
    });
    
    test('should show admin navigation and modules', async ({ page }) => {
      // Login first
      await loginToAdmin(page);
      
      // Check admin navigation elements
      await expect(page.getByRole('heading', { name: 'EPSX Admin' })).toBeVisible();
      await expect(page.getByRole('link', { name: 'Dashboard' })).toBeVisible();
      
      // Check admin modules
      await expect(page.getByText('User Management')).toBeVisible();
      await expect(page.getByText('Security & Access')).toBeVisible();
      await expect(page.getByText('Analytics & Reports')).toBeVisible();
      await expect(page.getByText('System Management')).toBeVisible();
      
      // Check specific admin links
      await expect(page.getByRole('link', { name: /User Accounts/ })).toBeVisible();
      await expect(page.getByRole('link', { name: /Developer Portal/ })).toBeVisible();
      await expect(page.getByRole('link', { name: /Analytics Dashboard/ })).toBeVisible();
    });
    
  });
  
  test.describe('Cross-App Authentication', () => {
    
    test('should maintain separate sessions between apps', async ({ browser }) => {
      // Create two contexts for testing separate sessions
      const frontendContext = await browser.newContext();
      const adminContext = await browser.newContext();
      
      const frontendPage = await frontendContext.newPage();
      const adminPage = await adminContext.newPage();
      
      // Login to frontend
      await loginToFrontend(frontendPage);
      await expect(frontendPage.getByRole('button', { name: 'Sign Out' })).toBeVisible();
      
      // Login to admin
      await loginToAdmin(adminPage);
      await expect(adminPage.getByText('test.user@example.com')).toBeVisible();
      
      // Both should be logged in independently
      await expect(frontendPage.getByRole('button', { name: 'Sign Out' })).toBeVisible();
      await expect(adminPage.getByText('10 modules')).toBeVisible();
      
      // Logout from frontend shouldn't affect admin
      await frontendPage.getByRole('button', { name: 'Sign Out' }).click();
      await expect(frontendPage).toHaveURL(/.*login.*/);
      await expect(adminPage.getByText('10 modules')).toBeVisible();
      
      await frontendContext.close();
      await adminContext.close();
    });
    
  });
  
});

// Helper functions
async function loginToFrontend(page: Page) {
  await page.goto(FRONTEND_URL);
  
  // Check if already logged in
  const signOutButton = page.getByRole('button', { name: 'Sign Out' });
  if (await signOutButton.isVisible()) {
    return; // Already logged in
  }
  
  // Wait for login page and login
  await expect(page.getByRole('button', { name: 'Continue with EPSX' })).toBeVisible();
  await page.getByRole('button', { name: 'Continue with EPSX' }).click();
  await page.getByRole('textbox', { name: 'Email' }).fill(TEST_CREDENTIALS.email);
  await page.getByRole('textbox', { name: 'Password' }).fill(TEST_CREDENTIALS.password);
  await page.getByRole('button', { name: 'Sign In' }).click();
  
  // Wait for successful login
  await expect(page).toHaveURL(FRONTEND_URL);
  await expect(page.getByRole('button', { name: 'Sign Out' })).toBeVisible();
}

async function loginToAdmin(page: Page) {
  await page.goto(ADMIN_URL);
  
  // Check if already logged in
  if (await page.getByText('test.user@example.com').isVisible()) {
    return; // Already logged in
  }
  
  // Login process
  await page.getByRole('link', { name: 'Sign in with EPSX Backend' }).click();
  await page.getByRole('textbox', { name: 'Email' }).fill(TEST_CREDENTIALS.email);
  await page.getByRole('textbox', { name: 'Password' }).fill(TEST_CREDENTIALS.password);
  await page.getByRole('button', { name: 'Sign In' }).click();
  
  // Wait for successful login
  await expect(page).toHaveURL(ADMIN_URL);
  await expect(page.getByText('test.user@example.com')).toBeVisible();
}