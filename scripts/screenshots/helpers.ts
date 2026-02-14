import type { Page } from '@playwright/test';
import path from 'path';
import fs from 'fs';

/** Build a fake JWT with proper base64url-encoded payload so jose.decodeJwt() works */
function makeJWT(claims: Record<string, unknown>): string {
  const hdr = Buffer.from('{"alg":"HS256","typ":"JWT"}').toString('base64url');
  const payload = Buffer.from(JSON.stringify(claims)).toString('base64url');
  return `${hdr}.${payload}.e2e_test_sig`;
}

export const USER_PERMS = [
  'epsx:analytics:view', 'epsx:portfolio:view', 'epsx:plans:view',
  'epsx:notifications:view', 'epsx:developer:view', 'epsx:profile:manage',
];

export const ADMIN_PERMS = [
  'admin:users:manage', 'admin:wallets:manage', 'admin:permissions:manage',
  'admin:analytics:view', 'admin:notifications:manage', 'admin:plans:manage',
  'admin:settings:manage', 'admin:audit:view', 'admin:developer:manage',
];

const USER_WALLET = '0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68';
const ADMIN_WALLET = '0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B';

const USER_TOKEN = makeJWT({
  sub: USER_WALLET, wallet_address: USER_WALLET,
  permissions: USER_PERMS, iat: 1700000000, exp: 9999999999,
});

const ADMIN_TOKEN = makeJWT({
  sub: ADMIN_WALLET, wallet_address: ADMIN_WALLET,
  permissions: ADMIN_PERMS, iat: 1700000000, exp: 9999999999,
});

/**
 * Complete UserInfoResponse objects matching shared/auth/client.ts interface.
 * Critical: `access` field is required for SharedWeb3AuthClient.isAuthenticated()
 */
export const MOCK_USER = {
  sub: USER_WALLET,
  wallet_address: USER_WALLET,
  tier_level: 'pro',
  auth_method: 'web3_siwe',
  permissions: USER_PERMS,
  email: 'e2e@epsx.test',
  access: USER_TOKEN,
  packageTier: 'pro',
  group: 'user',
  is_admin: false,
};

export const MOCK_ADMIN = {
  sub: ADMIN_WALLET,
  wallet_address: ADMIN_WALLET,
  tier_level: 'admin',
  auth_method: 'web3_siwe',
  permissions: ADMIN_PERMS,
  email: 'admin-e2e@epsx.test',
  access: ADMIN_TOKEN,
  packageTier: 'admin',
  group: 'admin',
  is_admin: true,
};

export async function setupAuth(page: Page, user: typeof MOCK_USER, perms: string[]) {
  const token = user === MOCK_ADMIN ? ADMIN_TOKEN : USER_TOKEN;

  await page.context().addCookies([
    { name: 'epsx.access_token', value: token, domain: 'localhost', path: '/', httpOnly: true, secure: false, sameSite: 'Lax' },
    { name: 'epsx.refresh_token', value: `refresh_${token}`, domain: 'localhost', path: '/', httpOnly: true, secure: false, sameSite: 'Lax' },
    { name: 'epsx.user', value: encodeURIComponent(JSON.stringify(user)), domain: 'localhost', path: '/', httpOnly: false, secure: false, sameSite: 'Lax' },
    { name: 'epsx.sid', value: `sid_${user.sub}`, domain: 'localhost', path: '/', httpOnly: false, secure: false, sameSite: 'Lax' },
    { name: 'epsx.expires_at', value: String(Date.now() + 86400000), domain: 'localhost', path: '/', httpOnly: false, secure: false, sameSite: 'Lax' },
    { name: 'epsx.auth_time', value: String(Date.now()), domain: 'localhost', path: '/', httpOnly: false, secure: false, sameSite: 'Lax' },
  ]);

  // Intercept all auth API calls (browser-side)
  await page.route('**/api/auth/**', route =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        data: {
          valid: true, authenticated: true, user, token,
          permissions: perms, wallet_address: user.wallet_address,
        },
        success: true,
      }),
    })
  );
}

/**
 * Wait for client-side auth to resolve after navigation.
 * Dismisses auth modals/overlays if they appear.
 */
export async function waitForAuth(page: Page) {
  // Wait for React hydration + auth effect to complete
  await page.waitForTimeout(2000);

  // Dismiss any auth modals/overlays that might be blocking content
  try {
    // Close shared AuthModal if open
    const closeBtn = page.locator('[class*="auth"] button:has-text("×"), [class*="modal"] button[aria-label="Close"], .fixed button:has-text("×")').first();
    if (await closeBtn.isVisible({ timeout: 500 })) {
      await closeBtn.click();
      await page.waitForTimeout(300);
    }
  } catch {
    // No auth modal to dismiss
  }

  // If "Admin Access Required" card is visible, try clicking away or waiting
  try {
    const authCard = page.locator('text=Admin Access Required, text=Connect Admin Wallet, text=Verify your admin permissions').first();
    if (await authCard.isVisible({ timeout: 500 })) {
      // Press Escape to close any overlays
      await page.keyboard.press('Escape');
      await page.waitForTimeout(500);
    }
  } catch {
    // No auth blocking element
  }
}

export async function capture(page: Page, dir: string, name: string) {
  const outDir = path.resolve(dir, 'public/screenshots');
  fs.mkdirSync(outDir, { recursive: true });
  await page.screenshot({ path: path.join(outDir, `${name}.png`), fullPage: false });
}

export async function waitFor(page: Page, selector: string, timeout = 5000) {
  try {
    await page.waitForSelector(selector, { state: 'visible', timeout });
  } catch {
    // Element may not appear in mocked state - continue
  }
}

export async function clickAndWait(page: Page, selector: string, waitMs = 500) {
  try {
    await page.click(selector, { timeout: 3000 });
    await page.waitForTimeout(waitMs);
  } catch {
    // Element may not be clickable in mocked state - continue
  }
}

export async function fillField(page: Page, selector: string, value: string) {
  try {
    await page.fill(selector, value, { timeout: 3000 });
    await page.waitForTimeout(300);
  } catch {
    // Field may not exist in mocked state - continue
  }
}

export async function tryClick(page: Page, selector: string) {
  try {
    const el = page.locator(selector).first();
    if (await el.isVisible({ timeout: 2000 })) {
      await el.click();
      await page.waitForTimeout(500);
    }
  } catch {
    // Continue silently
  }
}
