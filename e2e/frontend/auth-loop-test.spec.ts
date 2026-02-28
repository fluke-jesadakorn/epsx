import { test, expect } from '@playwright/test';

/**
 * Authentication Loop Detection Test
 * Using MCP Playwright to identify and fix auth loop issues
 */

test.describe('Authentication Loop Detection', () => {

  test('should not create infinite redirect loop on protected route access', async ({ page }) => {
    // Track redirects
    const redirects: string[] = [];

    page.on('response', (response) => {
      if ([301, 302, 303, 307, 308].includes(response.status())) {
        redirects.push(response.url());
      }
    });

    // Navigate to protected route without authentication
    await page.goto('http://localhost:3000/dashboard');

    // Wait for navigation to settle
    await page.waitForLoadState('networkidle');

    // Check current URL
    const currentUrl = page.url();
    console.log('Current URL:', currentUrl);
    console.log('Redirects:', redirects);

    // Should redirect to signin page
    expect(currentUrl).toContain('/auth/signin');

    // Should have returnUrl parameter
    const url = new URL(currentUrl);
    const returnUrl = url.searchParams.get('returnUrl');
    expect(returnUrl).toBe('/dashboard');

    // Should not have infinite redirects
    expect(redirects.length).toBeLessThan(5);
  });

  test('should not loop when already authenticated on signin page', async ({ page, context }) => {
    // Set authentication cookies (simulate authenticated user)
    await context.addCookies([
      {
        name: 'epsx_access_token',
        value: 'test-token',
        domain: 'localhost',
        path: '/',
      },
      {
        name: 'epsx_expires_at',
        value: String(Date.now() + 3600000),
        domain: 'localhost',
        path: '/',
      },
    ]);

    const redirects: string[] = [];

    page.on('response', (response) => {
      if ([301, 302, 303, 307, 308].includes(response.status())) {
        redirects.push(response.url());
      }
    });

    // Navigate to signin page when already authenticated
    await page.goto('http://localhost:3000/auth/signin?returnUrl=/dashboard');

    // Wait for navigation
    await page.waitForLoadState('networkidle');

    const currentUrl = page.url();
    console.log('Authenticated redirect URL:', currentUrl);
    console.log('Redirects:', redirects);

    // Should redirect to dashboard
    expect(currentUrl).toBe('http://localhost:3000/dashboard');

    // Should not loop
    expect(redirects.length).toBeLessThan(3);
  });

  test('should prevent auth page as returnUrl', async ({ page }) => {
    // Try to use signin as returnUrl (should fallback to dashboard)
    await page.goto('http://localhost:3000/auth/signin?returnUrl=/auth/signin');

    await page.waitForLoadState('networkidle');

    const currentUrl = page.url();
    const url = new URL(currentUrl);
    const returnUrl = url.searchParams.get('returnUrl');

    console.log('Return URL validation:', { currentUrl, returnUrl });

    // returnUrl should be corrected to /dashboard
    expect(returnUrl).not.toBe('/auth/signin');
  });

  test('should handle middleware validation without loop', async ({ page }) => {
    const validationAttempts: number[] = [];
    let requestCount = 0;

    // Track middleware validation requests
    page.on('request', (request) => {
      if (request.url().includes('/api/auth/')) {
        requestCount++;
        validationAttempts.push(Date.now());
      }
    });

    // Navigate to protected route
    await page.goto('http://localhost:3000/dashboard');
    await page.waitForLoadState('networkidle');

    console.log('Validation attempts:', requestCount);
    console.log('Timestamps:', validationAttempts);

    // Should not make excessive validation requests
    expect(requestCount).toBeLessThan(5);

    // Should not retry validation in rapid succession
    if (validationAttempts.length > 1) {
      for (let i = 1; i < validationAttempts.length; i++) {
        const timeDiff = validationAttempts[i] - validationAttempts[i - 1];
        expect(timeDiff).toBeGreaterThan(100); // At least 100ms between attempts
      }
    }
  });

  test('should handle challenge-verify flow without loop', async ({ page }) => {
    const apiCalls: { endpoint: string; timestamp: number }[] = [];

    page.on('request', (request) => {
      const url = request.url();
      if (url.includes('/api/auth/web3/')) {
        const endpoint = url.split('/api/auth/web3/')[1];
        apiCalls.push({ endpoint, timestamp: Date.now() });
      }
    });

    // Go to signin page
    await page.goto('http://localhost:3000/auth/signin');
    await page.waitForLoadState('networkidle');

    console.log('Web3 auth API calls:', apiCalls);

    // Count challenge requests
    const challengeCalls = apiCalls.filter(c => c.endpoint.startsWith('challenge'));
    const verifyCalls = apiCalls.filter(c => c.endpoint.startsWith('verify'));

    console.log('Challenge calls:', challengeCalls.length);
    console.log('Verify calls:', verifyCalls.length);

    // Should not spam challenge endpoint
    expect(challengeCalls.length).toBeLessThan(3);
  });

  test('should detect rapid redirect loops', async ({ page }) => {
    const redirectLog: { url: string; timestamp: number }[] = [];

    page.on('framenavigated', (frame) => {
      if (frame === page.mainFrame()) {
        redirectLog.push({ url: frame.url(), timestamp: Date.now() });
      }
    });

    await page.goto('http://localhost:3000/dashboard');
    await page.waitForTimeout(3000); // Wait 3 seconds

    console.log('Navigation log:', redirectLog);

    // Check for rapid redirects (more than 3 in 5 seconds is suspicious)
    const fiveSecondsAgo = Date.now() - 5000;
    const recentRedirects = redirectLog.filter(r => r.timestamp > fiveSecondsAgo);

    expect(recentRedirects.length).toBeLessThan(4);

    // Check for ping-pong redirects
    const urls = redirectLog.map(r => r.url);
    for (let i = 2; i < urls.length; i++) {
      const isPingPong = urls[i] === urls[i - 2];
      if (isPingPong) {
        console.error('PING-PONG DETECTED:', {
          url1: urls[i - 2],
          url2: urls[i - 1],
          url3: urls[i],
        });
      }
      expect(isPingPong).toBe(false);
    }
  });
});

test.describe('Auth Loop Fixes', () => {

  test('middleware should not validate on public routes', async ({ page }) => {
    const authRequests: string[] = [];

    page.on('request', (request) => {
      if (request.url().includes('/api/auth/')) {
        authRequests.push(request.url());
      }
    });

    // Navigate to public route
    await page.goto('http://localhost:3000/');
    await page.waitForLoadState('networkidle');

    console.log('Auth requests on public route:', authRequests);

    // Should not make auth requests for public routes
    expect(authRequests.length).toBe(0);
  });

  test('session check should use caching', async ({ page, context }) => {
    // Set auth cookies
    await context.addCookies([
      {
        name: 'epsx_access_token',
        value: 'test-token',
        domain: 'localhost',
        path: '/',
      },
    ]);

    const sessionRequests: number[] = [];

    page.on('request', (request) => {
      if (request.url().includes('/api/auth/session')) {
        sessionRequests.push(Date.now());
      }
    });

    // Navigate multiple times
    await page.goto('http://localhost:3000/dashboard');
    await page.waitForLoadState('networkidle');

    await page.goto('http://localhost:3000/analytics');
    await page.waitForLoadState('networkidle');

    console.log('Session check requests:', sessionRequests.length);

    // Should cache session and not check repeatedly
    expect(sessionRequests.length).toBeLessThan(3);
  });
});
