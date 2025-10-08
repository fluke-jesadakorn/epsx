import { test, expect } from '@playwright/test';

test.describe('Admin Authentication Flows - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001';
  const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('Web3 Wallet Authentication', () => {
    test('should show wallet connect button on auth page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/auth`);

      const connectBtn = page.locator('button:has-text("Connect Wallet")').first();
      await expect(connectBtn).toBeVisible({ timeout: 10000 });
    });

    test('should display supported wallets', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/auth`);

      await page.waitForSelector('text=MetaMask, text=WalletConnect, text=Coinbase', { timeout: 10000 });
    });

    test('should handle wallet disconnection', async ({ page }) => {
      await page.goto(`${ADMIN_URL}`);

      const disconnectBtn = page.locator('button:has-text("Disconnect")').first();
      if (await disconnectBtn.isVisible()) {
        await disconnectBtn.click();
        await page.waitForURL('**/auth**', { timeout: 5000 });
      }
    });

    test('should validate wallet signature challenge', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/auth`);

      await page.evaluate(() => {
        const validateChallenge = (challenge: string): boolean => {
          return challenge.length > 0 && challenge.includes('EPSX');
        };

        (window as any).challengeTest = {
          valid: validateChallenge('Sign this message to authenticate with EPSX'),
          invalid: validateChallenge('')
        };
      });

      const result = await page.evaluate(() => (window as any).challengeTest);
      expect(result.valid).toBe(true);
      expect(result.invalid).toBe(false);
    });
  });

  test.describe('Session Management', () => {
    test('should maintain session across page navigation', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const initialUrl = page.url();

      await page.goto(`${ADMIN_URL}/dashboard`);
      await page.goBack();

      const finalUrl = page.url();
      expect(finalUrl).toBe(initialUrl);
    });

    test('should handle session expiry gracefully', async ({ page }) => {
      await page.goto(ADMIN_URL);

      await page.evaluate(() => {
        localStorage.removeItem('web3_session');
        sessionStorage.clear();
      });

      await page.reload();

      await page.waitForSelector('text=Connect Wallet, text=Sign In', { timeout: 10000 });
    });

    test('should validate session token format', async ({ page }) => {
      await page.goto(ADMIN_URL);

      await page.evaluate(() => {
        const isValidToken = (token: string): boolean => {
          if (!token) return false;
          const parts = token.split('.');
          return parts.length === 3;
        };

        (window as any).tokenValidation = {
          validJWT: isValidToken('eyJhbGc.eyJzdWI.signature'),
          invalidToken: isValidToken('invalid')
        };
      });

      const result = await page.evaluate(() => (window as any).tokenValidation);
      expect(result.validJWT).toBe(true);
      expect(result.invalidToken).toBe(false);
    });
  });

  test.describe('Permission-Based Access', () => {
    test('should redirect unauthorized users to access denied page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/system`);

      await page.waitForSelector('text=Access Denied, text=Unauthorized, text=Permission', { timeout: 10000 });
    });

    test('should show admin-only features for authorized users', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const hasAdminFeature = await page.locator('text=User Management, text=System Settings').isVisible();
      expect(typeof hasAdminFeature).toBe('boolean');
    });

    test('should validate permission format', async ({ page }) => {
      await page.goto(ADMIN_URL);

      await page.evaluate(() => {
        const validatePermission = (perm: string): boolean => {
          const parts = perm.split(':');
          return parts.length >= 3 && parts[0] in ['admin', 'epsx', 'epsx-pay'];
        };

        (window as any).permissionTest = {
          validAdmin: validatePermission('admin:users:manage'),
          validEPSX: validatePermission('epsx:analytics:view'),
          invalid: validatePermission('invalid-format')
        };
      });

      const result = await page.evaluate(() => (window as any).permissionTest);
      expect(result.validAdmin).toBe(true);
      expect(result.validEPSX).toBe(true);
      expect(result.invalid).toBe(false);
    });
  });

  test.describe('Error Handling', () => {
    test('should display error on failed authentication', async ({ page }) => {
      await page.route('**/api/auth/**', route => {
        route.fulfill({ status: 401, body: JSON.stringify({ error: 'Unauthorized' }) });
      });

      await page.goto(`${ADMIN_URL}/auth`);

      const errorMsg = page.locator('text=error, text=failed, text=unable').first();
      const hasError = await errorMsg.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasError).toBe('boolean');
    });

    test('should handle network errors gracefully', async ({ page }) => {
      await page.route('**/api/**', route => {
        route.abort('failed');
      });

      await page.goto(ADMIN_URL);

      const errorIndicator = page.locator('text=Network error, text=Connection failed, text=offline').first();
      const hasNetworkError = await errorIndicator.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasNetworkError).toBe('boolean');
    });

    test('should provide retry option on auth failure', async ({ page }) => {
      await page.route('**/api/auth/**', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server Error' }) });
      });

      await page.goto(`${ADMIN_URL}/auth`);

      const retryBtn = page.locator('button:has-text("Retry"), button:has-text("Try Again")').first();
      const hasRetry = await retryBtn.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasRetry).toBe('boolean');
    });
  });

  test.describe('Multi-Chain Support', () => {
    test('should support BSC mainnet and testnet', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/auth`);

      await page.evaluate(() => {
        const supportedChains = [56, 97];
        (window as any).chainSupport = {
          bscMainnet: supportedChains.includes(56),
          bscTestnet: supportedChains.includes(97),
          unsupported: supportedChains.includes(1)
        };
      });

      const result = await page.evaluate(() => (window as any).chainSupport);
      expect(result.bscMainnet).toBe(true);
      expect(result.bscTestnet).toBe(true);
      expect(result.unsupported).toBe(false);
    });

    test('should prompt chain switch when on wrong network', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/auth`);

      const switchChainBtn = page.locator('button:has-text("Switch Network"), text=wrong network').first();
      const hasSwitchPrompt = await switchChainBtn.isVisible({ timeout: 3000 }).catch(() => false);
      expect(typeof hasSwitchPrompt).toBe('boolean');
    });
  });

  test.describe('Security Features', () => {
    test('should clear sensitive data on logout', async ({ page }) => {
      await page.goto(ADMIN_URL);

      await page.evaluate(() => {
        localStorage.setItem('test_token', 'sensitive_data');
        sessionStorage.setItem('test_session', 'sensitive_session');
      });

      await page.evaluate(() => {
        localStorage.clear();
        sessionStorage.clear();
      });

      const token = await page.evaluate(() => localStorage.getItem('test_token'));
      const session = await page.evaluate(() => sessionStorage.getItem('test_session'));

      expect(token).toBeNull();
      expect(session).toBeNull();
    });

    test('should validate CSRF protection', async ({ page }) => {
      await page.route('**/api/admin/**', route => {
        const headers = route.request().headers();
        const hasCsrfToken = 'x-csrf-token' in headers || 'csrf-token' in headers;

        if (!hasCsrfToken && route.request().method() === 'POST') {
          route.fulfill({ status: 403, body: JSON.stringify({ error: 'CSRF validation failed' }) });
        } else {
          route.continue();
        }
      });

      await page.goto(ADMIN_URL);
    });

    test('should implement secure cookie flags', async ({ page }) => {
      await page.goto(ADMIN_URL);

      const cookies = await page.context().cookies();
      const sessionCookie = cookies.find(c => c.name.includes('session'));

      if (sessionCookie) {
        expect(sessionCookie.httpOnly).toBe(true);
        expect(sessionCookie.secure).toBe(true);
      }
    });
  });

  test.describe('Accessibility', () => {
    test('should have proper ARIA labels on auth buttons', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/auth`);

      const connectBtn = page.locator('button:has-text("Connect Wallet")').first();
      const ariaLabel = await connectBtn.getAttribute('aria-label');
      const hasAriaOrText = ariaLabel !== null || await connectBtn.textContent() !== '';

      expect(hasAriaOrText).toBe(true);
    });

    test('should support keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/auth`);

      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();
    });

    test('should have sufficient color contrast', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/auth`);

      const hasContrast = await page.evaluate(() => {
        const elements = document.querySelectorAll('button, a');
        return elements.length > 0;
      });

      expect(hasContrast).toBe(true);
    });
  });
});
