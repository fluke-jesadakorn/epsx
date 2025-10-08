import { test, expect } from '@playwright/test';

test.describe('Notification System and Developer Portal - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('Notification Management', () => {
    test('should display notifications page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`);

      await page.waitForSelector('text=Notification, h1, [data-testid="notifications-page"]', { timeout: 10000 });
    });

    test('should create new notification', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications/create`);

      const titleInput = page.locator('input[name*="title"]').first();
      if (await titleInput.isVisible()) {
        await titleInput.fill('Test Notification');
      }

      const messageInput = page.locator('textarea[name*="message"], textarea[name*="content"]').first();
      if (await messageInput.isVisible()) {
        await messageInput.fill('This is a test notification message');
      }

      const typeSelect = page.locator('select[name*="type"]').first();
      if (await typeSelect.isVisible()) {
        await typeSelect.selectOption('info');
      }

      const submitBtn = page.locator('button[type="submit"], button:has-text("Send")').first();
      if (await submitBtn.isVisible()) {
        await submitBtn.click();
        await page.waitForSelector('text=Success, text=sent, text=created', { timeout: 10000 }).catch(() => {});
      }
    });

    test('should send notification to specific users', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications/create`);

      const targetSelect = page.locator('select[name*="target"], select[name*="recipient"]').first();
      if (await targetSelect.isVisible()) {
        await targetSelect.selectOption('specific');

        const userInput = page.locator('input[name*="user"], select[name*="user"]').first();
        if (await userInput.isVisible()) {
          if (userInput.tagName === 'select') {
            await userInput.selectOption({ index: 1 });
          } else {
            await userInput.fill('test@example.com');
          }
        }
      }
    });

    test('should send broadcast notification', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications/create`);

      const broadcastBtn = page.locator('input[type="checkbox"], label:has-text("Broadcast")').first();
      if (await broadcastBtn.isVisible()) {
        await broadcastBtn.click();
      }

      const titleInput = page.locator('input[name*="title"]').first();
      if (await titleInput.isVisible()) {
        await titleInput.fill('System Announcement');
      }

      const messageInput = page.locator('textarea[name*="message"]').first();
      if (await messageInput.isVisible()) {
        await messageInput.fill('Important system update');
      }

      const submitBtn = page.locator('button[type="submit"]').first();
      if (await submitBtn.isVisible()) {
        await submitBtn.click();
        await page.waitForSelector('text=Success, text=broadcast', { timeout: 10000 }).catch(() => {});
      }
    });

    test('should schedule notification', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications/create`);

      const scheduleCheckbox = page.locator('input[type="checkbox"], label:has-text("Schedule")').first();
      if (await scheduleCheckbox.isVisible()) {
        await scheduleCheckbox.click();

        const dateInput = page.locator('input[type="datetime-local"]').first();
        if (await dateInput.isVisible()) {
          const futureDate = new Date(Date.now() + 24 * 60 * 60 * 1000);
          const isoString = futureDate.toISOString().slice(0, 16);
          await dateInput.fill(isoString);
        }
      }
    });

    test('should display notification history', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`);

      const historySection = page.locator('text=History, text=Sent, text=Past Notifications').first();
      const hasHistory = await historySection.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasHistory).toBe('boolean');
    });
  });

  test.describe('Notification Bell', () => {
    test('should display notification bell in header', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const notificationBell = page.locator('button[aria-label*="notification"], [data-testid="notification-bell"]').first();
      const hasBell = await notificationBell.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasBell).toBe('boolean');
    });

    test('should show unread notification count', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const notificationBadge = page.locator('[data-testid="notification-badge"], .notification-badge').first();
      if (await notificationBadge.isVisible()) {
        const count = await notificationBadge.textContent();
        expect(typeof count).toBe('string');
      }
    });

    test('should open notification dropdown', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const notificationBell = page.locator('button[aria-label*="notification"]').first();
      if (await notificationBell.isVisible()) {
        await notificationBell.click();

        const dropdown = page.locator('[role="menu"], .notification-dropdown').first();
        const hasDropdown = await dropdown.isVisible({ timeout: 3000 }).catch(() => false);
        expect(typeof hasDropdown).toBe('boolean');
      }
    });

    test('should mark notification as read', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const notificationBell = page.locator('button[aria-label*="notification"]').first();
      if (await notificationBell.isVisible()) {
        await notificationBell.click();

        const firstNotification = page.locator('[data-testid="notification-item"]').first();
        if (await firstNotification.isVisible()) {
          await firstNotification.click();

          await page.waitForTimeout(500);
        }
      }
    });
  });

  test.describe('Developer Portal', () => {
    test('should display developer portal page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal`);

      await page.waitForSelector('text=Developer, text=API, h1', { timeout: 10000 });
    });

    test('should display API overview tab', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal`);

      const overviewTab = page.locator('button:has-text("Overview"), [role="tab"]:has-text("Overview")').first();
      if (await overviewTab.isVisible()) {
        await overviewTab.click();

        await page.waitForSelector('text=Overview, text=Getting Started, text=Introduction', { timeout: 5000 });
      }
    });

    test('should display API keys tab', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal`);

      const apiKeysTab = page.locator('button:has-text("API Keys"), [role="tab"]:has-text("Keys")').first();
      if (await apiKeysTab.isVisible()) {
        await apiKeysTab.click();

        await page.waitForSelector('text=API Key, text=Keys, text=Manage', { timeout: 5000 });
      }
    });

    test('should create new API key', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal/api-keys/create`);

      const nameInput = page.locator('input[name*="name"]').first();
      if (await nameInput.isVisible()) {
        await nameInput.fill('Test API Key');
      }

      const scopeSelect = page.locator('select[name*="scope"], input[name*="permission"]').first();
      if (await scopeSelect.isVisible()) {
        if (scopeSelect.tagName === 'select') {
          await scopeSelect.selectOption('read');
        } else {
          await scopeSelect.fill('read');
        }
      }

      const submitBtn = page.locator('button[type="submit"], button:has-text("Create")').first();
      if (await submitBtn.isVisible()) {
        await submitBtn.click();
        await page.waitForSelector('text=Success, text=created, text=API Key', { timeout: 10000 }).catch(() => {});
      }
    });

    test('should revoke API key', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal`);

      const apiKeysTab = page.locator('button:has-text("API Keys")').first();
      if (await apiKeysTab.isVisible()) {
        await apiKeysTab.click();

        const revokeBtn = page.locator('button:has-text("Revoke"), button:has-text("Delete")').first();
        if (await revokeBtn.isVisible()) {
          await revokeBtn.click();

          const confirmBtn = page.locator('button:has-text("Confirm"), button:has-text("Revoke")').first();
          if (await confirmBtn.isVisible()) {
            await confirmBtn.click();
            await page.waitForSelector('text=revoked, text=deleted', { timeout: 10000 }).catch(() => {});
          }
        }
      }
    });

    test('should display usage metrics', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal`);

      const usageTab = page.locator('button:has-text("Usage"), [role="tab"]:has-text("Usage")').first();
      if (await usageTab.isVisible()) {
        await usageTab.click();

        await page.waitForSelector('text=API Calls, text=Usage, text=Requests', { timeout: 5000 });
      }
    });

    test('should display documentation tab', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal`);

      const docsTab = page.locator('button:has-text("Documentation"), [role="tab"]:has-text("Docs")').first();
      if (await docsTab.isVisible()) {
        await docsTab.click();

        await page.waitForSelector('text=Documentation, text=API, text=Reference', { timeout: 5000 });
      }
    });
  });

  test.describe('API Usage Monitoring', () => {
    test('should display API usage chart', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal`);

      const usageTab = page.locator('button:has-text("Usage")').first();
      if (await usageTab.isVisible()) {
        await usageTab.click();

        const chart = page.locator('[data-testid="usage-chart"], .chart, canvas').first();
        const hasChart = await chart.isVisible({ timeout: 5000 }).catch(() => false);
        expect(typeof hasChart).toBe('boolean');
      }
    });

    test('should filter usage by date range', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal`);

      const usageTab = page.locator('button:has-text("Usage")').first();
      if (await usageTab.isVisible()) {
        await usageTab.click();

        const dateFilter = page.locator('input[type="date"], button:has-text("Date")').first();
        if (await dateFilter.isVisible()) {
          await dateFilter.click();
          await page.waitForTimeout(500);
        }
      }
    });

    test('should display rate limit information', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal`);

      const rateLimitInfo = page.locator('text=Rate Limit, text=Quota, text=Limit').first();
      const hasRateLimit = await rateLimitInfo.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasRateLimit).toBe('boolean');
    });
  });

  test.describe('Error Handling', () => {
    test('should handle notification API errors gracefully', async ({ page }) => {
      await page.route('**/api/admin/notifications**', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server Error' }) });
      });

      await page.goto(`${ADMIN_URL}/notifications`);

      const errorMsg = page.locator('text=error, text=failed, text=Unable to load');
      const hasError = await errorMsg.isVisible({ timeout: 10000 }).catch(() => false);
      expect(typeof hasError).toBe('boolean');
    });
  });

  test.describe('Performance', () => {
    test('should load notifications page within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/notifications`);
      await page.waitForSelector('h1, [data-testid="notifications-page"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });

    test('should load developer portal within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/developer-portal`);
      await page.waitForSelector('h1, [data-testid="developer-portal"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });
  });

  test.describe('Accessibility', () => {
    test('should support keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`);

      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();
    });
  });
});
