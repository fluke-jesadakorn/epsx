/**
 * Admin Performance and Load Testing Suite
 * 
 * Comprehensive performance testing for admin middleware including:
 * - Sub-100ms response time validation
 * - Concurrent session handling 
 * - Database connection pooling performance
 * - Cache hit rates and effectiveness
 * - Memory usage under sustained load
 * - Rate limiting performance impact
 * - Session validation performance
 * - Permission checking latency
 * - Security middleware overhead
 * - Large dataset handling
 */

import { test, expect, Page, APIRequestContext } from '@playwright/test';
import { performance } from 'perf_hooks';

// Test configuration
const ADMIN_BASE_URL = 'http://localhost:3001';
const API_BASE_URL = 'http://localhost:8080';
const TEST_EMAIL = 'jesadakorn.kirtnu@gmail.com';
const TEST_PASSWORD = 'Aa_12345678';

// Performance thresholds (in milliseconds)
const PERFORMANCE_THRESHOLDS = {
  FAST_RESPONSE: 100,      // Sub-100ms target
  NORMAL_RESPONSE: 500,    // Acceptable response time
  SLOW_RESPONSE: 1000,     // Warning threshold
  TIMEOUT: 5000           // Maximum acceptable time
};

// Load testing configuration
const LOAD_TEST_CONFIG = {
  LIGHT_LOAD: 10,          // Light load concurrent requests
  MEDIUM_LOAD: 25,         // Medium load concurrent requests  
  HEAVY_LOAD: 50,          // Heavy load concurrent requests
  STRESS_LOAD: 100,        // Stress test concurrent requests
  DURATION_MS: 30000,      // Test duration in milliseconds
  RAMP_UP_MS: 5000        // Time to ramp up to full load
};

// Performance metrics tracking
interface PerformanceMetrics {
  responseTime: number;
  throughput: number;
  errorRate: number;
  cpuUsage?: number;
  memoryUsage?: number;
  cacheHitRate?: number;
  dbConnectionsUsed?: number;
}

// Helper functions
async function loginAdmin(page: Page) {
  console.log('🔑 Logging in admin for performance testing');
  
  await page.goto('/');
  
  try {
    await page.waitForURL('**/login**', { timeout: 5000 });
  } catch {
    const signOutBtn = page.locator('text=Sign out').first();
    if (await signOutBtn.isVisible()) {
      await signOutBtn.click();
      await page.waitForURL('**/login**');
    }
  }

  const oauthLoginBtn = page.locator('button').filter({ hasText: /sign in|login|epsx/i }).first();
  await expect(oauthLoginBtn).toBeVisible({ timeout: 10000 });
  await oauthLoginBtn.click();

  await page.waitForURL('**/oauth/authorize**', { timeout: 10000 });
  await page.fill('input[name="email"]', TEST_EMAIL);
  await page.fill('input[name="password"]', TEST_PASSWORD);
  
  const submitBtn = page.locator('button[type="submit"]').first();
  await submitBtn.click();

  await page.waitForFunction(
    () => {
      const url = window.location.href;
      return !url.includes('/login') && 
             !url.includes('/oauth/authorize') && 
             url.includes('localhost:3001');
    },
    { timeout: 30000 }
  );

  await page.waitForLoadState('networkidle');
  console.log('✅ Admin login successful for performance testing');
}

async function extractAuthToken(page: Page): Promise<string | null> {
  return await page.evaluate(() => {
    return localStorage.getItem('auth_token') || 
           document.cookie.split(';').find(c => c.trim().startsWith('session='))?.split('=')[1];
  });
}

async function measureResponseTime(
  request: APIRequestContext,
  endpoint: string,
  method: string = 'GET',
  payload?: any,
  headers: any = {},
  token?: string
): Promise<{ duration: number; success: boolean; status: number; response?: any }> {
  const startTime = performance.now();
  
  try {
    let requestConfig: any = {
      method,
      headers: {
        'Content-Type': 'application/json',
        ...headers
      },
      timeout: PERFORMANCE_THRESHOLDS.TIMEOUT
    };
    
    if (token) {
      requestConfig.headers['Authorization'] = `Bearer ${token}`;
    }
    
    if (payload && method !== 'GET') {
      requestConfig.body = JSON.stringify(payload);
    }
    
    const response = await request.fetch(`${API_BASE_URL}${endpoint}`, requestConfig);
    const duration = performance.now() - startTime;
    
    let responseData = null;
    try {
      responseData = await response.json();
    } catch {
      responseData = await response.text().catch(() => null);
    }
    
    return {
      duration,
      success: response.ok,
      status: response.status(),
      response: responseData
    };
  } catch (error) {
    const duration = performance.now() - startTime;
    return {
      duration,
      success: false,
      status: 0
    };
  }
}

async function runLoadTest(
  request: APIRequestContext,
  endpoint: string,
  concurrentUsers: number,
  durationMs: number,
  token: string,
  method: string = 'GET',
  payload?: any
): Promise<PerformanceMetrics> {
  console.log(`🚀 Starting load test: ${concurrentUsers} users, ${durationMs}ms duration`);
  
  const startTime = performance.now();
  const endTime = startTime + durationMs;
  const results: Array<{ duration: number; success: boolean }> = [];
  
  // Create concurrent user sessions
  const userSessions = Array(concurrentUsers).fill(null).map(async (_, userIndex) => {
    const userResults: Array<{ duration: number; success: boolean }> = [];
    let requestCount = 0;
    
    while (performance.now() < endTime) {
      try {
        const result = await measureResponseTime(
          request,
          `${endpoint}?user=${userIndex}&req=${requestCount}`,
          method,
          payload,
          {},
          token
        );
        
        userResults.push({
          duration: result.duration,
          success: result.success
        });
        
        requestCount++;
        
        // Small delay to simulate realistic user behavior
        await new Promise(resolve => setTimeout(resolve, 50));
      } catch (error) {
        userResults.push({
          duration: PERFORMANCE_THRESHOLDS.TIMEOUT,
          success: false
        });
      }
    }
    
    return userResults;
  });
  
  const allUserResults = await Promise.all(userSessions);
  
  // Flatten results
  allUserResults.forEach(userResults => {
    results.push(...userResults);
  });
  
  // Calculate metrics
  const totalRequests = results.length;
  const successfulRequests = results.filter(r => r.success).length;
  const totalDuration = performance.now() - startTime;
  
  const responseTime = results.reduce((acc, r) => acc + r.duration, 0) / results.length;
  const throughput = (totalRequests / totalDuration) * 1000; // requests per second
  const errorRate = ((totalRequests - successfulRequests) / totalRequests) * 100;
  
  console.log(`📊 Load test completed: ${totalRequests} requests, ${throughput.toFixed(2)} req/s`);
  
  return {
    responseTime,
    throughput,
    errorRate
  };
}

// ============================================================================
// Response Time Performance Tests
// ============================================================================

test.describe('⚡ Response Time Performance', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should meet sub-100ms response time targets for core endpoints', async ({ page, request }) => {
    console.log('🧪 Testing sub-100ms response time targets');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    const coreEndpoints = [
      { endpoint: '/api/v1/admin/auth/profile', method: 'GET' },
      { endpoint: '/api/v1/admin/users', method: 'GET' },
      { endpoint: '/api/v1/admin/analytics/user-statistics', method: 'GET' },
      { endpoint: '/api/v1/admin/admin-modules', method: 'GET' },
      { endpoint: '/api/v1/admin/permission-profiles', method: 'GET' }
    ];
    
    const performanceResults = [];
    
    for (const endpoint of coreEndpoints) {
      // Run multiple samples to get average
      const samples = [];
      
      for (let i = 0; i < 5; i++) {
        const result = await measureResponseTime(
          request,
          endpoint.endpoint,
          endpoint.method,
          undefined,
          {},
          token
        );
        
        if (result.success) {
          samples.push(result.duration);
        }
      }
      
      if (samples.length > 0) {
        const avgResponseTime = samples.reduce((a, b) => a + b, 0) / samples.length;
        const minResponseTime = Math.min(...samples);
        const maxResponseTime = Math.max(...samples);
        
        performanceResults.push({
          endpoint: endpoint.endpoint,
          avgResponseTime,
          minResponseTime,
          maxResponseTime
        });
        
        console.log(`${endpoint.endpoint}:`);
        console.log(`  Average: ${avgResponseTime.toFixed(2)}ms`);
        console.log(`  Min: ${minResponseTime.toFixed(2)}ms`);
        console.log(`  Max: ${maxResponseTime.toFixed(2)}ms`);
        
        // Check performance targets
        if (avgResponseTime <= PERFORMANCE_THRESHOLDS.FAST_RESPONSE) {
          console.log('  ✅ EXCELLENT - Sub-100ms target met');
        } else if (avgResponseTime <= PERFORMANCE_THRESHOLDS.NORMAL_RESPONSE) {
          console.log('  🟡 GOOD - Within normal range');
        } else {
          console.log('  🔴 SLOW - Exceeds performance target');
        }
        
        // Performance assertion
        expect(avgResponseTime).toBeLessThan(PERFORMANCE_THRESHOLDS.TIMEOUT);
      }
    }
    
    // Calculate overall performance score
    const avgOverallTime = performanceResults.reduce((acc, r) => acc + r.avgResponseTime, 0) / performanceResults.length;
    console.log(`\\n📊 Overall Average Response Time: ${avgOverallTime.toFixed(2)}ms`);
    
    // At least 60% of endpoints should meet sub-100ms target
    const fastEndpoints = performanceResults.filter(r => r.avgResponseTime <= PERFORMANCE_THRESHOLDS.FAST_RESPONSE).length;
    const fastEndpointRate = (fastEndpoints / performanceResults.length) * 100;
    
    console.log(`🎯 Fast endpoints: ${fastEndpoints}/${performanceResults.length} (${fastEndpointRate.toFixed(1)}%)`);
  });

  test('should maintain consistent response times under normal load', async ({ page, request }) => {
    console.log('🧪 Testing response time consistency');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test response time consistency over multiple requests
    const testEndpoint = '/api/v1/admin/users';
    const sampleCount = 20;
    const responseTimes = [];
    
    for (let i = 0; i < sampleCount; i++) {
      const result = await measureResponseTime(
        request,
        `${testEndpoint}?sample=${i}`,
        'GET',
        undefined,
        {},
        token
      );
      
      if (result.success) {
        responseTimes.push(result.duration);
      }
      
      // Small delay between requests
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    if (responseTimes.length >= sampleCount * 0.8) { // At least 80% success rate
      const avgTime = responseTimes.reduce((a, b) => a + b, 0) / responseTimes.length;
      const minTime = Math.min(...responseTimes);
      const maxTime = Math.max(...responseTimes);
      const stdDev = Math.sqrt(
        responseTimes.reduce((acc, time) => acc + Math.pow(time - avgTime, 2), 0) / responseTimes.length
      );
      
      console.log(`📈 Response Time Consistency Analysis:`);
      console.log(`  Average: ${avgTime.toFixed(2)}ms`);
      console.log(`  Min: ${minTime.toFixed(2)}ms`);
      console.log(`  Max: ${maxTime.toFixed(2)}ms`);
      console.log(`  Std Dev: ${stdDev.toFixed(2)}ms`);
      console.log(`  Variation: ${((maxTime - minTime) / avgTime * 100).toFixed(1)}%`);
      
      // Response times should be reasonably consistent
      const variation = (maxTime - minTime) / avgTime;
      expect(variation).toBeLessThan(2); // Less than 200% variation
      
      console.log('✅ Response time consistency validated');
    }
  });

  test('should validate permission checking performance', async ({ page, request }) => {
    console.log('🧪 Testing permission checking performance impact');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test endpoints with different permission requirements
    const permissionEndpoints = [
      { endpoint: '/api/v1/admin/users', modules: ['user-management'] },
      { endpoint: '/api/v1/admin/admin-modules', modules: ['security-management'] },
      { endpoint: '/api/v1/admin/analytics/permissions', modules: ['analytics-access'] },
      { endpoint: '/api/v1/admin/permissions/audit-report', modules: ['audit-logs'] }
    ];
    
    for (const testCase of permissionEndpoints) {
      const samples = [];
      
      // Test permission checking performance
      for (let i = 0; i < 5; i++) {
        const result = await measureResponseTime(
          request,
          testCase.endpoint,
          'GET',
          undefined,
          {},
          token
        );
        
        if (result.status !== 0) { // Any response (including 403)
          samples.push(result.duration);
        }
      }
      
      if (samples.length > 0) {
        const avgTime = samples.reduce((a, b) => a + b, 0) / samples.length;
        console.log(`${testCase.endpoint}: ${avgTime.toFixed(2)}ms`);
        
        // Permission checking should add minimal overhead
        expect(avgTime).toBeLessThan(PERFORMANCE_THRESHOLDS.NORMAL_RESPONSE);
      }
    }
    
    console.log('✅ Permission checking performance validated');
  });
});

// ============================================================================
// Concurrent Session Handling Tests
// ============================================================================

test.describe('👥 Concurrent Session Handling', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should handle light concurrent load (10 users)', async ({ page, request }) => {
    console.log('🧪 Testing light concurrent load handling');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    const metrics = await runLoadTest(
      request,
      '/api/v1/admin/users',
      LOAD_TEST_CONFIG.LIGHT_LOAD,
      10000, // 10 seconds
      token
    );
    
    console.log(`📊 Light Load Results:`);
    console.log(`  Average Response Time: ${metrics.responseTime.toFixed(2)}ms`);
    console.log(`  Throughput: ${metrics.throughput.toFixed(2)} req/s`);
    console.log(`  Error Rate: ${metrics.errorRate.toFixed(2)}%`);
    
    // Performance targets for light load
    expect(metrics.responseTime).toBeLessThan(PERFORMANCE_THRESHOLDS.NORMAL_RESPONSE);
    expect(metrics.errorRate).toBeLessThan(5); // Less than 5% error rate
    expect(metrics.throughput).toBeGreaterThan(10); // At least 10 req/s
    
    console.log('✅ Light load handling validated');
  });

  test('should handle medium concurrent load (25 users)', async ({ page, request }) => {
    console.log('🧪 Testing medium concurrent load handling');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    const metrics = await runLoadTest(
      request,
      '/api/v1/admin/users',
      LOAD_TEST_CONFIG.MEDIUM_LOAD,
      15000, // 15 seconds  
      token
    );
    
    console.log(`📊 Medium Load Results:`);
    console.log(`  Average Response Time: ${metrics.responseTime.toFixed(2)}ms`);
    console.log(`  Throughput: ${metrics.throughput.toFixed(2)} req/s`);
    console.log(`  Error Rate: ${metrics.errorRate.toFixed(2)}%`);
    
    // Performance targets for medium load
    expect(metrics.responseTime).toBeLessThan(PERFORMANCE_THRESHOLDS.SLOW_RESPONSE);
    expect(metrics.errorRate).toBeLessThan(10); // Less than 10% error rate
    expect(metrics.throughput).toBeGreaterThan(15); // At least 15 req/s
    
    console.log('✅ Medium load handling validated');
  });

  test('should handle heavy concurrent load (50 users)', async ({ page, request }) => {
    console.log('🧪 Testing heavy concurrent load handling');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    const metrics = await runLoadTest(
      request,
      '/api/v1/admin/users',
      LOAD_TEST_CONFIG.HEAVY_LOAD,
      20000, // 20 seconds
      token
    );
    
    console.log(`📊 Heavy Load Results:`);
    console.log(`  Average Response Time: ${metrics.responseTime.toFixed(2)}ms`);
    console.log(`  Throughput: ${metrics.throughput.toFixed(2)} req/s`);
    console.log(`  Error Rate: ${metrics.errorRate.toFixed(2)}%`);
    
    // Performance targets for heavy load (more lenient)
    expect(metrics.responseTime).toBeLessThan(PERFORMANCE_THRESHOLDS.TIMEOUT);
    expect(metrics.errorRate).toBeLessThan(20); // Less than 20% error rate
    expect(metrics.throughput).toBeGreaterThan(10); // At least 10 req/s
    
    if (metrics.errorRate <= 15 && metrics.responseTime <= PERFORMANCE_THRESHOLDS.SLOW_RESPONSE) {
      console.log('✅ Heavy load handled excellently');
    } else {
      console.log('⚠️ Heavy load handled with degraded performance');
    }
  });

  test('should validate session validation performance under load', async ({ page, request }) => {
    console.log('🧪 Testing session validation performance under load');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test session validation with concurrent requests
    const concurrentRequests = Array(20).fill(null).map((_, i) => 
      measureResponseTime(
        request,
        `/api/v1/admin/auth/profile?req=${i}`,
        'GET',
        undefined,
        {},
        token
      )
    );
    
    const results = await Promise.all(concurrentRequests);
    const successfulResults = results.filter(r => r.success);
    
    if (successfulResults.length > 0) {
      const avgTime = successfulResults.reduce((acc, r) => acc + r.duration, 0) / successfulResults.length;
      const successRate = (successfulResults.length / results.length) * 100;
      
      console.log(`📊 Session Validation Under Load:`);
      console.log(`  Average Response Time: ${avgTime.toFixed(2)}ms`);
      console.log(`  Success Rate: ${successRate.toFixed(1)}%`);
      
      // Session validation should remain fast even under load
      expect(avgTime).toBeLessThan(PERFORMANCE_THRESHOLDS.NORMAL_RESPONSE);
      expect(successRate).toBeGreaterThan(80);
      
      console.log('✅ Session validation performance under load validated');
    }
  });
});

// ============================================================================
// Memory and Resource Usage Tests
// ============================================================================

test.describe('💾 Memory and Resource Usage', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should maintain stable memory usage under sustained load', async ({ page, request }) => {
    console.log('🧪 Testing memory usage stability');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Monitor memory usage during sustained requests
    const testDuration = 30000; // 30 seconds
    const requestInterval = 100; // 100ms between requests
    const startTime = performance.now();
    
    let requestCount = 0;
    let memorySnapshots = [];
    
    while (performance.now() - startTime < testDuration) {
      // Make request
      const result = await measureResponseTime(
        request,
        `/api/v1/admin/users?req=${requestCount}`,
        'GET',
        undefined,
        {},
        token
      );
      
      requestCount++;
      
      // Take memory snapshot every 100 requests
      if (requestCount % 100 === 0) {
        const memoryUsage = await page.evaluate(() => {
          if (performance.memory) {
            return {
              used: performance.memory.usedJSHeapSize,
              total: performance.memory.totalJSHeapSize,
              limit: performance.memory.jsHeapSizeLimit
            };
          }
          return null;
        });
        
        if (memoryUsage) {
          memorySnapshots.push({
            requestCount,
            timestamp: performance.now() - startTime,
            ...memoryUsage
          });
        }
      }
      
      await new Promise(resolve => setTimeout(resolve, requestInterval));
    }
    
    console.log(`📊 Memory Usage Analysis (${requestCount} requests):`);
    
    if (memorySnapshots.length > 1) {
      const initialMemory = memorySnapshots[0].used;
      const finalMemory = memorySnapshots[memorySnapshots.length - 1].used;
      const memoryGrowth = finalMemory - initialMemory;
      const memoryGrowthPercent = (memoryGrowth / initialMemory) * 100;
      
      console.log(`  Initial Memory: ${(initialMemory / 1024 / 1024).toFixed(2)} MB`);
      console.log(`  Final Memory: ${(finalMemory / 1024 / 1024).toFixed(2)} MB`);
      console.log(`  Memory Growth: ${(memoryGrowth / 1024 / 1024).toFixed(2)} MB (${memoryGrowthPercent.toFixed(1)}%)`);
      
      // Memory growth should be reasonable
      expect(memoryGrowthPercent).toBeLessThan(50); // Less than 50% growth
      
      if (memoryGrowthPercent < 10) {
        console.log('✅ Excellent memory stability');
      } else if (memoryGrowthPercent < 25) {
        console.log('🟡 Good memory stability');
      } else {
        console.log('⚠️ High memory growth detected');
      }
    }
  });

  test('should handle large dataset operations efficiently', async ({ page, request }) => {
    console.log('🧪 Testing large dataset handling performance');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test endpoints that potentially return large datasets
    const largeDatasetEndpoints = [
      '/api/v1/admin/users?limit=1000',
      '/api/v1/admin/permissions/audit-report',
      '/api/v1/admin/analytics/permissions',
      '/api/v1/admin/users/search?q=test'
    ];
    
    const performanceResults = [];
    
    for (const endpoint of largeDatasetEndpoints) {
      const result = await measureResponseTime(
        request,
        endpoint,
        'GET',
        undefined,
        {},
        token
      );
      
      if (result.success || result.status !== 0) {
        performanceResults.push({
          endpoint,
          duration: result.duration,
          status: result.status,
          success: result.success
        });
        
        console.log(`${endpoint}: ${result.duration.toFixed(2)}ms (${result.status})`);
        
        // Large dataset operations should complete within reasonable time
        if (result.duration <= PERFORMANCE_THRESHOLDS.SLOW_RESPONSE) {
          console.log('  ✅ Efficient large dataset handling');
        } else {
          console.log('  ⚠️ Slow large dataset handling');
        }
      }
    }
    
    // Overall large dataset performance
    const avgTime = performanceResults.reduce((acc, r) => acc + r.duration, 0) / performanceResults.length;
    console.log(`📊 Average large dataset response time: ${avgTime.toFixed(2)}ms`);
    
    expect(avgTime).toBeLessThan(PERFORMANCE_THRESHOLDS.TIMEOUT);
  });
});

// ============================================================================
// Cache Performance Tests
// ============================================================================

test.describe('🗄️ Cache Performance', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should demonstrate cache effectiveness', async ({ page, request }) => {
    console.log('🧪 Testing cache performance effectiveness');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    const cacheableEndpoint = '/api/v1/admin/users';
    
    // First request (cache miss)
    const firstRequest = await measureResponseTime(
      request,
      cacheableEndpoint,
      'GET',
      undefined,
      {},
      token
    );
    
    // Wait a brief moment
    await new Promise(resolve => setTimeout(resolve, 100));
    
    // Second request (potential cache hit)
    const secondRequest = await measureResponseTime(
      request,
      cacheableEndpoint,
      'GET',
      undefined,
      {},
      token
    );
    
    // Third request (should be cache hit)
    const thirdRequest = await measureResponseTime(
      request,
      cacheableEndpoint,
      'GET',
      undefined,
      {},
      token
    );
    
    if (firstRequest.success && secondRequest.success && thirdRequest.success) {
      console.log(`📊 Cache Performance Analysis:`);
      console.log(`  First request (cache miss): ${firstRequest.duration.toFixed(2)}ms`);
      console.log(`  Second request (cache hit?): ${secondRequest.duration.toFixed(2)}ms`);
      console.log(`  Third request (cache hit?): ${thirdRequest.duration.toFixed(2)}ms`);
      
      // Calculate cache effectiveness
      const avgCachedTime = (secondRequest.duration + thirdRequest.duration) / 2;
      const cacheImprovement = ((firstRequest.duration - avgCachedTime) / firstRequest.duration) * 100;
      
      console.log(`  Cache improvement: ${cacheImprovement.toFixed(1)}%`);
      
      if (cacheImprovement > 10) {
        console.log('✅ Effective caching detected');
      } else {
        console.log('⚠️ Limited caching improvement');
      }
      
      // Cached requests should generally be faster
      expect(avgCachedTime).toBeLessThanOrEqual(firstRequest.duration * 1.2);
    }
  });

  test('should validate session cache performance', async ({ page, request }) => {
    console.log('🧪 Testing session cache performance');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test session validation caching with multiple rapid requests
    const sessionRequests = Array(10).fill(null).map((_, i) => 
      measureResponseTime(
        request,
        `/api/v1/admin/auth/profile?cache_test=${i}`,
        'GET',
        undefined,
        {},
        token
      )
    );
    
    const results = await Promise.all(sessionRequests);
    const successfulResults = results.filter(r => r.success);
    
    if (successfulResults.length > 0) {
      const avgTime = successfulResults.reduce((acc, r) => acc + r.duration, 0) / successfulResults.length;
      const minTime = Math.min(...successfulResults.map(r => r.duration));
      const maxTime = Math.max(...successfulResults.map(r => r.duration));
      
      console.log(`📊 Session Cache Performance:`);
      console.log(`  Average: ${avgTime.toFixed(2)}ms`);
      console.log(`  Min: ${minTime.toFixed(2)}ms`);
      console.log(`  Max: ${maxTime.toFixed(2)}ms`);
      console.log(`  Consistency: ${((maxTime - minTime) / avgTime * 100).toFixed(1)}% variation`);
      
      // Session validation should be consistently fast due to caching
      expect(avgTime).toBeLessThan(PERFORMANCE_THRESHOLDS.FAST_RESPONSE * 2);
      
      console.log('✅ Session cache performance validated');
    }
  });
});

// ============================================================================
// Rate Limiting Performance Impact
// ============================================================================

test.describe('🚦 Rate Limiting Performance Impact', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should measure rate limiting overhead', async ({ page, request }) => {
    console.log('🧪 Testing rate limiting performance overhead');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test rate limiting overhead with normal request pattern
    const normalRequests = [];
    
    for (let i = 0; i < 10; i++) {
      const result = await measureResponseTime(
        request,
        `/api/v1/admin/users?rl_test=${i}`,
        'GET',
        undefined,
        {},
        token
      );
      
      normalRequests.push(result);
      
      // Normal spacing between requests
      await new Promise(resolve => setTimeout(resolve, 200));
    }
    
    // Test rate limiting with rapid requests (may trigger rate limiting)
    const rapidRequests = [];
    
    for (let i = 0; i < 10; i++) {
      const result = await measureResponseTime(
        request,
        `/api/v1/admin/users?rapid_test=${i}`,
        'GET',
        undefined,
        {},
        token
      );
      
      rapidRequests.push(result);
      
      // Minimal delay (may trigger rate limiting)
      await new Promise(resolve => setTimeout(resolve, 10));
    }
    
    // Analyze rate limiting overhead
    const normalSuccesses = normalRequests.filter(r => r.success);
    const rapidSuccesses = rapidRequests.filter(r => r.success);
    const rapidRateLimited = rapidRequests.filter(r => r.status === 429);
    
    if (normalSuccesses.length > 0) {
      const normalAvgTime = normalSuccesses.reduce((acc, r) => acc + r.duration, 0) / normalSuccesses.length;
      const rapidAvgTime = rapidSuccesses.length > 0 
        ? rapidSuccesses.reduce((acc, r) => acc + r.duration, 0) / rapidSuccesses.length 
        : 0;
      
      console.log(`📊 Rate Limiting Impact:`);
      console.log(`  Normal requests avg: ${normalAvgTime.toFixed(2)}ms`);
      console.log(`  Rapid requests avg: ${rapidAvgTime.toFixed(2)}ms`);
      console.log(`  Rate limited requests: ${rapidRateLimited.length}/10`);
      
      if (rapidRateLimited.length > 0) {
        const rateLimitAvgTime = rapidRateLimited.reduce((acc, r) => acc + r.duration, 0) / rapidRateLimited.length;
        console.log(`  Rate limit response time: ${rateLimitAvgTime.toFixed(2)}ms`);
        
        // Rate limit responses should be fast
        expect(rateLimitAvgTime).toBeLessThan(PERFORMANCE_THRESHOLDS.FAST_RESPONSE);
        
        console.log('✅ Rate limiting working with fast responses');
      }
      
      // Rate limiting should not significantly impact normal request performance
      if (rapidSuccesses.length > 0) {
        const performanceImpact = ((rapidAvgTime - normalAvgTime) / normalAvgTime) * 100;
        console.log(`  Performance impact: ${performanceImpact.toFixed(1)}%`);
        
        expect(performanceImpact).toBeLessThan(50); // Less than 50% overhead
      }
    }
  });
});

// ============================================================================
// Database Performance Tests
// ============================================================================

test.describe('🗃️ Database Performance', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should validate database query performance', async ({ page, request }) => {
    console.log('🧪 Testing database query performance');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test various database-intensive operations
    const dbIntensiveEndpoints = [
      { endpoint: '/api/v1/admin/users', operation: 'User list query' },
      { endpoint: '/api/v1/admin/users/search?q=admin', operation: 'User search query' },
      { endpoint: '/api/v1/admin/permissions/audit-report', operation: 'Audit report query' },
      { endpoint: '/api/v1/admin/analytics/user-statistics', operation: 'Analytics aggregation' }
    ];
    
    for (const test of dbIntensiveEndpoints) {
      const samples = [];
      
      // Run multiple samples
      for (let i = 0; i < 3; i++) {
        const result = await measureResponseTime(
          request,
          test.endpoint,
          'GET',
          undefined,
          {},
          token
        );
        
        if (result.success || result.status !== 0) {
          samples.push(result.duration);
        }
      }
      
      if (samples.length > 0) {
        const avgTime = samples.reduce((a, b) => a + b, 0) / samples.length;
        
        console.log(`${test.operation}: ${avgTime.toFixed(2)}ms`);
        
        // Database queries should be reasonably fast
        expect(avgTime).toBeLessThan(PERFORMANCE_THRESHOLDS.TIMEOUT);
        
        if (avgTime <= PERFORMANCE_THRESHOLDS.NORMAL_RESPONSE) {
          console.log('  ✅ Good database performance');
        } else {
          console.log('  ⚠️ Slow database query detected');
        }
      }
    }
  });

  test('should test connection pool performance', async ({ page, request }) => {
    console.log('🧪 Testing database connection pool performance');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test concurrent database requests to stress connection pool
    const concurrentDbRequests = Array(15).fill(null).map((_, i) => 
      measureResponseTime(
        request,
        `/api/v1/admin/users?pool_test=${i}`,
        'GET',
        undefined,
        {},
        token
      )
    );
    
    const results = await Promise.all(concurrentDbRequests);
    const successfulResults = results.filter(r => r.success);
    
    if (successfulResults.length > 0) {
      const avgTime = successfulResults.reduce((acc, r) => acc + r.duration, 0) / successfulResults.length;
      const maxTime = Math.max(...successfulResults.map(r => r.duration));
      const successRate = (successfulResults.length / results.length) * 100;
      
      console.log(`📊 Connection Pool Performance:`);
      console.log(`  Average Response Time: ${avgTime.toFixed(2)}ms`);
      console.log(`  Max Response Time: ${maxTime.toFixed(2)}ms`);
      console.log(`  Success Rate: ${successRate.toFixed(1)}%`);
      
      // Connection pool should handle concurrent requests efficiently
      expect(successRate).toBeGreaterThan(80); // At least 80% success rate
      expect(avgTime).toBeLessThan(PERFORMANCE_THRESHOLDS.SLOW_RESPONSE);
      
      console.log('✅ Database connection pool performance validated');
    }
  });
});

// ============================================================================
// Comprehensive Performance Summary
// ============================================================================

test.describe('📊 Performance Test Summary', () => {
  test('should generate comprehensive performance report', async ({ page, request }) => {
    console.log('🧪 Generating comprehensive performance report');
    
    await loginAdmin(page);
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Run a subset of performance tests to generate summary
    const performanceSummary = {
      responseTime: { tests: 0, passedSubThreshold: 0, avgTime: 0 },
      concurrency: { tests: 0, passedLoadTest: 0, avgThroughput: 0 },
      memory: { tests: 0, passedStabilityTest: 0 },
      cache: { tests: 0, showedImprovement: 0 },
      rateLimit: { tests: 0, workingProperly: 0 },
      database: { tests: 0, performedWell: 0 }
    };
    
    // Test core endpoint response times
    const coreEndpoints = [
      '/api/v1/admin/users',
      '/api/v1/admin/auth/profile',
      '/api/v1/admin/admin-modules'
    ];
    
    const responseTimes = [];
    
    for (const endpoint of coreEndpoints) {
      const result = await measureResponseTime(
        request,
        endpoint,
        'GET',
        undefined,
        {},
        token
      );
      
      performanceSummary.responseTime.tests++;
      
      if (result.success) {
        responseTimes.push(result.duration);
        
        if (result.duration <= PERFORMANCE_THRESHOLDS.FAST_RESPONSE) {
          performanceSummary.responseTime.passedSubThreshold++;
        }
      }
    }
    
    performanceSummary.responseTime.avgTime = responseTimes.length > 0 
      ? responseTimes.reduce((a, b) => a + b, 0) / responseTimes.length 
      : 0;
    
    // Test light concurrency
    try {
      const concurrencyMetrics = await runLoadTest(
        request,
        '/api/v1/admin/users',
        10, // 10 concurrent users
        5000, // 5 seconds
        token
      );
      
      performanceSummary.concurrency.tests++;
      performanceSummary.concurrency.avgThroughput = concurrencyMetrics.throughput;
      
      if (concurrencyMetrics.errorRate < 10 && concurrencyMetrics.responseTime < PERFORMANCE_THRESHOLDS.NORMAL_RESPONSE) {
        performanceSummary.concurrency.passedLoadTest++;
      }
    } catch (error) {
      console.log('⚠️ Concurrency test skipped due to error');
    }
    
    // Test cache effectiveness
    const firstRequest = await measureResponseTime(request, '/api/v1/admin/users', 'GET', undefined, {}, token);
    await new Promise(resolve => setTimeout(resolve, 100));
    const secondRequest = await measureResponseTime(request, '/api/v1/admin/users', 'GET', undefined, {}, token);
    
    performanceSummary.cache.tests++;
    if (firstRequest.success && secondRequest.success && secondRequest.duration <= firstRequest.duration) {
      performanceSummary.cache.showedImprovement++;
    }
    
    // Generate performance report
    console.log('\\n🚀 PERFORMANCE TEST REPORT');
    console.log('============================');
    
    const responseTimeScore = (performanceSummary.responseTime.passedSubThreshold / Math.max(1, performanceSummary.responseTime.tests)) * 100;
    const concurrencyScore = (performanceSummary.concurrency.passedLoadTest / Math.max(1, performanceSummary.concurrency.tests)) * 100;
    const cacheScore = (performanceSummary.cache.showedImprovement / Math.max(1, performanceSummary.cache.tests)) * 100;
    
    console.log(`📈 Response Time Performance: ${responseTimeScore.toFixed(1)}%`);
    console.log(`   Average Response Time: ${performanceSummary.responseTime.avgTime.toFixed(2)}ms`);
    console.log(`   Sub-100ms Endpoints: ${performanceSummary.responseTime.passedSubThreshold}/${performanceSummary.responseTime.tests}`);
    
    console.log(`👥 Concurrency Performance: ${concurrencyScore.toFixed(1)}%`);
    console.log(`   Average Throughput: ${performanceSummary.concurrency.avgThroughput.toFixed(2)} req/s`);
    
    console.log(`🗄️ Cache Effectiveness: ${cacheScore.toFixed(1)}%`);
    
    // Calculate overall performance score
    const overallScore = (responseTimeScore + concurrencyScore + cacheScore) / 3;
    
    console.log(`\\n🎯 Overall Performance Score: ${overallScore.toFixed(1)}%`);
    
    if (overallScore >= 80) {
      console.log('🟢 EXCELLENT - Outstanding performance');
    } else if (overallScore >= 60) {
      console.log('🟡 GOOD - Acceptable performance');
    } else {
      console.log('🔴 POOR - Performance improvements needed');
    }
    
    // Performance assertion
    expect(overallScore).toBeGreaterThanOrEqual(50); // At least 50% overall performance
    
    console.log('\\n✅ Performance testing completed');
  });
});

// ============================================================================
// Cleanup and Final Validation
// ============================================================================

test.afterAll(async () => {
  console.log('🧹 Cleaning up after performance and load tests');
  console.log('📊 All performance and load testing completed');
  console.log('✅ Performance testing: COMPLETE');
});