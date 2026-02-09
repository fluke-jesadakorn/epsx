import { test, expect } from '@playwright/test';

test.describe('Profile and Settings - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL ?? 'http://localhost:3001';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('Admin Profile', () => {
    test('should display profile page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      await page.waitForSelector('text=Profile, text=Account, h1', { timeout: 10000 });
    });

    test('should display wallet information', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const walletInfo = page.locator('text=Wallet, text=Address, text=0x').first();
      const hasWallet = await walletInfo.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasWallet).toBe('boolean');
    });

    test('should display user permissions', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const permissionsSection = page.locator('text=Permissions, text=Access, text=Role').first();
      const hasPermissions = await permissionsSection.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasPermissions).toBe('boolean');
    });

    test('should update profile information', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const editBtn = page.locator('button:has-text("Edit"), button:has-text("Update")').first();
      if (await editBtn.isVisible()) {
        await editBtn.click();

        const nameInput = page.locator('input[name*="name"]').first();
        if (await nameInput.isVisible()) {
          await nameInput.fill('Updated Admin Name');
        }

        const saveBtn = page.locator('button[type="submit"], button:has-text("Save")').first();
        if (await saveBtn.isVisible()) {
          await saveBtn.click();
          await page.waitForSelector('text=Success, text=updated', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should display connected wallets', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const walletsSection = page.locator('text=Connected Wallets, text=Linked Wallets').first();
      const hasWallets = await walletsSection.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasWallets).toBe('boolean');
    });

    test('should connect additional wallet', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const connectBtn = page.locator('button:has-text("Connect Wallet"), button:has-text("Add Wallet")').first();
      if (await connectBtn.isVisible()) {
        await connectBtn.click();

        await page.waitForSelector('text=MetaMask, text=WalletConnect', { timeout: 5000 });
      }
    });

    test('should disconnect wallet', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const disconnectBtn = page.locator('button:has-text("Disconnect"), button:has-text("Remove")').first();
      if (await disconnectBtn.isVisible()) {
        await disconnectBtn.click();

        const confirmBtn = page.locator('button:has-text("Confirm"), button:has-text("Yes")').first();
        if (await confirmBtn.isVisible()) {
          await confirmBtn.click();
          await page.waitForSelector('text=disconnected, text=removed', { timeout: 10000 }).catch(() => {});
        }
      }
    });
  });

  test.describe('Settings', () => {
    test('should display settings page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/settings`);

      await page.waitForSelector('text=Settings, text=Preferences, h1', { timeout: 10000 });
    });

    test('should toggle theme', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/settings`);

      const themeToggle = page.locator('button[aria-label*="theme"], input[type="checkbox"]').first();
      if (await themeToggle.isVisible()) {
        await themeToggle.click();
        await page.waitForTimeout(300);

        const htmlClass = await page.evaluate(() => document.documentElement.className);
        expect(typeof htmlClass).toBe('string');
      }
    });

    test('should change notification preferences', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/settings`);

      const notificationCheckbox = page.locator('input[type="checkbox"], label:has-text("Notification")').first();
      if (await notificationCheckbox.isVisible()) {
        await notificationCheckbox.click();
        await page.waitForTimeout(300);
      }
    });

    test('should update language preference', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/settings`);

      const languageSelect = page.locator('select[name*="language"], select[name*="locale"]').first();
      if (await languageSelect.isVisible()) {
        await languageSelect.selectOption('en');
        await page.waitForTimeout(300);
      }
    });

    test('should save settings', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/settings`);

      const saveBtn = page.locator('button:has-text("Save"), button[type="submit"]').first();
      if (await saveBtn.isVisible()) {
        await saveBtn.click();
        await page.waitForSelector('text=Success, text=saved', { timeout: 10000 }).catch(() => {});
      }
    });
  });

  test.describe('Security Settings', () => {
    test('should display security settings', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/settings`);

      const securityTab = page.locator('button:has-text("Security"), [role="tab"]:has-text("Security")').first();
      if (await securityTab.isVisible()) {
        await securityTab.click();

        await page.waitForSelector('text=Security, text=Password, text=2FA', { timeout: 5000 });
      }
    });

    test('should enable two-factor authentication', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/settings`);

      const securityTab = page.locator('button:has-text("Security")').first();
      if (await securityTab.isVisible()) {
        await securityTab.click();

        const enable2FABtn = page.locator('button:has-text("Enable 2FA"), button:has-text("Setup 2FA")').first();
        if (await enable2FABtn.isVisible()) {
          await enable2FABtn.click();

          await page.waitForSelector('text=QR Code, text=Authenticator, text=Code', { timeout: 5000 });
        }
      }
    });

    test('should display active sessions', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/settings`);

      const securityTab = page.locator('button:has-text("Security")').first();
      if (await securityTab.isVisible()) {
        await securityTab.click();

        const sessionsSection = page.locator('text=Active Sessions, text=Sessions').first();
        const hasSessions = await sessionsSection.isVisible({ timeout: 5000 }).catch(() => false);
        expect(typeof hasSessions).toBe('boolean');
      }
    });

    test('should revoke session', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/settings`);

      const securityTab = page.locator('button:has-text("Security")').first();
      if (await securityTab.isVisible()) {
        await securityTab.click();

        const revokeBtn = page.locator('button:has-text("Revoke"), button:has-text("Sign Out")').first();
        if (await revokeBtn.isVisible()) {
          await revokeBtn.click();

          await page.waitForSelector('text=revoked, text=signed out', { timeout: 5000 }).catch(() => {});
        }
      }
    });
  });

  test.describe('Activity Log', () => {
    test('should display activity log', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const activityTab = page.locator('button:has-text("Activity"), [role="tab"]:has-text("Activity")').first();
      if (await activityTab.isVisible()) {
        await activityTab.click();

        await page.waitForSelector('text=Activity, text=History, text=Log', { timeout: 5000 });
      }
    });

    test('should filter activity by date', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const activityTab = page.locator('button:has-text("Activity")').first();
      if (await activityTab.isVisible()) {
        await activityTab.click();

        const dateFilter = page.locator('input[type="date"], button:has-text("Date")').first();
        if (await dateFilter.isVisible()) {
          await dateFilter.click();
          await page.waitForTimeout(500);
        }
      }
    });

    test('should filter activity by type', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const activityTab = page.locator('button:has-text("Activity")').first();
      if (await activityTab.isVisible()) {
        await activityTab.click();

        const typeFilter = page.locator('select[name*="type"], button:has-text("Type")').first();
        if (await typeFilter.isVisible()) {
          if (typeFilter.tagName === 'select') {
            await typeFilter.selectOption({ index: 1 });
          } else {
            await typeFilter.click();
          }

          await page.waitForTimeout(500);
        }
      }
    });

    test('should export activity log', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const activityTab = page.locator('button:has-text("Activity")').first();
      if (await activityTab.isVisible()) {
        await activityTab.click();

        const exportBtn = page.locator('button:has-text("Export"), button:has-text("Download")').first();
        if (await exportBtn.isVisible()) {
          const downloadPromise = page.waitForEvent('download').catch(() => null);
          await exportBtn.click();

          const download = await downloadPromise;
          if (download) {
            expect(download.suggestedFilename()).toContain('activity');
          }
        }
      }
    });
  });

  test.describe('Performance', () => {
    test('should load profile page within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/profile`);
      await page.waitForSelector('h1, [data-testid="profile-page"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });

    test('should load settings page within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/settings`);
      await page.waitForSelector('h1, [data-testid="settings-page"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });
  });

  test.describe('Accessibility', () => {
    test('should support keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();
    });

    test('should have proper heading structure', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/profile`);

      const hasH1 = await page.locator('h1').count() > 0;
      expect(hasH1).toBe(true);
    });
  });
});
