import { test, expect, Page } from '@playwright/test';
import { 
  TEST_USERS, 
  TestUser, 
  generateMockJWT, 
  canUserAccessFeature,
  getAvailableFeatures
} from '../fixtures/user-fixtures';

/**
 * Feature Access Control Tests
 * Tests all trading platform features and their tier-based access controls
 */

test.describe('🎛️ Feature Access Control System', () => {

  // Helper function to authenticate user
  async function authenticateUser(page: Page, user: TestUser): Promise<void> {
    const jwtToken = generateMockJWT(user);
    
    await page.context().addCookies([{
      name: 'epsx_jwt',
      value: jwtToken,
      domain: 'localhost',
      path: '/',
      httpOnly: true,
      secure: false
    }]);
  }

  // Helper function to test feature visibility and functionality
  async function testFeatureAccess(
    page: Page, 
    user: TestUser, 
    featureName: string, 
    testId: string, 
    shouldHaveAccess: boolean
  ): Promise<void> {
    const hasAccess = canUserAccessFeature(user, featureName);
    expect(hasAccess).toBe(shouldHaveAccess);

    if (shouldHaveAccess) {
      // Feature should be fully accessible
      const element = page.locator(`[data-testid="${testId}"]`);
      await expect(element).toBeVisible();
      
      if (await element.isEnabled()) {
        // Test functionality if clickable
        await element.click();
        await expect(page.locator('[data-testid="loading-spinner"]')).not.toBeVisible({ timeout: 5000 });
      }
    } else {
      // Feature should be restricted or show upgrade prompt
      const element = page.locator(`[data-testid="${testId}"]`);
      const upgradePrompt = page.locator(`[data-testid="${testId}-upgrade"]`);
      
      const elementExists = await element.isVisible().catch(() => false);
      const upgradeExists = await upgradePrompt.isVisible().catch(() => false);
      
      if (elementExists) {
        // If feature is visible, it should be disabled or trigger upgrade
        await element.click();
        await expect(page.locator('[data-testid="upgrade-modal"]')).toBeVisible();
      } else if (upgradeExists) {
        // Should show upgrade prompt
        await expect(upgradePrompt).toBeVisible();
      }
    }

    console.log(`✅ ${user.package_tier} - ${featureName}: ${shouldHaveAccess ? 'ACCESSIBLE' : 'RESTRICTED'}`);
  }

  test.describe('🔀 Basic Trading Features', () => {
    
    test('basic trading should be available to ALL tiers', async ({ page }) => {
      const allUsers = [
        TEST_USERS.FREE_USER,
        TEST_USERS.BRONZE_USER,
        TEST_USERS.SILVER_USER,
        TEST_USERS.GOLD_USER,
        TEST_USERS.PLATINUM_USER,
        TEST_USERS.ENTERPRISE_USER
      ];

      for (const user of allUsers) {
        await authenticateUser(page, user);
        await page.goto('/trading');
        
        // Basic trading features should be available
        await testFeatureAccess(page, user, 'basic-trading', 'basic-buy-button', true);
        await testFeatureAccess(page, user, 'basic-trading', 'basic-sell-button', true);
        await testFeatureAccess(page, user, 'basic-trading', 'market-order-form', true);
        
        // Basic portfolio view
        await page.goto('/portfolio');
        await testFeatureAccess(page, user, 'portfolio-view', 'portfolio-balance', true);
        await testFeatureAccess(page, user, 'portfolio-view', 'holdings-table', true);
      }
    });

    test('basic notifications should be available to ALL tiers', async ({ page }) => {
      const allUsers = [TEST_USERS.FREE_USER, TEST_USERS.BRONZE_USER, TEST_USERS.SILVER_USER];

      for (const user of allUsers) {
        await authenticateUser(page, user);
        await page.goto('/settings');
        
        await testFeatureAccess(page, user, 'basic-notifications', 'notification-settings', true);
        await testFeatureAccess(page, user, 'basic-notifications', 'email-notifications-toggle', true);
      }
    });
  });

  test.describe('🥉 BRONZE+ Features', () => {
    
    test('enhanced notifications should require BRONZE+', async ({ page }) => {
      // FREE user should NOT have access
      await authenticateUser(page, TEST_USERS.FREE_USER);
      await page.goto('/settings');
      await testFeatureAccess(page, TEST_USERS.FREE_USER, 'enhanced-notifications', 'push-notifications-toggle', false);
      await testFeatureAccess(page, TEST_USERS.FREE_USER, 'enhanced-notifications', 'sms-notifications-toggle', false);
      
      // BRONZE+ users should have access
      const bronzePlusUsers = [
        TEST_USERS.BRONZE_USER,
        TEST_USERS.SILVER_USER,
        TEST_USERS.GOLD_USER,
        TEST_USERS.PLATINUM_USER,
        TEST_USERS.ENTERPRISE_USER
      ];

      for (const user of bronzePlusUsers) {
        await authenticateUser(page, user);
        await page.goto('/settings');
        
        await testFeatureAccess(page, user, 'enhanced-notifications', 'push-notifications-toggle', true);
        await testFeatureAccess(page, user, 'enhanced-notifications', 'sms-notifications-toggle', true);
        await testFeatureAccess(page, user, 'enhanced-notifications', 'advanced-alert-rules', true);
      }
    });

    test('portfolio history should require BRONZE+', async ({ page }) => {
      // FREE user should NOT have access
      await authenticateUser(page, TEST_USERS.FREE_USER);
      await page.goto('/portfolio');
      await testFeatureAccess(page, TEST_USERS.FREE_USER, 'portfolio-history', 'portfolio-history-chart', false);
      
      // BRONZE+ users should have access
      await authenticateUser(page, TEST_USERS.BRONZE_USER);
      await page.goto('/portfolio');
      await testFeatureAccess(page, TEST_USERS.BRONZE_USER, 'portfolio-history', 'portfolio-history-chart', true);
      await testFeatureAccess(page, TEST_USERS.BRONZE_USER, 'portfolio-history', 'performance-metrics', true);
    });
  });

  test.describe('🥈 SILVER+ Features', () => {
    
    test('advanced analytics should require SILVER+', async ({ page }) => {
      // FREE and BRONZE users should NOT have access
      const restrictedUsers = [TEST_USERS.FREE_USER, TEST_USERS.BRONZE_USER];
      
      for (const user of restrictedUsers) {
        await authenticateUser(page, user);
        await page.goto('/');
        await testFeatureAccess(page, user, 'advanced-analytics', 'advanced-analytics-button', false);
      }
      
      // SILVER+ users should have access
      const silverPlusUsers = [
        TEST_USERS.SILVER_USER,
        TEST_USERS.GOLD_USER,
        TEST_USERS.PLATINUM_USER,
        TEST_USERS.ENTERPRISE_USER
      ];

      for (const user of silverPlusUsers) {
        await authenticateUser(page, user);
        await page.goto('/advanced-analytics');
        
        await testFeatureAccess(page, user, 'advanced-analytics', 'technical-indicators', true);
        await testFeatureAccess(page, user, 'advanced-analytics', 'real-time-charts', true);
        await testFeatureAccess(page, user, 'advanced-analytics', 'market-sentiment', true);
      }
    });

    test('advanced trading features should require SILVER+', async ({ page }) => {
      // SILVER+ users should have advanced trading
      await authenticateUser(page, TEST_USERS.SILVER_USER);
      await page.goto('/trading');
      
      await testFeatureAccess(page, TEST_USERS.SILVER_USER, 'advanced-trading', 'limit-order-form', true);
      await testFeatureAccess(page, TEST_USERS.SILVER_USER, 'advanced-trading', 'stop-loss-form', true);
      await testFeatureAccess(page, TEST_USERS.SILVER_USER, 'advanced-trading', 'conditional-orders', true);
      
      // FREE users should NOT have advanced trading
      await authenticateUser(page, TEST_USERS.FREE_USER);
      await page.goto('/trading');
      await testFeatureAccess(page, TEST_USERS.FREE_USER, 'advanced-trading', 'limit-order-form', false);
    });
  });

  test.describe('🥇 GOLD+ Features', () => {
    
    test('portfolio tools should require GOLD+', async ({ page }) => {
      // SILVER and below should NOT have access
      const restrictedUsers = [TEST_USERS.FREE_USER, TEST_USERS.BRONZE_USER, TEST_USERS.SILVER_USER];
      
      for (const user of restrictedUsers) {
        await authenticateUser(page, user);
        await page.goto('/portfolio');
        await testFeatureAccess(page, user, 'portfolio-tools', 'portfolio-optimizer', false);
      }
      
      // GOLD+ users should have access
      const goldPlusUsers = [
        TEST_USERS.GOLD_USER,
        TEST_USERS.PLATINUM_USER,
        TEST_USERS.ENTERPRISE_USER
      ];

      for (const user of goldPlusUsers) {
        await authenticateUser(page, user);
        await page.goto('/portfolio');
        
        await testFeatureAccess(page, user, 'portfolio-tools', 'portfolio-optimizer', true);
        await testFeatureAccess(page, user, 'portfolio-tools', 'risk-analyzer', true);
        await testFeatureAccess(page, user, 'portfolio-tools', 'rebalancing-tool', true);
      }
    });

    test('priority support should require GOLD+', async ({ page }) => {
      // GOLD+ users should have priority support
      await authenticateUser(page, TEST_USERS.GOLD_USER);
      await page.goto('/priority-support');
      
      await testFeatureAccess(page, TEST_USERS.GOLD_USER, 'priority-support', 'priority-chat-widget', true);
      await testFeatureAccess(page, TEST_USERS.GOLD_USER, 'priority-support', 'dedicated-support-form', true);
      
      // SILVER and below should NOT have access
      await authenticateUser(page, TEST_USERS.SILVER_USER);
      await page.goto('/');
      await testFeatureAccess(page, TEST_USERS.SILVER_USER, 'priority-support', 'priority-support-button', false);
    });

    test('advanced order types should require GOLD+', async ({ page }) => {
      // GOLD+ users should have advanced orders
      await authenticateUser(page, TEST_USERS.GOLD_USER);
      await page.goto('/trading');
      
      await testFeatureAccess(page, TEST_USERS.GOLD_USER, 'advanced-order-types', 'trailing-stop-order', true);
      await testFeatureAccess(page, TEST_USERS.GOLD_USER, 'advanced-order-types', 'bracket-order', true);
      await testFeatureAccess(page, TEST_USERS.GOLD_USER, 'advanced-order-types', 'one-cancels-other', true);
    });
  });

  test.describe('🏆 PLATINUM+ Features', () => {
    
    test('research reports should require PLATINUM+', async ({ page }) => {
      // GOLD and below should NOT have access
      const restrictedUsers = [
        TEST_USERS.FREE_USER, 
        TEST_USERS.BRONZE_USER, 
        TEST_USERS.SILVER_USER, 
        TEST_USERS.GOLD_USER
      ];
      
      for (const user of restrictedUsers) {
        await authenticateUser(page, user);
        await page.goto('/');
        await testFeatureAccess(page, user, 'research-reports', 'research-reports-button', false);
      }
      
      // PLATINUM+ users should have access
      const platinumPlusUsers = [TEST_USERS.PLATINUM_USER, TEST_USERS.ENTERPRISE_USER];

      for (const user of platinumPlusUsers) {
        await authenticateUser(page, user);
        await page.goto('/reports');
        
        await testFeatureAccess(page, user, 'research-reports', 'analyst-reports', true);
        await testFeatureAccess(page, user, 'research-reports', 'market-research', true);
        await testFeatureAccess(page, user, 'research-reports', 'earnings-analysis', true);
      }
    });

    test('custom dashboards should require PLATINUM+', async ({ page }) => {
      // PLATINUM+ users should have custom dashboards
      await authenticateUser(page, TEST_USERS.PLATINUM_USER);
      await page.goto('/custom-dashboards');
      
      await testFeatureAccess(page, TEST_USERS.PLATINUM_USER, 'custom-dashboards', 'dashboard-builder', true);
      await testFeatureAccess(page, TEST_USERS.PLATINUM_USER, 'custom-dashboards', 'widget-library', true);
      await testFeatureAccess(page, TEST_USERS.PLATINUM_USER, 'custom-dashboards', 'layout-designer', true);
      
      // GOLD and below should NOT have access
      await authenticateUser(page, TEST_USERS.GOLD_USER);
      await page.goto('/');
      await testFeatureAccess(page, TEST_USERS.GOLD_USER, 'custom-dashboards', 'custom-dashboard-button', false);
    });
  });

  test.describe('🏢 ENTERPRISE Features', () => {
    
    test('API access should require ENTERPRISE', async ({ page }) => {
      // All non-enterprise users should NOT have access
      const nonEnterpriseUsers = [
        TEST_USERS.FREE_USER,
        TEST_USERS.BRONZE_USER,
        TEST_USERS.SILVER_USER,
        TEST_USERS.GOLD_USER,
        TEST_USERS.PLATINUM_USER
      ];
      
      for (const user of nonEnterpriseUsers) {
        await authenticateUser(page, user);
        await page.goto('/');
        await testFeatureAccess(page, user, 'api-access', 'api-access-button', false);
      }
      
      // ENTERPRISE users should have access
      await authenticateUser(page, TEST_USERS.ENTERPRISE_USER);
      await page.goto('/api-access');
      
      await testFeatureAccess(page, TEST_USERS.ENTERPRISE_USER, 'api-access', 'api-key-manager', true);
      await testFeatureAccess(page, TEST_USERS.ENTERPRISE_USER, 'api-access', 'api-documentation', true);
      await testFeatureAccess(page, TEST_USERS.ENTERPRISE_USER, 'api-access', 'usage-analytics', true);
    });

    test('institutional features should require ENTERPRISE', async ({ page }) => {
      // ENTERPRISE users should have institutional features
      await authenticateUser(page, TEST_USERS.ENTERPRISE_USER);
      await page.goto('/enterprise');
      
      await testFeatureAccess(page, TEST_USERS.ENTERPRISE_USER, 'institutional-features', 'bulk-operations', true);
      await testFeatureAccess(page, TEST_USERS.ENTERPRISE_USER, 'institutional-features', 'white-label-options', true);
      await testFeatureAccess(page, TEST_USERS.ENTERPRISE_USER, 'institutional-features', 'dedicated-support', true);
      
      // All other users should NOT have access
      await authenticateUser(page, TEST_USERS.PLATINUM_USER);
      await page.goto('/');
      await testFeatureAccess(page, TEST_USERS.PLATINUM_USER, 'institutional-features', 'enterprise-button', false);
    });

    test('bulk operations should require ENTERPRISE', async ({ page }) => {
      // ENTERPRISE users should have bulk operations
      await authenticateUser(page, TEST_USERS.ENTERPRISE_USER);
      await page.goto('/trading');
      
      await testFeatureAccess(page, TEST_USERS.ENTERPRISE_USER, 'bulk-operations', 'bulk-trade-form', true);
      await testFeatureAccess(page, TEST_USERS.ENTERPRISE_USER, 'bulk-operations', 'csv-upload', true);
      await testFeatureAccess(page, TEST_USERS.ENTERPRISE_USER, 'bulk-operations', 'batch-processing', true);
    });
  });

  test.describe('🎚️ Dynamic Feature Testing', () => {
    
    test('should dynamically show/hide features based on user tier', async ({ page }) => {
      const testUsers = [
        TEST_USERS.FREE_USER,
        TEST_USERS.SILVER_USER,
        TEST_USERS.GOLD_USER,
        TEST_USERS.ENTERPRISE_USER
      ];

      for (const user of testUsers) {
        await authenticateUser(page, user);
        await page.goto('/');
        
        const availableFeatures = getAvailableFeatures(user);
        console.log(`📊 ${user.package_tier} available features:`, availableFeatures);
        
        // Check navigation menu
        for (const feature of availableFeatures) {
          const navItem = page.locator(`[data-testid="nav-${feature}"]`);
          const isVisible = await navItem.isVisible().catch(() => false);
          
          if (['basic_data', 'limited_api'].includes(feature)) {
            // Basic features should always be visible
            expect(isVisible).toBe(true);
          }
        }
        
        // Check feature cards on dashboard
        if (availableFeatures.includes('advanced_analytics')) {
          await expect(page.locator('[data-testid="analytics-card"]')).toBeVisible();
        }
        
        if (availableFeatures.includes('premium_data')) {
          await expect(page.locator('[data-testid="premium-data-card"]')).toBeVisible();
        }
        
        if (availableFeatures.includes('all_features')) {
          await expect(page.locator('[data-testid="enterprise-features-section"]')).toBeVisible();
        }
      }
    });

    test('should validate feature combinations and dependencies', async ({ page }) => {
      // Test that higher-tier features depend on lower-tier features
      await authenticateUser(page, TEST_USERS.GOLD_USER);
      await page.goto('/portfolio');
      
      // Portfolio tools should be available
      await expect(page.locator('[data-testid="portfolio-optimizer"]')).toBeVisible();
      
      // But should still have basic portfolio features
      await expect(page.locator('[data-testid="portfolio-balance"]')).toBeVisible();
      await expect(page.locator('[data-testid="holdings-table"]')).toBeVisible();
      
      // And should have portfolio history from BRONZE tier
      await expect(page.locator('[data-testid="portfolio-history-chart"]')).toBeVisible();
    });

    test('should handle feature access during tier transitions', async ({ page }) => {
      // Start with SILVER user
      let user = { ...TEST_USERS.SILVER_USER };
      await authenticateUser(page, user);
      await page.goto('/professional');
      
      // Should access professional features
      await expect(page.locator('[data-testid="professional-dashboard"]')).toBeVisible();
      
      // Simulate downgrade to FREE (expired subscription)
      user.package_tier = 'FREE';
      user.permissions = ['trading:basic', 'portfolio:view', 'notifications:basic'];
      await authenticateUser(page, user);
      
      // Should be redirected when trying to access professional features
      const response = await page.goto('/professional');
      expect(page.url()).not.toContain('/professional');
    });
  });

  test.describe('🔒 Feature Security and Validation', () => {
    
    test('should validate API endpoint access by feature', async ({ page }) => {
      const testCases = [
        { 
          user: TEST_USERS.FREE_USER, 
          endpoint: '/api/analytics/advanced', 
          shouldAccess: false 
        },
        { 
          user: TEST_USERS.SILVER_USER, 
          endpoint: '/api/analytics/advanced', 
          shouldAccess: true 
        },
        { 
          user: TEST_USERS.GOLD_USER, 
          endpoint: '/api/portfolio/optimization', 
          shouldAccess: true 
        },
        { 
          user: TEST_USERS.PLATINUM_USER, 
          endpoint: '/api/research/reports', 
          shouldAccess: true 
        },
        { 
          user: TEST_USERS.ENTERPRISE_USER, 
          endpoint: '/api/enterprise/bulk', 
          shouldAccess: true 
        }
      ];

      for (const testCase of testCases) {
        await authenticateUser(page, testCase.user);
        
        const response = await page.request.get(testCase.endpoint);
        
        if (testCase.shouldAccess) {
          expect(response.status()).toBeLessThan(400);
        } else {
          expect(response.status()).toBeGreaterThanOrEqual(403);
        }
        
        console.log(`🔐 API Access: ${testCase.user.package_tier} -> ${testCase.endpoint}: ${response.status()}`);
      }
    });

    test('should prevent feature manipulation via client-side changes', async ({ page }) => {
      await authenticateUser(page, TEST_USERS.FREE_USER);
      await page.goto('/');
      
      // Try to inject premium features via DOM manipulation
      await page.evaluate(() => {
        const premiumButton = document.createElement('button');
        premiumButton.setAttribute('data-testid', 'premium-feature-injected');
        premiumButton.textContent = 'Premium Feature';
        document.body.appendChild(premiumButton);
      });
      
      // Click the injected button
      await page.locator('[data-testid="premium-feature-injected"]').click();
      
      // Should still be blocked at the API level
      const response = await page.request.get('/api/premium/feature');
      expect(response.status()).toBeGreaterThanOrEqual(403);
    });

    test('should validate feature access with expired tokens', async ({ page }) => {
      // Create an expired token
      const expiredUser = {
        ...TEST_USERS.GOLD_USER,
        jwt_token: 'expired_token_12345'
      };
      
      await page.context().addCookies([{
        name: 'epsx_jwt',
        value: 'expired_token_12345',
        domain: 'localhost',
        path: '/',
        httpOnly: true,
        secure: false
      }]);
      
      // Should be redirected to login
      const response = await page.goto('/vip');
      expect(page.url()).toContain('/oauth/authorize');
    });
  });

  test.describe('📈 Feature Usage Analytics', () => {
    
    test('should track feature usage by tier', async ({ page }) => {
      const users = [TEST_USERS.SILVER_USER, TEST_USERS.GOLD_USER, TEST_USERS.ENTERPRISE_USER];
      
      for (const user of users) {
        await authenticateUser(page, user);
        await page.goto('/');
        
        // Use available features
        const features = getAvailableFeatures(user);
        
        if (features.includes('advanced_analytics')) {
          await page.goto('/advanced-analytics');
          await page.locator('[data-testid="technical-indicators"]').click();
        }
        
        if (features.includes('portfolio_tools')) {
          await page.goto('/portfolio');
          await page.locator('[data-testid="portfolio-optimizer"]').click();
        }
        
        if (features.includes('api_access')) {
          await page.goto('/api-access');
          await page.locator('[data-testid="api-key-manager"]').click();
        }
        
        console.log(`📊 Feature usage tracking completed for ${user.package_tier}`);
      }
    });
  });
});