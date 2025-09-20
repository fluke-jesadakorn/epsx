import { test, expect, Page, BrowserContext } from '@playwright/test';
import { 
  TEST_USERS, 
  TIER_FEATURES, 
  getUserByTier, 
  getPermissionTestUsers, // NEW: Permission-based testing
  getSpecialCaseUsers,
  canUserAccessRoute, 
  canUserAccessFeature,
  isFeatureRestricted,
  generateMockJWT,
  TestUser,
  // NEW: Permission-based helpers
  getUserRankingLimit,
  userHasPermission,
  canUserViewRanking,
  deriveTierFromUserPermissions,
  createTestUserWithPermissions
} from '../fixtures/user-fixtures';

/**
 * Comprehensive Permission-Based Access Control Tests
 * Tests structured permissions (e.g., "epsx:rankings:view:25")
 * Validates dynamic ranking limits and permission-based access control
 * Maintains UI compatibility with familiar tier names
 */

test.describe('🎯 Permission-Based Access Control System', () => {
  
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
    
    // Log both legacy tier and actual permissions for debugging
    const derivedTier = deriveTierFromUserPermissions(user);
    const rankingLimit = getUserRankingLimit(user);
    console.log(`🔐 Authenticated user: ${user.email} (${derivedTier}, rankings: ${rankingLimit === -1 ? 'unlimited' : rankingLimit})`);
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

  // Helper function to verify feature availability based on permissions
  async function verifyFeatureAvailability(page: Page, user: TestUser): Promise<void> {
    // Check navigation menu for permission-specific features
    if (userHasPermission(user, 'epsx:analytics:advanced')) {
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
    
    if (userHasPermission(user, 'epsx:portfolio:tools')) {
      await expect(page.locator('[data-testid="nav-portfolio-tools"]')).toBeVisible();
    }
    
    if (userHasPermission(user, 'admin:*:*') || userHasPermission(user, 'epsx:*:*')) {
      await expect(page.locator('[data-testid="nav-api-access"]')).toBeVisible();
    }
  }

  // NEW: Helper function to verify ranking limits
  async function verifyRankingLimits(page: Page, user: TestUser): Promise<void> {
    await page.goto('/rankings');
    
    const rankingLimit = getUserRankingLimit(user);
    const derivedTier = deriveTierFromUserPermissions(user);
    
    if (rankingLimit === -1) {
      // Unlimited access - should see all rankings
      const allRankings = page.locator('[data-testid="ranking-item"]');
      const count = await allRankings.count();
      console.log(`✅ ${derivedTier} user sees ${count} rankings (unlimited)`);
    } else {
      // Limited access - should only see up to the limit
      const visibleRankings = page.locator('[data-testid="ranking-item"]:not([data-locked="true"])');
      const lockedRankings = page.locator('[data-testid="ranking-item"][data-locked="true"]');
      
      const visibleCount = await visibleRankings.count();
      const lockedCount = await lockedRankings.count();
      
      expect(visibleCount).toBeLessThanOrEqual(rankingLimit);
      console.log(`✅ ${derivedTier} user sees ${visibleCount} visible + ${lockedCount} locked rankings (limit: ${rankingLimit})`);
      
      if (visibleCount < rankingLimit) {
        // Should see upgrade prompts for additional rankings
        await expect(page.locator('[data-testid="ranking-upgrade-prompt"]')).toBeVisible();
      }
    }
  }

  test.describe('🔓 FREE Tier (Basic Access - 3 Rankings)', () => {
    const freeUser = TEST_USERS.FREE_USER;

    test.beforeEach(async ({ page }) => {
      await authenticateUser(page, freeUser);
    });

    test('should validate permission-based access control', async ({ page }) => {
      // Verify user has expected permissions
      expect(userHasPermission(freeUser, 'epsx:rankings:view:3')).toBe(true);
      expect(userHasPermission(freeUser, 'epsx:trading:basic')).toBe(true);
      expect(userHasPermission(freeUser, 'epsx:portfolio:view')).toBe(true);
      expect(userHasPermission(freeUser, 'epsx:trading:advanced')).toBe(false);
      
      // Verify derived tier matches expectation
      expect(deriveTierFromUserPermissions(freeUser)).toBe('FREE');
      expect(getUserRankingLimit(freeUser)).toBe(3);
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

    test('should enforce 3-ranking limit', async ({ page }) => {
      await verifyRankingLimits(page, freeUser);
      
      // Verify specific ranking positions
      expect(canUserViewRanking(freeUser, 1)).toBe(true);  // Position 1
      expect(canUserViewRanking(freeUser, 3)).toBe(true);  // Position 3
      expect(canUserViewRanking(freeUser, 4)).toBe(false); // Position 4 (beyond limit)
      expect(canUserViewRanking(freeUser, 10)).toBe(false); // Position 10
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

  test.describe('🥉 BRONZE Tier (Enhanced Access - 5 Rankings)', () => {
    const bronzeUser = TEST_USERS.BRONZE_USER;

    test.beforeEach(async ({ page }) => {
      await authenticateUser(page, bronzeUser);
    });

    test('should validate permission-based access control', async ({ page }) => {
      // Verify user has expected permissions
      expect(userHasPermission(bronzeUser, 'epsx:rankings:view:5')).toBe(true);
      expect(userHasPermission(bronzeUser, 'epsx:trading:basic')).toBe(true);
      expect(userHasPermission(bronzeUser, 'epsx:portfolio:history')).toBe(true);
      expect(userHasPermission(bronzeUser, 'epsx:analytics:basic')).toBe(true);
      expect(userHasPermission(bronzeUser, 'epsx:trading:premium')).toBe(false); // Not in bronze
      
      // Verify derived tier matches expectation
      expect(deriveTierFromUserPermissions(bronzeUser)).toBe('BRONZE');
      expect(getUserRankingLimit(bronzeUser)).toBe(5);
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

    test('should enforce 5-ranking limit', async ({ page }) => {
      await verifyRankingLimits(page, bronzeUser);
      
      // Verify specific ranking positions
      expect(canUserViewRanking(bronzeUser, 1)).toBe(true);  // Position 1
      expect(canUserViewRanking(bronzeUser, 5)).toBe(true);  // Position 5
      expect(canUserViewRanking(bronzeUser, 6)).toBe(false); // Position 6 (beyond limit)
      expect(canUserViewRanking(bronzeUser, 25)).toBe(false); // Position 25 (Silver limit)
    });

    test('should show enhanced notifications and portfolio history', async ({ page }) => {
      await page.goto('/portfolio');
      
      // Should see enhanced features based on permissions
      if (userHasPermission(bronzeUser, 'epsx:portfolio:history')) {
        await expect(page.locator('[data-testid="portfolio-history-chart"]')).toBeVisible();
      }
      if (userHasPermission(bronzeUser, 'epsx:notifications:enhanced')) {
        await expect(page.locator('[data-testid="enhanced-notifications-panel"]')).toBeVisible();
      }
      
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

  test.describe('🏢 ENTERPRISE Tier (Unlimited Access)', () => {
    const enterpriseUser = TEST_USERS.ENTERPRISE_USER;

    test.beforeEach(async ({ page }) => {
      await authenticateUser(page, enterpriseUser);
    });

    test('should validate unlimited permission structure', async ({ page }) => {
      // Verify user has expected unlimited permissions
      expect(userHasPermission(enterpriseUser, 'epsx:rankings:view:unlimited')).toBe(true);
      expect(userHasPermission(enterpriseUser, 'epsx:*:*')).toBe(true);
      expect(userHasPermission(enterpriseUser, 'epsx-pay:*:*')).toBe(true);
      expect(userHasPermission(enterpriseUser, 'epsx-token:*:*')).toBe(true);
      expect(userHasPermission(enterpriseUser, 'admin:*:*')).toBe(true);
      
      // Verify derived tier matches expectation
      expect(deriveTierFromUserPermissions(enterpriseUser)).toBe('ENTERPRISE');
      expect(getUserRankingLimit(enterpriseUser)).toBe(-1); // Unlimited
    });

    test('should have unlimited ranking access', async ({ page }) => {
      await verifyRankingLimits(page, enterpriseUser);
      
      // Verify unlimited access to any ranking position
      expect(canUserViewRanking(enterpriseUser, 1)).toBe(true);
      expect(canUserViewRanking(enterpriseUser, 100)).toBe(true);
      expect(canUserViewRanking(enterpriseUser, 1000)).toBe(true);
      expect(canUserViewRanking(enterpriseUser, 9999)).toBe(true);
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

    test('should have wildcard permissions across all platforms', async ({ page }) => {
      // Test EPSX platform wildcard
      expect(userHasPermission(enterpriseUser, 'epsx:trading:any')).toBe(true);
      expect(userHasPermission(enterpriseUser, 'epsx:analytics:premium')).toBe(true);
      expect(userHasPermission(enterpriseUser, 'epsx:research:reports')).toBe(true);
      
      // Test EPSX Pay platform wildcard
      expect(userHasPermission(enterpriseUser, 'epsx-pay:transactions:manage')).toBe(true);
      expect(userHasPermission(enterpriseUser, 'epsx-pay:payments:process')).toBe(true);
      
      // Test EPSX Token platform wildcard
      expect(userHasPermission(enterpriseUser, 'epsx-token:governance:vote')).toBe(true);
      expect(userHasPermission(enterpriseUser, 'epsx-token:tokens:stake')).toBe(true);
      
      // Test Admin platform wildcard
      expect(userHasPermission(enterpriseUser, 'admin:users:manage')).toBe(true);
      expect(userHasPermission(enterpriseUser, 'admin:system:configure')).toBe(true);
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
      await expect(page.locator('[data-testid="ranking-upgrade-prompt"]')).not.toBeVisible();
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
        const derivedTier = deriveTierFromUserPermissions(testCase.user);
        console.log(`✅ ${derivedTier} -> ${testCase.route}: ${testCase.shouldAccess ? 'ALLOWED' : 'BLOCKED'}`);
      }
    });

    test('should validate permission-based feature access across all tiers', async ({ page }) => {
      const allUsers = getPermissionTestUsers(); // Use new permission-based function
      
      for (const user of allUsers) {
        await authenticateUser(page, user);
        await page.goto('/');
        await verifyFeatureAvailability(page, user);
        await verifyRankingLimits(page, user);
        
        const derivedTier = deriveTierFromUserPermissions(user);
        const rankingLimit = getUserRankingLimit(user);
        console.log(`✅ Permission validation completed for ${derivedTier} (${rankingLimit === -1 ? 'unlimited' : rankingLimit} rankings)`);
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

  test.describe('🚀 Dynamic Permission System (NEW)', () => {
    
    test('should support custom ranking limits', async ({ page }) => {
      // Create custom users with non-standard limits
      const customUsers = [
        createTestUserWithPermissions(['epsx:rankings:view:7', 'epsx:trading:basic'], { email: 'custom7@test.com' }),
        createTestUserWithPermissions(['epsx:rankings:view:37', 'epsx:trading:advanced'], { email: 'custom37@test.com' }),
        createTestUserWithPermissions(['epsx:rankings:view:123', 'epsx:portfolio:tools'], { email: 'custom123@test.com' })
      ];
      
      for (const user of customUsers) {
        await authenticateUser(page, user);
        
        const rankingLimit = getUserRankingLimit(user);
        const derivedTier = deriveTierFromUserPermissions(user);
        
        console.log(`🔧 Testing custom user: ${user.email} (limit: ${rankingLimit}, tier: ${derivedTier})`);
        
        // Verify ranking access follows custom limit
        expect(canUserViewRanking(user, 1)).toBe(true);
        expect(canUserViewRanking(user, rankingLimit)).toBe(true);
        if (rankingLimit > 0) {
          expect(canUserViewRanking(user, rankingLimit + 1)).toBe(false);
        }
        
        // Verify UI derives appropriate tier
        if (rankingLimit <= 10) expect(derivedTier).toBe('BRONZE');
        else if (rankingLimit <= 30) expect(derivedTier).toBe('SILVER');
        else if (rankingLimit <= 75) expect(derivedTier).toBe('GOLD');
        else if (rankingLimit <= 150) expect(derivedTier).toBe('PLATINUM');
        else expect(derivedTier).toBe('ENTERPRISE');
      }
    });

    test('should handle unlimited access correctly', async ({ page }) => {
      const unlimitedUser = createTestUserWithPermissions([
        'epsx:rankings:view:unlimited',
        'epsx:*:*'
      ], { email: 'unlimited@test.com' });
      
      await authenticateUser(page, unlimitedUser);
      
      // Verify unlimited access
      expect(getUserRankingLimit(unlimitedUser)).toBe(-1);
      expect(deriveTierFromUserPermissions(unlimitedUser)).toBe('ENTERPRISE');
      
      // Should access any ranking position
      expect(canUserViewRanking(unlimitedUser, 1)).toBe(true);
      expect(canUserViewRanking(unlimitedUser, 100)).toBe(true);
      expect(canUserViewRanking(unlimitedUser, 1000)).toBe(true);
      
      // Should have wildcard permissions
      expect(userHasPermission(unlimitedUser, 'epsx:any:feature')).toBe(true);
    });

    test('should validate permission wildcards', async ({ page }) => {
      const wildcardTests = [
        {
          permissions: ['epsx:*:*'],
          shouldHave: ['epsx:trading:basic', 'epsx:portfolio:view', 'epsx:analytics:advanced'],
          shouldNotHave: ['admin:users:manage', 'epsx-pay:transactions:view']
        },
        {
          permissions: ['epsx:trading:*'],
          shouldHave: ['epsx:trading:basic', 'epsx:trading:advanced'],
          shouldNotHave: ['epsx:portfolio:view', 'epsx:analytics:basic']
        },
        {
          permissions: ['admin:*:*'],
          shouldHave: ['admin:users:manage', 'admin:system:configure'],
          shouldNotHave: ['epsx:trading:basic']
        }
      ];
      
      for (const test of wildcardTests) {
        const testUser = createTestUserWithPermissions(test.permissions);
        
        for (const perm of test.shouldHave) {
          expect(userHasPermission(testUser, perm)).toBe(true);
        }
        
        for (const perm of test.shouldNotHave) {
          expect(userHasPermission(testUser, perm)).toBe(false);
        }
      }
    });

    test('should support multi-platform permissions', async ({ page }) => {
      const multiPlatformUser = createTestUserWithPermissions([
        'epsx:rankings:view:50',
        'epsx:trading:advanced',
        'epsx-pay:transactions:view',
        'epsx-token:tokens:stake'
      ], { email: 'multiplatform@test.com' });
      
      await authenticateUser(page, multiPlatformUser);
      
      // Should have EPSX permissions
      expect(userHasPermission(multiPlatformUser, 'epsx:trading:advanced')).toBe(true);
      expect(getUserRankingLimit(multiPlatformUser)).toBe(50);
      
      // Should have EPSX Pay permissions
      expect(userHasPermission(multiPlatformUser, 'epsx-pay:transactions:view')).toBe(true);
      
      // Should have EPSX Token permissions
      expect(userHasPermission(multiPlatformUser, 'epsx-token:tokens:stake')).toBe(true);
      
      // Should NOT have admin permissions
      expect(userHasPermission(multiPlatformUser, 'admin:users:manage')).toBe(false);
    });

    test('should handle edge cases in permission parsing', async ({ page }) => {
      // Test malformed or edge case permissions
      const edgeCaseTests = [
        {
          permissions: ['epsx:rankings:view:0'], // Zero limit
          expectedLimit: 0,
          canView1: false
        },
        {
          permissions: ['epsx:rankings:view:abc'], // Invalid number
          expectedLimit: 5, // Should fallback to default
          canView1: true
        },
        {
          permissions: ['epsx:rankings:view'], // Missing limit
          expectedLimit: 5, // Should fallback to default
          canView1: true
        },
        {
          permissions: ['invalid:permission:format'], // No ranking permission
          expectedLimit: 5, // Should fallback to default
          canView1: true
        }
      ];
      
      for (const test of edgeCaseTests) {
        const testUser = createTestUserWithPermissions(test.permissions);
        
        expect(getUserRankingLimit(testUser)).toBe(test.expectedLimit);
        expect(canUserViewRanking(testUser, 1)).toBe(test.canView1);
        
        console.log(`✅ Edge case handled: ${JSON.stringify(test.permissions)} -> limit: ${test.expectedLimit}`);
      }
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
        
        const derivedTier = deriveTierFromUserPermissions(user);
        console.log(`📊 ${derivedTier} dashboard load time: ${loadTime}ms`);
        
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