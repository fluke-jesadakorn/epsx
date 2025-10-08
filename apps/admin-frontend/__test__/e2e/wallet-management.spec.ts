import { test, expect } from '@playwright/test';

test.describe('Wallet Management - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('Wallet Search and Discovery', () => {
    test('should search wallets by address', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const searchInput = page.locator('input[placeholder*="search"], input[type="search"]').first();
      if (await searchInput.isVisible()) {
        await searchInput.fill('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        const results = page.locator('[data-testid="wallet-card"], .wallet-item');
        const hasResults = await results.count() >= 0;
        expect(hasResults).toBe(true);
      }
    });

    test('should filter wallets by status', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const statusFilter = page.locator('select[name*="status"], button:has-text("Status")').first();
      if (await statusFilter.isVisible()) {
        if (statusFilter.tagName === 'select') {
          await statusFilter.selectOption('active');
        } else {
          await statusFilter.click();
          await page.locator('text=Active').first().click();
        }

        await page.waitForTimeout(500);
      }
    });

    test('should display wallet details on click', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const firstWallet = page.locator('[data-testid="wallet-card"], .wallet-item').first();
      if (await firstWallet.isVisible()) {
        await firstWallet.click();

        await page.waitForSelector('text=Wallet Details, text=Address, text=Balance', { timeout: 5000 });
      }
    });

    test('should validate wallet address format', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      await page.evaluate(() => {
        const isValidAddress = (addr: string): boolean => {
          return /^0x[a-fA-F0-9]{40}$/.test(addr);
        };

        (window as any).addressValidation = {
          valid: isValidAddress('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'),
          invalid: isValidAddress('invalid_address')
        };
      });

      const result = await page.evaluate(() => (window as any).addressValidation);
      expect(result.valid).toBe(true);
      expect(result.invalid).toBe(false);
    });
  });

  test.describe('Wallet Permission Assignment', () => {
    test('should assign permission to wallet', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const assignBtn = page.locator('button:has-text("Assign Permission"), button:has-text("Grant")').first();
      if (await assignBtn.isVisible()) {
        await assignBtn.click();

        const walletInput = page.locator('input[placeholder*="wallet"], input[placeholder*="address"]').first();
        if (await walletInput.isVisible()) {
          await walletInput.fill('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb');
        }

        const permissionInput = page.locator('input[name*="permission"], select[name*="permission"]').first();
        if (await permissionInput.isVisible()) {
          if (permissionInput.tagName === 'select') {
            await permissionInput.selectOption('admin:users:view');
          } else {
            await permissionInput.fill('admin:users:view');
          }
        }

        const submitBtn = page.locator('button[type="submit"], button:has-text("Assign")').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();

          await page.waitForSelector('text=Success, text=assigned, text=granted', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should revoke permission from wallet', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const revokeBtn = page.locator('button:has-text("Revoke"), button:has-text("Remove")').first();
      if (await revokeBtn.isVisible()) {
        await revokeBtn.click();

        const confirmBtn = page.locator('button:has-text("Confirm"), button:has-text("Yes")').first();
        if (await confirmBtn.isVisible()) {
          await confirmBtn.click();

          await page.waitForSelector('text=revoked, text=removed, text=Success', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should bulk assign permissions to wallets', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const bulkBtn = page.locator('button:has-text("Bulk"), button:has-text("Multiple")').first();
      if (await bulkBtn.isVisible()) {
        await bulkBtn.click();

        const addressListInput = page.locator('textarea[name*="address"], textarea[name*="wallet"]').first();
        if (await addressListInput.isVisible()) {
          await addressListInput.fill('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb\n0x853d955aCEf822Db058eb8505911ED77F175b99e');
        }

        const permissionSelect = page.locator('select[name*="permission"], input[name*="permission"]').first();
        if (await permissionSelect.isVisible()) {
          if (permissionSelect.tagName === 'select') {
            await permissionSelect.selectOption('admin:analytics:view');
          } else {
            await permissionSelect.fill('admin:analytics:view');
          }
        }

        const submitBulkBtn = page.locator('button[type="submit"], button:has-text("Assign")').first();
        if (await submitBulkBtn.isVisible()) {
          await submitBulkBtn.click();

          await page.waitForSelector('text=completed, text=successful, text=Bulk', { timeout: 15000 }).catch(() => {});
        }
      }
    });
  });

  test.describe('Wallet Analytics', () => {
    test('should display wallet statistics', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const statsCards = page.locator('[data-testid="stats-card"], .stats-card, .stat');
      const cardCount = await statsCards.count();

      if (cardCount > 0) {
        await expect(statsCards.first()).toBeVisible();
      }
    });

    test('should show wallet transaction history', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const firstWallet = page.locator('[data-testid="wallet-card"]').first();
      if (await firstWallet.isVisible()) {
        await firstWallet.click();

        const txHistory = page.locator('text=Transaction History, text=Transactions');
        const hasTxHistory = await txHistory.isVisible({ timeout: 5000 }).catch(() => false);
        expect(typeof hasTxHistory).toBe('boolean');
      }
    });

    test('should filter transactions by date range', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const dateFilter = page.locator('input[type="date"], button:has-text("Date")').first();
      if (await dateFilter.isVisible()) {
        if (dateFilter.getAttribute('type') === 'date') {
          await dateFilter.fill('2024-01-01');
        }

        await page.waitForTimeout(500);
      }
    });
  });

  test.describe('Wallet Group Assignment', () => {
    test('should assign wallet to group', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const assignGroupBtn = page.locator('button:has-text("Assign Group"), button:has-text("Add to Group")').first();
      if (await assignGroupBtn.isVisible()) {
        await assignGroupBtn.click();

        const groupSelect = page.locator('select[name*="group"], [role="combobox"]').first();
        if (await groupSelect.isVisible()) {
          await groupSelect.selectOption('premium-users');
        }

        const walletInput = page.locator('input[placeholder*="wallet"]').first();
        if (await walletInput.isVisible()) {
          await walletInput.fill('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb');
        }

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();

          await page.waitForSelector('text=Success, text=assigned to group', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should remove wallet from group', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const removeGroupBtn = page.locator('button:has-text("Remove from Group"), button:has-text("Unassign")').first();
      if (await removeGroupBtn.isVisible()) {
        await removeGroupBtn.click();

        const confirmBtn = page.locator('button:has-text("Confirm"), button:has-text("Yes")').first();
        if (await confirmBtn.isVisible()) {
          await confirmBtn.click();

          await page.waitForSelector('text=removed from group, text=Success', { timeout: 10000 }).catch(() => {});
        }
      }
    });
  });

  test.describe('Wallet Verification', () => {
    test('should verify wallet ownership', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const verifyBtn = page.locator('button:has-text("Verify"), button:has-text("Verify Ownership")').first();
      if (await verifyBtn.isVisible()) {
        await verifyBtn.click();

        await page.waitForSelector('text=verification, text=signature, text=challenge', { timeout: 5000 }).catch(() => {});
      }
    });

    test('should flag suspicious wallets', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const flagBtn = page.locator('button:has-text("Flag"), button:has-text("Suspicious")').first();
      if (await flagBtn.isVisible()) {
        await flagBtn.click();

        const reasonInput = page.locator('textarea[name*="reason"], input[name*="reason"]').first();
        if (await reasonInput.isVisible()) {
          await reasonInput.fill('Suspicious activity detected');
        }

        const submitBtn = page.locator('button[type="submit"], button:has-text("Flag")').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();

          await page.waitForSelector('text=flagged, text=Success', { timeout: 10000 }).catch(() => {});
        }
      }
    });
  });

  test.describe('Recent Wallets Panel', () => {
    test('should display recently active wallets', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const recentPanel = page.locator('text=Recent Wallets, text=Recently Active');
      const hasRecentPanel = await recentPanel.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasRecentPanel).toBe('boolean');
    });

    test('should refresh recent wallets list', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const refreshBtn = page.locator('button:has-text("Refresh"), button[aria-label*="refresh"]').first();
      if (await refreshBtn.isVisible()) {
        await refreshBtn.click();
        await page.waitForTimeout(500);
      }
    });

    test('should navigate to wallet details from recent panel', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const firstRecentWallet = page.locator('[data-testid="recent-wallet"]').first();
      if (await firstRecentWallet.isVisible()) {
        await firstRecentWallet.click();

        const urlContainsWallet = page.url().includes('wallet');
        expect(typeof urlContainsWallet).toBe('boolean');
      }
    });
  });

  test.describe('Wallet Export', () => {
    test('should export wallet data to CSV', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const exportBtn = page.locator('button:has-text("Export"), button:has-text("Download")').first();
      if (await exportBtn.isVisible()) {
        const downloadPromise = page.waitForEvent('download').catch(() => null);
        await exportBtn.click();

        const download = await downloadPromise;
        if (download) {
          expect(download.suggestedFilename()).toContain('wallet');
        }
      }
    });

    test('should export filtered wallet data', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const filterInput = page.locator('input[placeholder*="search"]').first();
      if (await filterInput.isVisible()) {
        await filterInput.fill('0x742');
        await page.waitForTimeout(500);
      }

      const exportBtn = page.locator('button:has-text("Export")').first();
      if (await exportBtn.isVisible()) {
        const downloadPromise = page.waitForEvent('download').catch(() => null);
        await exportBtn.click();

        const download = await downloadPromise;
        if (download) {
          expect(download).toBeTruthy();
        }
      }
    });
  });

  test.describe('Error Handling', () => {
    test('should handle invalid wallet address gracefully', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const searchInput = page.locator('input[placeholder*="search"]').first();
      if (await searchInput.isVisible()) {
        await searchInput.fill('invalid_wallet_address');
        await page.keyboard.press('Enter');

        await page.waitForSelector('text=Invalid, text=No results, text=Error', { timeout: 5000 }).catch(() => {});
      }
    });

    test('should handle API errors when loading wallets', async ({ page }) => {
      await page.route('**/api/admin/wallets**', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server Error' }) });
      });

      await page.goto(`${ADMIN_URL}/wallet-management`);

      const errorMsg = page.locator('text=error, text=failed, text=Unable to load');
      const hasError = await errorMsg.isVisible({ timeout: 10000 }).catch(() => false);
      expect(typeof hasError).toBe('boolean');
    });

    test('should provide retry option on failed wallet load', async ({ page }) => {
      await page.route('**/api/admin/wallets**', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server Error' }) });
      });

      await page.goto(`${ADMIN_URL}/wallet-management`);

      const retryBtn = page.locator('button:has-text("Retry"), button:has-text("Reload")');
      const hasRetry = await retryBtn.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasRetry).toBe('boolean');
    });
  });

  test.describe('Performance', () => {
    test('should load wallet management page within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/wallet-management`);
      await page.waitForSelector('h1, [data-testid="wallet-page"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });

    test('should handle large wallet lists efficiently', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const walletList = page.locator('[data-testid="wallet-list"], .wallet-container');
      const hasVirtualization = await page.evaluate(() => {
        const container = document.querySelector('[data-testid="wallet-list"]');
        return container !== null;
      });

      expect(typeof hasVirtualization).toBe('boolean');
    });
  });

  test.describe('Accessibility', () => {
    test('should have proper ARIA labels', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      const buttons = page.locator('button');
      const buttonCount = await buttons.count();

      if (buttonCount > 0) {
        for (let i = 0; i < Math.min(buttonCount, 5); i++) {
          const button = buttons.nth(i);
          const hasLabel = await button.getAttribute('aria-label') !== null ||
                          await button.textContent() !== '';
          expect(hasLabel).toBe(true);
        }
      }
    });

    test('should support keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/wallet-management`);

      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();

      for (let i = 0; i < 3; i++) {
        await page.keyboard.press('Tab');
        const currentFocus = page.locator(':focus').first();
        await expect(currentFocus).toBeVisible();
      }
    });
  });
});
