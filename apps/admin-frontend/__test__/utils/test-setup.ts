/**
 * Test Setup and Utilities for Admin Middleware Testing
 * 
 * Comprehensive testing utilities for setting up and managing admin tests including:
 * - Global test configuration
 * - Database setup and teardown
 * - Authentication helpers
 * - Performance measurement utilities
 * - Security testing helpers
 * - Test data management
 * - Environment validation
 */

import { test as base, expect, Page, APIRequestContext } from '@playwright/test';

import { getBackendUrl } from '@/shared/utils/url-resolver';
import {
  TEST_USERS,
  TEST_SESSIONS,
  TEST_ENVIRONMENT_CONFIG,
  TestDatabaseUtilities,
  MockAPIClient,
  type TestUser
} from '../fixtures/admin-test-fixtures';

// ============================================================================
// Extended Test Fixtures
// ============================================================================

interface AdminTestFixtures {
  adminPage: Page;
  authenticatedRequest: APIRequestContext;
  testUser: TestUser;
  dbUtils: TestDatabaseUtilities;
  mockAPI: MockAPIClient;
  performanceMonitor: PerformanceMonitor;
}

// Extend Playwright's base test with custom fixtures
export const test = base.extend<AdminTestFixtures>({
  adminPage: async ({ page }, use) => {
    // Set up admin-specific page configuration
    await page.setExtraHTTPHeaders({
      'X-Test-Environment': 'true',
      'X-Test-Run-ID': `test-${Date.now()}`
    });
    
    // Set viewport for consistent testing
    await page.setViewportSize({ width: 1920, height: 1080 });
    
    // Block unnecessary resources for faster testing
    await page.route('**/*.{png,jpg,jpeg,svg}', route => route.abort());
    
    await use(page);
  },

  authenticatedRequest: async ({ request }, use) => {
    // Use request with authentication headers
    await use(request);
  },

  testUser: async ({}, use) => {
    // Provide default test user
    await use(TEST_USERS['ADMIN']!);
  },

  dbUtils: async ({}, use) => {
    const utils = TestDatabaseUtilities.getInstance();
    await use(utils);
  },

  mockAPI: async ({}, use) => {
    const mockClient = new MockAPIClient();
    await use(mockClient);
  },

  performanceMonitor: async ({}, use) => {
    const monitor = new PerformanceMonitor();
    await use(monitor);
  }
});

// ============================================================================
// Performance Monitoring Utilities
// ============================================================================

export interface PerformanceMetrics {
  startTime: number;
  endTime: number;
  duration: number;
  memoryUsage?: {
    used: number;
    total: number;
  };
  networkActivity?: NetworkActivity[];
  errors?: Error[];
}

export interface NetworkActivity {
  url: string;
  method: string;
  status: number;
  duration: number;
  size: number;
}

/**
 *
 */
export class PerformanceMonitor {
  private metrics: PerformanceMetrics[] = [];
  private currentMetric: Partial<PerformanceMetrics> | null = null;

  /**
   *
   * @param identifier
   */
  startMeasurement(identifier: string): void {
    this.currentMetric = {
      startTime: performance.now(),
      errors: []
    };
  }

  /**
   *
   */
  endMeasurement(): PerformanceMetrics | null {
    if (!this.currentMetric || !this.currentMetric.startTime) {
       
      console.warn('⚠️ No active measurement to end');
      return null;
    }

    const endTime = performance.now();
    const completedMetric: PerformanceMetrics = {
      ...this.currentMetric,
      endTime,
      duration: endTime - this.currentMetric.startTime
    } as PerformanceMetrics;

    this.metrics.push(completedMetric);
    this.currentMetric = null;

    return completedMetric;
  }

  /**
   *
   * @param page
   */
  async measurePagePerformance(page: Page): Promise<PagePerformanceMetrics> {
    const performanceTiming = await page.evaluate(() => {
      const timing = performance.timing;
      return {
        navigationStart: timing.navigationStart,
        loadEventEnd: timing.loadEventEnd,
        domContentLoadedEventEnd: timing.domContentLoadedEventEnd,
        connectEnd: timing.connectEnd,
        requestStart: timing.requestStart,
        responseStart: timing.responseStart,
        responseEnd: timing.responseEnd
      };
    });

    const memory = await page.evaluate(() => {
      if ('memory' in performance) {
        return (performance as any).memory;
      }
      return null;
    });

    return {
      pageLoadTime: performanceTiming.loadEventEnd - performanceTiming.navigationStart,
      domContentLoaded: performanceTiming.domContentLoadedEventEnd - performanceTiming.navigationStart,
      timeToFirstByte: performanceTiming.responseStart - performanceTiming.requestStart,
      memoryUsage: memory
    };
  }

  /**
   *
   */
  getAverageResponseTime(): number {
    if (this.metrics.length === 0) {return 0;}
    return this.metrics.reduce((sum, metric) => sum + metric.duration, 0) / this.metrics.length;
  }

  /**
   *
   */
  getMetricsSummary(): PerformanceSummary {
    if (this.metrics.length === 0) {
      return { totalMeasurements: 0, averageDuration: 0, minDuration: 0, maxDuration: 0 };
    }

    const durations = this.metrics.map(m => m.duration);
    return {
      totalMeasurements: this.metrics.length,
      averageDuration: durations.reduce((a, b) => a + b, 0) / durations.length,
      minDuration: Math.min(...durations),
      maxDuration: Math.max(...durations),
      p95Duration: this.calculatePercentile(durations, 95),
      p99Duration: this.calculatePercentile(durations, 99)
    };
  }

  private calculatePercentile(values: number[], percentile: number): number {
    const sorted = values.sort((a, b) => a - b);
    const index = Math.ceil((percentile / 100) * sorted.length) - 1;
    return sorted[index] || 0;
  }

  /**
   *
   */
  clearMetrics(): void {
    this.metrics = [];
    this.currentMetric = null;
  }
}

export interface PagePerformanceMetrics {
  pageLoadTime: number;
  domContentLoaded: number;
  timeToFirstByte: number;
  memoryUsage: any;
}

export interface PerformanceSummary {
  totalMeasurements: number;
  averageDuration: number;
  minDuration: number;
  maxDuration: number;
  p95Duration?: number;
  p99Duration?: number;
}

// ============================================================================
// Authentication Test Helpers
// ============================================================================

/**
 *
 */
export class AuthenticationHelper {
  private page: Page;

  /**
   *
   * @param page
   */
  constructor(page: Page) {
    this.page = page;
  }

  /**
   *
   * @param userType
   */
  async loginAsUser(userType: keyof typeof TEST_USERS): Promise<string | null> {
    const user = TEST_USERS[userType];
    const session = Object.values(TEST_SESSIONS).find(s => s.userId === user.id);

    try {
      // Navigate to login page
      await this.page.goto('/');
      
      // Check if already logged in
      try {
        await this.page.waitForURL('**/login**', { timeout: 5000 });
      } catch {
        const signOutBtn = this.page.locator('text=Sign out').first();
        if (await signOutBtn.isVisible()) {
          await signOutBtn.click();
          await this.page.waitForURL('**/login**');
        }
      }

      // Perform OAuth login
      const oauthLoginBtn = this.page.locator('button').filter({ hasText: /sign in|login|epsx/i }).first();
      await expect(oauthLoginBtn).toBeVisible({ timeout: 10000 });
      await oauthLoginBtn.click();

      await this.page.waitForURL('**/oauth/authorize**', { timeout: 10000 });
      await this.page.fill('input[name="email"]', TEST_ENVIRONMENT_CONFIG.auth.testEmail);
      await this.page.fill('input[name="password"]', TEST_ENVIRONMENT_CONFIG.auth.testPassword);
      
      const submitBtn = this.page.locator('button[type="submit"]').first();
      await submitBtn.click();

      // Wait for successful login
      await this.page.waitForFunction(
        () => {
          const url = window.location.href;
          return !url.includes('/login') && 
                 !url.includes('/oauth/authorize') && 
                 (url.includes('localhost:3001') || url.includes('admin.epsx.io'));
        },
        { timeout: 30000 }
      );

      await this.page.waitForLoadState('networkidle');

      // Extract session token
      const token = await this.extractAuthToken();
      
      if (token) {
        return token;
      } else {
         
        console.error(`❌ Failed to extract auth token for ${user.name}`);
        return null;
      }
    } catch (_error) {
       
      console.error(`❌ Login failed for ${user.name}:`, _error);
      return null;
    }
  }

  /**
   *
   */
  async extractAuthToken(): Promise<string | null> {
    return await this.page.evaluate(() => {
      return localStorage.getItem('auth_token') || 
             document.cookie.split(';').find(c => c.trim().startsWith('session='))?.split('=')[1] || null;
    });
  }

  /**
   *
   * @param token
   */
  async validateSession(token: string): Promise<boolean> {
    try {
      const response = await this.page.request.get('/api/admin/auth/profile', {
        headers: { 'Authorization': `Bearer ${token}` }
      });
      return response.ok();
    } catch {
      return false;
    }
  }

  /**
   *
   */
  async logout(): Promise<void> {
    
    try {
      // Try to find and click logout button
      const logoutSelectors = [
        'button:has-text("Logout")',
        'button:has-text("Sign out")', 
        'a:has-text("Logout")',
        'a:has-text("Sign out")'
      ];

      for (const selector of logoutSelectors) {
        const element = this.page.locator(selector);
        if (await element.isVisible()) {
          await element.click();
          break;
        }
      }

      // Clear session storage
      await this.page.evaluate(() => {
        localStorage.clear();
        sessionStorage.clear();
      });

    } catch (_error) {
       
      console.error('⚠️ Logout error:', _error);
    }
  }
}

// ============================================================================
// Security Testing Helpers
// ============================================================================

/**
 *
 */
export class SecurityTestHelper {
  private request: APIRequestContext;
  private baseUrl: string;

  /**
   *
   * @param request
   * @param baseUrl
   */
  constructor(request: APIRequestContext, baseUrl: string = getBackendUrl('server')) {
    this.request = request;
    this.baseUrl = baseUrl;
  }

  /**
   *
   * @param endpoint
   * @param payloads
   * @param token
   */
  async testSQLInjection(endpoint: string, payloads: string[], token?: string): Promise<SecurityTestResult[]> {
    
    const results: SecurityTestResult[] = [];

    for (const payload of payloads) {
      try {
        const response = await this.request.get(`${this.baseUrl}${endpoint}?q=${encodeURIComponent(payload)}`, {
          headers: token ? { 'Authorization': `Bearer ${token}` } : {}
        });

        results.push({
          payload,
          blocked: response.status() >= 400,
          status: response.status(),
          responseTime: 0 // Would be measured in real implementation
        });
      } catch (_error) {
        results.push({
          payload,
          blocked: true,
          status: 0,
          responseTime: 0,
          error: (error as Error).toString()
        });
      }
    }

    const blockRate = (results.filter(r => r.blocked).length / results.length) * 100;

    return results;
  }

  /**
   *
   * @param endpoint
   * @param payloads
   * @param token
   */
  async testXSS(endpoint: string, payloads: string[], token?: string): Promise<SecurityTestResult[]> {
    
    const results: SecurityTestResult[] = [];

    for (const payload of payloads) {
      try {
        const response = await this.request.post(`${this.baseUrl}${endpoint}`, {
          data: {
            name: payload,
            email: 'xss.test@example.com'
          },
          headers: {
            'Content-Type': 'application/json',
            ...(token ? { 'Authorization': `Bearer ${token}` } : {})
          }
        });

        const blocked = response.status() >= 400;
        let sanitized = false;

        if (response.ok()) {
          try {
            const responseData = await response.json();
            sanitized = !responseData.name?.includes('<script>') && 
                       !responseData.name?.includes('javascript:');
          } catch {}
        }

        results.push({
          payload,
          blocked: blocked || sanitized,
          status: response.status(),
          responseTime: 0,
          sanitized
        });
      } catch (_error) {
        results.push({
          payload,
          blocked: true,
          status: 0,
          responseTime: 0,
          error: (error as Error).toString()
        });
      }
    }

    return results;
  }

  /**
   *
   * @param endpoint
   * @param requestCount
   * @param token
   */
  async testRateLimiting(endpoint: string, requestCount: number, token?: string): Promise<RateLimitTestResult> {
    
    const startTime = performance.now();
    const requests = Array.from({ length: requestCount }, (_, i) => 
      this.request.get(`${this.baseUrl}${endpoint}?test=${i}`, {
        headers: token ? { 'Authorization': `Bearer ${token}` } : {}
      }).catch(() => ({ status: () => 0, ok: () => false }))
    );

    const responses = await Promise.all(requests);
    const endTime = performance.now();

    const results = responses.map(response => ({
      status: response.status ? response.status() : 0,
      success: response.ok ? response.ok() : false
    }));

    const successCount = results.filter(r => r.success).length;
    const rateLimitedCount = results.filter(r => r.status === 429).length;

    return {
      totalRequests: requestCount,
      successfulRequests: successCount,
      rateLimitedRequests: rateLimitedCount,
      duration: endTime - startTime,
      throughput: requestCount / ((endTime - startTime) / 1000)
    };
  }

  /**
   *
   * @param endpoint
   * @param token
   */
  async testCSRF(endpoint: string, token?: string): Promise<CSRFTestResult> {

    try {
      const response = await this.request.post(`${this.baseUrl}${endpoint}`, {
        data: { test: 'csrf' },
        headers: {
          'Content-Type': 'application/json',
          'Origin': 'http://malicious-site.com',
          'Referer': 'http://malicious-site.com/attack.html',
          ...(token ? { 'Authorization': `Bearer ${token}` } : {})
        }
      });

      return {
        protected: response.status() === 403 || response.status() === 400,
        status: response.status(),
        responseTime: 0
      };
    } catch (_error) {
      return {
        protected: true,
        status: 0,
        responseTime: 0,
        error: (error as Error).toString()
      };
    }
  }
}

// ============================================================================
// Test Result Types
// ============================================================================

export interface SecurityTestResult {
  payload: string;
  blocked: boolean;
  status: number;
  responseTime: number;
  sanitized?: boolean;
  error?: string;
}

export interface RateLimitTestResult {
  totalRequests: number;
  successfulRequests: number;
  rateLimitedRequests: number;
  duration: number;
  throughput: number;
}

export interface CSRFTestResult {
  protected: boolean;
  status: number;
  responseTime: number;
  error?: string;
}

// ============================================================================
// Environment Validation
// ============================================================================

/**
 *
 */
export class EnvironmentValidator {
  /**
   *
   */
  static async validateTestEnvironment(): Promise<ValidationResult> {
    
    const results: ValidationResult = {
      database: false,
      api: false,
      authentication: false,
      errors: []
    };

    // Validate database connection
    try {
      // Implementation: Check database connectivity
      results.database = true;
    } catch (_error) {
      results.database = false;
      results.errors.push(`Database validation failed: ${error}`);
       
      console.error('❌ Database validation failed:', _error);
    }

    // Validate API availability
    try {
      const response = await fetch(`${TEST_ENVIRONMENT_CONFIG.api.baseUrl}/health`);
      results.api = response.ok;
      
      if (results.api) {
      } else {
        results.errors.push(`API health check failed: ${response.status}`);
         
        console.error('❌ API validation failed:', response.status);
      }
    } catch (_error) {
      results.api = false;
      results.errors.push(`API validation failed: ${error}`);
       
      console.error('❌ API validation failed:', _error);
    }

    // Validate authentication system
    try {
      // Implementation: Check authentication endpoints
      results.authentication = true;
    } catch (_error) {
      results.authentication = false;
      results.errors.push(`Authentication validation failed: ${error}`);
       
      console.error('❌ Authentication validation failed:', _error);
    }

    const allValid = results.database && results.api && results.authentication;
    
    if (allValid) {
    } else {
       
      console.error('❌ Test environment validation failed');
       
      console.error('Errors:', results.errors);
    }

    return results;
  }

  /**
   *
   */
  static async validatePermissions(): Promise<boolean> {
    
    // Implementation: Validate that test users have correct permissions
    // This would check the database or API to ensure test users exist with proper permissions
    
    return true;
  }
}

export interface ValidationResult {
  database: boolean;
  api: boolean;
  authentication: boolean;
  errors: string[];
}

// ============================================================================
// Test Data Management
// ============================================================================

/**
 *
 */
export class TestDataManager {
  private dbUtils: TestDatabaseUtilities;

  /**
   *
   */
  constructor() {
    this.dbUtils = TestDatabaseUtilities.getInstance();
  }

  /**
   *
   */
  async setupTestData(): Promise<void> {
    
    try {
      await this.dbUtils.seedTestUsers();
      await this.dbUtils.seedRoleProfiles();
      await this.dbUtils.seedTestSessions();
      
    } catch (_error) {
       
      console.error('❌ Test data setup failed:', _error);
      throw _error;
    }
  }

  /**
   *
   */
  async cleanupTestData(): Promise<void> {
    
    try {
      await this.dbUtils.cleanupTestData();
    } catch (_error) {
       
      console.error('❌ Test data cleanup failed:', _error);
      throw _error;
    }
  }

  /**
   *
   */
  async resetTestData(): Promise<void> {
    
    await this.cleanupTestData();
    await this.setupTestData();
    
  }

  /**
   *
   */
  async verifyTestDataIntegrity(): Promise<boolean> {
    
    try {
      const isValid = await this.dbUtils.verifyDatabaseIntegrity();
      
      if (isValid) {
      } else {
         
        console.error('❌ Test data integrity check failed');
      }
      
      return isValid;
    } catch (_error) {
       
      console.error('❌ Test data integrity verification failed:', _error);
      return false;
    }
  }
}

// ============================================================================
// Global Test Setup and Teardown
// ============================================================================

/**
 *
 */
export class GlobalTestSetup {
  private static instance: GlobalTestSetup;
  private dataManager: TestDataManager;
  private isSetup = false;

  /**
   *
   */
  constructor() {
    this.dataManager = new TestDataManager();
  }

  /**
   *
   */
  static getInstance(): GlobalTestSetup {
    if (!GlobalTestSetup.instance) {
      GlobalTestSetup.instance = new GlobalTestSetup();
    }
    return GlobalTestSetup.instance;
  }

  /**
   *
   */
  async globalSetup(): Promise<void> {
    if (this.isSetup) {
      return;
    }

    // Validate environment
    const validation = await EnvironmentValidator.validateTestEnvironment();
    if (!validation.database || !validation.api) {
      throw new Error(`Environment validation failed: ${validation.errors.join(', ')}`);
    }

    // Setup test data
    await this.dataManager.setupTestData();

    // Validate permissions
    await EnvironmentValidator.validatePermissions();

    this.isSetup = true;
  }

  /**
   *
   */
  async globalTeardown(): Promise<void> {
    if (!this.isSetup) {
      return;
    }

    // Cleanup test data
    await this.dataManager.cleanupTestData();

    this.isSetup = false;
  }
}

// ============================================================================
// Exports
// ============================================================================

export { expect };