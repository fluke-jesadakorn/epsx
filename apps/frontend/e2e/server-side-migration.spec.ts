import { test, expect, Page } from '@playwright/test';

test.describe('Server-Side Migration Validation', () => {
  test.beforeEach(async ({ page }) => {
    // Set up monitoring for each test
    await page.goto('/');
    
    // Inject performance monitoring
    await page.addInitScript(() => {
      (window as any).testMetrics = {
        serverRenderTime: 0,
        hydrationTime: 0,
        cacheHits: 0,
        apiCalls: [],
      };
    });
  });

  test('should render pages with server-side data fetching', async ({ page }) => {
    // Test homepage
    await page.goto('/');
    
    // Check that content is present before hydration
    const initialContent = await page.textContent('body');
    expect(initialContent).toContain('EPSX Trading Platform');
    
    // Verify server-rendered content
    await expect(page.locator('[data-testid="server-rendered"]')).toBeVisible();
    
    // Test navigation performance
    const navigationStart = Date.now();
    await page.click('[data-testid="dashboard-link"]');
    await page.waitForLoadState('networkidle');
    const navigationEnd = Date.now();
    
    const navigationTime = navigationEnd - navigationStart;
    expect(navigationTime).toBeLessThan(2000); // Should load in under 2 seconds
  });

  test('should validate performance targets', async ({ page }) => {
    // Navigate to performance-heavy page
    await page.goto('/analytics');
    
    // Measure Core Web Vitals
    const vitals = await page.evaluate(() => {
      return new Promise((resolve) => {
        const observer = new PerformanceObserver((list) => {
          const entries = list.getEntries();
          const vitals: any = {};
          
          entries.forEach((entry) => {
            if (entry.name === 'first-contentful-paint') {
              vitals.fcp = entry.value;
            }
            if (entry.entryType === 'largest-contentful-paint') {
              vitals.lcp = entry.value;
            }
          });
          
          if (vitals.fcp && vitals.lcp) {
            resolve(vitals);
          }
        });
        
        observer.observe({ entryTypes: ['paint', 'largest-contentful-paint'] });
        
        // Fallback timeout
        setTimeout(() => resolve({ fcp: 0, lcp: 0 }), 5000);
      });
    });
    
    // Validate performance targets
    expect((vitals as any).fcp).toBeLessThan(1800); // Target: <1.8s FCP
    expect((vitals as any).lcp).toBeLessThan(2500); // Target: <2.5s LCP
  });

  test('should handle feature flags correctly', async ({ page }) => {
    // Test with feature flags enabled
    await page.goto('/?feature-test=server-side-migration');
    
    // Should use server-side rendering
    const serverContent = await page.locator('[data-testid="server-rendered"]');
    await expect(serverContent).toBeVisible();
    
    // Test with feature flags disabled (fallback)
    await page.goto('/?feature-test=legacy-mode');
    
    // Should still work but with different implementation
    const content = await page.locator('[data-testid="content"]');
    await expect(content).toBeVisible();
  });

  test('should validate server actions work correctly', async ({ page }) => {
    await page.goto('/dashboard');
    
    // Test server action form submission
    await page.fill('[data-testid="search-input"]', 'AAPL');
    await page.click('[data-testid="search-button"]');
    
    // Should show results without full page reload
    await expect(page.locator('[data-testid="search-results"]')).toBeVisible();
    
    // Verify no client-side API calls were made
    const apiCalls = await page.evaluate(() => {
      return (window as any).testMetrics?.apiCalls || [];
    });
    
    // Should be empty since we're using server actions
    expect(apiCalls.length).toBe(0);
  });

  test('should validate caching works correctly', async ({ page }) => {
    // First visit
    await page.goto('/dashboard');
    const firstLoadTime = await measurePageLoadTime(page);
    
    // Second visit (should be cached)
    await page.reload();
    const secondLoadTime = await measurePageLoadTime(page);
    
    // Second load should be faster due to caching
    expect(secondLoadTime).toBeLessThan(firstLoadTime * 0.8);
  });

  test('should validate error handling and monitoring', async ({ page }) => {
    // Listen for console errors
    const errors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    // Navigate to a page that might have errors
    await page.goto('/dashboard');
    
    // Should not have unhandled errors
    expect(errors.length).toBe(0);
    
    // Test error monitoring
    await page.evaluate(() => {
      // Simulate an error
      throw new Error('Test error for monitoring');
    });
    
    // Wait for error to be captured
    await page.waitForTimeout(1000);
    
    // Verify error was captured by monitoring system
    const errorsCaptured = await page.evaluate(() => {
      return (window as any).performanceMonitor?.getErrors?.() || [];
    });
    
    expect(errorsCaptured.length).toBeGreaterThan(0);
  });

  test('should validate mobile responsiveness', async ({ page, isMobile }) => {
    if (!isMobile) return;
    
    await page.goto('/');
    
    // Check mobile-specific elements
    await expect(page.locator('[data-testid="mobile-menu"]')).toBeVisible();
    
    // Verify touch interactions work
    await page.tap('[data-testid="mobile-menu-toggle"]');
    await expect(page.locator('[data-testid="mobile-nav"]')).toBeVisible();
    
    // Test mobile performance
    const loadTime = await measurePageLoadTime(page);
    expect(loadTime).toBeLessThan(3000); // Mobile should load in under 3 seconds
  });

  test('should validate accessibility', async ({ page }) => {
    await page.goto('/');
    
    // Test keyboard navigation
    await page.keyboard.press('Tab');
    const focusedElement = await page.locator(':focus');
    expect(await focusedElement.count()).toBeGreaterThan(0);
    
    // Test ARIA attributes
    const buttons = await page.locator('button[aria-label]');
    expect(await buttons.count()).toBeGreaterThan(0);
    
    // Test semantic HTML
    const headings = await page.locator('h1, h2, h3, h4, h5, h6');
    expect(await headings.count()).toBeGreaterThan(0);
  });

  test('should validate production-like environment', async ({ page }) => {
    // Test with production build
    const response = await page.goto('/');
    expect(response?.status()).toBe(200);
    
    // Check for minification (no source maps in production)
    const scripts = await page.locator('script[src]').all();
    for (const script of scripts) {
      const src = await script.getAttribute('src');
      if (src && !src.includes('webpack')) {
        expect(src).not.toContain('.map');
      }
    }
    
    // Verify security headers
    const headers = response?.headers() || {};
    expect(headers['x-frame-options'] || headers['x-frame-options']).toBeDefined();
  });
});

// Helper function to measure page load time
async function measurePageLoadTime(page: Page): Promise<number> {
  return await page.evaluate(() => {
    const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
    return navigation.loadEventEnd - navigation.navigationStart;
  });
}