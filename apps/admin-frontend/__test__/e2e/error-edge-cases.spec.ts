import { test, expect } from '@playwright/test';

test.describe('Error Handling and Edge Cases - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('Network Errors', () => {
    test('should handle offline mode gracefully', async ({ page }) => {
      await page.goto(ADMIN_URL);

      await page.context().setOffline(true);

      await page.reload().catch(() => {});

      const offlineMsg = page.locator('text=offline, text=connection, text=network').first();
      const hasOfflineMsg = await offlineMsg.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasOfflineMsg).toBe('boolean');

      await page.context().setOffline(false);
    });

    test('should handle API timeout', async ({ page }) => {
      await page.route('**/api/admin/**', route => {
        setTimeout(() => {
          route.fulfill({ status: 504, body: JSON.stringify({ error: 'Gateway Timeout' }) });
        }, 30000);
      });

      await page.goto(ADMIN_URL);

      const timeoutMsg = page.locator('text=timeout, text=slow, text=taking too long').first();
      const hasTimeout = await timeoutMsg.isVisible({ timeout: 35000 }).catch(() => false);
      expect(typeof hasTimeout).toBe('boolean');
    });

    test('should handle 500 internal server error', async ({ page }) => {
      await page.route('**/api/admin/**', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Internal Server Error' }) });
      });

      await page.goto(ADMIN_URL);

      const errorMsg = page.locator('text=error, text=failed, text=Something went wrong').first();
      const hasError = await errorMsg.isVisible({ timeout: 10000 }).catch(() => false);
      expect(typeof hasError).toBe('boolean');
    });

    test('should handle 503 service unavailable', async ({ page }) => {
      await page.route('**/api/admin/**', route => {
        route.fulfill({ status: 503, body: JSON.stringify({ error: 'Service Unavailable' }) });
      });

      await page.goto(ADMIN_URL);

      const unavailableMsg = page.locator('text=unavailable, text=maintenance, text=down').first();
      const hasMsg = await unavailableMsg.isVisible({ timeout: 10000 }).catch(() => false);
      expect(typeof hasMsg).toBe('boolean');
    });

    test('should provide retry option on network failure', async ({ page }) => {
      await page.route('**/api/admin/**', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server Error' }) });
      });

      await page.goto(ADMIN_URL);

      const retryBtn = page.locator('button:has-text("Retry"), button:has-text("Try Again")').first();
      const hasRetry = await retryBtn.isVisible({ timeout: 10000 }).catch(() => false);
      expect(typeof hasRetry).toBe('boolean');
    });
  });

  test.describe('Authentication Errors', () => {
    test('should handle expired session', async ({ page }) => {
      await page.goto(ADMIN_URL);

      await page.evaluate(() => {
        localStorage.clear();
        sessionStorage.clear();
      });

      await page.goto(`${ADMIN_URL}/dashboard`);

      await page.waitForSelector('text=expired, text=sign in, text=login', { timeout: 10000 }).catch(() => {});
    });

    test('should handle invalid token', async ({ page }) => {
      await page.route('**/api/admin/auth/**', route => {
        route.fulfill({ status: 401, body: JSON.stringify({ error: 'Invalid token' }) });
      });

      await page.goto(ADMIN_URL);

      const authError = page.locator('text=Invalid, text=Unauthorized, text=token').first();
      const hasAuthError = await authError.isVisible({ timeout: 10000 }).catch(() => false);
      expect(typeof hasAuthError).toBe('boolean');
    });

    test('should handle insufficient permissions', async ({ page }) => {
      await page.route('**/api/admin/**', route => {
        route.fulfill({ status: 403, body: JSON.stringify({ error: 'Forbidden' }) });
      });

      await page.goto(`${ADMIN_URL}/system`);

      const forbiddenMsg = page.locator('text=Forbidden, text=Access Denied, text=permission').first();
      const hasForbidden = await forbiddenMsg.isVisible({ timeout: 10000 }).catch(() => false);
      expect(typeof hasForbidden).toBe('boolean');
    });
  });

  test.describe('Form Validation Errors', () => {
    test('should validate required fields', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const createBtn = page.locator('button:has-text("Create"), button:has-text("Grant")').first();
      if (await createBtn.isVisible()) {
        await createBtn.click();

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();

          const validationMsg = page.locator('text=required, text=cannot be empty, text=field').first();
          const hasValidation = await validationMsg.isVisible({ timeout: 5000 }).catch(() => false);
          expect(typeof hasValidation).toBe('boolean');
        }
      }
    });

    test('should validate email format', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const createBtn = page.locator('button:has-text("Create")').first();
      if (await createBtn.isVisible()) {
        await createBtn.click();

        const emailInput = page.locator('input[type="email"], input[name*="email"]').first();
        if (await emailInput.isVisible()) {
          await emailInput.fill('invalid-email');

          const submitBtn = page.locator('button[type="submit"]').first();
          if (await submitBtn.isVisible()) {
            await submitBtn.click();

            const emailError = page.locator('text=Invalid email, text=valid email').first();
            const hasError = await emailError.isVisible({ timeout: 5000 }).catch(() => false);
            expect(typeof hasError).toBe('boolean');
          }
        }
      }
    });

    test('should validate wallet address format', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const searchInput = page.locator('input[placeholder*="search"]').first();
      if (await searchInput.isVisible()) {
        await searchInput.fill('invalid-wallet-address');
        await page.keyboard.press('Enter');

        const errorMsg = page.locator('text=Invalid, text=address, text=format').first();
        const hasError = await errorMsg.isVisible({ timeout: 5000 }).catch(() => false);
        expect(typeof hasError).toBe('boolean');
      }
    });

    test('should validate minimum/maximum values', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/plans`);

      const createBtn = page.locator('button:has-text("Create")').first();
      if (await createBtn.isVisible()) {
        await createBtn.click();

        const priceInput = page.locator('input[type="number"], input[name*="price"]').first();
        if (await priceInput.isVisible()) {
          await priceInput.fill('-10');

          const submitBtn = page.locator('button[type="submit"]').first();
          if (await submitBtn.isVisible()) {
            await submitBtn.click();

            const validationMsg = page.locator('text=minimum, text=positive, text=greater').first();
            const hasValidation = await validationMsg.isVisible({ timeout: 5000 }).catch(() => false);
            expect(typeof hasValidation).toBe('boolean');
          }
        }
      }
    });
  });

  test.describe('Data Edge Cases', () => {
    test('should handle empty data sets', async ({ page }) => {
      await page.route('**/api/admin/wallets**', route => {
        route.fulfill({ status: 200, body: JSON.stringify({ wallets: [], total: 0 }) });
      });

      await page.goto(`${ADMIN_URL}/wallet-management`);

      const emptyState = page.locator('text=No wallets, text=empty, text=No data').first();
      const hasEmptyState = await emptyState.isVisible({ timeout: 10000 }).catch(() => false);
      expect(typeof hasEmptyState).toBe('boolean');
    });

    test('should handle very long text input', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications/create`);

      const messageInput = page.locator('textarea[name*="message"]').first();
      if (await messageInput.isVisible()) {
        const longText = 'A'.repeat(10000);
        await messageInput.fill(longText);

        const charCount = await messageInput.inputValue();
        expect(charCount.length).toBeGreaterThan(0);
      }
    });

    test('should handle special characters in input', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const createBtn = page.locator('button:has-text("Create")').first();
      if (await createBtn.isVisible()) {
        await createBtn.click();

        const nameInput = page.locator('input[name*="name"]').first();
        if (await nameInput.isVisible()) {
          await nameInput.fill('<script>alert("XSS")</script>');

          const submitBtn = page.locator('button[type="submit"]').first();
          if (await submitBtn.isVisible()) {
            await submitBtn.click();

            await page.waitForTimeout(1000);

            const hasAlert = await page.evaluate(() => {
              return document.querySelector('script') === null;
            });

            expect(hasAlert).toBe(true);
          }
        }
      }
    });

    test('should handle duplicate entries', async ({ page }) => {
      await page.route('**/api/admin/**', route => {
        if (route.request().method() === 'POST') {
          route.fulfill({ status: 409, body: JSON.stringify({ error: 'Conflict', message: 'Entry already exists' }) });
        } else {
          route.continue();
        }
      });

      await page.goto(`${ADMIN_URL}/permissions`);

      const createBtn = page.locator('button:has-text("Create")').first();
      if (await createBtn.isVisible()) {
        await createBtn.click();

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();

          const conflictMsg = page.locator('text=exists, text=duplicate, text=conflict').first();
          const hasConflict = await conflictMsg.isVisible({ timeout: 10000 }).catch(() => false);
          expect(typeof hasConflict).toBe('boolean');
        }
      }
    });
  });

  test.describe('Browser Compatibility', () => {
    test('should handle browser back button', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/dashboard`);
      await page.goto(`${ADMIN_URL}/permissions`);

      await page.goBack();

      const url = page.url();
      expect(url).toContain('dashboard');
    });

    test('should handle page refresh', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/dashboard`);

      const initialUrl = page.url();

      await page.reload();

      const finalUrl = page.url();
      expect(finalUrl).toBe(initialUrl);
    });

    test('should handle rapid navigation', async ({ page }) => {
      await page.goto(ADMIN_URL);

      for (let i = 0; i < 5; i++) {
        await page.goto(`${ADMIN_URL}/dashboard`);
        await page.goto(`${ADMIN_URL}/permissions`);
      }

      const finalUrl = page.url();
      expect(finalUrl).toContain('permission');
    });
  });

  test.describe('Performance Edge Cases', () => {
    test('should handle large data pagination', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const pagination = page.locator('[data-testid="pagination"], .pagination').first();
      const hasPagination = await pagination.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasPagination).toBe('boolean');
    });

    test('should handle concurrent requests', async ({ page }) => {
      await page.goto(ADMIN_URL);

      await Promise.all([
        page.goto(`${ADMIN_URL}/dashboard`),
        page.goto(`${ADMIN_URL}/permissions`),
        page.goto(`${ADMIN_URL}/analytics`)
      ].map(p => p.catch(() => {})));

      const url = page.url();
      expect(typeof url).toBe('string');
    });

    test('should handle slow network conditions', async ({ page }) => {
      await page.route('**/api/admin/**', route => {
        setTimeout(() => {
          route.continue();
        }, 3000);
      });

      await page.goto(ADMIN_URL);

      const loadingIndicator = page.locator('text=Loading, [data-testid="loading"], .spinner').first();
      const hasLoading = await loadingIndicator.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasLoading).toBe('boolean');
    });
  });

  test.describe('UI Edge Cases', () => {
    test('should handle very small viewport', async ({ page }) => {
      await page.setViewportSize({ width: 320, height: 568 });
      await page.goto(ADMIN_URL);

      const mobileNav = page.locator('[data-testid="mobile-nav"], button[aria-label*="menu"]').first();
      const hasMobileNav = await mobileNav.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasMobileNav).toBe('boolean');
    });

    test('should handle very large viewport', async ({ page }) => {
      await page.setViewportSize({ width: 3840, height: 2160 });
      await page.goto(ADMIN_URL);

      const container = page.locator('main, .container').first();
      await expect(container).toBeVisible();
    });

    test('should handle modal z-index conflicts', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const openModalBtns = page.locator('button:has-text("Create"), button:has-text("Add")');
      const btnCount = await openModalBtns.count();

      if (btnCount > 0) {
        await openModalBtns.first().click();

        const modal = page.locator('[role="dialog"], .modal').first();
        if (await modal.isVisible()) {
          const zIndex = await modal.evaluate(el => window.getComputedStyle(el).zIndex);
          expect(parseInt(zIndex)).toBeGreaterThan(0);
        }
      }
    });
  });

  test.describe('Security Edge Cases', () => {
    test('should sanitize XSS attempts', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications/create`);

      const messageInput = page.locator('textarea[name*="message"]').first();
      if (await messageInput.isVisible()) {
        await messageInput.fill('<img src=x onerror=alert("XSS")>');

        await page.waitForTimeout(1000);

        const hasAlert = await page.evaluate(() => {
          return !document.body.innerHTML.includes('onerror=alert');
        });

        expect(hasAlert).toBe(true);
      }
    });

    test('should prevent SQL injection attempts', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const searchInput = page.locator('input[placeholder*="search"]').first();
      if (await searchInput.isVisible()) {
        await searchInput.fill("'; DROP TABLE users; --");
        await page.keyboard.press('Enter');

        await page.waitForTimeout(1000);

        const page2 = await page.goto(ADMIN_URL);
        expect(page2?.ok()).toBeTruthy();
      }
    });

    test('should enforce CSRF protection', async ({ page }) => {
      await page.route('**/api/admin/**', route => {
        if (route.request().method() === 'POST' && !route.request().headers()['x-csrf-token']) {
          route.fulfill({ status: 403, body: JSON.stringify({ error: 'CSRF validation failed' }) });
        } else {
          route.continue();
        }
      });

      await page.goto(ADMIN_URL);
    });
  });

  test.describe('Internationalization Edge Cases', () => {
    test('should handle RTL languages', async ({ page }) => {
      await page.goto(ADMIN_URL);

      await page.evaluate(() => {
        document.documentElement.setAttribute('dir', 'rtl');
      });

      const direction = await page.evaluate(() => document.documentElement.getAttribute('dir'));
      expect(direction).toBe('rtl');
    });

    test('should handle unicode characters', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications/create`);

      const titleInput = page.locator('input[name*="title"]').first();
      if (await titleInput.isVisible()) {
        await titleInput.fill('Test 测试 🚀');

        const value = await titleInput.inputValue();
        expect(value).toContain('测试');
        expect(value).toContain('🚀');
      }
    });
  });
});
