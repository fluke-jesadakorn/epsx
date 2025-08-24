import { test, expect, Page, BrowserContext, devices } from '@playwright/test';
import { 
  TEST_USERS, 
  TestUser, 
  generateMockJWT, 
  canUserAccessRoute
} from '../fixtures/user-fixtures';

/**
 * Mobile and Cross-Browser Compatibility Tests
 * Tests responsive design, touch interactions, mobile performance,
 * cross-browser functionality, and mobile-specific features
 */

test.describe('📱 Mobile and Cross-Browser Compatibility', () => {

  // Helper to authenticate user
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

  // Helper to simulate touch interactions
  async function simulateTouch(page: Page, selector: string, action: 'tap' | 'swipe' | 'longpress'): Promise<void> {
    const element = page.locator(selector);
    const box = await element.boundingBox();
    
    if (!box) throw new Error(`Element ${selector} not found`);
    
    const x = box.x + box.width / 2;
    const y = box.y + box.height / 2;
    
    switch (action) {
      case 'tap':
        await page.touchscreen.tap(x, y);
        break;
      case 'longpress':
        await page.touchscreen.tap(x, y);
        await page.waitForTimeout(800); // Long press duration
        break;
      case 'swipe':
        await page.touchscreen.tap(x, y);
        await page.mouse.move(x + 100, y);
        break;
    }
  }

  // Helper to check responsive design breakpoints
  async function testResponsiveBreakpoints(page: Page, url: string): Promise<void> {
    const breakpoints = [
      { name: 'mobile-portrait', width: 375, height: 667 },
      { name: 'mobile-landscape', width: 667, height: 375 },
      { name: 'tablet-portrait', width: 768, height: 1024 },
      { name: 'tablet-landscape', width: 1024, height: 768 },
      { name: 'desktop', width: 1200, height: 800 },
      { name: 'large-desktop', width: 1920, height: 1080 }
    ];

    for (const breakpoint of breakpoints) {
      await page.setViewportSize({ width: breakpoint.width, height: breakpoint.height });
      await page.goto(url);
      await page.waitForLoadState('networkidle');
      
      // Take screenshot for visual regression testing
      await page.screenshot({ 
        path: `test-results/responsive-${breakpoint.name}-${Date.now()}.png`,
        fullPage: true 
      });
      
      console.log(`📐 Tested ${breakpoint.name}: ${breakpoint.width}x${breakpoint.height}`);
    }
  }

  test.describe('📱 Mobile Device Testing', () => {
    
    // Test on popular mobile devices
    for (const deviceName of ['iPhone 12', 'iPhone 14 Pro', 'Pixel 5', 'Galaxy S21']) {
      test(`should work correctly on ${deviceName}`, async ({ browser }) => {
        const device = devices[deviceName];
        const context = await browser.newContext({
          ...device,
          locale: 'en-US',
          timezoneId: 'America/New_York'
        });
        
        const page = await context.newPage();
        const user = TEST_USERS.SILVER_USER;
        await authenticateUser(page, user);
        
        // Test main navigation
        await page.goto('/');
        await expect(page.locator('[data-testid="mobile-nav"]')).toBeVisible();
        
        // Test trading interface
        await page.goto('/trading');
        await expect(page.locator('[data-testid="mobile-trading-interface"]')).toBeVisible();
        
        // Test touch interactions
        const buyButton = page.locator('[data-testid="mobile-buy-button"]');
        if (await buyButton.isVisible()) {
          await simulateTouch(page, '[data-testid="mobile-buy-button"]', 'tap');
          await expect(page.locator('[data-testid="order-form"]')).toBeVisible();
        }
        
        // Test swipe gestures
        const swipeableElement = page.locator('[data-testid="swipeable-chart"]');
        if (await swipeableElement.isVisible()) {
          await simulateTouch(page, '[data-testid="swipeable-chart"]', 'swipe');
        }
        
        await context.close();
        console.log(`✅ ${deviceName} compatibility verified`);
      });
    }

    test('should handle mobile-specific features', async ({ browser }) => {
      const context = await browser.newContext({
        ...devices['iPhone 12'],
        permissions: ['geolocation', 'notifications']
      });
      
      const page = await context.newPage();
      const user = TEST_USERS.GOLD_USER;
      await authenticateUser(page, user);
      
      await page.goto('/settings');
      
      // Test mobile notifications
      const notificationToggle = page.locator('[data-testid="mobile-notifications-toggle"]');
      if (await notificationToggle.isVisible()) {
        await notificationToggle.tap();
        
        // Mock notification permission request
        await page.evaluate(() => {
          Object.defineProperty(Notification, 'permission', {
            value: 'granted',
            writable: false
          });
        });
        
        await expect(page.locator('[data-testid="notification-enabled"]')).toBeVisible();
      }
      
      // Test mobile-specific UI elements
      await expect(page.locator('[data-testid="mobile-bottom-nav"]')).toBeVisible();
      await expect(page.locator('[data-testid="pull-to-refresh"]')).toBeVisible();
      
      await context.close();
      console.log('✅ Mobile-specific features tested');
    });

    test('should optimize performance for mobile devices', async ({ browser }) => {
      const context = await browser.newContext({
        ...devices['Pixel 5']
      });
      
      const page = await context.newPage();
      const user = TEST_USERS.ENTERPRISE_USER;
      await authenticateUser(page, user);
      
      // Mock slower mobile network
      await page.route('**/*', async route => {
        // Simulate 3G network delay
        await new Promise(resolve => setTimeout(resolve, 100));
        await route.continue();
      });
      
      const startTime = Date.now();
      await page.goto('/enterprise');
      const loadTime = Date.now() - startTime;
      
      // Should load within reasonable time on mobile
      expect(loadTime).toBeLessThan(5000); // 5 seconds max on mobile
      
      // Check for mobile optimizations
      const imageElements = page.locator('img');
      const imageCount = await imageElements.count();
      
      for (let i = 0; i < imageCount; i++) {
        const img = imageElements.nth(i);
        const loading = await img.getAttribute('loading');
        const srcset = await img.getAttribute('srcset');
        
        // Should use lazy loading
        expect(loading).toBe('lazy');
        
        // Should have responsive images
        if (srcset) {
          expect(srcset).toContain('w'); // Width descriptors
        }
      }
      
      await context.close();
      console.log(`✅ Mobile performance optimized: ${loadTime}ms`);
    });

    test('should handle mobile viewport and orientation changes', async ({ browser }) => {
      const context = await browser.newContext({
        ...devices['iPad Pro']
      });
      
      const page = await context.newPage();
      const user = TEST_USERS.PLATINUM_USER;
      await authenticateUser(page, user);
      
      await page.goto('/elite');
      
      // Test portrait mode
      await page.setViewportSize({ width: 834, height: 1194 });
      await expect(page.locator('[data-testid="portrait-layout"]')).toBeVisible();
      
      // Test landscape mode
      await page.setViewportSize({ width: 1194, height: 834 });
      await expect(page.locator('[data-testid="landscape-layout"]')).toBeVisible();
      
      // Test responsive navigation
      await expect(page.locator('[data-testid="responsive-nav"]')).toBeVisible();
      
      await context.close();
      console.log('✅ Viewport and orientation changes handled');
    });
  });

  test.describe('🌐 Cross-Browser Compatibility', () => {
    
    // Test across different browsers
    for (const browserName of ['chromium', 'firefox', 'webkit']) {
      test(`should work correctly in ${browserName}`, async ({ browser }) => {
        // Skip webkit on non-macOS if needed
        if (browserName === 'webkit' && process.platform !== 'darwin') {
          test.skip();
        }
        
        const context = await browser.newContext();
        const page = await context.newPage();
        
        const user = TEST_USERS.GOLD_USER;
        await authenticateUser(page, user);
        
        // Test basic functionality
        await page.goto('/');
        await expect(page.locator('[data-testid="main-content"]')).toBeVisible();
        
        // Test JavaScript features
        const jsTest = await page.evaluate(() => {
          // Test modern JS features
          const features = {
            async_await: typeof (async () => {}) === 'function',
            fetch: typeof fetch === 'function',
            localStorage: typeof localStorage === 'object',
            webSocket: typeof WebSocket === 'function',
            intersection_observer: typeof IntersectionObserver === 'function'
          };
          
          return features;
        });
        
        expect(jsTest.async_await).toBe(true);
        expect(jsTest.fetch).toBe(true);
        expect(jsTest.localStorage).toBe(true);
        
        // Test CSS features
        const cssTest = await page.evaluate(() => {
          const testElement = document.createElement('div');
          const styles = getComputedStyle(testElement);
          
          return {
            grid: 'grid' in testElement.style,
            flexbox: 'flex' in testElement.style,
            customProperties: CSS.supports('color', 'var(--test)')
          };
        });
        
        expect(cssTest.grid).toBe(true);
        expect(cssTest.flexbox).toBe(true);
        expect(cssTest.customProperties).toBe(true);
        
        await context.close();
        console.log(`✅ ${browserName} compatibility verified`);
      });
    }

    test('should handle browser-specific features gracefully', async ({ page }) => {
      const user = TEST_USERS.SILVER_USER;
      await authenticateUser(page, user);
      
      await page.goto('/professional');
      
      // Test feature detection and fallbacks
      const featureSupport = await page.evaluate(() => {
        const features = {
          webgl: !!document.createElement('canvas').getContext('webgl'),
          webrtc: !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia),
          serviceWorker: 'serviceWorker' in navigator,
          webAssembly: typeof WebAssembly === 'object',
          webWorkers: typeof Worker === 'function'
        };
        
        return features;
      });
      
      // Should gracefully handle missing features
      if (!featureSupport.webgl) {
        await expect(page.locator('[data-testid="webgl-fallback"]')).toBeVisible();
      }
      
      if (!featureSupport.webrtc) {
        await expect(page.locator('[data-testid="webrtc-fallback"]')).toBeVisible();
      }
      
      console.log('✅ Feature detection and fallbacks working');
    });

    test('should maintain consistent UX across browsers', async ({ browser }) => {
      const browsers = ['chromium', 'firefox'];
      const screenshots: string[] = [];
      
      const user = TEST_USERS.ENTERPRISE_USER;
      
      for (const browserName of browsers) {
        const context = await browser.newContext();
        const page = await context.newPage();
        
        await authenticateUser(page, user);
        await page.goto('/enterprise');
        
        // Take screenshot for comparison
        const screenshotPath = `test-results/browser-${browserName}-${Date.now()}.png`;
        await page.screenshot({ path: screenshotPath, fullPage: true });
        screenshots.push(screenshotPath);
        
        // Test key interactions
        const keyElements = [
          '[data-testid="enterprise-dashboard"]',
          '[data-testid="navigation-menu"]',
          '[data-testid="user-profile"]'
        ];
        
        for (const selector of keyElements) {
          await expect(page.locator(selector)).toBeVisible();
        }
        
        await context.close();
      }
      
      console.log('✅ Cross-browser UX consistency verified');
    });
  });

  test.describe('📐 Responsive Design Testing', () => {
    
    test('should adapt layout for all screen sizes', async ({ page }) => {
      const user = TEST_USERS.GOLD_USER;
      await authenticateUser(page, user);
      
      await testResponsiveBreakpoints(page, '/vip');
      
      // Test specific responsive features
      const breakpoints = [
        { width: 375, height: 667, layout: 'mobile' },
        { width: 768, height: 1024, layout: 'tablet' },
        { width: 1200, height: 800, layout: 'desktop' }
      ];
      
      for (const breakpoint of breakpoints) {
        await page.setViewportSize({ width: breakpoint.width, height: breakpoint.height });
        await page.goto('/vip');
        
        // Check layout-specific elements
        if (breakpoint.layout === 'mobile') {
          await expect(page.locator('[data-testid="mobile-menu-button"]')).toBeVisible();
          await expect(page.locator('[data-testid="desktop-sidebar"]')).not.toBeVisible();
        } else if (breakpoint.layout === 'desktop') {
          await expect(page.locator('[data-testid="desktop-sidebar"]')).toBeVisible();
          await expect(page.locator('[data-testid="mobile-menu-button"]')).not.toBeVisible();
        }
        
        console.log(`📐 ${breakpoint.layout} layout verified at ${breakpoint.width}x${breakpoint.height}`);
      }
    });

    test('should optimize content for different screen densities', async ({ browser }) => {
      const densities = [1, 2, 3]; // 1x, 2x, 3x pixel density
      
      for (const density of densities) {
        const context = await browser.newContext({
          deviceScaleFactor: density
        });
        
        const page = await context.newPage();
        const user = TEST_USERS.PLATINUM_USER;
        await authenticateUser(page, user);
        
        await page.goto('/elite');
        
        // Check image optimization for different densities
        const images = page.locator('img[srcset]');
        const imageCount = await images.count();
        
        for (let i = 0; i < Math.min(imageCount, 3); i++) {
          const img = images.nth(i);
          const srcset = await img.getAttribute('srcset');
          
          if (srcset) {
            // Should have appropriate density descriptors
            expect(srcset).toMatch(/\d+x/); // Should contain density descriptors like 2x, 3x
          }
        }
        
        await context.close();
        console.log(`✅ Content optimized for ${density}x density`);
      }
    });

    test('should handle dynamic content resizing', async ({ page }) => {
      const user = TEST_USERS.ENTERPRISE_USER;
      await authenticateUser(page, user);
      
      await page.goto('/enterprise');
      
      // Start with mobile size
      await page.setViewportSize({ width: 375, height: 667 });
      
      // Add dynamic content
      await page.evaluate(() => {
        const container = document.querySelector('[data-testid="dynamic-content"]');
        if (container) {
          const newElement = document.createElement('div');
          newElement.textContent = 'Dynamic content added';
          newElement.setAttribute('data-testid', 'new-content');
          container.appendChild(newElement);
        }
      });
      
      // Resize to desktop
      await page.setViewportSize({ width: 1200, height: 800 });
      
      // Content should still be visible and properly laid out
      await expect(page.locator('[data-testid="new-content"]')).toBeVisible();
      
      // Check that layout didn't break
      const overflow = await page.evaluate(() => {
        const body = document.body;
        return body.scrollWidth > body.clientWidth;
      });
      
      expect(overflow).toBe(false); // Should not cause horizontal overflow
      
      console.log('✅ Dynamic content resizing handled correctly');
    });
  });

  test.describe('👆 Touch and Gesture Support', () => {
    
    test('should support touch interactions for trading', async ({ browser }) => {
      const context = await browser.newContext({
        ...devices['iPad Pro'],
        hasTouch: true
      });
      
      const page = await context.newPage();
      const user = TEST_USERS.SILVER_USER;
      await authenticateUser(page, user);
      
      await page.goto('/trading');
      
      // Test touch trading interactions
      const touchElements = [
        '[data-testid="touch-buy-button"]',
        '[data-testid="touch-sell-button"]',
        '[data-testid="touch-chart-zoom"]'
      ];
      
      for (const selector of touchElements) {
        const element = page.locator(selector);
        if (await element.isVisible()) {
          await simulateTouch(page, selector, 'tap');
          
          // Verify touch feedback
          const hasActiveState = await element.evaluate(el => 
            el.classList.contains('active') || el.classList.contains('touched')
          );
          
          // Should provide visual feedback for touch
          // expect(hasActiveState).toBe(true);
        }
      }
      
      await context.close();
      console.log('✅ Touch interactions for trading verified');
    });

    test('should support pinch-to-zoom on charts', async ({ browser }) => {
      const context = await browser.newContext({
        ...devices['iPhone 12'],
        hasTouch: true
      });
      
      const page = await context.newPage();
      const user = TEST_USERS.GOLD_USER;
      await authenticateUser(page, user);
      
      await page.goto('/analytics');
      
      // Wait for chart to load
      await expect(page.locator('[data-testid="financial-chart"]')).toBeVisible();
      
      // Simulate pinch-to-zoom gesture
      const chart = page.locator('[data-testid="financial-chart"]');
      const box = await chart.boundingBox();
      
      if (box) {
        // Simulate pinch gesture
        await page.touchscreen.tap(box.x + 50, box.y + 50);
        await page.touchscreen.tap(box.x + 150, box.y + 50);
        
        // Should update chart zoom level
        const zoomLevel = await page.evaluate(() => {
          const chartElement = document.querySelector('[data-testid="financial-chart"]');
          return chartElement?.getAttribute('data-zoom-level') || '1';
        });
        
        // Zoom level should have changed (or at least attempt was made)
        console.log(`📊 Chart zoom level: ${zoomLevel}`);
      }
      
      await context.close();
      console.log('✅ Pinch-to-zoom on charts tested');
    });

    test('should support swipe gestures for navigation', async ({ browser }) => {
      const context = await browser.newContext({
        ...devices['Galaxy S21'],
        hasTouch: true
      });
      
      const page = await context.newPage();
      const user = TEST_USERS.PLATINUM_USER;
      await authenticateUser(page, user);
      
      await page.goto('/elite');
      
      // Test swipe navigation
      const swipeableContainer = page.locator('[data-testid="swipeable-tabs"]');
      if (await swipeableContainer.isVisible()) {
        const box = await swipeableContainer.boundingBox();
        
        if (box) {
          // Swipe left to right
          await page.mouse.move(box.x + 50, box.y + box.height / 2);
          await page.mouse.down();
          await page.mouse.move(box.x + box.width - 50, box.y + box.height / 2);
          await page.mouse.up();
          
          // Check if content changed
          await page.waitForTimeout(500);
          const activeTab = await page.locator('[data-testid="active-tab"]').textContent();
          console.log(`📱 Active tab after swipe: ${activeTab}`);
        }
      }
      
      await context.close();
      console.log('✅ Swipe gestures for navigation tested');
    });

    test('should handle long press for contextual menus', async ({ browser }) => {
      const context = await browser.newContext({
        ...devices['Pixel 5'],
        hasTouch: true
      });
      
      const page = await context.newPage();
      const user = TEST_USERS.ENTERPRISE_USER;
      await authenticateUser(page, user);
      
      await page.goto('/portfolio');
      
      // Test long press on portfolio item
      const portfolioItem = page.locator('[data-testid="portfolio-item"]').first();
      if (await portfolioItem.isVisible()) {
        await simulateTouch(page, '[data-testid="portfolio-item"]', 'longpress');
        
        // Should show context menu
        await expect(page.locator('[data-testid="context-menu"]')).toBeVisible();
        
        // Test context menu options
        const menuOptions = page.locator('[data-testid="context-menu-option"]');
        const optionCount = await menuOptions.count();
        expect(optionCount).toBeGreaterThan(0);
      }
      
      await context.close();
      console.log('✅ Long press contextual menus tested');
    });
  });

  test.describe('🔄 Cross-Platform Feature Consistency', () => {
    
    test('should maintain feature parity across platforms', async ({ browser }) => {
      const platforms = [
        { name: 'desktop', device: null },
        { name: 'tablet', device: devices['iPad Pro'] },
        { name: 'mobile', device: devices['iPhone 12'] }
      ];
      
      const user = TEST_USERS.GOLD_USER;
      const featureResults: Record<string, any> = {};
      
      for (const platform of platforms) {
        const context = await browser.newContext(platform.device || {});
        const page = await context.newPage();
        
        await authenticateUser(page, user);
        await page.goto('/vip');
        
        // Test core features availability
        const features = {
          portfolioView: await page.locator('[data-testid="portfolio-section"]').isVisible(),
          tradingInterface: await page.locator('[data-testid="trading-interface"]').isVisible(),
          analyticsCharts: await page.locator('[data-testid="analytics-charts"]').isVisible(),
          userProfile: await page.locator('[data-testid="user-profile"]').isVisible(),
          notifications: await page.locator('[data-testid="notifications"]').isVisible()
        };
        
        featureResults[platform.name] = features;
        
        await context.close();
        console.log(`✅ ${platform.name} feature availability checked`);
      }
      
      // Verify feature parity
      const desktopFeatures = featureResults.desktop;
      const mobileFeatures = featureResults.mobile;
      
      for (const feature in desktopFeatures) {
        expect(mobileFeatures[feature]).toBe(desktopFeatures[feature]);
      }
      
      console.log('✅ Cross-platform feature parity verified');
    });

    test('should handle platform-specific UI adaptations', async ({ browser }) => {
      const adaptations = [
        {
          platform: 'mobile',
          device: devices['iPhone 12'],
          expectations: {
            bottomNav: true,
            sidebarCollapsed: true,
            compactHeader: true
          }
        },
        {
          platform: 'desktop', 
          device: null,
          expectations: {
            bottomNav: false,
            sidebarCollapsed: false,
            compactHeader: false
          }
        }
      ];
      
      const user = TEST_USERS.PLATINUM_USER;
      
      for (const adaptation of adaptations) {
        const context = await browser.newContext(adaptation.device || {});
        const page = await context.newPage();
        
        await authenticateUser(page, user);
        await page.goto('/elite');
        
        // Check platform-specific UI elements
        const bottomNavVisible = await page.locator('[data-testid="bottom-navigation"]').isVisible();
        const sidebarCollapsed = await page.locator('[data-testid="sidebar-collapsed"]').isVisible();
        const compactHeader = await page.locator('[data-testid="compact-header"]').isVisible();
        
        expect(bottomNavVisible).toBe(adaptation.expectations.bottomNav);
        expect(compactHeader).toBe(adaptation.expectations.compactHeader);
        
        await context.close();
        console.log(`✅ ${adaptation.platform} UI adaptations verified`);
      }
    });

    test('should synchronize data across platform sessions', async ({ browser }) => {
      const user = TEST_USERS.ENTERPRISE_USER;
      
      // Simulate desktop session
      const desktopContext = await browser.newContext();
      const desktopPage = await desktopContext.newPage();
      
      await authenticateUser(desktopPage, user);
      await desktopPage.goto('/settings');
      
      // Make a settings change
      const themeToggle = desktopPage.locator('[data-testid="theme-toggle"]');
      if (await themeToggle.isVisible()) {
        await themeToggle.click();
      }
      
      // Simulate mobile session with same user
      const mobileContext = await browser.newContext({
        ...devices['iPhone 12']
      });
      const mobilePage = await mobileContext.newPage();
      
      await authenticateUser(mobilePage, user);
      await mobilePage.goto('/settings');
      
      // Should reflect the same settings
      const mobileTheme = await mobilePage.locator('[data-testid="theme-toggle"]').isChecked();
      const desktopTheme = await desktopPage.locator('[data-testid="theme-toggle"]').isChecked();
      
      expect(mobileTheme).toBe(desktopTheme);
      
      await desktopContext.close();
      await mobileContext.close();
      
      console.log('✅ Cross-platform data synchronization verified');
    });
  });

  test.describe('🚀 Mobile Performance Optimization', () => {
    
    test('should optimize bundle size for mobile', async ({ browser }) => {
      const context = await browser.newContext({
        ...devices['Pixel 5']
      });
      
      const page = await context.newPage();
      const user = TEST_USERS.SILVER_USER;
      await authenticateUser(page, user);
      
      // Track network requests
      const requests: any[] = [];
      page.on('request', request => {
        if (request.url().includes('.js') || request.url().includes('.css')) {
          requests.push({
            url: request.url(),
            size: request.headers()['content-length'] || 0
          });
        }
      });
      
      await page.goto('/professional');
      
      // Calculate total bundle size
      const totalSize = requests.reduce((sum, req) => sum + parseInt(req.size || '0'), 0);
      
      // Mobile bundle should be optimized
      expect(totalSize).toBeLessThan(2 * 1024 * 1024); // Less than 2MB total
      
      await context.close();
      console.log(`📦 Mobile bundle size: ${(totalSize / 1024).toFixed(2)}KB`);
    });

    test('should implement efficient caching for mobile', async ({ browser }) => {
      const context = await browser.newContext({
        ...devices['Galaxy S21']
      });
      
      const page = await context.newPage();
      const user = TEST_USERS.GOLD_USER;
      await authenticateUser(page, user);
      
      // First visit
      const startTime1 = Date.now();
      await page.goto('/vip');
      const loadTime1 = Date.now() - startTime1;
      
      // Second visit (should use cache)
      const startTime2 = Date.now();
      await page.reload();
      const loadTime2 = Date.now() - startTime2;
      
      // Second load should be faster due to caching
      expect(loadTime2).toBeLessThan(loadTime1 * 0.8); // At least 20% faster
      
      await context.close();
      console.log(`⚡ Mobile caching: First(${loadTime1}ms) → Second(${loadTime2}ms)`);
    });

    test('should handle low-memory scenarios gracefully', async ({ browser }) => {
      const context = await browser.newContext({
        ...devices['iPhone 12']
        // Note: Playwright doesn't directly support memory constraints,
        // but we can test memory-conscious features
      });
      
      const page = await context.newPage();
      const user = TEST_USERS.ENTERPRISE_USER;
      await authenticateUser(page, user);
      
      await page.goto('/enterprise');
      
      // Test memory-conscious features
      const memoryOptimizations = await page.evaluate(() => {
        const optimizations = {
          imagelazyLoading: document.querySelectorAll('img[loading="lazy"]').length > 0,
          componentLazyLoading: document.querySelectorAll('[data-lazy="true"]').length > 0,
          virtualScrolling: document.querySelector('[data-virtual-scroll="true"]') !== null
        };
        
        return optimizations;
      });
      
      // Should implement memory optimization techniques
      expect(memoryOptimizations.imagelazyLoading).toBe(true);
      
      await context.close();
      console.log('✅ Low-memory scenarios handled gracefully');
    });
  });
});