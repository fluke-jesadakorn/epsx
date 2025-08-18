import { test, expect } from '@playwright/test';

// Test credentials
const TEST_CREDENTIALS = {
  email: 'info@epsx.io',
  password: 'P@ssword'
};

const ADMIN_URL = 'http://localhost:3001';

test.describe('Admin Frontend Comprehensive Tests', () => {
  
  test('complete admin authentication flow', async ({ page }) => {
    // Navigate to admin
    await page.goto(ADMIN_URL);
    
    // Should show login page
    await expect(page.getByText('Admin Login')).toBeVisible();
    await expect(page.getByText('Administrative access for EPSX platform')).toBeVisible();
    
    // Start OAuth flow
    await page.getByRole('link', { name: 'Sign in with EPSX Backend' }).click();
    
    // Verify OAuth parameters
    await expect(page).toHaveURL(/.*localhost:8080.*oauth\/authorize.*/);
    await expect(page.getByText('epsx-admin-frontend')).toBeVisible();
    await expect(page.getByText('admin_modules')).toBeVisible();
    await expect(page.getByText('✅ Enabled')).toBeVisible(); // PKCE
    
    // Login
    await page.getByRole('textbox', { name: 'Email' }).fill(TEST_CREDENTIALS.email);
    await page.getByRole('textbox', { name: 'Password' }).fill(TEST_CREDENTIALS.password);
    await page.getByRole('button', { name: 'Sign In' }).click();
    
    // Should be logged into admin dashboard
    await expect(page).toHaveURL(ADMIN_URL);
    await expect(page.getByRole('heading', { name: 'Admin Dashboard' })).toBeVisible();
    await expect(page.getByText('info@epsx.io')).toBeVisible();
    await expect(page.getByText('10 modules')).toBeVisible();
    
    // Test dashboard stats
    await expect(page.getByText('Total Users')).toBeVisible();
    await expect(page.getByText('Active Sessions')).toBeVisible();
    await expect(page.getByText('System Health')).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Performance' })).toBeVisible();
    
    // Test navigation menu
    await expect(page.getByRole('button', { name: 'User Management 1 items' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Security & Access 1 items' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Analytics & Reports 1 items' })).toBeVisible();
    await expect(page.getByText('System Management')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Configuration 1 items' })).toBeVisible();
  });
  
  test('admin navigation and module access', async ({ page }) => {
    // Login first
    await page.goto(ADMIN_URL);
    await page.getByRole('link', { name: 'Sign in with EPSX Backend' }).click();
    await page.getByRole('textbox', { name: 'Email' }).fill(TEST_CREDENTIALS.email);
    await page.getByRole('textbox', { name: 'Password' }).fill(TEST_CREDENTIALS.password);
    await page.getByRole('button', { name: 'Sign In' }).click();
    await expect(page).toHaveURL(ADMIN_URL);
    
    // Test Quick Actions
    await expect(page.getByRole('link', { name: 'Manage Users' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'View Analytics' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'System Settings' })).toBeVisible();
    
    // Test System Status
    await expect(page.getByText('API Server')).toBeVisible();
    await expect(page.getByRole('link', { name: 'Database Database management' })).toBeVisible();
    await expect(page.getByText('Memory Usage')).toBeVisible();
    
    // Test admin module links work
    await page.getByRole('link', { name: /User Accounts/ }).click();
    await expect(page).toHaveURL(/.*\/users.*/);
  });
  
  test('admin logout functionality works correctly', async ({ page }) => {
    // Use the already-authenticated session from manual testing
    await page.goto(ADMIN_URL);
    
    // Verify we're on the admin dashboard (user should be logged in from manual test)
    const isLoggedIn = await page.getByText('info@epsx.io').isVisible().catch(() => false);
    
    if (!isLoggedIn) {
      // If not logged in, perform quick login
      await expect(page.getByText('Admin Login')).toBeVisible();
      await page.getByRole('link', { name: 'Sign in with EPSX Backend' }).click();
      await page.waitForURL(/.*localhost:8080.*oauth.*/, { timeout: 5000 });
      await page.getByRole('textbox', { name: 'Email' }).fill(TEST_CREDENTIALS.email);
      await page.getByRole('textbox', { name: 'Password' }).fill(TEST_CREDENTIALS.password);
      await page.getByRole('button', { name: 'Sign In' }).click();
      await page.waitForURL(ADMIN_URL, { timeout: 5000 });
    }
    
    // Verify we're logged in
    await expect(page.getByText('info@epsx.io')).toBeVisible();
    
    // Test logout using JavaScript since the button can be outside viewport
    await page.evaluate(() => {
      const buttons = Array.from(document.querySelectorAll('button'));
      const signOutButton = buttons.find(btn => btn.textContent?.includes('Sign out'));
      if (signOutButton) {
        signOutButton.click();
      }
    });
    
    // Verify logout redirects to login page
    await page.waitForURL(/.*\/login.*/, { timeout: 5000 });
    await expect(page.getByText('Admin Login')).toBeVisible();
    
    // Verify logout worked by trying to access protected route
    await page.goto(ADMIN_URL);
    await expect(page).toHaveURL(/.*\/login.*/);
  });
  
  test('admin breadcrumb navigation', async ({ page }) => {
    // Login and test breadcrumbs
    await page.goto(ADMIN_URL);
    await page.getByRole('link', { name: 'Sign in with EPSX Backend' }).click();
    await page.getByRole('textbox', { name: 'Email' }).fill(TEST_CREDENTIALS.email);
    await page.getByRole('textbox', { name: 'Password' }).fill(TEST_CREDENTIALS.password);
    await page.getByRole('button', { name: 'Sign In' }).click();
    await expect(page).toHaveURL(ADMIN_URL);
    
    // Check breadcrumb elements
    await expect(page.getByRole('navigation', { name: 'Breadcrumb' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'Admin' })).toBeVisible();
    await expect(page.getByLabel('Breadcrumb').getByText('Dashboard')).toBeVisible();
    
    // Check date display
    await expect(page.getByText(/Mon, Aug 18, 2025/)).toBeVisible();
  });
  
  test('admin user profile display', async ({ page }) => {
    // Login
    await page.goto(ADMIN_URL);
    await page.getByRole('link', { name: 'Sign in with EPSX Backend' }).click();
    await page.getByRole('textbox', { name: 'Email' }).fill(TEST_CREDENTIALS.email);
    await page.getByRole('textbox', { name: 'Password' }).fill(TEST_CREDENTIALS.password);
    await page.getByRole('button', { name: 'Sign In' }).click();
    await expect(page).toHaveURL(ADMIN_URL);
    
    // Check user profile information
    await expect(page.getByText('info@epsx.io')).toBeVisible();
    await expect(page.getByText('10 modules')).toBeVisible();
    await expect(page.locator('div').filter({ hasText: /^Online$/ }).locator('span')).toBeVisible();
    
    // Check user avatar/initial in user profile section
    await expect(page.locator('[class*="rounded-full"]').getByText('I')).toBeVisible(); // User initial
  });
  
});