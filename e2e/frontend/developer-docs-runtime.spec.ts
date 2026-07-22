import { expect, test, type Page } from '@playwright/test';

const accessToken = process.env.A7_DEVELOPER_DOCS_ACCESS_TOKEN;
const viewports = [
  { name: 'mobile', width: 390, height: 844 },
  { name: 'desktop', width: 1440, height: 900 },
] as const;

const sections = [
  ['auth', 'Authentication', 1],
  ['analytics', 'Analytics', 4],
  ['portfolio', 'Portfolio & Watchlist', 3],
  ['user', 'User', 2],
] as const;

const endpoints = [
  ['GET', '/api/auth/session/verify'],
  ['GET', '/api/analytics/rankings'],
  ['GET', '/api/analytics/filters'],
  ['GET', '/api/analytics/countries'],
  ['GET', '/api/analytics/sectors'],
  ['GET', '/api/users/watchlist'],
  ['POST', '/api/users/watchlist'],
  ['DELETE', '/api/users/watchlist'],
  ['GET', '/api/users/profile'],
  ['GET', '/api/users/access-overview'],
] as const;

const sourceCopy = [
  'Integrate EPSX analytics into your applications. Use your API key as a Bearer token — same endpoints, same data.',
  'All requests use the Authorization: Bearer <token> header. Your API key works like a JWT — the middleware auto-detects the type.',
  'API keys use the same Authorization header as JWT tokens. Pass your key as a Bearer token.',
  'Market data, stock rankings, filters, countries, and sector breakdowns.',
  'Manage your stock watchlist. Requires authentication.',
  'User profile and access information.',
  'Verify that your API key is valid and return associated permissions.',
  'Returns paginated EPS rankings with optional filters. Free tier gets limited columns; premium unlocks all fields.',
  'Returns available filter values for countries, sectors, and sort columns.',
  'Returns list of countries with stock data available.',
  'Returns available sector categories.',
  'Returns current user watchlist with stock data.',
  'Add a stock ticker to your watchlist.',
  'Remove a stock ticker from your watchlist.',
  'Returns the authenticated user profile including wallet address and plan info.',
  'Returns a summary of permissions and plan features available to the user.',
] as const;

async function expectNoHorizontalOverflow(page: Page) {
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth
  );
  expect(overflow).toBeLessThanOrEqual(1);
}

test.describe('A7 /developer/docs pinned-source runtime proof', () => {
  test.skip(
    !accessToken,
    'Run through scripts/migration/run-developer-docs-runtime-proof.sh'
  );

  test.beforeEach(async ({ context }) => {
    await context.addCookies([
      {
        name: 'epsx.frontend.access_token',
        value: accessToken!,
        url: 'http://localhost:3000',
        httpOnly: true,
        sameSite: 'Lax',
      },
    ]);
  });

  test('proves content, metadata, order, responsive navigation, and fail-closed controls', async ({ page }) => {
    const pageErrors: string[] = [];
    const consoleErrors: string[] = [];
    page.on('pageerror', error => pageErrors.push(error.message));
    page.on('console', message => {
      if (message.type() === 'error') consoleErrors.push(message.text());
    });

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      const response = await page.goto('/developer/docs', { waitUntil: 'domcontentloaded' });
      expect(response?.status(), `${viewport.name} status`).toBe(200);
      await expect(page).toHaveTitle('EPSX - Stock Analytics Platform');
      await expect(page.locator('meta[name="description"]')).toHaveAttribute(
        'content',
        'Advanced stock data analytics platform'
      );
      await expect(page.locator('meta[name="keywords"]')).toHaveAttribute(
        'content',
        'stock analytics,financial data,EPSX,market insights'
      );
      await expect(page.locator('.developer-docs-page')).toHaveAttribute(
        'data-docs-source-baseline',
        'origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db'
      );

      await expect(page.getByRole('heading', { level: 1, name: 'API Reference' })).toHaveCount(1);
      await expect(page.locator('.developer-docs-hero')).toContainText(
        'Integrate EPSX analytics into your applications. Use your API key as a Bearer token — same endpoints, same data.'
      );
      await expect(page.locator('.developer-docs-auth-card')).toContainText(
        'Your API key works like a JWT — the middleware auto-detects the type.'
      );
      await expect(page.locator('.docs-endpoint-card')).toHaveCount(10);
      await expect(page.locator('.docs-try-it')).toHaveCount(10);
      await expect(page.locator('.docs-send-button')).toHaveCount(10);
      expect(
        await page.locator('.docs-send-button').evaluateAll(buttons =>
          buttons.every(button => (button as HTMLButtonElement).disabled)
        )
      ).toBe(true);
      await expect(page.locator('.docs-try-it-status').first()).toContainText(
        'Live requests stay disabled until the A4/A5'
      );
      for (const copy of sourceCopy) {
        await expect(page.locator('.developer-docs-page')).toContainText(copy);
      }
      await expect(page.locator('.developer-docs-page')).not.toContainText(
        'REST endpoints, request/response schemas, and examples'
      );

      const actualSections = await page.locator('.docs-endpoint-section').evaluateAll(nodes =>
        nodes.map(node => ({
          id: node.id,
          heading: node.querySelector('h2')?.textContent?.trim(),
          cards: node.querySelectorAll('.docs-endpoint-card').length,
        }))
      );
      expect(actualSections).toEqual(
        sections.map(([id, heading, cards]) => ({ id: `section-${id}`, heading, cards }))
      );
      const actualEndpoints = await page.locator('[data-docs-endpoint-toggle="true"]').evaluateAll(buttons =>
        buttons.map(button => [
          button.querySelector('span')?.textContent?.trim(),
          button.querySelector('code')?.textContent?.trim(),
        ])
      );
      expect(actualEndpoints).toEqual(endpoints);

      const firstToggle = page.locator('[data-docs-endpoint-toggle="true"]').first();
      await firstToggle.focus();
      await expect(firstToggle).toBeFocused();
      await page.keyboard.press('Enter');
      await expect(firstToggle).toHaveAttribute('aria-expanded', 'true');
      const body = page.locator(`#${await firstToggle.getAttribute('aria-controls')}`);
      await expect(body).toBeVisible();

      const tabs = body.getByRole('tab');
      await expect(tabs).toHaveCount(3);
      await tabs.first().focus();
      await page.keyboard.press('ArrowRight');
      await expect(tabs.nth(1)).toBeFocused();
      await expect(tabs.nth(1)).toHaveAttribute('aria-selected', 'true');
      await expect(body.locator('[data-docs-code-panel="javascript"]')).toBeVisible();
      await expect(body.locator('[data-docs-code-panel="curl"]')).toBeHidden();
      await body.locator('[data-docs-copy-code="true"]').click();
      await expect(body.locator('[data-docs-copy-code="true"]')).toContainText('Copied');
      await body.locator('[data-docs-copy-response="true"]').click();
      await expect(body.locator('[data-docs-copy-response="true"]')).toContainText('Copied');

      if (viewport.name === 'mobile') {
        const toggle = page.locator('[data-docs-sidebar-toggle="true"]');
        await expect(toggle).toBeVisible();
        await expect(toggle).toHaveAccessibleName('Open API reference navigation');
        await toggle.focus();
        await page.keyboard.press('Enter');
        await expect(toggle).toHaveAttribute('aria-expanded', 'true');
        await expect(toggle).toHaveAccessibleName('Close API reference navigation');
        const analyticsLink = page.locator('[data-docs-section-link="analytics"]');
        await expect(page.locator('[data-docs-section-link="auth"]')).toBeFocused();
        await analyticsLink.focus();
        await page.keyboard.press('Enter');
        await expect(toggle).toHaveAttribute('aria-expanded', 'false');
        await expect(toggle).toBeFocused();
      } else {
        await expect(page.locator('#developer-docs-sidebar')).toBeVisible();
        const analyticsLink = page.locator('[data-docs-section-link="analytics"]');
        await analyticsLink.focus();
        await expect(analyticsLink).toBeFocused();
        await page.keyboard.press('Enter');
        await expect(analyticsLink).toHaveClass(/active/);
      }

      const theme = page.getByRole('button', { name: 'Toggle theme' });
      await theme.focus();
      await expect(theme).toBeFocused();
      await page.keyboard.press('Enter');
      await expectNoHorizontalOverflow(page);
    }

    expect(pageErrors).toEqual([]);
    expect(consoleErrors).toEqual([]);
  });
});
