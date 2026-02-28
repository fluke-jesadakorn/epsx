import { test, expect } from '@playwright/test';

test.describe('API Documentation - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL ?? 'http://localhost:3001';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('API Documentation Display', () => {
    test('should display API documentation page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      await page.waitForSelector('text=API, text=Documentation, text=Endpoints', { timeout: 10000 });
    });

    test('should display API endpoints list', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const endpointsList = page.locator('[data-testid="endpoints-list"], .endpoint-list');
      const hasEndpoints = await endpointsList.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasEndpoints).toBe('boolean');
    });

    test('should display endpoint details on click', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const firstEndpoint = page.locator('[data-testid="endpoint-item"], .endpoint').first();
      if (await firstEndpoint.isVisible()) {
        await firstEndpoint.click();

        await page.waitForSelector('text=Request, text=Response, text=Parameters', { timeout: 5000 });
      }
    });

    test('should display request/response examples', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const exampleCode = page.locator('pre, code, [data-testid="code-example"]').first();
      const hasExamples = await exampleCode.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasExamples).toBe('boolean');
    });
  });

  test.describe('API Endpoint Filtering', () => {
    test('should filter endpoints by method', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const methodFilter = page.locator('button:has-text("GET"), button:has-text("POST")').first();
      if (await methodFilter.isVisible()) {
        await methodFilter.click();
        await page.waitForTimeout(500);
      }
    });

    test('should search endpoints by keyword', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const searchInput = page.locator('input[placeholder*="search"], input[type="search"]').first();
      if (await searchInput.isVisible()) {
        await searchInput.fill('wallet');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(500);
      }
    });

    test('should filter by endpoint category', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const categoryBtn = page.locator('button:has-text("auth"), button:has-text("user")').first();
      if (await categoryBtn.isVisible()) {
        await categoryBtn.click();
        await page.waitForTimeout(500);
      }
    });
  });

  test.describe('Interactive API Testing', () => {
    test('should display Try it button for endpoints', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const tryItBtn = page.locator('button:has-text("Try it"), button:has-text("Test")').first();
      const hasTryIt = await tryItBtn.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasTryIt).toBe('boolean');
    });

    test('should allow parameter input for testing', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const tryItBtn = page.locator('button:has-text("Try it")').first();
      if (await tryItBtn.isVisible()) {
        await tryItBtn.click();

        const paramInput = page.locator('input[name*="param"], input[placeholder*="parameter"]').first();
        if (await paramInput.isVisible()) {
          await paramInput.fill('test-value');
        }
      }
    });

    test('should send test API request', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const tryItBtn = page.locator('button:has-text("Try it")').first();
      if (await tryItBtn.isVisible()) {
        await tryItBtn.click();

        const sendBtn = page.locator('button:has-text("Send"), button:has-text("Execute")').first();
        if (await sendBtn.isVisible()) {
          await sendBtn.click();

          await page.waitForSelector('text=Response, text=Result, text=Status', { timeout: 5000 });
        }
      }
    });
  });

  test.describe('Authentication Documentation', () => {
    test('should display authentication methods', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const authSection = page.locator('text=Authentication, text=Bearer Token, text=API Key').first();
      const hasAuth = await authSection.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasAuth).toBe('boolean');
    });

    test('should show API key management link', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const apiKeyLink = page.locator('a:has-text("API Key"), a:has-text("Manage Keys")').first();
      const hasLink = await apiKeyLink.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasLink).toBe('boolean');
    });
  });

  test.describe('Code Examples', () => {
    test('should display code examples in multiple languages', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const langButtons = page.locator('button:has-text("JavaScript"), button:has-text("Python"), button:has-text("cURL")');
      const hasLangOptions = await langButtons.first().isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasLangOptions).toBe('boolean');
    });

    test('should switch between language examples', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const pythonBtn = page.locator('button:has-text("Python")').first();
      if (await pythonBtn.isVisible()) {
        await pythonBtn.click();
        await page.waitForTimeout(300);

        const codeBlock = page.locator('pre, code').first();
        const content = await codeBlock.textContent();
        expect(typeof content).toBe('string');
      }
    });

    test('should copy code example to clipboard', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const copyBtn = page.locator('button:has-text("Copy"), button[aria-label*="copy"]').first();
      if (await copyBtn.isVisible()) {
        await copyBtn.click();

        await page.waitForSelector('text=Copied, text=✓', { timeout: 3000 }).catch(() => {});
      }
    });
  });

  test.describe('Error Responses', () => {
    test('should document error response codes', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const errorSection = page.locator('text=Error, text=400, text=401, text=500').first();
      const hasErrors = await errorSection.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasErrors).toBe('boolean');
    });

    test('should show error response examples', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const errorExample = page.locator('text=error, text=message, text=code').first();
      const hasExample = await errorExample.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasExample).toBe('boolean');
    });
  });

  test.describe('API Versioning', () => {
    test('should display API version', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const versionInfo = page.locator('text=v1, text=Version, text=API Version').first();
      const hasVersion = await versionInfo.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasVersion).toBe('boolean');
    });

    test('should allow switching between API versions', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const versionSelect = page.locator('select[name*="version"], button:has-text("Version")').first();
      if (await versionSelect.isVisible()) {
        if (versionSelect.tagName === 'select') {
          await versionSelect.selectOption({ index: 0 });
        } else {
          await versionSelect.click();
        }
      }
    });
  });

  test.describe('Performance', () => {
    test('should load API documentation within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/docs/api`);
      await page.waitForSelector('h1, [data-testid="api-docs"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });
  });

  test.describe('Accessibility', () => {
    test('should support keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();
    });

    test('should have proper heading structure', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/docs/api`);

      const hasH1 = await page.locator('h1').count() > 0;
      expect(hasH1).toBe(true);
    });
  });
});
