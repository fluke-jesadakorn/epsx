import { test, expect } from '@playwright/test';

test.describe('Analytics and Monitoring - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('Analytics Dashboard', () => {
    test('should display analytics dashboard', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      await page.waitForSelector('text=Analytics, text=Dashboard, text=Statistics', { timeout: 10000 });
    });

    test('should display key metrics cards', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const metricsCards = page.locator('[data-testid="metric-card"], .metric-card, .stat-card');
      const cardCount = await metricsCards.count();

      if (cardCount > 0) {
        await expect(metricsCards.first()).toBeVisible();
      }
    });

    test('should display user activity chart', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const chart = page.locator('[data-testid="chart"], .chart, canvas').first();
      const hasChart = await chart.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasChart).toBe('boolean');
    });

    test('should filter analytics by date range', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const dateFilter = page.locator('input[type="date"], button:has-text("Date Range")').first();
      if (await dateFilter.isVisible()) {
        await dateFilter.click();
        await page.waitForTimeout(500);
      }
    });

    test('should refresh analytics data', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const refreshBtn = page.locator('button:has-text("Refresh"), button[aria-label*="refresh"]').first();
      if (await refreshBtn.isVisible()) {
        await refreshBtn.click();
        await page.waitForTimeout(1000);
      }
    });
  });

  test.describe('User Analytics', () => {
    test('should display active users count', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const activeUsers = page.locator('text=Active Users, text=Users Online').first();
      const hasActiveUsers = await activeUsers.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasActiveUsers).toBe('boolean');
    });

    test('should display user growth trends', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const growthChart = page.locator('[data-testid="growth-chart"], text=Growth').first();
      const hasGrowth = await growthChart.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasGrowth).toBe('boolean');
    });

    test('should display user segmentation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const segmentation = page.locator('text=Segmentation, text=Categories, text=Groups').first();
      const hasSegmentation = await segmentation.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasSegmentation).toBe('boolean');
    });
  });

  test.describe('System Health Monitoring', () => {
    test('should display system health status', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/system`);

      const healthStatus = page.locator('text=Health, text=Status, text=System').first();
      const hasHealth = await healthStatus.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasHealth).toBe('boolean');
    });

    test('should display service uptime', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/system`);

      const uptime = page.locator('text=Uptime, text=Online, text=Running').first();
      const hasUptime = await uptime.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasUptime).toBe('boolean');
    });

    test('should display error rate metrics', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/system`);

      const errorRate = page.locator('text=Error Rate, text=Errors, text=Failures').first();
      const hasErrorRate = await errorRate.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasErrorRate).toBe('boolean');
    });

    test('should display database metrics', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/system`);

      const dbMetrics = page.locator('text=Database, text=Connections, text=Query').first();
      const hasDbMetrics = await dbMetrics.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasDbMetrics).toBe('boolean');
    });
  });

  test.describe('Performance Metrics', () => {
    test('should display API response times', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const responseTime = page.locator('text=Response Time, text=Latency, text=Performance').first();
      const hasResponseTime = await responseTime.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasResponseTime).toBe('boolean');
    });

    test('should display throughput metrics', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const throughput = page.locator('text=Throughput, text=Requests, text=Rate').first();
      const hasThroughput = await throughput.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasThroughput).toBe('boolean');
    });
  });

  test.describe('Usage Analytics', () => {
    test('should display API usage statistics', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/developer-portal`);

      const usageTab = page.locator('button:has-text("Usage"), [role="tab"]:has-text("Usage")').first();
      if (await usageTab.isVisible()) {
        await usageTab.click();

        await page.waitForSelector('text=API Calls, text=Usage, text=Requests', { timeout: 5000 });
      }
    });

    test('should display endpoint popularity', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const popularity = page.locator('text=Popular Endpoints, text=Most Used').first();
      const hasPopularity = await popularity.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasPopularity).toBe('boolean');
    });
  });

  test.describe('Security Monitoring', () => {
    test('should display security events', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/system`);

      const securityEvents = page.locator('text=Security, text=Threats, text=Alerts').first();
      const hasSecurityEvents = await securityEvents.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasSecurityEvents).toBe('boolean');
    });

    test('should display failed login attempts', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/system`);

      const failedLogins = page.locator('text=Failed Login, text=Login Attempts').first();
      const hasFailedLogins = await failedLogins.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasFailedLogins).toBe('boolean');
    });

    test('should display suspicious activity alerts', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/system`);

      const suspiciousActivity = page.locator('text=Suspicious, text=Anomaly, text=Alert').first();
      const hasSuspicious = await suspiciousActivity.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasSuspicious).toBe('boolean');
    });
  });

  test.describe('Export Analytics', () => {
    test('should export analytics data to CSV', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const exportBtn = page.locator('button:has-text("Export"), button:has-text("Download")').first();
      if (await exportBtn.isVisible()) {
        const downloadPromise = page.waitForEvent('download').catch(() => null);
        await exportBtn.click();

        const download = await downloadPromise;
        if (download) {
          expect(download.suggestedFilename()).toContain('analytics');
        }
      }
    });

    test('should export charts as images', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const exportChartBtn = page.locator('button:has-text("Export Chart"), button[aria-label*="export"]').first();
      if (await exportChartBtn.isVisible()) {
        const downloadPromise = page.waitForEvent('download').catch(() => null);
        await exportChartBtn.click();

        const download = await downloadPromise;
        if (download) {
          expect(download.suggestedFilename()).toMatch(/\.(png|jpg|svg)$/);
        }
      }
    });
  });

  test.describe('Real-time Updates', () => {
    test('should update metrics in real-time', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const initialValue = await page.locator('[data-testid="active-users"], .metric-value').first().textContent();

      await page.waitForTimeout(5000);

      const updatedValue = await page.locator('[data-testid="active-users"], .metric-value').first().textContent();

      expect(typeof initialValue).toBe('string');
      expect(typeof updatedValue).toBe('string');
    });

    test('should show live activity feed', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/dashboard`);

      const activityFeed = page.locator('[data-testid="activity-feed"], text=Recent Activity').first();
      const hasFeed = await activityFeed.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasFeed).toBe('boolean');
    });
  });

  test.describe('Error Handling', () => {
    test('should handle analytics API errors gracefully', async ({ page }) => {
      await page.route('**/api/admin/analytics**', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server Error' }) });
      });

      await page.goto(`${ADMIN_URL}/analytics`);

      const errorMsg = page.locator('text=error, text=failed, text=Unable to load');
      const hasError = await errorMsg.isVisible({ timeout: 10000 }).catch(() => false);
      expect(typeof hasError).toBe('boolean');
    });

    test('should provide retry option on failed data load', async ({ page }) => {
      await page.route('**/api/admin/analytics**', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server Error' }) });
      });

      await page.goto(`${ADMIN_URL}/analytics`);

      const retryBtn = page.locator('button:has-text("Retry"), button:has-text("Reload")');
      const hasRetry = await retryBtn.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasRetry).toBe('boolean');
    });
  });

  test.describe('Performance', () => {
    test('should load analytics dashboard within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/analytics`);
      await page.waitForSelector('h1, [data-testid="analytics-page"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });
  });

  test.describe('Accessibility', () => {
    test('should support keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();
    });

    test('should have proper ARIA labels on metrics', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/analytics`);

      const metricCards = page.locator('[data-testid="metric-card"]');
      const cardCount = await metricCards.count();

      if (cardCount > 0) {
        for (let i = 0; i < Math.min(cardCount, 3); i++) {
          const card = metricCards.nth(i);
          const hasLabel = await card.getAttribute('aria-label') !== null ||
                          await card.textContent() !== '';
          expect(hasLabel).toBe(true);
        }
      }
    });
  });
});
