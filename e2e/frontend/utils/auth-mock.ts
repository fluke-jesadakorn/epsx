/**
 * Auth Mock Helper for Frontend E2E Tests
 * Sets cookies and intercepts auth endpoints to simulate an authenticated session.
 */
import { type Page } from '@playwright/test';
import { MOCK_USER, MOCK_TOKEN, MOCK_PERMISSIONS } from '../fixtures/api-mocks';

const COOKIE_DOMAIN = 'localhost';

export async function mockAuth(page: Page, user = MOCK_USER, token = MOCK_TOKEN) {
  const rawUrl = page.context().pages()[0]?.url() ?? '';
  const baseURL = rawUrl.startsWith('http') ? rawUrl : 'http://localhost:3000';
  const url = new URL(baseURL);
  const domain = url.hostname;

  // Set auth cookies (dev mode uses epsx.* prefix without __Host-)
  await page.context().addCookies([
    { name: 'epsx.access_token', value: token, domain: domain || COOKIE_DOMAIN, path: '/', httpOnly: true, secure: false, sameSite: 'Lax' },
    { name: 'epsx.refresh_token', value: `refresh_${token}`, domain: domain || COOKIE_DOMAIN, path: '/', httpOnly: true, secure: false, sameSite: 'Lax' },
    { name: 'epsx.user', value: encodeURIComponent(JSON.stringify(user)), domain: domain || COOKIE_DOMAIN, path: '/', httpOnly: false, secure: false, sameSite: 'Lax' },
    { name: 'epsx.sid', value: `sid_${user.id}`, domain: domain || COOKIE_DOMAIN, path: '/', httpOnly: false, secure: false, sameSite: 'Lax' },
    { name: 'epsx.expires_at', value: String(Date.now() + 86400000), domain: domain || COOKIE_DOMAIN, path: '/', httpOnly: false, secure: false, sameSite: 'Lax' },
    { name: 'epsx.auth_time', value: String(Date.now()), domain: domain || COOKIE_DOMAIN, path: '/', httpOnly: false, secure: false, sameSite: 'Lax' },
  ]);

  // Intercept auth verification endpoints
  await page.route('**/api/auth/session/verify', route =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ data: { valid: true, user }, success: true }) })
  );

  await page.route('**/api/auth/web3/session', route =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ data: { authenticated: true, user, token }, success: true }) })
  );

  await page.route('**/api/auth/users/profile', route =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ data: user, success: true }) })
  );

  await page.route('**/api/auth/users/permissions', route =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ data: { permissions: MOCK_PERMISSIONS }, success: true }) })
  );

  return { user, token };
}
