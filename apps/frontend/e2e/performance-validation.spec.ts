import { test, expect } from '@playwright/test';

test.describe('Performance Validation', () => {
  test.beforeEach(async ({ page }) => {
    // Clear caches for accurate performance testing
    await page.context().clearCookies();
    await page.evaluate(() => {
      if ('serviceWorker' in navigator) {
        navigator.serviceWorker.getRegistrations().then(registrations => {
          registrations.forEach(registration => registration.unregister());
        });
      }
    });
  });

  test('should meet bundle size targets', async ({ page }) => {
    await page.goto('/');
    
    // Get all network requests
    const requests: any[] = [];
    page.on('response', (response) => {
      requests.push({
        url: response.url(),
        size: response.headers()['content-length'],
        type: response.request().resourceType(),
      });
    });
    
    await page.waitForLoadState('networkidle');
    
    // Calculate total bundle size
    const jsRequests = requests.filter(req => req.type === 'script');
    const totalBundleSize = jsRequests.reduce((sum, req) => {
      return sum + (parseInt(req.size) || 0);
    }, 0);
    
    // Should be under 3.5MB (30% reduction target)
    expect(totalBundleSize).toBeLessThan(3.5 * 1024 * 1024);
  });

  test('should meet page load time targets', async ({ page }) => {
    const measurements = [];
    
    const pages = ['/', '/dashboard', '/analytics'];
    
    for (const pagePath of pages) {
      const startTime = Date.now();
      await page.goto(pagePath);
      await page.waitForLoadState('networkidle');
      const endTime = Date.now();
      
      const loadTime = endTime - startTime;
      measurements.push({ page: pagePath, loadTime });
      
      // Each page should load in under 2.5 seconds (40% improvement target)
      expect(loadTime).toBeLessThan(2500);
    }
    
    console.log('Page load measurements:', measurements);
  });

  test('should meet Core Web Vitals targets', async ({ page }) => {
    await page.goto('/');
    
    // Measure Core Web Vitals
    const vitals = await page.evaluate(() => {
      return new Promise((resolve) => {
        const measurements: any = {};
        let measurementCount = 0;
        const expectedMeasurements = 3;
        
        const observer = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            if (entry.name === 'first-contentful-paint') {
              measurements.fcp = entry.value;
              measurementCount++;
            }
            
            if (entry.entryType === 'largest-contentful-paint') {
              measurements.lcp = entry.value;
              measurementCount++;
            }
            
            if (entry.entryType === 'first-input') {
              measurements.fid = entry.duration;
              measurementCount++;
            }
          }
          
          if (measurementCount >= expectedMeasurements) {
            observer.disconnect();
            resolve(measurements);
          }
        });
        
        observer.observe({ 
          entryTypes: ['paint', 'largest-contentful-paint', 'first-input'] 
        });
        
        // Simulate user interaction for FID
        setTimeout(() => {
          const button = document.querySelector('button');
          if (button) {
            button.click();
          }
        }, 100);
        
        // Timeout fallback
        setTimeout(() => {
          observer.disconnect();
          resolve(measurements);
        }, 10000);
      });
    });
    
    // Validate Core Web Vitals targets (40% improvement)
    if ((vitals as any).fcp) {
      expect((vitals as any).fcp).toBeLessThan(1500); // Target: <1.5s
    }
    
    if ((vitals as any).lcp) {
      expect((vitals as any).lcp).toBeLessThan(2400); // Target: <2.4s
    }
    
    if ((vitals as any).fid) {
      expect((vitals as any).fid).toBeLessThan(150); // Target: <150ms
    }
  });

  test('should validate caching performance', async ({ page }) => {
    // First visit (cold cache)
    const firstVisitStart = Date.now();
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    const firstVisitTime = Date.now() - firstVisitStart;
    
    // Second visit (warm cache)
    const secondVisitStart = Date.now();
    await page.reload();
    await page.waitForLoadState('networkidle');
    const secondVisitTime = Date.now() - secondVisitStart;
    
    // Third visit (should be even faster)
    const thirdVisitStart = Date.now();
    await page.reload();
    await page.waitForLoadState('networkidle');
    const thirdVisitTime = Date.now() - thirdVisitStart;
    
    // Cached visits should be significantly faster
    expect(secondVisitTime).toBeLessThan(firstVisitTime * 0.7);
    expect(thirdVisitTime).toBeLessThan(firstVisitTime * 0.6);
    
    // Cache hit rate should be good
    const cacheMetrics = await page.evaluate(() => {
      return (window as any).performanceMonitor?.getMetrics?.()
        ?.filter((m: any) => m.metadata?.cached) || [];
    });
    
    const totalMetrics = await page.evaluate(() => {
      return (window as any).performanceMonitor?.getMetrics?.() || [];
    });
    
    if (totalMetrics.length > 0) {
      const cacheHitRate = cacheMetrics.length / totalMetrics.length;
      expect(cacheHitRate).toBeGreaterThan(0.5); // At least 50% cache hit rate
    }
  });

  test('should validate server-side rendering performance', async ({ page }) => {
    // Test SSR timing
    await page.goto('/dashboard');
    
    const ssrMetrics = await page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      
      return {
        serverResponseTime: navigation.responseEnd - navigation.requestStart,
        domProcessingTime: navigation.domContentLoadedEventEnd - navigation.responseEnd,
        totalRenderTime: navigation.loadEventEnd - navigation.navigationStart,
      };
    });
    
    // Server response should be fast
    expect(ssrMetrics.serverResponseTime).toBeLessThan(800); // <800ms server response
    
    // DOM processing should be efficient
    expect(ssrMetrics.domProcessingTime).toBeLessThan(600); // <600ms DOM processing
    
    // Total render time should meet targets
    expect(ssrMetrics.totalRenderTime).toBeLessThan(2000); // <2s total render
  });

  test('should validate dynamic import performance', async ({ page }) => {
    await page.goto('/analytics');
    
    // Check that analytics components are loaded dynamically
    const scriptTags = await page.locator('script[src*="analytics"]').count();
    expect(scriptTags).toBeGreaterThan(0);
    
    // Measure dynamic import timing
    const dynamicImportTime = await page.evaluate(() => {
      return new Promise((resolve) => {
        const startTime = performance.now();
        
        // Trigger dynamic import (simulate interaction)
        const observer = new MutationObserver(() => {
          const analyticsComponent = document.querySelector('[data-testid="analytics-dashboard"]');
          if (analyticsComponent) {
            const endTime = performance.now();
            observer.disconnect();
            resolve(endTime - startTime);
          }
        });
        
        observer.observe(document.body, {
          childList: true,
          subtree: true,
        });
        
        // Timeout fallback
        setTimeout(() => {
          observer.disconnect();
          resolve(5000); // Max time if component doesn't load
        }, 5000);
      });
    });
    
    // Dynamic imports should load quickly
    expect(dynamicImportTime).toBeLessThan(1000); // <1s for dynamic imports
  });

  test('should validate memory usage', async ({ page }) => {
    await page.goto('/dashboard');
    
    // Get initial memory usage
    const initialMemory = await page.evaluate(() => {
      return (performance as any).memory ? {
        usedJSHeapSize: (performance as any).memory.usedJSHeapSize,
        totalJSHeapSize: (performance as any).memory.totalJSHeapSize,
      } : null;
    });
    
    if (initialMemory) {
      // Navigate to a heavy page
      await page.goto('/analytics');
      await page.waitForTimeout(2000);
      
      const afterNavigationMemory = await page.evaluate(() => {
        return {
          usedJSHeapSize: (performance as any).memory.usedJSHeapSize,
          totalJSHeapSize: (performance as any).memory.totalJSHeapSize,
        };
      });
      
      // Memory usage shouldn't increase dramatically
      const memoryIncrease = afterNavigationMemory.usedJSHeapSize - initialMemory.usedJSHeapSize;
      const memoryIncreasePercentage = (memoryIncrease / initialMemory.usedJSHeapSize) * 100;
      
      expect(memoryIncreasePercentage).toBeLessThan(50); // <50% memory increase
    }
  });

  test('should validate network efficiency', async ({ page }) => {
    const networkRequests: any[] = [];
    
    page.on('response', (response) => {
      networkRequests.push({
        url: response.url(),
        status: response.status(),
        size: response.headers()['content-length'],
        type: response.request().resourceType(),
        cached: response.fromCache(),
      });
    });
    
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    
    // Filter out non-app requests
    const appRequests = networkRequests.filter(req => 
      req.url.includes('localhost') || req.url.includes('epsx')
    );
    
    // Should have reasonable number of requests
    expect(appRequests.length).toBeLessThan(20); // <20 network requests
    
    // Should have good cache hit rate
    const cachedRequests = appRequests.filter(req => req.cached);
    const cacheHitRate = cachedRequests.length / appRequests.length;
    
    if (appRequests.length > 0) {
      expect(cacheHitRate).toBeGreaterThan(0.3); // >30% cache hit rate
    }
    
    // No failed requests
    const failedRequests = appRequests.filter(req => req.status >= 400);
    expect(failedRequests.length).toBe(0);
  });
});

test.describe('Performance Regression Tests', () => {
  test('should not regress from baseline performance', async ({ page }) => {
    // Load baseline metrics if available
    const baselineMetrics = {
      bundleSize: 5 * 1024 * 1024, // 5MB baseline
      pageLoadTime: 3000, // 3s baseline
      fcp: 2500, // 2.5s baseline
      lcp: 4000, // 4s baseline
    };
    
    // Measure current performance
    const startTime = Date.now();
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    const currentLoadTime = Date.now() - startTime;
    
    // Should be better than baseline
    expect(currentLoadTime).toBeLessThan(baselineMetrics.pageLoadTime);
    
    // Validate improvement targets
    const improvementTarget = 0.6; // 40% improvement = 60% of original
    expect(currentLoadTime).toBeLessThan(baselineMetrics.pageLoadTime * improvementTarget);
  });
});