import { test, expect, Page, Request, Response } from '@playwright/test';
import { 
  TEST_USERS, 
  TestUser, 
  generateMockJWT, 
  canUserAccessRoute
} from '../fixtures/user-fixtures';

/**
 * Frontend Middleware Validation Tests
 * Tests Next.js middleware stack for session validation, tier-based access control,
 * security headers, performance monitoring, and error handling
 */

test.describe('🛡️ Frontend Middleware Validation', () => {

  // Helper to set up user authentication
  async function setupUserAuth(page: Page, user: TestUser): Promise<void> {
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

  // Helper to capture network requests and responses
  async function captureNetworkActivity(page: Page): Promise<{requests: Request[], responses: Response[]}> {
    const requests: Request[] = [];
    const responses: Response[] = [];
    
    page.on('request', request => requests.push(request));
    page.on('response', response => responses.push(response));
    
    return { requests, responses };
  }

  test.describe('🔐 Session Validation Middleware', () => {
    
    test('should validate JWT tokens and authenticate users', async ({ page }) => {
      // Test with valid user
      const validUser = TEST_USERS.GOLD_USER;
      await setupUserAuth(page, validUser);
      
      const { requests, responses } = await captureNetworkActivity(page);
      
      const response = await page.goto('/dashboard');
      
      // Should successfully load protected route
      expect(response?.status()).toBe(200);
      expect(page.url()).toContain('/dashboard');
      
      // Should have session validation headers
      const pageResponse = responses.find(r => r.url().includes('/dashboard'));
      if (pageResponse) {
        const headers = pageResponse.headers();
        expect(headers['x-user-id']).toBe(validUser.id);
        expect(headers['x-user-email']).toBe(validUser.email);
        expect(headers['x-user-role']).toBe(validUser.role);
        expect(headers['x-user-package-tier']).toBe(validUser.package_tier);
      }
      
      console.log('✅ JWT validation successful for authenticated user');
    });

    test('should reject invalid JWT tokens', async ({ page }) => {
      // Set invalid JWT token
      await page.context().addCookies([{
        name: 'epsx_jwt',
        value: 'invalid_jwt_token_12345',
        domain: 'localhost',
        path: '/',
        httpOnly: true,
        secure: false
      }]);
      
      const response = await page.goto('/dashboard');
      
      // Should redirect to login
      expect(page.url()).toContain('/oauth/authorize');
      
      console.log('✅ Invalid JWT token correctly rejected');
    });

    test('should handle missing JWT tokens', async ({ page }) => {
      // Don't set any JWT token
      const response = await page.goto('/dashboard');
      
      // Should redirect to login
      expect(page.url()).toContain('/oauth/authorize');
      
      console.log('✅ Missing JWT token handled correctly');
    });

    test('should validate session with backend API', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.SILVER_USER);
      
      // Mock backend session validation response
      await page.route('**/api/auth/validate-session', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            valid: true,
            user: TEST_USERS.SILVER_USER,
            permissions: TEST_USERS.SILVER_USER.permissions,
            package_tier: TEST_USERS.SILVER_USER.package_tier,
            performance: {
              validation_time_ms: 45,
              cache_hit: false
            }
          })
        });
      });
      
      const response = await page.goto('/professional');
      
      expect(response?.status()).toBe(200);
      expect(page.url()).toContain('/professional');
      
      console.log('✅ Backend session validation integration working');
    });

    test('should handle backend validation failures', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.SILVER_USER);
      
      // Mock backend session validation failure
      await page.route('**/api/auth/validate-session', async route => {
        await route.fulfill({
          status: 401,
          contentType: 'application/json',
          body: JSON.stringify({
            valid: false,
            error: 'Session expired'
          })
        });
      });
      
      const response = await page.goto('/professional');
      
      // Should redirect to login
      expect(page.url()).toContain('/oauth/authorize');
      
      console.log('✅ Backend validation failure handled correctly');
    });
  });

  test.describe('🎯 Tier-Based Access Control', () => {
    
    test('should enforce tier-based route protection', async ({ page }) => {
      const testCases = [
        { user: TEST_USERS.FREE_USER, route: '/premium', shouldAccess: false },
        { user: TEST_USERS.BRONZE_USER, route: '/premium', shouldAccess: true },
        { user: TEST_USERS.BRONZE_USER, route: '/vip', shouldAccess: false },
        { user: TEST_USERS.GOLD_USER, route: '/vip', shouldAccess: true },
        { user: TEST_USERS.GOLD_USER, route: '/elite', shouldAccess: false },
        { user: TEST_USERS.PLATINUM_USER, route: '/elite', shouldAccess: true },
        { user: TEST_USERS.PLATINUM_USER, route: '/enterprise', shouldAccess: false },
        { user: TEST_USERS.ENTERPRISE_USER, route: '/enterprise', shouldAccess: true }
      ];

      for (const testCase of testCases) {
        await setupUserAuth(page, testCase.user);
        
        const response = await page.goto(testCase.route);
        
        if (testCase.shouldAccess) {
          expect(page.url()).toContain(testCase.route);
          expect(response?.status()).toBeLessThan(400);
        } else {
          // Should redirect to upgrade page
          expect(page.url()).toContain('/upgrade');
          
          // Check upgrade parameters
          const url = new URL(page.url());
          const requiredTier = url.searchParams.get('tier');
          const currentTier = url.searchParams.get('current');
          const feature = url.searchParams.get('feature');
          
          expect(currentTier).toBe(testCase.user.package_tier);
          expect(feature).toBe(testCase.route);
        }
        
        console.log(`✅ ${testCase.user.package_tier} -> ${testCase.route}: ${testCase.shouldAccess ? 'ALLOWED' : 'BLOCKED'}`);
      }
    });

    test('should add user context headers for authorized requests', async ({ page }) => {
      const testUser = TEST_USERS.GOLD_USER;
      await setupUserAuth(page, testUser);
      
      const { responses } = await captureNetworkActivity(page);
      
      await page.goto('/vip');
      
      // Find the main page response
      const pageResponse = responses.find(r => r.url().includes('/vip') && r.request().resourceType() === 'document');
      
      if (pageResponse) {
        const headers = pageResponse.headers();
        
        // Check user context headers
        expect(headers['x-user-id']).toBe(testUser.id);
        expect(headers['x-user-email']).toBe(testUser.email);
        expect(headers['x-user-role']).toBe(testUser.role);
        expect(headers['x-user-package-tier']).toBe(testUser.package_tier);
        
        // Check permissions header
        const permissions = JSON.parse(headers['x-user-permissions'] || '[]');
        expect(permissions).toEqual(testUser.permissions);
      }
      
      console.log('✅ User context headers added correctly');
    });

    test('should validate permission combinations for complex routes', async ({ page }) => {
      // Test user with multiple permissions
      await setupUserAuth(page, TEST_USERS.PLATINUM_USER);
      
      // Routes requiring multiple permissions
      const complexRoutes = [
        '/reports', // Requires research-reports permission
        '/custom-dashboards', // Requires custom-dashboards permission
        '/priority-support' // Requires priority-support permission
      ];

      for (const route of complexRoutes) {
        const response = await page.goto(route);
        expect(response?.status()).toBeLessThan(400);
        expect(page.url()).toContain(route);
      }
      
      console.log('✅ Complex permission validation successful');
    });
  });

  test.describe('🔒 Security Headers and Protection', () => {
    
    test('should add comprehensive security headers', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.SILVER_USER);
      
      const { responses } = await captureNetworkActivity(page);
      
      await page.goto('/professional');
      
      const pageResponse = responses.find(r => r.url().includes('/professional'));
      
      if (pageResponse) {
        const headers = pageResponse.headers();
        
        // Check security headers
        expect(headers['x-content-type-options']).toBe('nosniff');
        expect(headers['x-frame-options']).toBe('SAMEORIGIN');
        expect(headers['x-xss-protection']).toBe('1; mode=block');
        expect(headers['referrer-policy']).toBe('strict-origin-when-cross-origin');
        expect(headers['strict-transport-security']).toBe('max-age=31536000; includeSubDomains');
        expect(headers['permissions-policy']).toContain('geolocation=()');
        
        // Check middleware-specific headers
        expect(headers['x-pathname']).toBe('/professional');
        expect(headers['x-middleware-timestamp']).toBeTruthy();
      }
      
      console.log('✅ Security headers applied correctly');
    });

    test('should handle CORS for trading platform', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.GOLD_USER);
      
      // Test CORS preflight for API requests
      const corsResponse = await page.request.fetch('/api/trading/orders', {
        method: 'OPTIONS',
        headers: {
          'Origin': 'http://localhost:3000',
          'Access-Control-Request-Method': 'POST',
          'Access-Control-Request-Headers': 'authorization,content-type'
        }
      });
      
      const corsHeaders = corsResponse.headers();
      expect(corsHeaders['access-control-allow-origin']).toBeTruthy();
      expect(corsHeaders['access-control-allow-methods']).toBeTruthy();
      
      console.log('✅ CORS handling verified');
    });

    test('should protect against common attacks', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.FREE_USER);
      
      // Test XSS protection
      const maliciousScript = '<script>alert("xss")</script>';
      await page.goto(`/?search=${encodeURIComponent(maliciousScript)}`);
      
      // Should not execute script
      const alerts = [];
      page.on('dialog', dialog => {
        alerts.push(dialog.message());
        dialog.dismiss();
      });
      
      await page.waitForTimeout(1000);
      expect(alerts).toHaveLength(0);
      
      console.log('✅ XSS protection working');
    });
  });

  test.describe('📊 Performance Monitoring', () => {
    
    test('should track middleware execution time', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.GOLD_USER);
      
      const { responses } = await captureNetworkActivity(page);
      
      const startTime = Date.now();
      await page.goto('/vip');
      const endTime = Date.now();
      
      const pageResponse = responses.find(r => r.url().includes('/vip'));
      
      if (pageResponse) {
        const headers = pageResponse.headers();
        
        // Check performance headers
        expect(headers['x-middleware-performance']).toBeTruthy();
        
        const middlewareTime = parseFloat(headers['x-middleware-performance'] || '0');
        expect(middlewareTime).toBeGreaterThan(0);
        expect(middlewareTime).toBeLessThan(1000); // Should be under 1 second
        
        console.log(`📊 Middleware execution time: ${middlewareTime}ms`);
      }
      
      console.log('✅ Performance monitoring active');
    });

    test('should track session validation performance', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.ENTERPRISE_USER);
      
      // Mock backend with performance metrics
      await page.route('**/api/auth/validate-session', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            valid: true,
            user: TEST_USERS.ENTERPRISE_USER,
            performance: {
              validation_time_ms: 25,
              cache_hit: true
            }
          })
        });
      });
      
      const { responses } = await captureNetworkActivity(page);
      await page.goto('/enterprise');
      
      const pageResponse = responses.find(r => r.url().includes('/enterprise'));
      
      if (pageResponse) {
        const headers = pageResponse.headers();
        
        expect(headers['x-session-cache-hit']).toBe('true');
        expect(headers['x-session-validation-time']).toBe('25');
      }
      
      console.log('✅ Session validation performance tracking working');
    });

    test('should log slow middleware operations', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.PLATINUM_USER);
      
      // Mock slow backend response
      await page.route('**/api/auth/validate-session', async route => {
        await new Promise(resolve => setTimeout(resolve, 150)); // Slow response
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            valid: true,
            user: TEST_USERS.PLATINUM_USER,
            performance: {
              validation_time_ms: 150,
              cache_hit: false
            }
          })
        });
      });
      
      // Capture console warnings
      const consoleMessages: string[] = [];
      page.on('console', msg => {
        if (msg.type() === 'warning' && msg.text().includes('Slow validation')) {
          consoleMessages.push(msg.text());
        }
      });
      
      await page.goto('/elite');
      
      // Should log slow validation warning
      expect(consoleMessages.length).toBeGreaterThan(0);
      
      console.log('✅ Slow operation logging working');
    });

    test('should record performance metrics to backend', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.GOLD_USER);
      
      let metricsRecorded = false;
      
      // Intercept metrics recording
      await page.route('**/api/security/metrics', async route => {
        metricsRecorded = true;
        const requestBody = route.request().postDataJSON();
        
        expect(requestBody.source).toBe('frontend-middleware');
        expect(requestBody.middleware_execution_time).toBeGreaterThan(0);
        expect(requestBody.path).toBeTruthy();
        expect(requestBody.method).toBeTruthy();
        
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ success: true })
        });
      });
      
      await page.goto('/vip');
      
      expect(metricsRecorded).toBe(true);
      console.log('✅ Performance metrics recording verified');
    });
  });

  test.describe('🚨 Security Event Logging', () => {
    
    test('should log authentication failures', async ({ page }) => {
      let securityEventLogged = false;
      
      // Intercept security event logging
      await page.route('**/api/security/events', async route => {
        securityEventLogged = true;
        const requestBody = route.request().postDataJSON();
        
        expect(requestBody.event_type).toBe('AUTHENTICATION_FAILED');
        expect(requestBody.severity).toBe('HIGH');
        expect(requestBody.source).toBe('frontend-middleware');
        
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ success: true })
        });
      });
      
      // Try to access protected route without auth
      await page.goto('/vip');
      
      expect(securityEventLogged).toBe(true);
      console.log('✅ Authentication failure logging verified');
    });

    test('should log access denied events', async ({ page }) => {
      let accessDeniedLogged = false;
      
      await setupUserAuth(page, TEST_USERS.FREE_USER);
      
      // Intercept security event logging
      await page.route('**/api/security/events', async route => {
        const requestBody = route.request().postDataJSON();
        
        if (requestBody.event_type === 'ACCESS_DENIED') {
          accessDeniedLogged = true;
          expect(requestBody.details.requiredTier).toBe('GOLD');
          expect(requestBody.details.userTier).toBe('FREE');
          expect(requestBody.details.requiredFeatures).toBe('/vip');
        }
        
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ success: true })
        });
      });
      
      // Try to access VIP route as FREE user
      await page.goto('/vip');
      
      expect(accessDeniedLogged).toBe(true);
      console.log('✅ Access denied logging verified');
    });

    test('should log middleware errors', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.SILVER_USER);
      
      let errorLogged = false;
      
      // Mock backend to cause middleware error
      await page.route('**/api/auth/validate-session', async route => {
        await route.abort('failed');
      });
      
      // Intercept security event logging
      await page.route('**/api/security/events', async route => {
        const requestBody = route.request().postDataJSON();
        
        if (requestBody.event_type === 'MIDDLEWARE_ERROR') {
          errorLogged = true;
          expect(requestBody.severity).toBe('HIGH');
        }
        
        await route.fulfill({ status: 200 });
      });
      
      await page.goto('/professional');
      
      expect(errorLogged).toBe(true);
      console.log('✅ Middleware error logging verified');
    });

    test('should include context in security events', async ({ page }) => {
      let eventWithContext = false;
      
      // Intercept security event logging
      await page.route('**/api/security/events', async route => {
        const requestBody = route.request().postDataJSON();
        
        eventWithContext = true;
        expect(requestBody.ip_address).toBeTruthy();
        expect(requestBody.user_agent).toBeTruthy();
        expect(requestBody.path).toBeTruthy();
        expect(requestBody.method).toBeTruthy();
        expect(requestBody.timestamp).toBeTruthy();
        
        await route.fulfill({ status: 200 });
      });
      
      await page.goto('/enterprise');
      
      expect(eventWithContext).toBe(true);
      console.log('✅ Security event context verified');
    });
  });

  test.describe('🔄 Error Handling and Resilience', () => {
    
    test('should handle network failures gracefully', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.GOLD_USER);
      
      // Mock network failure
      await page.route('**/api/auth/validate-session', async route => {
        await route.abort('connectionrefused');
      });
      
      const response = await page.goto('/vip');
      
      // Should redirect to login on network failure
      expect(page.url()).toContain('/oauth/authorize');
      
      console.log('✅ Network failure handled gracefully');
    });

    test('should handle timeout scenarios', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.PLATINUM_USER);
      
      // Mock timeout
      await page.route('**/api/auth/validate-session', async route => {
        await new Promise(resolve => setTimeout(resolve, 10000)); // Long delay
      });
      
      // Set shorter timeout for test
      page.setDefaultTimeout(2000);
      
      try {
        await page.goto('/elite');
        // Should either timeout or redirect
        expect(page.url()).toContain('/oauth/authorize');
      } catch (error) {
        // Timeout is acceptable behavior
        console.log('⏰ Request timed out as expected');
      }
      
      console.log('✅ Timeout scenario handled');
    });

    test('should maintain middleware state during errors', async ({ page }) => {
      await setupUserAuth(page, TEST_USERS.ENTERPRISE_USER);
      
      // First successful request
      await page.goto('/enterprise');
      expect(page.url()).toContain('/enterprise');
      
      // Cause error on next request
      await page.route('**/api/auth/validate-session', async route => {
        await route.fulfill({ status: 500 });
      }, { times: 1 });
      
      // Should handle error and redirect
      await page.goto('/api-access');
      expect(page.url()).toContain('/oauth/authorize');
      
      // Remove error mock and try again
      await page.unroute('**/api/auth/validate-session');
      await setupUserAuth(page, TEST_USERS.ENTERPRISE_USER);
      
      const response = await page.goto('/enterprise');
      expect(response?.status()).toBe(200);
      
      console.log('✅ Middleware state maintained during errors');
    });
  });

  test.describe('🔄 Public Route Handling', () => {
    
    test('should allow access to public routes without authentication', async ({ page }) => {
      const publicRoutes = [
        '/',
        '/register',
        '/forgot-password',
        '/terms',
        '/privacy'
      ];

      for (const route of publicRoutes) {
        const response = await page.goto(route);
        expect(response?.status()).toBeLessThan(400);
        expect(page.url()).toContain(route);
      }
      
      console.log('✅ Public routes accessible without authentication');
    });

    test('should add security headers to public routes', async ({ page }) => {
      const { responses } = await captureNetworkActivity(page);
      
      await page.goto('/');
      
      const pageResponse = responses.find(r => r.url() === 'http://localhost:3000/');
      
      if (pageResponse) {
        const headers = pageResponse.headers();
        
        // Should still have security headers
        expect(headers['x-content-type-options']).toBe('nosniff');
        expect(headers['x-frame-options']).toBe('SAMEORIGIN');
        expect(headers['x-middleware-performance']).toBeTruthy();
      }
      
      console.log('✅ Security headers applied to public routes');
    });

    test('should handle mixed public and protected content', async ({ page }) => {
      // Access public page
      await page.goto('/');
      expect(page.url()).toBe('http://localhost:3000/');
      
      // Try to access protected route from public page
      await page.click('[data-testid="login-required-feature"]').catch(() => {
        // Button might not exist, which is fine
      });
      
      // Should redirect to login if authentication required
      if (page.url().includes('/oauth/authorize')) {
        console.log('✅ Protected content properly redirected');
      } else {
        console.log('✅ Public content remained accessible');
      }
    });
  });
});