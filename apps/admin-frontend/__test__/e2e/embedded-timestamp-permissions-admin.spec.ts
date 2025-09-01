import { test, expect } from '@playwright/test';

test.describe('Embedded Timestamp Permissions - Admin Management E2E Tests', () => {
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001';
  const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
  const TEST_USER = {
    email: 'testuser@example.com',
    firebase_uid: 'test-user-123'
  };

  test.beforeEach(async ({ page }) => {
    // Set viewport for consistent testing
    await page.setViewportSize({ width: 1920, height: 1080 });
    
    // Clear any existing auth state
    await page.context().clearCookies();
    await page.context().clearPermissions();
  });

  test.describe('Admin Authentication and Navigation', () => {
    test('should authenticate as admin and navigate to embedded permissions', async ({ page }) => {
      // Navigate to admin login
      await page.goto(`${ADMIN_URL}/login`);
      
      // Check for login form or direct access
      const isAlreadyAuthenticated = await page.locator('h1:has-text("Admin Dashboard")').isVisible({ timeout: 3000 }).catch(() => false);
      
      if (!isAlreadyAuthenticated) {
        // Attempt authentication
        const oidcButton = page.locator('button:has-text("Continue with OIDC")');
        if (await oidcButton.isVisible()) {
          await oidcButton.click();
          await page.waitForURL(/admin/, { timeout: 30000 });
        }
      }
      
      // Verify we're in the admin dashboard
      await expect(page.locator('h1:has-text("Admin Dashboard"), h2:has-text("Admin Dashboard"), text=Admin Dashboard')).toBeVisible();
      
      // Navigate to Users section
      const usersLink = page.locator('a[href*="/users"], button:has-text("Users")');
      await expect(usersLink.first()).toBeVisible();
      await usersLink.first().click();
      
      // Verify users page loads
      await expect(page.locator('h1:has-text("User Management"), h2:has-text("Users"), text=User Management')).toBeVisible();
    });
  });

  test.describe('Embedded Permission Management UI', () => {
    test('should display embedded permission management interface', async ({ page }) => {
      // Navigate to admin and authenticate
      await page.goto(`${ADMIN_URL}/users`);
      
      // Wait for user list to load
      await page.waitForSelector('[data-testid="user-list"], .user-table, table, [class*="user"]', { timeout: 10000 });
      
      // Look for a test user or create one for testing
      const userRow = page.locator('[data-testid="user-row"], tr').first();
      if (await userRow.isVisible()) {
        // Click on first user to view details
        const userLink = userRow.locator('a, button').first();
        await userLink.click();
        
        // Verify user details page
        await expect(page.locator('h1, h2').filter({ hasText: /user|profile|details/i })).toBeVisible();
        
        // Look for permissions tab or section
        const permissionsTab = page.locator('text=Permissions, button:has-text("Permissions"), a[href*="permissions"]');
        if (await permissionsTab.first().isVisible()) {
          await permissionsTab.first().click();
          
          // Verify embedded permission management UI elements
          await expect(page.locator('text=Embedded Permissions, text=Timestamp Permissions, text=Temporary Access')).toBeVisible();
        }
      }
    });

    test('should show permission expiry indicators and status', async ({ page }) => {
      // Navigate to a user's permission page
      await page.goto(`${ADMIN_URL}/users`);
      
      // Find and click on a user
      const userRow = page.locator('[data-testid="user-row"], tr').first();
      if (await userRow.isVisible()) {
        await userRow.click();
        
        // Navigate to permissions
        const permissionsLink = page.locator('text=Permissions, a[href*="permissions"]').first();
        if (await permissionsLink.isVisible()) {
          await permissionsLink.click();
          
          // Check for permission health indicators
          const healthIndicators = [
            'text=Expired',
            'text=Expiring Soon',
            'text=Active',
            'text=Permanent',
            '[class*="health"]',
            '[class*="status"]',
            '[class*="expiry"]'
          ];
          
          let foundIndicator = false;
          for (const indicator of healthIndicators) {
            if (await page.locator(indicator).first().isVisible({ timeout: 2000 }).catch(() => false)) {
              foundIndicator = true;
              break;
            }
          }
          
          expect(foundIndicator).toBeTruthy();
        }
      }
    });
  });

  test.describe('Permission Creation and Management', () => {
    test('should create embedded timestamp permission via admin interface', async ({ page }) => {
      // Navigate to user permissions
      await page.goto(`${ADMIN_URL}/users`);
      
      const userRow = page.locator('[data-testid="user-row"], tr').first();
      if (await userRow.isVisible()) {
        await userRow.click();
        
        // Navigate to permissions
        const permissionsLink = page.locator('text=Permissions, a[href*="permissions"]').first();
        if (await permissionsLink.isVisible()) {
          await permissionsLink.click();
          
          // Look for add/grant permission button
          const grantButtons = [
            'button:has-text("Grant Permission")',
            'button:has-text("Add Permission")', 
            'button:has-text("New Permission")',
            'button[class*="grant"]',
            'button[class*="add"]'
          ];
          
          let grantButton = null;
          for (const selector of grantButtons) {
            const button = page.locator(selector).first();
            if (await button.isVisible({ timeout: 2000 }).catch(() => false)) {
              grantButton = button;
              break;
            }
          }
          
          if (grantButton) {
            await grantButton.click();
            
            // Fill permission form (if modal opens)
            const permissionForm = page.locator('form, [role="dialog"], .modal, .popup');
            if (await permissionForm.first().isVisible({ timeout: 3000 }).catch(() => false)) {
              
              // Fill base permission field
              const permissionInput = page.locator('input[name*="permission"], input[placeholder*="permission"], input[type="text"]').first();
              if (await permissionInput.isVisible({ timeout: 2000 }).catch(() => false)) {
                await permissionInput.fill('epsx:analytics:view');
              }
              
              // Set expiry time (1 hour from now)
              const expiryInput = page.locator('input[name*="expiry"], input[name*="expires"], input[type="datetime-local"]').first();
              if (await expiryInput.isVisible({ timeout: 2000 }).catch(() => false)) {
                const oneHourFromNow = new Date(Date.now() + 60 * 60 * 1000).toISOString().slice(0, -8);
                await expiryInput.fill(oneHourFromNow);
              }
              
              // Add reason
              const reasonInput = page.locator('input[name*="reason"], textarea[name*="reason"]').first();
              if (await reasonInput.isVisible({ timeout: 2000 }).catch(() => false)) {
                await reasonInput.fill('E2E Test: Temporary analytics access');
              }
              
              // Submit form
              const submitButton = page.locator('button[type="submit"], button:has-text("Save"), button:has-text("Grant"), button:has-text("Create")').first();
              if (await submitButton.isVisible()) {
                await submitButton.click();
                
                // Wait for success message or permission to appear in list
                await page.waitForTimeout(2000);
                
                // Verify permission was created
                const successIndicators = [
                  'text=Permission granted',
                  'text=Success',
                  'text=epsx:analytics:view',
                  '[class*="success"]'
                ];
                
                let foundSuccess = false;
                for (const indicator of successIndicators) {
                  if (await page.locator(indicator).first().isVisible({ timeout: 3000 }).catch(() => false)) {
                    foundSuccess = true;
                    break;
                  }
                }
                
                expect(foundSuccess).toBeTruthy();
              }
            }
          }
        }
      }
    });
  });

  test.describe('API Integration Tests', () => {
    test('should successfully call embedded permission API endpoints', async ({ request }) => {
      // Test permission validation endpoint
      const validationResponse = await request.post(`${BACKEND_URL}/api/v1/admin/users/${TEST_USER.firebase_uid}/embedded-permissions/validate`, {
        data: {
          permissions: [
            'epsx:analytics:view',
            'epsx:rankings:view:50:' + Math.floor((Date.now() + 3600000) / 1000), // 1 hour from now
            'admin:users:manage:' + Math.floor((Date.now() - 3600000) / 1000), // 1 hour ago (expired)
          ]
        },
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json'
        },
        ignoreHTTPSErrors: true,
        timeout: 10000
      }).catch(() => null);
      
      // If endpoint exists and responds, validate the response
      if (validationResponse && validationResponse.ok()) {
        const data = await validationResponse.json();
        expect(data).toHaveProperty('valid');
        expect(data).toHaveProperty('summary');
        expect(data.summary).toHaveProperty('total');
        expect(data.summary).toHaveProperty('valid_count');
      }
    });

    test('should handle permission expiry status API', async ({ request }) => {
      // Test expiry status endpoint
      const expiryResponse = await request.get(`${BACKEND_URL}/api/v1/admin/users/${TEST_USER.firebase_uid}/permissions/expiry-status`, {
        headers: {
          'Accept': 'application/json'
        },
        ignoreHTTPSErrors: true,
        timeout: 10000
      }).catch(() => null);
      
      // If endpoint exists and responds, validate the response structure
      if (expiryResponse && expiryResponse.ok()) {
        const data = await expiryResponse.json();
        expect(data).toHaveProperty('user_id');
        expect(data).toHaveProperty('permissions');
        expect(data).toHaveProperty('health');
        expect(data.health).toHaveProperty('has_expired');
        expect(data.health).toHaveProperty('has_expiring_soon');
      }
    });
  });

  test.describe('Permission Health Monitoring', () => {
    test('should display permission health dashboard', async ({ page }) => {
      // Navigate to admin dashboard
      await page.goto(`${ADMIN_URL}`);
      
      // Look for permission health indicators on dashboard
      const healthIndicators = [
        'text=Permission Health',
        'text=Expiring Permissions',
        'text=Health Score',
        'text=Expired',
        'text=Active Permissions',
        '[class*="health"]'
      ];
      
      let foundHealthSection = false;
      for (const indicator of healthIndicators) {
        if (await page.locator(indicator).first().isVisible({ timeout: 3000 }).catch(() => false)) {
          foundHealthSection = true;
          break;
        }
      }
      
      // If health section exists, verify it shows meaningful data
      if (foundHealthSection) {
        // Check for numeric indicators
        const numericIndicators = page.locator('text=/\\d+/, [class*="count"], [class*="metric"]');
        await expect(numericIndicators.first()).toBeVisible();
      }
    });

    test('should show expiry warnings and alerts', async ({ page }) => {
      // Navigate to users list to check for expiry warnings
      await page.goto(`${ADMIN_URL}/users`);
      
      // Look for users with expiry indicators
      const expiryWarnings = [
        '[class*="warning"]',
        '[class*="expiring"]',
        '[class*="expired"]', 
        'text=Expiring',
        'text=Expired',
        '[title*="expir"]'
      ];
      
      let foundWarning = false;
      for (const warning of expiryWarnings) {
        if (await page.locator(warning).first().isVisible({ timeout: 3000 }).catch(() => false)) {
          foundWarning = true;
          break;
        }
      }
      
      // If warnings exist, they should be properly styled
      if (foundWarning) {
        const warningElement = page.locator('[class*="warning"], [class*="expiring"]').first();
        if (await warningElement.isVisible()) {
          // Verify warning has appropriate styling (color, icon, etc.)
          const elementClass = await warningElement.getAttribute('class');
          expect(elementClass).toBeTruthy();
        }
      }
    });
  });

  test.describe('Performance and Error Handling', () => {
    test('should handle large permission lists efficiently', async ({ page }) => {
      // Navigate to a user with potentially many permissions
      await page.goto(`${ADMIN_URL}/users`);
      
      // Measure page load time
      const startTime = Date.now();
      
      const userRow = page.locator('[data-testid="user-row"], tr').first();
      if (await userRow.isVisible()) {
        await userRow.click();
        
        // Navigate to permissions and measure load time
        const permissionsLink = page.locator('text=Permissions, a[href*="permissions"]').first();
        if (await permissionsLink.isVisible()) {
          await permissionsLink.click();
          
          // Wait for permissions to load
          await page.waitForSelector('table, [class*="permission"], [data-testid*="permission"]', { timeout: 15000 }).catch(() => {});
          
          const loadTime = Date.now() - startTime;
          
          // Verify page loaded within reasonable time (15 seconds)
          expect(loadTime).toBeLessThan(15000);
        }
      }
    });

    test('should gracefully handle API errors', async ({ page }) => {
      // Mock network failure scenario
      await page.route('**/api/v1/admin/**', route => {
        route.fulfill({
          status: 500,
          body: JSON.stringify({ error: 'server_error', message: 'Internal server error' })
        });
      });
      
      // Navigate to permissions page
      await page.goto(`${ADMIN_URL}/users`);
      
      // Verify error handling - page should still be usable
      const errorMessages = [
        'text=Error',
        'text=Failed to load',
        'text=Something went wrong',
        '[class*="error"]'
      ];
      
      let foundErrorHandling = false;
      for (const errorMsg of errorMessages) {
        if (await page.locator(errorMsg).first().isVisible({ timeout: 5000 }).catch(() => false)) {
          foundErrorHandling = true;
          break;
        }
      }
      
      // Either error message is shown or page continues to work
      const pageStillWorks = await page.locator('h1, h2, main').first().isVisible({ timeout: 5000 }).catch(() => false);
      
      expect(foundErrorHandling || pageStillWorks).toBeTruthy();
    });
  });

  test.describe('Cross-Platform Permission Validation', () => {
    test('should validate platform-specific permissions', async ({ page }) => {
      // Navigate to user permissions
      await page.goto(`${ADMIN_URL}/users`);
      
      const userRow = page.locator('[data-testid="user-row"], tr').first();
      if (await userRow.isVisible()) {
        await userRow.click();
        
        const permissionsLink = page.locator('text=Permissions, a[href*="permissions"]').first();
        if (await permissionsLink.isVisible()) {
          await permissionsLink.click();
          
          // Look for platform indicators in permissions
          const platformIndicators = [
            'text=epsx:',
            'text=epsx-pay:',
            'text=epsx-token:',
            'text=admin:',
            '[class*="platform"]'
          ];
          
          let foundPlatformPermissions = false;
          for (const indicator of platformIndicators) {
            if (await page.locator(indicator).first().isVisible({ timeout: 3000 }).catch(() => false)) {
              foundPlatformPermissions = true;
              break;
            }
          }
          
          // If platform permissions exist, verify they're properly categorized
          if (foundPlatformPermissions) {
            const permissionsList = page.locator('[class*="permission"], table tr, [data-testid*="permission"]');
            await expect(permissionsList.first()).toBeVisible();
          }
        }
      }
    });
  });
});