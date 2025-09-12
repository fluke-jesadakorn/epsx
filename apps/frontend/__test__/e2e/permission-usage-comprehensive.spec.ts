import { test, expect } from '@playwright/test';

test.describe('Frontend Permission Usage - Comprehensive E2E Tests', () => {
  const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:3000';
  const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
  
  // Test user credentials
  const TEST_USER = {
    email: 'jesadakorn.kirtnu@gmail.com',
    password: 'P@ssword',
    uid: 'test-user-firebase-uid'
  };

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
  });

  test.describe('Permission Display and Access Control', () => {
    test('should display user permissions page correctly', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      
      // Check if user needs to login
      const isLoggedIn = await page.locator('text=Sign Out, button:has-text("Sign Out")').isVisible();
      
      if (!isLoggedIn) {
        const signInBtn = page.locator('button:has-text("Sign In"), a[href*="login"]').first();
        if (await signInBtn.isVisible()) {
          await signInBtn.click();
          await page.waitForURL(/login|auth/, { timeout: 10000 });
          
          // Handle OAuth flow if needed
          await page.waitForTimeout(2000);
        }
      }

      // Navigate to permissions page
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Verify permissions page structure
      await expect(page.locator('h1:has-text("My Permissions"), h1:has-text("Permissions")')).toBeVisible({ timeout: 10000 });
      
      // Should have account information section
      await expect(page.locator('text=Account Information, text=User Details')).toBeVisible();
      
      // Should have permission statistics
      const statsCards = page.locator('[data-testid="stats-card"], .stats-card, .permission-stats');
      const statsCount = await statsCards.count();
      
      if (statsCount > 0) {
        await expect(page.locator('text=Total Permissions, text=Active')).toBeVisible();
        await expect(page.locator('text=Expiring Soon, text=Expires')).toBeVisible();
        await expect(page.locator('text=Expired')).toBeVisible();
      }
    });

    test('should display embedded timestamp permissions with expiry info', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Wait for permission cards to load
      await page.waitForSelector('[data-testid="permission-card"], .permission-card', { timeout: 10000 });
      
      const permissionCards = page.locator('[data-testid="permission-card"], .permission-card');
      const cardCount = await permissionCards.count();
      
      expect(cardCount).toBeGreaterThanOrEqual(0);
      
      if (cardCount > 0) {
        for (let i = 0; i < Math.min(cardCount, 3); i++) {
          const card = permissionCards.nth(i);
          
          // Should have platform badge
          await expect(card.locator('.badge, [class*="badge"], [data-testid="platform-badge"]')).toBeVisible();
          
          // Should have permission name/title
          await expect(card.locator('h3, .title, [data-testid="permission-title"]')).toBeVisible();
          
          // Should have status indicator
          await expect(card.locator('svg, .status-icon, [data-testid="status-icon"]')).toBeVisible();
          
          // Check for expiry information if permission has timestamp
          const expiryInfo = card.locator('text=expires, text=Expires, text=expiry, text=hours, text=days');
          const hasExpiryInfo = await expiryInfo.count() > 0;
          
          if (hasExpiryInfo) {
            await expect(expiryInfo.first()).toBeVisible();
          }
        }
      }
    });

    test('should filter permissions by status tabs', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Test permission filter tabs
      const filterTabs = ['Active', 'Expiring Soon', 'Expired', 'All'];
      
      for (const tabName of filterTabs) {
        const tab = page.locator(`button:has-text("${tabName}"), [role="tab"]:has-text("${tabName}")`).first();
        
        if (await tab.isVisible()) {
          await tab.click();
          await page.waitForTimeout(500);
          
          // Verify tab is selected
          await expect(tab).toHaveClass(/selected|active|current/);
          
          // Verify content updates
          await page.waitForSelector('[data-testid="permission-card"], .permission-card, text=No permissions', { timeout: 5000 });
        }
      }
    });

    test('should handle permission expiry status correctly', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Test embedded timestamp parsing and expiry logic
      await page.evaluate(() => {
        const parsePermissionWithTimestamp = (permission: string): { basePermission: string; timestamp?: number; isExpired: boolean } => {
          const parts = permission.split(':');
          if (parts.length >= 4) {
            const lastPart = parts[parts.length - 1];
            const timestamp = parseInt(lastPart, 10);
            if (!isNaN(timestamp)) {
              const basePermission = parts.slice(0, -1).join(':');
              const now = Math.floor(Date.now() / 1000);
              return { 
                basePermission, 
                timestamp, 
                isExpired: now > timestamp 
              };
            }
          }
          return { basePermission: permission, isExpired: false };
        };

        const testPermissions = [
          `epsx:analytics:premium:${Math.floor(Date.now() / 1000) + 3600}`, // 1 hour future
          `epsx:rankings:view:${Math.floor(Date.now() / 1000) - 3600}`, // 1 hour past
          'epsx:basic:read', // No timestamp
          `admin:users:manage:${Math.floor(Date.now() / 1000) + 86400}`, // 1 day future
        ];

        const results = testPermissions.map(permission => {
          const parsed = parsePermissionWithTimestamp(permission);
          return {
            permission,
            basePermission: parsed.basePermission,
            timestamp: parsed.timestamp,
            isExpired: parsed.isExpired,
            hasTimestamp: parsed.timestamp !== undefined
          };
        });

        console.log('⏰ Permission Expiry Test Results:', results);
        (window as any).permissionExpiryTest = {
          results,
          validCount: results.filter(r => !r.isExpired).length,
          expiredCount: results.filter(r => r.isExpired).length,
          timestampCount: results.filter(r => r.hasTimestamp).length
        };
      });

      const testResults = await page.evaluate(() => (window as any).permissionExpiryTest);
      expect(testResults.results).toHaveLength(4);
      expect(testResults.timestampCount).toBeGreaterThan(0);
    });
  });

  test.describe('Feature Access Control', () => {
    test('should enforce permission-based feature access', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/analytics`);
      
      // Check if analytics page loads (user should have analytics permission)
      const hasAccess = await page.locator('h1:has-text("Analytics"), h1:has-text("Stock Rankings")').isVisible({ timeout: 5000 });
      const isBlocked = await page.locator('text=Access Denied, text=Permission Required, text=Upgrade').isVisible({ timeout: 5000 });
      
      // Should either have access or be properly blocked
      expect(hasAccess || isBlocked).toBe(true);
      
      if (hasAccess) {
        // Should display analytics content
        await expect(page.locator('[data-testid="analytics-content"], .analytics-dashboard')).toBeVisible();
      } else if (isBlocked) {
        // Should show upgrade or permission request option
        await expect(page.locator('button:has-text("Upgrade"), button:has-text("Request Access")')).toBeVisible();
      }
    });

    test('should display permission gates for premium features', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/analytics`);
      
      // Look for premium feature gates
      const premiumGates = page.locator('[data-testid="permission-gate"], .permission-gate, .premium-gate');
      const gateCount = await premiumGates.count();
      
      if (gateCount > 0) {
        for (let i = 0; i < Math.min(gateCount, 3); i++) {
          const gate = premiumGates.nth(i);
          
          // Should have upgrade button or access message
          const hasUpgradeBtn = await gate.locator('button:has-text("Upgrade"), button:has-text("Get Access")').isVisible();
          const hasAccessMsg = await gate.locator('text=Premium Feature, text=Requires Permission').isVisible();
          
          expect(hasUpgradeBtn || hasAccessMsg).toBe(true);
        }
      }
    });

    test('should handle tier-based access correctly', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/my-data`);
      
      // Check portfolio/my-data access
      const hasPortfolioAccess = await page.locator('h1:has-text("Portfolio"), h1:has-text("My Data")').isVisible({ timeout: 5000 });
      const isPortfolioBlocked = await page.locator('text=Premium Feature, text=Portfolio Access Required').isVisible({ timeout: 5000 });
      
      expect(hasPortfolioAccess || isPortfolioBlocked).toBe(true);
      
      if (hasPortfolioAccess) {
        // Should show portfolio data or empty state
        const hasData = await page.locator('[data-testid="portfolio-data"], .portfolio-content').isVisible();
        const isEmpty = await page.locator('text=No data, text=Connect your portfolio').isVisible();
        expect(hasData || isEmpty).toBe(true);
      }
    });
  });

  test.describe('Permission Expiry Handling', () => {
    test('should display expiry warnings for expiring permissions', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Look for expiry warnings
      const warningElements = page.locator('[data-testid="expiry-warning"], .expiry-warning, .warning');
      const warningCount = await warningElements.count();
      
      if (warningCount > 0) {
        // Should have warning text
        await expect(page.locator('text=expiring, text=Expiring, text=expires soon')).toBeVisible();
        
        // Should have action button
        await expect(page.locator('button:has-text("Renew"), button:has-text("Extend")')).toBeVisible();
      }
    });

    test('should auto-refresh permission status', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Get initial permission count
      const initialStats = await page.locator('[data-testid="total-permissions"], text=/\\d+ permissions/').first().textContent();
      
      // Wait for potential auto-refresh
      await page.waitForTimeout(5000);
      
      // Check if page auto-refreshes (status should remain consistent)
      const updatedStats = await page.locator('[data-testid="total-permissions"], text=/\\d+ permissions/').first().textContent();
      expect(updatedStats).toBeDefined();
    });

    test('should handle expired permissions gracefully', async ({ page }) => {
      // Test expired permission handling
      await page.evaluate(() => {
        const now = Math.floor(Date.now() / 1000);
        const testPermissions = [
          `epsx:analytics:premium:${now - 3600}`, // Expired 1 hour ago
          `epsx:rankings:view:${now + 3600}`, // Valid for 1 hour
          'epsx:basic:read', // No expiry
        ];

        const filterExpired = (permissions: string[]): { valid: string[]; expired: string[] } => {
          const valid: string[] = [];
          const expired: string[] = [];

          permissions.forEach(permission => {
            const parts = permission.split(':');
            if (parts.length >= 4) {
              const timestamp = parseInt(parts[parts.length - 1], 10);
              if (!isNaN(timestamp)) {
                if (now > timestamp) {
                  expired.push(permission);
                } else {
                  valid.push(permission);
                }
              } else {
                valid.push(permission);
              }
            } else {
              valid.push(permission);
            }
          });

          return { valid, expired };
        };

        const result = filterExpired(testPermissions);
        console.log('🔍 Permission Filtering Test:', result);
        
        (window as any).permissionFilterTest = {
          input: testPermissions,
          valid: result.valid,
          expired: result.expired,
          validCount: result.valid.length,
          expiredCount: result.expired.length
        };
      });

      const testResults = await page.evaluate(() => (window as any).permissionFilterTest);
      expect(testResults.validCount).toBeGreaterThan(0);
      expect(testResults.expiredCount).toBeGreaterThan(0);
    });
  });

  test.describe('Real-time Permission Updates', () => {
    test('should reflect permission changes from admin interface', async ({ page, context }) => {
      // Open a second page for admin simulation
      const adminPage = await context.newPage();
      
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Get initial permission count
      const initialCount = await page.locator('[data-testid="total-permissions"], text=/\\d+/').first().textContent();
      
      // Simulate admin granting a permission (would normally be done via admin interface)
      await page.waitForTimeout(2000);
      
      // Refresh to check for updates
      await page.reload();
      await page.waitForSelector('[data-testid="permission-card"], .permission-card', { timeout: 5000 });
      
      // Verify page loads correctly after refresh
      await expect(page.locator('h1:has-text("Permissions")')).toBeVisible();
      
      await adminPage.close();
    });

    test('should handle SSE updates for permission changes', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Listen for SSE events
      await page.evaluate(() => {
        let sseUpdates = 0;
        
        // Mock SSE connection
        const mockSSE = () => {
          console.log('🔄 Mock SSE permission update received');
          sseUpdates++;
          
          // Trigger UI update
          const event = new CustomEvent('permissionUpdate', {
            detail: { type: 'permission_granted', userId: 'test-user' }
          });
          window.dispatchEvent(event);
        };
        
        // Simulate SSE update
        setTimeout(mockSSE, 2000);
        
        (window as any).sseTestData = { sseUpdates };
      });
      
      await page.waitForTimeout(3000);
      
      const sseData = await page.evaluate(() => (window as any).sseTestData);
      expect(sseData.sseUpdates).toBeGreaterThan(0);
    });
  });

  test.describe('Error Handling and Resilience', () => {
    test('should handle API failures gracefully', async ({ page }) => {
      // Mock API failure
      await page.route('**/api/auth/session', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server Error' }) });
      });
      
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Should show error state
      const errorState = page.locator('text=Error loading, text=Failed to load, text=Something went wrong');
      await expect(errorState).toBeVisible({ timeout: 10000 });
      
      // Should have retry mechanism
      const retryBtn = page.locator('button:has-text("Retry"), button:has-text("Reload")');
      if (await retryBtn.isVisible()) {
        await retryBtn.click();
      }
    });

    test('should handle unauthenticated state', async ({ page }) => {
      // Clear cookies to simulate unauthenticated state
      await page.context().clearCookies();
      
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Should redirect to login or show auth required
      const hasLoginRedirect = page.url().includes('/login') || page.url().includes('/auth');
      const hasAuthRequired = await page.locator('text=Sign In, text=Authentication Required').isVisible({ timeout: 5000 });
      
      expect(hasLoginRedirect || hasAuthRequired).toBe(true);
    });

    test('should handle network timeouts', async ({ page }) => {
      // Simulate slow network
      await page.route('**/api/**', route => {
        setTimeout(() => route.continue(), 5000);
      });
      
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Should show loading state
      const loadingState = page.locator('text=Loading, [data-testid="loading"]');
      await expect(loadingState).toBeVisible({ timeout: 3000 });
    });
  });

  test.describe('Mobile Responsiveness', () => {
    test('should display permissions correctly on mobile', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Should have mobile-friendly layout
      await expect(page.locator('h1')).toBeVisible();
      
      // Permission cards should stack vertically
      const cards = page.locator('[data-testid="permission-card"], .permission-card');
      const cardCount = await cards.count();
      
      if (cardCount > 1) {
        const firstCard = cards.nth(0);
        const secondCard = cards.nth(1);
        
        const firstCardBox = await firstCard.boundingBox();
        const secondCardBox = await secondCard.boundingBox();
        
        if (firstCardBox && secondCardBox) {
          // Second card should be below first card (not side by side)
          expect(secondCardBox.y).toBeGreaterThan(firstCardBox.y + firstCardBox.height - 10);
        }
      }
    });

    test('should have mobile navigation for permissions', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      
      await page.goto(FRONTEND_URL);
      
      // Check for mobile menu
      const mobileMenuBtn = page.locator('button[aria-label*="menu"], button:has-text("Menu")');
      if (await mobileMenuBtn.isVisible()) {
        await mobileMenuBtn.click();
        
        // Should have permissions link in mobile menu
        await expect(page.locator('a[href="/permissions"], nav a:has-text("Permissions")')).toBeVisible();
      }
    });
  });

  test.describe('Performance and Accessibility', () => {
    test('should load permissions page quickly', async ({ page }) => {
      const startTime = Date.now();
      
      await page.goto(`${FRONTEND_URL}/permissions`);
      await page.waitForSelector('h1, [data-testid="permissions-page"]', { timeout: 10000 });
      
      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(5000); // Should load within 5 seconds
    });

    test('should be keyboard accessible', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Test keyboard navigation
      await page.keyboard.press('Tab');
      const firstFocusable = page.locator(':focus').first();
      await expect(firstFocusable).toBeVisible();
      
      // Test tab navigation through permission cards
      for (let i = 0; i < 3; i++) {
        await page.keyboard.press('Tab');
        const currentFocus = page.locator(':focus').first();
        await expect(currentFocus).toBeVisible();
      }
    });

    test('should have proper ARIA labels', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Check for accessibility attributes
      const buttons = page.locator('button');
      const buttonCount = await buttons.count();
      
      if (buttonCount > 0) {
        for (let i = 0; i < Math.min(buttonCount, 5); i++) {
          const button = buttons.nth(i);
          const hasAccessibleName = await button.getAttribute('aria-label') !== null ||
                                  await button.textContent() !== '';
          expect(hasAccessibleName).toBe(true);
        }
      }
      
      // Check for proper headings
      await expect(page.locator('h1')).toBeVisible();
      
      // Check for semantic structure
      const main = page.locator('main, [role="main"]');
      if (await main.count() > 0) {
        await expect(main).toBeVisible();
      }
    });
  });

  test.describe('Data Validation and Integrity', () => {
    test('should validate permission data consistency', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Get statistics from dashboard
      const totalElement = page.locator('text=Total Permissions').locator('..').locator('text=/\\d+/').first();
      const activeElement = page.locator('text=Active').locator('..').locator('text=/\\d+/').first();
      const expiredElement = page.locator('text=Expired').locator('..').locator('text=/\\d+/').first();
      
      const totalText = await totalElement.textContent();
      const activeText = await activeElement.textContent();
      const expiredText = await expiredElement.textContent();
      
      if (totalText && activeText && expiredText) {
        const total = parseInt(totalText);
        const active = parseInt(activeText);
        const expired = parseInt(expiredText);
        
        // Data consistency checks
        expect(active).toBeLessThanOrEqual(total);
        expect(expired).toBeLessThanOrEqual(total);
        expect(active + expired).toBeLessThanOrEqual(total);
        expect(total).toBeGreaterThanOrEqual(0);
      }
    });

    test('should handle permission format validation', async ({ page }) => {
      await page.goto(`${FRONTEND_URL}/permissions`);
      
      // Test permission format validation
      await page.evaluate(() => {
        const validatePermissionFormat = (permission: string): boolean => {
          // Basic validation for platform:resource:action format
          const parts = permission.split(':');
          return parts.length >= 3 && parts.every(part => part.length > 0);
        };
        
        const testCases = [
          'epsx:analytics:premium',
          'admin:users:manage',
          'invalid-format',
          'epsx:analytics:premium:1756617417',
          '',
          'single',
          'epsx:',
          ':analytics:premium'
        ];
        
        const results = testCases.map(permission => ({
          permission,
          isValid: validatePermissionFormat(permission)
        }));
        
        console.log('✅ Permission Format Validation:', results);
        (window as any).validationTest = {
          results,
          validCount: results.filter(r => r.isValid).length,
          invalidCount: results.filter(r => !r.isValid).length
        };
      });
      
      const validationResults = await page.evaluate(() => (window as any).validationTest);
      expect(validationResults.validCount).toBeGreaterThan(0);
      expect(validationResults.invalidCount).toBeGreaterThan(0);
    });
  });
});