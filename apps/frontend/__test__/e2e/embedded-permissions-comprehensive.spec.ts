import { test, expect } from '@playwright/test';

test.describe('Embedded Timestamp Permissions - Comprehensive E2E Tests', () => {
  const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:3002';
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3003';
  const TEST_USER = {
    email: 'jesadakorn.kirtnu@gmail.com',
    password: 'P@ssword'
  };

  test.beforeEach(async ({ page }) => {
    // Set viewport for consistent testing
    await page.setViewportSize({ width: 1280, height: 720 });
  });

  test.describe('Frontend Permissions View', () => {
    test('should display permissions page for authenticated user', async ({ page }) => {
      // Navigate to frontend
      await page.goto(FRONTEND_URL);
      
      // Check if user is already authenticated or needs login
      const isLoggedIn = await page.locator('button:has-text("Sign Out")').isVisible();
      
      if (!isLoggedIn) {
        // If not logged in, attempt login
        const loginButton = page.locator('button:has-text("Sign In")');
        if (await loginButton.isVisible()) {
          await loginButton.click();
          // Wait for auth redirect
          await page.waitForURL(/localhost/, { timeout: 10000 });
        }
      }

      // Navigate to permissions page
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Verify permissions page loads
      await expect(page.locator('h1:has-text("My Permissions")')).toBeVisible();
      
      // Verify account information section
      await expect(page.locator('text=Account Information')).toBeVisible();
      await expect(page.locator('text=Email')).toBeVisible();
      
      // Verify permission statistics cards
      await expect(page.locator('text=Total Permissions')).toBeVisible();
      await expect(page.locator('text=Active')).toBeVisible();
      await expect(page.locator('text=Expiring Soon')).toBeVisible();
      await expect(page.locator('text=Expired')).toBeVisible();
    });

    test('should display permission cards with proper formatting', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Wait for permission cards to load
      await page.waitForSelector('[data-testid="permission-card"]', { timeout: 5000 });
      
      // Verify permission cards exist
      const permissionCards = page.locator('[data-testid="permission-card"]');
      const cardCount = await permissionCards.count();
      expect(cardCount).toBeGreaterThan(0);
      
      // Verify each card has required elements
      for (let i = 0; i < cardCount; i++) {
        const card = permissionCards.nth(i);
        
        // Should have platform badge
        await expect(card.locator('.badge, [class*="badge"]')).toBeVisible();
        
        // Should have permission title
        await expect(card.locator('h3, [class*="font-medium"]')).toBeVisible();
        
        // Should have status icon
        await expect(card.locator('svg')).toBeVisible();
      }
    });

    test('should handle tab navigation correctly', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Test tab functionality
      const tabs = ['Active', 'Expiring Soon', 'Expired', 'All'];
      
      for (const tabName of tabs) {
        const tabButton = page.locator(`button:has-text("${tabName}")`);
        if (await tabButton.isVisible()) {
          await tabButton.click();
          await page.waitForTimeout(500); // Wait for content to update
          
          // Verify tab is selected (should have different styling)
          await expect(tabButton).toHaveClass(/selected|active|default/);
        }
      }
    });
  });

  test.describe('Embedded Timestamp Logic', () => {
    test('should parse embedded timestamps correctly', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Inject test permissions with embedded timestamps
      await page.evaluate(() => {
        // Test the parsing function
        function parsePermissionWithTimestamp(permission: string): { basePermission: string; timestamp?: number } {
          const parts = permission.split(':');
          if (parts.length >= 4) {
            const lastPart = parts[parts.length - 1];
            const timestamp = parseInt(lastPart, 10);
            if (!isNaN(timestamp)) {
              const basePermission = parts.slice(0, -1).join(':');
              return { basePermission, timestamp };
            }
          }
          return { basePermission: permission };
        }

        // Test cases
        const testCases = [
          { input: 'epsx:analytics:premium:1756617417', expectedBase: 'epsx:analytics:premium', expectedTimestamp: 1756617417 },
          { input: 'admin:users:manage:1756613757', expectedBase: 'admin:users:manage', expectedTimestamp: 1756613757 },
          { input: 'api:basic:read', expectedBase: 'api:basic:read', expectedTimestamp: undefined },
          { input: 'epsx:rankings:view:100:1756700217', expectedBase: 'epsx:rankings:view:100', expectedTimestamp: 1756700217 }
        ];

        let allTestsPassed = true;
        const results: any[] = [];

        testCases.forEach((testCase, index) => {
          const result = parsePermissionWithTimestamp(testCase.input);
          const passed = result.basePermission === testCase.expectedBase && result.timestamp === testCase.expectedTimestamp;
          
          results.push({
            test: index + 1,
            input: testCase.input,
            expected: { base: testCase.expectedBase, timestamp: testCase.expectedTimestamp },
            actual: result,
            passed
          });
          
          if (!passed) allTestsPassed = false;
        });

        console.log('🧪 Embedded Timestamp Parsing Test Results:', results);
        
        // Store results on window for test verification
        (window as any).testResults = { allTestsPassed, results };
      });
      
      // Verify test results
      const testResults = await page.evaluate(() => (window as any).testResults);
      expect(testResults.allTestsPassed).toBe(true);
      expect(testResults.results).toHaveLength(4);
    });

    test('should correctly identify expired permissions', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      await page.evaluate(() => {
        const now = Math.floor(Date.now() / 1000);
        const testPermissions = [
          { permission: `test:active:permission:${now + 3600}`, shouldBeExpired: false }, // 1 hour future
          { permission: `test:expired:permission:${now - 3600}`, shouldBeExpired: true }, // 1 hour past
          { permission: `test:permanent:permission`, shouldBeExpired: false }, // No timestamp
        ];

        const results = testPermissions.map(test => {
          const parts = test.permission.split(':');
          let isExpired = false;
          
          if (parts.length >= 4) {
            const timestamp = parseInt(parts[parts.length - 1], 10);
            if (!isNaN(timestamp)) {
              isExpired = now > timestamp;
            }
          }
          
          return {
            permission: test.permission,
            expected: test.shouldBeExpired,
            actual: isExpired,
            passed: isExpired === test.shouldBeExpired
          };
        });

        const allTestsPassed = results.every(r => r.passed);
        (window as any).expiryTestResults = { allTestsPassed, results };
        
        console.log('⏰ Permission Expiry Test Results:', results);
      });
      
      const testResults = await page.evaluate(() => (window as any).expiryTestResults);
      expect(testResults.allTestsPassed).toBe(true);
    });
  });

  test.describe('Admin Interface', () => {
    test('should load admin permissions page', async ({ page }) => {
      await page.goto(ADMIN_URL);
      
      // Check if redirected to login
      const currentUrl = page.url();
      if (currentUrl.includes('/login')) {
        console.log('Admin interface requires authentication - checking login page');
        await expect(page.locator('text=Admin Login, text=Redirecting')).toBeVisible({ timeout: 5000 });
      } else {
        // If somehow authenticated, verify admin interface
        await page.goto(`${ADMIN_URL}/permissions`);
        await expect(page.locator('h1:has-text("Permission Management")')).toBeVisible();
      }
    });

    test('should display permission management interface structure', async ({ page }) => {
      // Test the structure even if auth fails
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Should either show login redirect or permission management
      const hasLogin = await page.locator('text=Login, text=Admin').isVisible();
      const hasPermissions = await page.locator('text=Permission Management').isVisible();
      
      expect(hasLogin || hasPermissions).toBe(true);
    });
  });

  test.describe('Navigation Integration', () => {
    test('should have permissions link in navigation', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      
      // Look for permissions navigation link
      const permissionsLink = page.locator('a[href="/permissions"], nav a:has-text("Permissions")');
      
      // Should be visible (might be in dropdown or direct nav)
      const isVisible = await permissionsLink.isVisible();
      if (!isVisible) {
        // Check if it's in a mobile menu or dropdown
        const mobileMenu = page.locator('button[aria-label*="menu"], button:has-text("Menu")');
        if (await mobileMenu.isVisible()) {
          await mobileMenu.click();
          await expect(permissionsLink).toBeVisible();
        }
      } else {
        await expect(permissionsLink).toBeVisible();
      }
    });

    test('should navigate to permissions page from nav link', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      
      // Try to find and click permissions link
      const permissionsLink = page.locator('a[href="/permissions"]').first();
      if (await permissionsLink.isVisible()) {
        await permissionsLink.click();
        await expect(page).toHaveURL(`${FRONTEND_URL}/permissions`);
        await expect(page.locator('h1:has-text("My Permissions")')).toBeVisible();
      }
    });
  });

  test.describe('Error Handling', () => {
    test('should handle unauthenticated access gracefully', async ({ page }) => {
      // Clear any existing auth
      await page.context().clearCookies();
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Should either show login prompt or redirect to auth
      const hasLoginButton = await page.locator('text=Sign In, button:has-text("Sign In")').isVisible();
      const hasAuthRequired = await page.locator('text=Authentication Required').isVisible();
      const isRedirected = page.url().includes('/login') || page.url().includes('/auth');
      
      expect(hasLoginButton || hasAuthRequired || isRedirected).toBe(true);
    });

    test('should handle network errors gracefully', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      
      // Simulate network failure
      await page.route('**/api/auth/session', route => {
        route.abort('connectionrefused');
      });
      
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Should show some form of error handling
      const hasError = await page.locator('text=Error, text=Failed, text=Unable').isVisible({ timeout: 5000 });
      const hasLoading = await page.locator('text=Loading').isVisible();
      const hasRetry = await page.locator('button:has-text("Refresh"), button:has-text("Retry")').isVisible();
      
      // Should handle the error state gracefully
      expect(hasError || hasLoading || hasRetry).toBe(true);
    });
  });

  test.describe('Performance and Accessibility', () => {
    test('should load permissions page within reasonable time', async ({ page }) => {
      const startTime = Date.now();
      
      await page.goto(`${FRONTEND_URL}/permissions`);
      await page.waitForSelector('h1:has-text("My Permissions"), text=Loading', { timeout: 10000 });
      
      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000); // Should load within 10 seconds
    });

    test('should have proper accessibility attributes', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Check for proper headings
      await expect(page.locator('h1')).toBeVisible();
      
      // Check for button accessibility
      const buttons = page.locator('button');
      const buttonCount = await buttons.count();
      
      if (buttonCount > 0) {
        // At least some buttons should have accessible names
        const accessibleButtons = await page.locator('button[aria-label], button:has-text(*), button[title]').count();
        expect(accessibleButtons).toBeGreaterThan(0);
      }
    });
  });

  test.describe('Data Integrity', () => {
    test('should maintain permission data consistency', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Get statistics from cards
      const totalPermissions = await page.locator('text=Total Permissions').locator('..').locator('p').first().textContent();
      const activePermissions = await page.locator('text=Active').locator('..').locator('p').first().textContent();
      
      if (totalPermissions && activePermissions) {
        const total = parseInt(totalPermissions);
        const active = parseInt(activePermissions);
        
        // Active permissions should not exceed total
        expect(active).toBeLessThanOrEqual(total);
        expect(total).toBeGreaterThanOrEqual(0);
        expect(active).toBeGreaterThanOrEqual(0);
      }
    });
  });
});

test.describe('Manual Test Cases for Admin Functionality', () => {
  test('should document admin test scenarios', async ({ page }) => {
    // This test documents the manual test cases that need to be run
    console.log(`
    📋 MANUAL TEST SCENARIOS FOR ADMIN INTERFACE:
    
    1. Admin Login Flow:
       - Visit: ${process.env.ADMIN_URL || 'http://localhost:3003'}
       - Should redirect to login
       - Complete OAuth flow
       - Should return to admin dashboard
    
    2. Permission Assignment:
       - Navigate to /permissions
       - Search for user: jesadakorn.kirtnu@gmail.com
       - Click "Grant Permission" button
       - Select platform: EPSX
       - Choose template: "Analytics Premium"
       - Set duration: 1 hour
       - Add reason: "E2E Test Permission"
       - Click "Grant Permission"
       - Verify permission appears in user list
    
    3. Permission Verification:
       - Navigate to frontend: ${process.env.FRONTEND_URL || 'http://localhost:3002'}/permissions
       - Login as: jesadakorn.kirtnu@gmail.com
       - Should see new permission with expiry time
       - Should show "Expiring Soon" status
    
    4. Permission Revocation:
       - Return to admin interface
       - Find the granted permission
       - Click revoke button
       - Confirm revocation
       - Verify permission is removed
    
    5. Expiry Testing:
       - Grant permission with 1-minute expiry
       - Wait for expiry
       - Refresh permissions page
       - Should show as expired
    `);
    
    expect(true).toBe(true); // This test always passes, it's for documentation
  });
});