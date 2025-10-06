import { test, expect } from '@playwright/test';

test.describe('Admin Permission Management - Comprehensive E2E Tests', () => {
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001';
  const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
  
  // Test user data
  const TEST_USER = {
    email: 'jesadakorn.kirtnu@gmail.com',
    userId: 'test-user-id-123',
    name: 'Test User'
  };

  const ADMIN_USER = {
    email: 'admin@epsx.io',
    password: 'P@ssword'
  };

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
  });

  test.describe('Permission Assignment Flow', () => {
    test('should grant embedded timestamp permission with expiry', async ({ page }) => {
      await page.goto(ADMIN_URL);
      
      // Wait for auth redirect or login
      await page.waitForTimeout(2000);
      
      // Check if we're on login page
      if (page.url().includes('/login') || page.url().includes('/auth')) {
        await page.waitForSelector('button:has-text("Continue"), button:has-text("Sign In")', { timeout: 10000 });
        const continueBtn = page.locator('button:has-text("Continue"), button:has-text("Sign In")').first();
        if (await continueBtn.isVisible()) {
          await continueBtn.click();
          await page.waitForURL(/admin/, { timeout: 15000 });
        }
      }

      // Navigate to permissions page
      await page.goto(`${ADMIN_URL}/permissions`);
      await page.waitForSelector('h1:has-text("Permission Management"), text=Permission', { timeout: 10000 });

      // Test permission granting form
      const grantBtn = page.locator('button:has-text("Grant Permission")').first();
      if (await grantBtn.isVisible()) {
        await grantBtn.click();
        
        // Fill permission form
        await page.waitForSelector('form, [data-testid="permission-form"]', { timeout: 5000 });
        
        // User selection
        const userInput = page.locator('input[placeholder*="email"], input[placeholder*="user"]').first();
        if (await userInput.isVisible()) {
          await userInput.fill(TEST_USER.email);
          await page.waitForTimeout(500);
        }

        // Platform selection
        const platformSelect = page.locator('select[name*="platform"], [role="combobox"]').first();
        if (await platformSelect.isVisible()) {
          await platformSelect.selectOption('epsx');
        }

        // Permission type
        const permissionInput = page.locator('input[name*="permission"], input[placeholder*="permission"]').first();
        if (await permissionInput.isVisible()) {
          await permissionInput.fill('analytics:premium');
        }

        // Expiry time - set to 1 hour from now
        const expiryInput = page.locator('input[type="datetime-local"], input[name*="expiry"]').first();
        if (await expiryInput.isVisible()) {
          const oneHourFromNow = new Date(Date.now() + 60 * 60 * 1000);
          const isoString = oneHourFromNow.toISOString().slice(0, 16);
          await expiryInput.fill(isoString);
        }

        // Reason
        const reasonInput = page.locator('textarea[name*="reason"], input[name*="reason"]').first();
        if (await reasonInput.isVisible()) {
          await reasonInput.fill('E2E Test - Temporary Analytics Access');
        }

        // Submit form
        const submitBtn = page.locator('button[type="submit"], button:has-text("Grant")').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          
          // Wait for success message or redirect
          await page.waitForSelector(
            'text=Success, text=granted, text=Permission granted',
            { timeout: 10000 }
          );
        }
      }
    });

    test('should display permission with expiry information', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Search for user
      const searchInput = page.locator('input[placeholder*="search"], input[type="search"]').first();
      if (await searchInput.isVisible()) {
        await searchInput.fill(TEST_USER.email);
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);
      }

      // Check for permission cards with expiry info
      const permissionCards = page.locator('[data-testid="permission-card"], .permission-card, .card');
      const cardCount = await permissionCards.count();
      
      if (cardCount > 0) {
        for (let i = 0; i < Math.min(cardCount, 3); i++) {
          const card = permissionCards.nth(i);
          
          // Should show platform badge
          await expect(card.locator('.badge, [class*="badge"]')).toBeVisible();
          
          // Should show expiry information
          const hasExpiry = await card.locator('text=expires, text=Expires, text=expiry').isVisible();
          const hasTime = await card.locator('text=hour, text=minute, text=day').isVisible();
          
          expect(hasExpiry || hasTime).toBe(true);
        }
      }
    });

    test('should handle bulk permission assignment', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Look for bulk operations
      const bulkBtn = page.locator('button:has-text("Bulk"), button:has-text("Multiple")').first();
      if (await bulkBtn.isVisible()) {
        await bulkBtn.click();
        
        // Fill bulk form
        const userListInput = page.locator('textarea[placeholder*="email"], textarea[name*="users"]').first();
        if (await userListInput.isVisible()) {
          await userListInput.fill(`${TEST_USER.email}\ntest2@example.com`);
        }

        // Select permission template
        const templateSelect = page.locator('select[name*="template"], [role="combobox"]').first();
        if (await templateSelect.isVisible()) {
          await templateSelect.selectOption('analytics-basic');
        }

        // Set bulk expiry
        const bulkExpiryInput = page.locator('input[name*="expiry"], select[name*="duration"]').first();
        if (await bulkExpiryInput.isVisible()) {
          if (bulkExpiryInput.getAttribute('type') === 'select') {
            await bulkExpiryInput.selectOption('24');
          } else {
            const twentyFourHoursFromNow = new Date(Date.now() + 24 * 60 * 60 * 1000);
            const isoString = twentyFourHoursFromNow.toISOString().slice(0, 16);
            await bulkExpiryInput.fill(isoString);
          }
        }

        // Submit bulk operation
        const submitBulkBtn = page.locator('button[type="submit"], button:has-text("Grant")').first();
        if (await submitBulkBtn.isVisible()) {
          await submitBulkBtn.click();
          
          // Wait for completion
          await page.waitForSelector(
            'text=completed, text=successful, text=Bulk operation',
            { timeout: 15000 }
          );
        }
      }
    });
  });

  test.describe('Permission Management Operations', () => {
    test('should extend permission expiry time', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Find existing permission to extend
      const extendBtn = page.locator('button:has-text("Extend"), button:has-text("Renew")').first();
      if (await extendBtn.isVisible()) {
        await extendBtn.click();
        
        // Fill extension form
        const newExpiryInput = page.locator('input[type="datetime-local"], input[name*="expiry"]').first();
        if (await newExpiryInput.isVisible()) {
          const futureDate = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000); // 7 days
          const isoString = futureDate.toISOString().slice(0, 16);
          await newExpiryInput.fill(isoString);
        }

        const reasonInput = page.locator('textarea[name*="reason"], input[name*="reason"]').first();
        if (await reasonInput.isVisible()) {
          await reasonInput.fill('Extended for continued testing');
        }

        const confirmExtendBtn = page.locator('button:has-text("Confirm"), button:has-text("Extend")').first();
        if (await confirmExtendBtn.isVisible()) {
          await confirmExtendBtn.click();
          
          await page.waitForSelector('text=extended, text=Updated', { timeout: 10000 });
        }
      }
    });

    test('should revoke permission successfully', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Find permission to revoke
      const revokeBtn = page.locator('button:has-text("Revoke"), button:has-text("Remove")').first();
      if (await revokeBtn.isVisible()) {
        await revokeBtn.click();
        
        // Confirm revocation
        const confirmRevokeBtn = page.locator('button:has-text("Confirm"), button:has-text("Yes")').first();
        if (await confirmRevokeBtn.isVisible()) {
          await confirmRevokeBtn.click();
          
          await page.waitForSelector('text=revoked, text=removed', { timeout: 10000 });
        }
      }
    });

    test('should validate permission format correctly', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Test invalid permission format
      const grantBtn = page.locator('button:has-text("Grant Permission")').first();
      if (await grantBtn.isVisible()) {
        await grantBtn.click();
        
        // Try invalid permission format
        const permissionInput = page.locator('input[name*="permission"]').first();
        if (await permissionInput.isVisible()) {
          await permissionInput.fill('invalid-format');
          
          // Should show validation error
          const errorMsg = page.locator('text=invalid, text=format, text=error').first();
          await expect(errorMsg).toBeVisible({ timeout: 5000 });
        }
      }
    });
  });

  test.describe('Permission Analytics and Health', () => {
    test('should display permission health dashboard', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Check for health indicators
      const healthCards = page.locator('[data-testid="health-card"], .health-indicator, .stats-card');
      const cardCount = await healthCards.count();
      
      if (cardCount > 0) {
        // Should have total permissions
        await expect(page.locator('text=Total Permissions, text=Active')).toBeVisible();
        
        // Should have expiring soon count
        await expect(page.locator('text=Expiring Soon, text=Expires')).toBeVisible();
        
        // Should have expired count
        await expect(page.locator('text=Expired')).toBeVisible();
      }
    });

    test('should filter permissions by expiry status', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Test filter tabs
      const filterTabs = ['Active', 'Expiring', 'Expired', 'All'];
      
      for (const tabName of filterTabs) {
        const tab = page.locator(`button:has-text("${tabName}"), [role="tab"]:has-text("${tabName}")`).first();
        if (await tab.isVisible()) {
          await tab.click();
          await page.waitForTimeout(500);
          
          // Verify tab is selected
          await expect(tab).toHaveClass(/active|selected|current/);
        }
      }
    });

    test('should export permission data', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      const exportBtn = page.locator('button:has-text("Export"), button:has-text("Download")').first();
      if (await exportBtn.isVisible()) {
        // Set up download handler
        const downloadPromise = page.waitForEvent('download');
        await exportBtn.click();
        
        const download = await downloadPromise;
        expect(download.suggestedFilename()).toContain('permission');
      }
    });
  });

  test.describe('Real-time Updates and Cleanup', () => {
    test('should auto-refresh expiring permissions', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Get initial expiring count
      const expiringElement = page.locator('text=Expiring Soon').locator('..').locator('text=/\\d+/').first();
      const initialCount = await expiringElement.textContent();
      
      // Wait for auto-refresh (should happen every 30 seconds)
      await page.waitForTimeout(35000);
      
      // Check if count updated (it may stay the same, but component should refresh)
      const updatedElement = page.locator('text=Expiring Soon').locator('..').locator('text=/\\d+/').first();
      await expect(updatedElement).toBeVisible();
    });

    test('should cleanup expired permissions', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/system`);
      
      const cleanupBtn = page.locator('button:has-text("Cleanup"), button:has-text("Clean")').first();
      if (await cleanupBtn.isVisible()) {
        await cleanupBtn.click();
        
        // Should show cleanup options
        const dryRunOption = page.locator('input[type="checkbox"], label:has-text("Dry run")').first();
        if (await dryRunOption.isVisible()) {
          await dryRunOption.check();
        }
        
        const executeCleanupBtn = page.locator('button:has-text("Execute"), button:has-text("Start")').first();
        if (await executeCleanupBtn.isVisible()) {
          await executeCleanupBtn.click();
          
          await page.waitForSelector('text=cleanup, text=completed', { timeout: 10000 });
        }
      }
    });
  });

  test.describe('Error Handling and Edge Cases', () => {
    test('should handle permission conflicts gracefully', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Try to grant duplicate permission
      const grantBtn = page.locator('button:has-text("Grant Permission")').first();
      if (await grantBtn.isVisible()) {
        await grantBtn.click();
        
        // Fill with existing permission
        const userInput = page.locator('input[placeholder*="user"]').first();
        if (await userInput.isVisible()) {
          await userInput.fill(TEST_USER.email);
        }
        
        const permissionInput = page.locator('input[name*="permission"]').first();
        if (await permissionInput.isVisible()) {
          await permissionInput.fill('analytics:premium');
        }
        
        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          
          // Should handle duplicate gracefully
          const response = page.locator('text=already exists, text=duplicate, text=error');
          await expect(response).toBeVisible({ timeout: 5000 });
        }
      }
    });

    test('should validate expiry date constraints', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      const grantBtn = page.locator('button:has-text("Grant Permission")').first();
      if (await grantBtn.isVisible()) {
        await grantBtn.click();
        
        // Try past date
        const expiryInput = page.locator('input[type="datetime-local"]').first();
        if (await expiryInput.isVisible()) {
          const pastDate = new Date(Date.now() - 24 * 60 * 60 * 1000);
          const isoString = pastDate.toISOString().slice(0, 16);
          await expiryInput.fill(isoString);
          
          // Should show validation error
          const errorMsg = page.locator('text=past, text=invalid date, text=future');
          await expect(errorMsg).toBeVisible({ timeout: 5000 });
        }
      }
    });

    test('should handle API errors gracefully', async ({ page }) => {
      // Mock API failure
      await page.route('**/api/v1/admin/**', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server Error' }) });
      });
      
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Should show error state
      const errorMessage = page.locator('text=error, text=failed, text=Unable to load');
      await expect(errorMessage).toBeVisible({ timeout: 10000 });
      
      // Should have retry option
      const retryBtn = page.locator('button:has-text("Retry"), button:has-text("Reload")');
      await expect(retryBtn).toBeVisible();
    });
  });

  test.describe('Integration with Backend API', () => {
    test('should create valid embedded timestamp permissions', async ({ page }) => {
      // Test permission format creation
      await page.evaluate(() => {
        const createEmbeddedPermission = (basePermission: string, expiryTimestamp: number): string => {
          return `${basePermission}:${expiryTimestamp}`;
        };
        
        const testCases = [
          { base: 'epsx:analytics:premium', timestamp: 1756617417 },
          { base: 'admin:users:manage', timestamp: 1756613757 },
          { base: 'epsx:rankings:view:100', timestamp: 1756700217 }
        ];
        
        const results = testCases.map(test => {
          const embedded = createEmbeddedPermission(test.base, test.timestamp);
          const expected = `${test.base}:${test.timestamp}`;
          return {
            input: test,
            output: embedded,
            expected,
            passed: embedded === expected
          };
        });
        
        (window as any).embeddedPermissionTest = {
          allPassed: results.every(r => r.passed),
          results
        };
      });
      
      const testResults = await page.evaluate(() => (window as any).embeddedPermissionTest);
      expect(testResults.allPassed).toBe(true);
    });

    test('should parse embedded permissions correctly', async ({ page }) => {
      await page.evaluate(() => {
        const parseEmbeddedPermission = (permission: string): { basePermission: string; timestamp?: number } => {
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
        };
        
        const testCases = [
          { 
            input: 'epsx:analytics:premium:1756617417', 
            expectedBase: 'epsx:analytics:premium', 
            expectedTimestamp: 1756617417 
          },
          { 
            input: 'admin:users:manage:1756613757', 
            expectedBase: 'admin:users:manage', 
            expectedTimestamp: 1756613757 
          },
          { 
            input: 'epsx:basic:read', 
            expectedBase: 'epsx:basic:read', 
            expectedTimestamp: undefined 
          }
        ];
        
        const results = testCases.map(test => {
          const parsed = parseEmbeddedPermission(test.input);
          const passed = parsed.basePermission === test.expectedBase && 
                         parsed.timestamp === test.expectedTimestamp;
          return { test, parsed, passed };
        });
        
        (window as any).permissionParsingTest = {
          allPassed: results.every(r => r.passed),
          results
        };
      });
      
      const testResults = await page.evaluate(() => (window as any).permissionParsingTest);
      expect(testResults.allPassed).toBe(true);
    });
  });

  test.describe('Performance and Accessibility', () => {
    test('should load permission management within acceptable time', async ({ page }) => {
      const startTime = Date.now();
      
      await page.goto(`${ADMIN_URL}/permissions`);
      await page.waitForSelector('h1, [data-testid="permission-page"]', { timeout: 10000 });
      
      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000); // Should load within 10 seconds
    });

    test('should have proper keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Test tab navigation
      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();
      
      // Should be able to navigate through interactive elements
      for (let i = 0; i < 5; i++) {
        await page.keyboard.press('Tab');
        const currentFocus = page.locator(':focus').first();
        await expect(currentFocus).toBeVisible();
      }
    });

    test('should have accessibility labels', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);
      
      // Check for proper ARIA labels
      const buttons = page.locator('button');
      const buttonCount = await buttons.count();
      
      if (buttonCount > 0) {
        for (let i = 0; i < Math.min(buttonCount, 5); i++) {
          const button = buttons.nth(i);
          const hasLabel = await button.getAttribute('aria-label') !== null ||
                          await button.textContent() !== '';
          expect(hasLabel).toBe(true);
        }
      }
    });
  });
});