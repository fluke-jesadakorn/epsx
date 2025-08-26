/**
 * Test Helper Utilities
 * Provides common utilities, setup functions, and helper methods for E2E tests
 */

import { Page, BrowserContext, expect } from '@playwright/test';
import { 
  TEST_USERS, 
  TestUser, 
  generateMockJWT, 
  getUserByTier,
  initializeTestUsers
} from '../fixtures/user-fixtures';

// Initialize test users on import
initializeTestUsers();

export interface TestConfig {
  baseUrl: string;
  apiUrl: string;
  timeout: number;
  retries: number;
  slowMotion: number;
}

export const DEFAULT_CONFIG: TestConfig = {
  baseUrl: process.env.PLAYWRIGHT_BASE_URL || 'http://localhost:3000',
  apiUrl: process.env.PLAYWRIGHT_API_URL || 'http://localhost:8080',
  timeout: 30000,
  retries: 2,
  slowMotion: 0
};

/**
 * Authentication Helper Class
 */
export class AuthHelper {
  constructor(private page: Page) {}

  /**
   * Authenticate user with JWT token
   */
  async loginUser(user: TestUser): Promise<void> {
    const jwtToken = generateMockJWT(user);
    
    await this.page.context().addCookies([{
      name: 'epsx_jwt',
      value: jwtToken,
      domain: new URL(DEFAULT_CONFIG.baseUrl).hostname,
      path: '/',
      httpOnly: true,
      secure: false
    }]);

    console.log(`🔐 Authenticated: ${user.email} (${user.package_tier})`);
  }

  /**
   * Logout user by clearing cookies
   */
  async logoutUser(): Promise<void> {
    await this.page.context().clearCookies();
    console.log('🚪 User logged out');
  }

  /**
   * Quick login by tier
   */
  async loginByTier(tier: string): Promise<TestUser> {
    const user = getUserByTier(tier);
    await this.loginUser(user);
    return user;
  }

  /**
   * Verify user is authenticated
   */
  async verifyAuthenticated(): Promise<boolean> {
    const cookies = await this.page.context().cookies();
    return cookies.some(cookie => cookie.name === 'epsx_jwt');
  }
}

/**
 * API Mock Helper Class
 */
export class ApiMockHelper {
  constructor(private page: Page) {}

  /**
   * Mock successful authentication
   */
  async mockAuthentication(user: TestUser): Promise<void> {
    await this.page.route('**/api/auth/validate-session', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          valid: true,
          user: user,
          permissions: user.permissions,
          package_tier: user.package_tier,
          performance: {
            validation_time_ms: Math.random() * 100,
            cache_hit: Math.random() > 0.5
          }
        })
      });
    });
  }

  /**
   * Mock API endpoints based on user tier
   */
  async mockTierBasedApis(user: TestUser): Promise<void> {
    // Portfolio API
    await this.page.route('**/api/portfolio/**', async route => {
      const hasAccess = this.checkRoleAccess(user, 'profile');
      
      if (hasAccess) {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            balance: 125000.50,
            positions: [],
            role: this.mapTierToRole(user.package_tier)
          })
        });
      } else {
        await route.fulfill({
          status: 403,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Insufficient permissions' })
        });
      }
    });

    // Analytics API
    await this.page.route('**/api/analytics/**', async route => {
      const hasAccess = this.checkRoleAccess(user, 'view_eps');
      
      if (hasAccess) {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            data: 'analytics_data',
            role: this.mapTierToRole(user.package_tier)
          })
        });
      } else {
        await route.fulfill({
          status: 403,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Insufficient permissions' })
        });
      }
    });
  }

  /**
   * Mock payment processing
   */
  async mockPaymentFlow(shouldSucceed: boolean = true): Promise<void> {
    await this.page.route('**/api/payments/**', async route => {
      const url = route.request().url();
      
      if (url.includes('create-payment-intent')) {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            client_secret: 'pi_test_12345',
            payment_intent_id: 'pi_test_12345'
          })
        });
      } else if (url.includes('confirm-payment')) {
        if (shouldSucceed) {
          await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({
              success: true,
              subscription_id: 'sub_12345'
            })
          });
        } else {
          await route.fulfill({
            status: 400,
            contentType: 'application/json',
            body: JSON.stringify({
              error: 'Payment failed'
            })
          });
        }
      }
    });
  }

  /**
   * Check if user role has access to feature
   */
  private checkRoleAccess(user: TestUser, feature: string): boolean {
    // Map tier to simple role
    const role = this.mapTierToRole(user.package_tier);
    
    // Import role checking logic
    const { checkFeatureAccess, Role } = require('@/lib/auth/roles');
    
    return checkFeatureAccess(role, feature);
  }

  /**
   * Map package tier to simple role
   */
  private mapTierToRole(tier: string): string {
    const tierRoleMap = {
      FREE: 'guest',
      BRONZE: 'guest', 
      SILVER: 'user',
      GOLD: 'user',
      PLATINUM: 'user',
      ENTERPRISE: 'admin'
    };
    
    return tierRoleMap[tier] || 'guest';
  }
}

/**
 * Navigation Helper Class
 */
export class NavigationHelper {
  constructor(private page: Page) {}

  /**
   * Navigate to route with tier-based access check
   */
  async navigateToRoute(route: string, expectedAccess: boolean = true): Promise<void> {
    const response = await this.page.goto(route);
    
    if (expectedAccess) {
      expect(response?.status()).toBeLessThan(400);
      expect(this.page.url()).toContain(route);
    } else {
      // Should redirect to upgrade or login
      const currentUrl = this.page.url();
      const isRedirected = currentUrl.includes('/upgrade') || 
                          currentUrl.includes('/oauth/authorize') ||
                          currentUrl.includes('/access-denied');
      expect(isRedirected).toBe(true);
    }
  }

  /**
   * Wait for page to be fully loaded
   */
  async waitForPageLoad(): Promise<void> {
    await this.page.waitForLoadState('networkidle');
    await this.page.waitForSelector('[data-testid="page-loaded"]', { timeout: 10000 }).catch(() => {
      // Fallback if page-loaded indicator doesn't exist
      console.log('Page loaded indicator not found, continuing...');
    });
  }

  /**
   * Navigate and verify tier-specific content
   */
  async navigateAndVerifyTier(route: string, tier: string): Promise<void> {
    await this.navigateToRoute(route);
    await this.waitForPageLoad();
    
    // Verify tier badge or indicator
    const tierIndicator = this.page.locator('[data-testid="tier-badge"]');
    if (await tierIndicator.isVisible()) {
      await expect(tierIndicator).toContainText(tier);
    }
  }
}

/**
 * Performance Helper Class
 */
export class PerformanceHelper {
  constructor(private page: Page) {}

  /**
   * Measure page load performance
   */
  async measurePageLoad(url: string): Promise<{
    loadTime: number;
    domContentLoaded: number;
    firstContentfulPaint: number;
    networkRequests: number;
  }> {
    const requests: any[] = [];
    this.page.on('request', request => requests.push(request));

    const startTime = Date.now();
    await this.page.goto(url);
    const loadTime = Date.now() - startTime;

    const performanceMetrics = await this.page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      const paints = performance.getEntriesByType('paint');
      
      return {
        domContentLoaded: navigation.domContentLoadedEventEnd - navigation.navigationStart,
        firstContentfulPaint: paints.find(p => p.name === 'first-contentful-paint')?.startTime || 0
      };
    });

    return {
      loadTime,
      domContentLoaded: performanceMetrics.domContentLoaded,
      firstContentfulPaint: performanceMetrics.firstContentfulPaint,
      networkRequests: requests.length
    };
  }

  /**
   * Measure API response time
   */
  async measureApiResponse(endpoint: string): Promise<number> {
    const startTime = Date.now();
    await this.page.request.get(endpoint);
    return Date.now() - startTime;
  }

  /**
   * Check Core Web Vitals
   */
  async getCoreWebVitals(): Promise<{
    lcp: number;
    fcp: number;
    cls: number;
  }> {
    return await this.page.evaluate(() => {
      return new Promise((resolve) => {
        setTimeout(() => {
          const paints = performance.getEntriesByType('paint');
          
          resolve({
            lcp: paints.find(p => p.name === 'largest-contentful-paint')?.startTime || 0,
            fcp: paints.find(p => p.name === 'first-contentful-paint')?.startTime || 0,
            cls: 0 // Would need layout shift observer in real implementation
          });
        }, 2000);
      });
    });
  }
}

/**
 * Accessibility Helper Class
 */
export class AccessibilityHelper {
  constructor(private page: Page) {}

  /**
   * Check basic accessibility compliance
   */
  async checkAccessibility(): Promise<{
    hasHeadings: boolean;
    hasAltText: boolean;
    hasLabels: boolean;
    hasAriaAttributes: boolean;
  }> {
    return await this.page.evaluate(() => {
      const headings = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
      const images = document.querySelectorAll('img');
      const inputs = document.querySelectorAll('input, select, textarea');
      const ariaElements = document.querySelectorAll('[aria-label], [aria-labelledby], [role]');

      const imagesWithAlt = Array.from(images).filter(img => 
        img.hasAttribute('alt') && img.getAttribute('alt')?.trim() !== ''
      );

      const inputsWithLabels = Array.from(inputs).filter(input => {
        const id = input.getAttribute('id');
        const label = id ? document.querySelector(`label[for="${id}"]`) : null;
        const ariaLabel = input.getAttribute('aria-label');
        const ariaLabelledBy = input.getAttribute('aria-labelledby');
        
        return label || ariaLabel || ariaLabelledBy;
      });

      return {
        hasHeadings: headings.length > 0,
        hasAltText: images.length === 0 || imagesWithAlt.length === images.length,
        hasLabels: inputs.length === 0 || inputsWithLabels.length === inputs.length,
        hasAriaAttributes: ariaElements.length > 0
      };
    });
  }

  /**
   * Test keyboard navigation
   */
  async testKeyboardNavigation(): Promise<boolean> {
    await this.page.keyboard.press('Tab');
    const activeElement = await this.page.evaluate(() => document.activeElement?.tagName);
    return activeElement !== 'BODY';
  }
}

/**
 * Data Management Helper Class
 */
export class DataHelper {
  constructor(private page: Page) {}

  /**
   * Clear all test data
   */
  async clearTestData(): Promise<void> {
    await this.page.evaluate(() => {
      localStorage.clear();
      sessionStorage.clear();
    });

    await this.page.context().clearCookies();
  }

  /**
   * Seed test data
   */
  async seedTestData(data: Record<string, any>): Promise<void> {
    await this.page.evaluate((testData) => {
      Object.entries(testData).forEach(([key, value]) => {
        localStorage.setItem(key, JSON.stringify(value));
      });
    }, data);
  }

  /**
   * Get test data
   */
  async getTestData(key: string): Promise<any> {
    return await this.page.evaluate((dataKey) => {
      const data = localStorage.getItem(dataKey);
      return data ? JSON.parse(data) : null;
    }, key);
  }

  /**
   * Generate test transaction data
   */
  generateTransactionData(count: number = 10): any[] {
    const transactions = [];
    const symbols = ['AAPL', 'GOOGL', 'MSFT', 'AMZN', 'TSLA'];
    const actions = ['BUY', 'SELL'];

    for (let i = 0; i < count; i++) {
      transactions.push({
        id: `tx_${Date.now()}_${i}`,
        symbol: symbols[Math.floor(Math.random() * symbols.length)],
        action: actions[Math.floor(Math.random() * actions.length)],
        quantity: Math.floor(Math.random() * 100) + 1,
        price: Math.random() * 1000 + 50,
        timestamp: new Date(Date.now() - Math.random() * 30 * 24 * 60 * 60 * 1000).toISOString()
      });
    }

    return transactions;
  }
}

/**
 * Screenshot Helper Class
 */
export class ScreenshotHelper {
  constructor(private page: Page) {}

  /**
   * Take comparison screenshot
   */
  async takeComparisonScreenshot(name: string, options?: {
    fullPage?: boolean;
    clip?: { x: number; y: number; width: number; height: number };
  }): Promise<void> {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const filename = `${name}-${timestamp}.png`;

    await this.page.screenshot({
      path: `test-results/screenshots/${filename}`,
      fullPage: options?.fullPage || false,
      clip: options?.clip
    });

    console.log(`📸 Screenshot saved: ${filename}`);
  }

  /**
   * Take responsive screenshots
   */
  async takeResponsiveScreenshots(name: string): Promise<void> {
    const viewports = [
      { name: 'mobile', width: 375, height: 667 },
      { name: 'tablet', width: 768, height: 1024 },
      { name: 'desktop', width: 1200, height: 800 }
    ];

    for (const viewport of viewports) {
      await this.page.setViewportSize(viewport);
      await this.page.waitForTimeout(500); // Allow layout to adjust
      
      await this.takeComparisonScreenshot(`${name}-${viewport.name}`, {
        fullPage: true
      });
    }
  }
}

/**
 * Test Suite Helper Class
 */
export class TestSuiteHelper {
  public auth: AuthHelper;
  public api: ApiMockHelper;
  public nav: NavigationHelper;
  public perf: PerformanceHelper;
  public a11y: AccessibilityHelper;
  public data: DataHelper;
  public screenshot: ScreenshotHelper;

  constructor(private page: Page) {
    this.auth = new AuthHelper(page);
    this.api = new ApiMockHelper(page);
    this.nav = new NavigationHelper(page);
    this.perf = new PerformanceHelper(page);
    this.a11y = new AccessibilityHelper(page);
    this.data = new DataHelper(page);
    this.screenshot = new ScreenshotHelper(page);
  }

  /**
   * Complete test setup for a user tier
   */
  async setupTierTest(tier: string): Promise<TestUser> {
    const user = await this.auth.loginByTier(tier);
    await this.api.mockAuthentication(user);
    await this.api.mockTierBasedApis(user);
    return user;
  }

  /**
   * Complete test teardown
   */
  async teardown(): Promise<void> {
    await this.data.clearTestData();
    await this.auth.logoutUser();
  }

  /**
   * Run comprehensive page test
   */
  async runComprehensivePageTest(route: string, tier: string): Promise<{
    user: TestUser;
    performance: any;
    accessibility: any;
  }> {
    const user = await this.setupTierTest(tier);
    
    // Navigate and test performance
    const performance = await this.perf.measurePageLoad(route);
    
    // Test accessibility
    const accessibility = await this.a11y.checkAccessibility();
    
    // Take screenshot
    await this.screenshot.takeComparisonScreenshot(`${tier}-${route.replace(/\//g, '-')}`);
    
    return { user, performance, accessibility };
  }
}

/**
 * Utility Functions
 */
export const utils = {
  /**
   * Wait for condition with timeout
   */
  async waitForCondition(
    condition: () => Promise<boolean>,
    timeout: number = 5000,
    interval: number = 100
  ): Promise<boolean> {
    const startTime = Date.now();
    
    while (Date.now() - startTime < timeout) {
      if (await condition()) {
        return true;
      }
      await new Promise(resolve => setTimeout(resolve, interval));
    }
    
    return false;
  },

  /**
   * Generate random test data
   */
  generateRandomData: {
    email: () => `test-${Date.now()}@example.com`,
    username: () => `testuser_${Date.now()}`,
    price: () => Math.round((Math.random() * 1000 + 50) * 100) / 100,
    percentage: () => Math.round((Math.random() * 10 - 5) * 100) / 100,
    uuid: () => crypto.randomUUID?.() || `uuid-${Date.now()}-${Math.random()}`
  },

  /**
   * Format currency for testing
   */
  formatCurrency: (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(amount);
  },

  /**
   * Calculate performance score
   */
  calculatePerformanceScore: (metrics: any) => {
    const { loadTime, firstContentfulPaint } = metrics;
    let score = 100;
    
    if (loadTime > 3000) score -= 30;
    else if (loadTime > 2000) score -= 20;
    else if (loadTime > 1000) score -= 10;
    
    if (firstContentfulPaint > 2000) score -= 20;
    else if (firstContentfulPaint > 1500) score -= 10;
    
    return Math.max(0, score);
  }
};

/**
 * Export test users for convenience
 */
export { TEST_USERS } from '../fixtures/user-fixtures';

/**
 * Create test suite helper
 */
export function createTestSuite(page: Page): TestSuiteHelper {
  return new TestSuiteHelper(page);
}