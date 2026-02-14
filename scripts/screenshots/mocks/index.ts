import type { Page } from '@playwright/test';
import type { MockHandler } from '../types';
import { authMocks } from './auth';
import { analyticsMocks } from './analytics';
import { notificationsMocks, emptyNotificationsMocks } from './notifications';
import { plansMocks } from './plans';
import { walletsMocks } from './wallets';
import { usersMocks } from './users';
import { adminMocks } from './admin';

const allMocks: MockHandler[] = [
  // Auth routes first (most specific)
  ...authMocks,
  // Domain-specific routes
  ...analyticsMocks,
  ...notificationsMocks,
  ...plansMocks,
  ...walletsMocks,
  ...usersMocks,
  ...adminMocks,
];

function matchPattern(urlStr: string, pattern: string): boolean {
  // Convert glob pattern to regex
  const regex = pattern
    .replace(/\*\*/g, '___GLOBSTAR___')
    .replace(/\*/g, '[^/]*')
    .replace(/___GLOBSTAR___/g, '.*');
  return new RegExp(regex).test(urlStr);
}

function findHandler(url: URL, mocks: MockHandler[]): MockHandler | undefined {
  return mocks.find(m => {
    if (typeof m.pattern === 'string') return matchPattern(url.href, m.pattern);
    return m.pattern.test(url.href);
  });
}

export interface MockOverrides {
  emptyNotifications?: boolean;
  extraHandlers?: MockHandler[];
}

export async function setupMocks(page: Page, overrides?: MockOverrides) {
  const mocks = [...allMocks];

  // Apply overrides
  if (overrides?.emptyNotifications) {
    // Remove default notification mocks, add empty ones
    const filtered = mocks.filter(m => {
      const p = typeof m.pattern === 'string' ? m.pattern : m.pattern.source;
      return !p.includes('notifications');
    });
    filtered.push(...emptyNotificationsMocks);
    mocks.length = 0;
    mocks.push(...filtered);
  }

  if (overrides?.extraHandlers) {
    mocks.unshift(...overrides.extraHandlers);
  }

  // Register catch-all route handler
  await page.route('**/api/**', route => {
    const url = new URL(route.request().url());

    // Skip auth routes - handled separately by setupAuth
    if (url.pathname.includes('/api/auth/')) {
      return route.fallback();
    }

    const handler = findHandler(url, mocks);
    if (handler) {
      const data = handler.handler(url);
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ data, success: true }),
      });
    }

    // Fallback for unmatched routes
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ data: {}, success: true }),
    });
  });
}
