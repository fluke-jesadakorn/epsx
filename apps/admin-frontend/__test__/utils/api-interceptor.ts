/**
 * API Route Interceptor for Admin Frontend E2E Tests
 * Sets up page.route() interception for all API endpoints using mock data.
 */
import { type Page } from '@playwright/test';
import { API_MOCKS } from '../fixtures/api-mocks';

type MockOverrides = Partial<Record<string, unknown>>;

/**
 * Parse "METHOD /path" keys and set up route interception for all API mocks.
 */
export async function mockAllApis(page: Page, overrides: MockOverrides = {}) {
  const merged = { ...API_MOCKS, ...overrides };

  for (const [key, response] of Object.entries(merged)) {
    const spaceIdx = key.indexOf(' ');
    if (spaceIdx === -1) continue;

    const method = key.slice(0, spaceIdx).toUpperCase();
    const path = key.slice(spaceIdx + 1);

    // Convert :param patterns to wildcard for route matching
    const urlPattern = `**${path.replace(/:[^/]+/g, '*')}`;

    await page.route(urlPattern, (route, request) => {
      if (request.method() !== method) {
        return route.fallback();
      }
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(response),
      });
    });
  }

  // Fallback: any unmatched /api/ calls return 200 with empty success
  await page.route('**/api/**', (route, request) => {
    const url = request.url();
    if (url.includes('/stream') || url.includes('/sse')) {
      return route.fulfill({ status: 200, contentType: 'text/event-stream', body: 'data: {"type":"connected"}\n\n' });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ data: null, success: true }),
    });
  });
}
