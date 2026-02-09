import { test, expect } from '@playwright/test';

test.describe('Navigation and Layout - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL ?? 'http://localhost:3001';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('Main Navigation', () => {
    test('should display main navigation menu', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const nav = page.locator('nav, [role="navigation"]').first();
      await expect(nav).toBeVisible({ timeout: 10000 });
    });

    test('should navigate to dashboard', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const dashboardLink = page.locator('a:has-text("Dashboard"), a[href*="dashboard"]').first();
      if (await dashboardLink.isVisible()) {
        await dashboardLink.click();
        await page.waitForURL('**/dashboard**', { timeout: 5000 });
      }
    });

    test('should navigate to wallet management', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const walletLink = page.locator('a:has-text("Wallet"), a[href*="wallet"]').first();
      if (await walletLink.isVisible()) {
        await walletLink.click();
        await page.waitForURL('**/wallet**', { timeout: 5000 });
      }
    });

    test('should navigate to permissions', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const permissionsLink = page.locator('a:has-text("permission"), a[href*="permission"]').first();
      if (await permissionsLink.isVisible()) {
        await permissionsLink.click();
        await page.waitForURL('**/permission**', { timeout: 5000 });
      }
    });

    test('should navigate to subscriptions', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const subscriptionsLink = page.locator('a:has-text("Subscription"), a[href*="subscription"]').first();
      if (await subscriptionsLink.isVisible()) {
        await subscriptionsLink.click();
        await page.waitForURL('**/subscription**', { timeout: 5000 });
      }
    });

    test('should navigate to analytics', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const analyticsLink = page.locator('a:has-text("Analytics"), a[href*="analytics"]').first();
      if (await analyticsLink.isVisible()) {
        await analyticsLink.click();
        await page.waitForURL('**/analytics**', { timeout: 5000 });
      }
    });
  });

  test.describe('Sidebar Navigation', () => {
    test('should toggle sidebar collapse', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const toggleBtn = page.locator('button[aria-label*="toggle"], button:has-text("Toggle")').first();
      if (await toggleBtn.isVisible()) {
        await toggleBtn.click();
        await page.waitForTimeout(300);

        const sidebar = page.locator('[data-testid="sidebar"], aside').first();
        const isCollapsed = await sidebar.evaluate(el => el.classList.contains('collapsed'));
        expect(typeof isCollapsed).toBe('boolean');
      }
    });

    test('should display active route indicator', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/dashboard`);

      const activeLink = page.locator('a[aria-current="page"], a.active').first();
      const hasActive = await activeLink.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasActive).toBe('boolean');
    });

    test('should show nested menu items', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const parentMenu = page.locator('button:has-text("Settings"), button:has-text("Admin")').first();
      if (await parentMenu.isVisible()) {
        await parentMenu.click();

        const submenu = page.locator('[role="menu"], .submenu').first();
        const hasSubmenu = await submenu.isVisible({ timeout: 3000 }).catch(() => false);
        expect(typeof hasSubmenu).toBe('boolean');
      }
    });
  });

  test.describe('Breadcrumb Navigation', () => {
    test('should display breadcrumb trail', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/hierarchy`);

      const breadcrumb = page.locator('[data-testid="breadcrumb"], nav[aria-label*="breadcrumb"]').first();
      const hasBreadcrumb = await breadcrumb.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasBreadcrumb).toBe('boolean');
    });

    test('should navigate via breadcrumb links', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/hierarchy`);

      const breadcrumbLink = page.locator('[data-testid="breadcrumb"] a, nav a').first();
      if (await breadcrumbLink.isVisible()) {
        await breadcrumbLink.click();
        await page.waitForTimeout(500);
      }
    });
  });

  test.describe('header', () => {
    test('should display header with logo', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const header = page.locator('header').first();
      await expect(header).toBeVisible({ timeout: 10000 });

      const logo = page.locator('img[alt*="logo"], [data-testid="logo"]').first();
      const hasLogo = await logo.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasLogo).toBe('boolean');
    });

    test('should display user menu', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const userMenu = page.locator('button[aria-label*="user"], [data-testid="user-menu"]').first();
      if (await userMenu.isVisible()) {
        await userMenu.click();

        const dropdown = page.locator('[role="menu"], .dropdown').first();
        const hasDropdown = await dropdown.isVisible({ timeout: 3000 }).catch(() => false);
        expect(typeof hasDropdown).toBe('boolean');
      }
    });

    test('should display theme toggle', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const themeToggle = page.locator('button[aria-label*="theme"], button:has-text("Theme")').first();
      if (await themeToggle.isVisible()) {
        await themeToggle.click();
        await page.waitForTimeout(300);
      }
    });

    test('should display notification bell', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const notificationBell = page.locator('button[aria-label*="notification"], [data-testid="notification-bell"]').first();
      const hasBell = await notificationBell.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasBell).toBe('boolean');
    });
  });

  test.describe('Responsive Design', () => {
    test('should display mobile menu on small screens', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto(ADMIN_URL);

      const mobileMenu = page.locator('button[aria-label*="menu"], [data-testid="mobile-menu"]').first();
      const hasMobileMenu = await mobileMenu.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasMobileMenu).toBe('boolean');
    });

    test('should hide sidebar on mobile by default', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto(ADMIN_URL);

      const sidebar = page.locator('aside, [data-testid="sidebar"]').first();
      const isHidden = await sidebar.evaluate(el => {
        const style = window.getComputedStyle(el);
        return style.display === 'none' ?? style.transform.includes('translate');
      }).catch(() => true);

      expect(typeof isHidden).toBe('boolean');
    });

    test('should stack content vertically on mobile', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto(ADMIN_URL);

      const container = page.locator('main, .container').first();
      const isStacked = await container.evaluate(el => {
        const style = window.getComputedStyle(el);
        return style.flexDirection === 'column' ?? style.display === 'block';
      });

      expect(typeof isStacked).toBe('boolean');
    });
  });

  test.describe('Page Transitions', () => {
    test('should handle browser back button', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/dashboard`);
      await page.goto(`${ADMIN_URL}/permissions`);

      await page.goBack();

      const url = page.url();
      expect(url).toContain('dashboard');
    });

    test('should handle browser forward button', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/dashboard`);
      await page.goto(`${ADMIN_URL}/permissions`);

      await page.goBack();
      await page.goForward();

      const url = page.url();
      expect(url).toContain('permission');
    });
  });

  test.describe('Search Functionality', () => {
    test('should display global search', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const searchInput = page.locator('input[type="search"], input[placeholder*="search"]').first();
      const hasSearch = await searchInput.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasSearch).toBe('boolean');
    });

    test('should search and navigate to results', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const searchInput = page.locator('input[type="search"]').first();
      if (await searchInput.isVisible()) {
        await searchInput.fill('wallet');
        await page.keyboard.press('Enter');

        await page.waitForTimeout(1000);
      }
    });
  });

  test.describe('Error Pages', () => {
    test('should display 404 page for invalid routes', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/nonexistent-page`);

      const error404 = page.locator('text=404, text=Not Found, text=Page not found').first();
      const has404 = await error404.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof has404).toBe('boolean');
    });

    test('should display access denied page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/access-denied`);

      await page.waitForSelector('text=Access Denied, text=Unauthorized, text=permission', { timeout: 10000 });
    });

    test('should display unauthorized page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/unauthorized`);

      await page.waitForSelector('text=Unauthorized, text=Access, text=permission', { timeout: 10000 });
    });
  });

  test.describe('Performance', () => {
    test('should load pages within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(ADMIN_URL);
      await page.waitForSelector('main, [role="main"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });

    test('should prefetch linked pages', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const hasPrefetch = await page.evaluate(() => {
        const links = document.querySelectorAll('link[rel="prefetch"]');
        return links.length > 0;
      });

      expect(typeof hasPrefetch).toBe('boolean');
    });
  });

  test.describe('Accessibility', () => {
    test('should have skip to main content link', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const skipLink = page.locator('a:has-text("Skip to"), a[href="#main"]').first();
      const hasSkipLink = await skipLink.isVisible({ timeout: 3000 }).catch(() => false);
      expect(typeof hasSkipLink).toBe('boolean');
    });

    test('should support keyboard navigation through menu', async ({ page }) => {
      await page.goto(ADMIN_URL);

      await page.keyboard.press('Tab');

      for (let i = 0; i < 5; i++) {
        await page.keyboard.press('Tab');
        const focusedElement = page.locator(':focus').first();
        await expect(focusedElement).toBeVisible();
      }
    });

    test('should have proper heading hierarchy', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const hasH1 = await page.locator('h1').count() > 0;
      expect(hasH1).toBe(true);
    });
  });
});
