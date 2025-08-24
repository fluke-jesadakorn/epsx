import { test, expect, Page, BrowserContext } from '@playwright/test';
import { 
  TEST_USERS, 
  TIER_FEATURES, 
  getUserByTier, 
  getUsersForTierTesting, 
  getSpecialCaseUsers,
  canUserAccessRoute, 
  canUserAccessFeature,
  isFeatureRestricted,
  generateMockJWT,
  TestUser
} from '../fixtures/user-fixtures';

/**
 * Comprehensive Package Tier Permission Tests
 * Tests all 6 package tiers (FREE, BRONZE, SILVER, GOLD, PLATINUM, ENTERPRISE)
 * Validates feature access control, route protection, and tier-based restrictions
 */

test.describe('🎯 Package Tier Permission System', () => {
  
  // Helper function to authenticate user and set JWT token
  async function authenticateUser(page: Page, user: TestUser): Promise<void> {
    const jwtToken = generateMockJWT(user);
    
    // Set JWT token in cookie
    await page.context().addCookies([{
      name: 'epsx_jwt',
      value: jwtToken,
      domain: 'localhost',
      path: '/',
      httpOnly: true,
      secure: false
    }]);
    
    console.log(`🔐 Authenticated user: ${user.email} (${user.package_tier})`);
  }

  // Helper function to verify route access
  async function verifyRouteAccess(page: Page, route: string, shouldHaveAccess: boolean): Promise<void> {
    console.log(`🔍 Testing route access: ${route} (expected: ${shouldHaveAccess})`);
    
    const response = await page.goto(route);
    
    if (shouldHaveAccess) {
      // Should not be redirected to login or upgrade page
      expect(page.url()).not.toContain('/oauth/authorize');
      expect(page.url()).not.toContain('/upgrade');
      expect(page.url()).not.toContain('/access-denied');
      expect(response?.status()).toBeLessThan(400);
    } else {
      // Should be redirected or show access denied
      const currentUrl = page.url();
      const isRedirected = currentUrl.includes('/oauth/authorize') || 
                          currentUrl.includes('/upgrade') || 
                          currentUrl.includes('/access-denied');
      expect(isRedirected).toBe(true);
    }
  }

  // Helper function to verify feature availability in UI
  async function verifyFeatureAvailability(page: Page, user: TestUser): Promise<void> {
    const tierFeatures = TIER_FEATURES[user.package_tier];
    
    // Check navigation menu for tier-specific features
    if (tierFeatures.features.includes('advanced-analytics')) {
      await expect(page.locator('[data-testid="nav-analytics"]')).toBeVisible();
    } else {
      // Should either be hidden or show upgrade prompt
      const analyticsNav = page.locator('[data-testid="nav-analytics"]');
      const isVisible = await analyticsNav.isVisible().catch(() => false);
      if (isVisible) {
        // If visible, clicking should show upgrade prompt
        await analyticsNav.click();
        await expect(page.locator('[data-testid="upgrade-prompt"]')).toBeVisible();
      }
    }
    
    if (tierFeatures.features.includes('portfolio-tools')) {
      await expect(page.locator('[data-testid="nav-portfolio-tools"]')).toBeVisible();
    }
    
    if (tierFeatures.features.includes('api-access')) {
      await expect(page.locator('[data-testid="nav-api-access"]')).toBeVisible();
    }
  }

  test.describe('🔓 FREE Tier (Basic Access)', () => {
    const freeUser = TEST_USERS.FREE_USER;

    test.beforeEach(async ({ page }) => {
      await authenticateUser(page, freeUser);
    });

    test('should access basic trading features only', async ({ page }) => {
      // Should access basic routes
      await verifyRouteAccess(page, '/', true);
      await verifyRouteAccess(page, '/trading', true);
      await verifyRouteAccess(page, '/portfolio', true);
      
      // Should NOT access premium routes
      await verifyRouteAccess(page, '/premium', false);
      await verifyRouteAccess(page, '/advanced-analytics', false);
      await verifyRouteAccess(page, '/professional', false);
      await verifyRouteAccess(page, '/vip', false);
      await verifyRouteAccess(page, '/elite', false);
      await verifyRouteAccess(page, '/enterprise', false);
      await verifyRouteAccess(page, '/api-access', false);
    });

    test('should show upgrade prompts for premium features', async ({ page }) => {
      await page.goto('/');
      
      // Try to access premium feature - should show upgrade prompt
      const premiumFeature = page.locator('[data-testid="premium-analytics-button"]');
      if (await premiumFeature.isVisible()) {
        await premiumFeature.click();
        await expect(page.locator('[data-testid="upgrade-modal"]')).toBeVisible();
        await expect(page.locator('[data-testid="upgrade-to-bronze"]')).toBeVisible();
      }
    });

    test('should have correct rate limits', async ({ page }) => {
      const user = freeUser;
      expect(user.rate_limits.per_minute).toBe(10);
      expect(user.rate_limits.per_hour).toBe(100);
    });
  });

  test.describe('🥉 BRONZE Tier (Enhanced Access)', () => {
    const bronzeUser = TEST_USERS.BRONZE_USER;

    test.beforeEach(async ({ page }) => {
      await authenticateUser(page, bronzeUser);
    });

    test('should access bronze-level features', async ({ page }) => {
      // Should access free + bronze routes
      await verifyRouteAccess(page, '/', true);
      await verifyRouteAccess(page, '/trading', true);
      await verifyRouteAccess(page, '/portfolio', true);
      await verifyRouteAccess(page, '/premium', true);
      await verifyRouteAccess(page, '/advanced-analytics', true);
      
      // Should NOT access higher-tier routes
      await verifyRouteAccess(page, '/professional', false);
      await verifyRouteAccess(page, '/vip', false);
      await verifyRouteAccess(page, '/elite', false);
      await verifyRouteAccess(page, '/enterprise', false);
      await verifyRouteAccess(page, '/api-access', false);
    });

    test('should show enhanced notifications and portfolio history', async ({ page }) => {
      await page.goto('/portfolio');
      
      // Should see enhanced features
      await expect(page.locator('[data-testid="portfolio-history-chart"]')).toBeVisible();
      await expect(page.locator('[data-testid="enhanced-notifications-panel"]')).toBeVisible();
      
      // Should still see upgrade options for higher tiers
      await expect(page.locator('[data-testid="upgrade-to-silver"]')).toBeVisible();
    });

    test('should have increased rate limits', async ({ page }) => {
      const user = bronzeUser;
      expect(user.rate_limits.per_minute).toBe(30);
      expect(user.rate_limits.per_hour).toBe(500);
    });
  });

  test.describe('🥈 SILVER Tier (Professional Access)', () => {
    const silverUser = TEST_USERS.SILVER_USER;

    test.beforeEach(async ({ page }) => {
      await authenticateUser(page, silverUser);
    });

    test('should access silver-level trading features', async ({ page }) => {
      // Should access free + bronze + silver routes
      await verifyRouteAccess(page, '/', true);
      await verifyRouteAccess(page, '/trading', true);
      await verifyRouteAccess(page, '/portfolio', true);
      await verifyRouteAccess(page, '/premium', true);
      await verifyRouteAccess(page, '/advanced-analytics', true);
      await verifyRouteAccess(page, '/professional', true);
      await verifyRouteAccess(page, '/alerts', true);
      
      // Should NOT access gold+ routes
      await verifyRouteAccess(page, '/vip', false);
      await verifyRouteAccess(page, '/priority-support', false);
      await verifyRouteAccess(page, '/elite', false);
      await verifyRouteAccess(page, '/enterprise', false);
      await verifyRouteAccess(page, '/api-access', false);
    });

    test('should access advanced trading tools', async ({ page }) => {
      await page.goto('/trading');
      
      // Should see advanced order types
      await expect(page.locator('[data-testid="advanced-order-types"]')).toBeVisible();
      await expect(page.locator('[data-testid="technical-indicators"]')).toBeVisible();
      await expect(page.locator('[data-testid="stop-loss-orders"]')).toBeVisible();
      
      // Go to analytics page
      await page.goto('/advanced-analytics');
      await expect(page.locator('[data-testid="real-time-data-feed"]')).toBeVisible();
      await expect(page.locator('[data-testid="advanced-charts"]')).toBeVisible();
    });

    test('should have professional rate limits', async ({ page }) => {
      const user = silverUser;
      expect(user.rate_limits.per_minute).toBe(60);
      expect(user.rate_limits.per_hour).toBe(1500);
    });
  });

  test.describe('🥇 GOLD Tier (VIP Access)', () => {
    const goldUser = TEST_USERS.GOLD_USER;

    test.beforeEach(async ({ page }) => {
      await authenticateUser(page, goldUser);
    });

    test('should access gold-level premium features', async ({ page }) => {
      // Should access all routes except platinum/enterprise
      await verifyRouteAccess(page, '/', true);
      await verifyRouteAccess(page, '/trading', true);
      await verifyRouteAccess(page, '/portfolio', true);
      await verifyRouteAccess(page, '/premium', true);
      await verifyRouteAccess(page, '/advanced-analytics', true);
      await verifyRouteAccess(page, '/professional', true);
      await verifyRouteAccess(page, '/alerts', true);
      await verifyRouteAccess(page, '/vip', true);
      await verifyRouteAccess(page, '/priority-support', true);
      
      // Should NOT access platinum+ routes
      await verifyRouteAccess(page, '/elite', false);
      await verifyRouteAccess(page, '/custom-dashboards', false);
      await verifyRouteAccess(page, '/reports', false);
      await verifyRouteAccess(page, '/enterprise', false);
      await verifyRouteAccess(page, '/api-access', false);
    });

    test('should access portfolio optimization tools', async ({ page }) => {
      await page.goto('/portfolio');
      
      // Should see portfolio tools
      await expect(page.locator('[data-testid="portfolio-optimizer"]')).toBeVisible();
      await expect(page.locator('[data-testid="risk-management-tools"]')).toBeVisible();
      await expect(page.locator('[data-testid="performance-analytics"]')).toBeVisible();
      
      // Should see VIP features
      await page.goto('/vip');
      await expect(page.locator('[data-testid="vip-dashboard"]')).toBeVisible();
      await expect(page.locator('[data-testid="priority-support-widget"]')).toBeVisible();
    });

    test('should have VIP rate limits', async ({ page }) => {
      const user = goldUser;
      expect(user.rate_limits.per_minute).toBe(120);
      expect(user.rate_limits.per_hour).toBe(5000);
    });

    test('should show research report previews but not full access', async ({ page }) => {
      await page.goto('/');
      
      // Should see research report previews with upgrade prompt
      const researchPreview = page.locator('[data-testid="research-reports-preview"]');
      if (await researchPreview.isVisible()) {
        await researchPreview.click();
        await expect(page.locator('[data-testid="upgrade-to-platinum"]')).toBeVisible();
      }
    });
  });

  test.describe('🏆 PLATINUM Tier (Elite Access)', () => {
    const platinumUser = TEST_USERS.PLATINUM_USER;

    test.beforeEach(async ({ page }) => {
      await authenticateUser(page, platinumUser);
    });

    test('should access platinum-level research features', async ({ page }) => {
      // Should access all routes except enterprise
      await verifyRouteAccess(page, '/', true);
      await verifyRouteAccess(page, '/trading', true);
      await verifyRouteAccess(page, '/portfolio', true);
      await verifyRouteAccess(page, '/premium', true);
      await verifyRouteAccess(page, '/advanced-analytics', true);
      await verifyRouteAccess(page, '/professional', true);
      await verifyRouteAccess(page, '/alerts', true);
      await verifyRouteAccess(page, '/vip', true);
      await verifyRouteAccess(page, '/priority-support', true);
      await verifyRouteAccess(page, '/elite', true);
      await verifyRouteAccess(page, '/custom-dashboards', true);
      await verifyRouteAccess(page, '/reports', true);
      
      // Should NOT access enterprise routes
      await verifyRouteAccess(page, '/enterprise', false);
      await verifyRouteAccess(page, '/api-access', false);
    });

    test('should access research reports and custom dashboards', async ({ page }) => {
      await page.goto('/reports');
      
      // Should see full research reports
      await expect(page.locator('[data-testid="research-reports-full"]')).toBeVisible();
      await expect(page.locator('[data-testid="analyst-recommendations"]')).toBeVisible();
      await expect(page.locator('[data-testid="market-insights"]')).toBeVisible();
      
      // Should access custom dashboards
      await page.goto('/custom-dashboards');
      await expect(page.locator('[data-testid="dashboard-builder"]')).toBeVisible();
      await expect(page.locator('[data-testid="custom-widgets"]')).toBeVisible();
    });

    test('should have premium rate limits', async ({ page }) => {
      const user = platinumUser;
      expect(user.rate_limits.per_minute).toBe(300);
      expect(user.rate_limits.per_hour).toBe(15000);
    });

    test('should show API access upgrade prompts', async ({ page }) => {
      await page.goto('/');
      
      // Should see API access promotion but not full access
      const apiPromotion = page.locator('[data-testid="api-access-promotion"]');
      if (await apiPromotion.isVisible()) {
        await apiPromotion.click();
        await expect(page.locator('[data-testid="upgrade-to-enterprise"]')).toBeVisible();
      }
    });
  });

  test.describe('🏢 ENTERPRISE Tier (Full Access)', () => {
    const enterpriseUser = TEST_USERS.ENTERPRISE_USER;

    test.beforeEach(async ({ page }) => {
      await authenticateUser(page, enterpriseUser);
    });

    test('should access all enterprise features', async ({ page }) => {
      // Should access ALL routes
      await verifyRouteAccess(page, '/', true);
      await verifyRouteAccess(page, '/trading', true);
      await verifyRouteAccess(page, '/portfolio', true);
      await verifyRouteAccess(page, '/premium', true);
      await verifyRouteAccess(page, '/advanced-analytics', true);
      await verifyRouteAccess(page, '/professional', true);
      await verifyRouteAccess(page, '/alerts', true);
      await verifyRouteAccess(page, '/vip', true);
      await verifyRouteAccess(page, '/priority-support', true);
      await verifyRouteAccess(page, '/elite', true);
      await verifyRouteAccess(page, '/custom-dashboards', true);
      await verifyRouteAccess(page, '/reports', true);
      await verifyRouteAccess(page, '/enterprise', true);
      await verifyRouteAccess(page, '/api-access', true);
    });

    test('should access institutional features and API management', async ({ page }) => {
      await page.goto('/enterprise');
      
      // Should see enterprise features
      await expect(page.locator('[data-testid="institutional-dashboard"]')).toBeVisible();
      await expect(page.locator('[data-testid="bulk-operations"]')).toBeVisible();
      await expect(page.locator('[data-testid="white-label-options"]')).toBeVisible();
      
      // Should access API management
      await page.goto('/api-access');
      await expect(page.locator('[data-testid="api-key-management"]')).toBeVisible();
      await expect(page.locator('[data-testid="api-usage-analytics"]')).toBeVisible();
      await expect(page.locator('[data-testid="webhook-configuration"]')).toBeVisible();
    });

    test('should have maximum rate limits', async ({ page }) => {
      const user = enterpriseUser;
      expect(user.rate_limits.per_minute).toBe(1000);
      expect(user.rate_limits.per_hour).toBe(50000);
    });

    test('should not show any upgrade prompts', async ({ page }) => {
      await page.goto('/');
      
      // Should NOT see any upgrade prompts
      await expect(page.locator('[data-testid="upgrade-modal"]')).not.toBeVisible();
      await expect(page.locator('[data-testid="upgrade-to-platinum"]')).not.toBeVisible();
      await expect(page.locator('[data-testid="upgrade-banner"]')).not.toBeVisible();
    });
  });

  test.describe('⚠️ Special Cases (Expired, Trial, Cancelled)', () => {
    
    test('expired user should be downgraded to FREE tier', async ({ page }) => {
      const expiredUser = TEST_USERS.EXPIRED_USER;
      await authenticateUser(page, expiredUser);
      
      // Should only access free routes
      await verifyRouteAccess(page, '/', true);
      await verifyRouteAccess(page, '/trading', true);
      await verifyRouteAccess(page, '/portfolio', true);
      await verifyRouteAccess(page, '/premium', false);
      await verifyRouteAccess(page, '/advanced-analytics', false);
      
      // Should show reactivation prompts
      await page.goto('/');
      await expect(page.locator('[data-testid="reactivate-subscription"]')).toBeVisible();
    });

    test('trial user should have temporary access to GOLD features', async ({ page }) => {
      const trialUser = TEST_USERS.TRIAL_USER;
      await authenticateUser(page, trialUser);
      
      // Should access gold-level routes during trial
      await verifyRouteAccess(page, '/vip', true);
      await verifyRouteAccess(page, '/priority-support', true);
      
      // Should NOT access platinum+ routes
      await verifyRouteAccess(page, '/elite', false);
      await verifyRouteAccess(page, '/reports', false);
      
      // Should show trial notification
      await page.goto('/');
      await expect(page.locator('[data-testid="trial-notification"]')).toBeVisible();
      await expect(page.locator('[data-testid="days-remaining"]')).toContainText('30'); // Approximate
    });

    test('cancelled user should maintain access until expiry', async ({ page }) => {
      const cancelledUser = TEST_USERS.CANCELLED_USER;
      await authenticateUser(page, cancelledUser);
      
      // Should still access SILVER features until expiry
      await verifyRouteAccess(page, '/professional', true);
      await verifyRouteAccess(page, '/alerts', true);
      
      // Should NOT access higher tiers
      await verifyRouteAccess(page, '/vip', false);
      
      // Should show cancellation notice
      await page.goto('/');
      await expect(page.locator('[data-testid="cancellation-notice"]')).toBeVisible();
      await expect(page.locator('[data-testid="reactivate-option"]')).toBeVisible();
    });
  });

  test.describe('🔄 Cross-Tier Access Validation', () => {
    
    test('should enforce strict tier boundaries', async ({ page }) => {
      const testCases = [
        { user: TEST_USERS.FREE_USER, route: '/enterprise', shouldAccess: false },
        { user: TEST_USERS.BRONZE_USER, route: '/vip', shouldAccess: false },
        { user: TEST_USERS.SILVER_USER, route: '/reports', shouldAccess: false },
        { user: TEST_USERS.GOLD_USER, route: '/api-access', shouldAccess: false },
        { user: TEST_USERS.PLATINUM_USER, route: '/enterprise', shouldAccess: false },
        { user: TEST_USERS.ENTERPRISE_USER, route: '/api-access', shouldAccess: true }
      ];

      for (const testCase of testCases) {
        await authenticateUser(page, testCase.user);
        await verifyRouteAccess(page, testCase.route, testCase.shouldAccess);
        console.log(`✅ ${testCase.user.package_tier} -> ${testCase.route}: ${testCase.shouldAccess ? 'ALLOWED' : 'BLOCKED'}`);
      }
    });

    test('should validate feature access across all tiers', async ({ page }) => {
      const allUsers = getUsersForTierTesting();
      
      for (const user of allUsers) {
        await authenticateUser(page, user);
        await page.goto('/');
        await verifyFeatureAvailability(page, user);
        console.log(`✅ Feature validation completed for ${user.package_tier}`);
      }
    });

    test('should handle tier transition scenarios', async ({ page, context }) => {
      // Simulate user upgrading from SILVER to GOLD
      let user = TEST_USERS.SILVER_USER;
      await authenticateUser(page, user);
      
      // Initially should NOT access VIP features
      await verifyRouteAccess(page, '/vip', false);
      
      // Simulate upgrade (in real app, this would be through payment flow)
      const upgradedUser = { ...user, package_tier: 'GOLD', permissions: TEST_USERS.GOLD_USER.permissions };
      await authenticateUser(page, upgradedUser);
      
      // Now should access VIP features
      await verifyRouteAccess(page, '/vip', true);
      
      console.log('✅ Tier upgrade scenario validated');
    });
  });

  test.describe('🚫 Security Boundary Testing', () => {
    
    test('should prevent privilege escalation attempts', async ({ page }) => {
      const freeUser = TEST_USERS.FREE_USER;
      await authenticateUser(page, freeUser);
      
      // Attempt direct navigation to premium routes
      const premiumRoutes = ['/enterprise', '/api-access', '/elite', '/vip'];
      
      for (const route of premiumRoutes) {
        const response = await page.goto(route);
        
        // Should be redirected or blocked
        expect(page.url()).not.toBe(`http://localhost:3000${route}`);
        
        // Should either be on upgrade page or access denied
        const isBlocked = page.url().includes('/upgrade') || 
                         page.url().includes('/access-denied') ||
                         page.url().includes('/oauth/authorize');
        expect(isBlocked).toBe(true);
      }
    });

    test('should validate API endpoint access restrictions', async ({ page }) => {
      const silverUser = TEST_USERS.SILVER_USER;
      await authenticateUser(page, silverUser);
      
      // Should access silver-level API endpoints
      const silverResponse = await page.request.get('/api/analytics/basic');
      expect(silverResponse.status()).toBeLessThan(400);
      
      // Should NOT access enterprise API endpoints
      const enterpriseResponse = await page.request.get('/api/enterprise/bulk');
      expect(enterpriseResponse.status()).toBeGreaterThanOrEqual(403);
    });

    test('should enforce rate limiting by tier', async ({ page }) => {
      const freeUser = TEST_USERS.FREE_USER;
      await authenticateUser(page, freeUser);
      
      // Make rapid requests to test rate limiting
      const requests = [];
      for (let i = 0; i < 15; i++) { // Exceed FREE tier limit of 10/minute
        requests.push(page.request.get('/api/portfolio/basic'));
      }
      
      const responses = await Promise.all(requests);
      const rateLimitedResponses = responses.filter(r => r.status() === 429);
      
      // Should have at least some rate limited responses
      expect(rateLimitedResponses.length).toBeGreaterThan(0);
    });
  });

  test.describe('📊 Performance and Metrics', () => {
    
    test('should track tier-based performance metrics', async ({ page }) => {
      const users = [TEST_USERS.FREE_USER, TEST_USERS.GOLD_USER, TEST_USERS.ENTERPRISE_USER];
      
      for (const user of users) {
        await authenticateUser(page, user);
        
        const startTime = Date.now();
        await page.goto('/dashboard');
        const loadTime = Date.now() - startTime;
        
        console.log(`📊 ${user.package_tier} dashboard load time: ${loadTime}ms`);
        
        // Higher tiers might have more features, so allow reasonable variance
        expect(loadTime).toBeLessThan(5000); // 5 second maximum
      }
    });

    test('should validate caching behavior by tier', async ({ page }) => {
      const enterpriseUser = TEST_USERS.ENTERPRISE_USER;
      await authenticateUser(page, enterpriseUser);
      
      // First request
      const start1 = Date.now();
      await page.goto('/advanced-analytics');
      const time1 = Date.now() - start1;
      
      // Second request (should be faster due to caching)
      const start2 = Date.now();
      await page.reload();
      const time2 = Date.now() - start2;
      
      console.log(`📊 Cache performance: First: ${time1}ms, Second: ${time2}ms`);
      
      // Second request should be faster (accounting for cache)
      expect(time2).toBeLessThan(time1 * 1.5); // Allow 50% variance
    });
  });
});