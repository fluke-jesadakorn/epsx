#!/usr/bin/env bun
/**
 * Capture screenshots of every route from a running EPSX frontend.
 *
 * Usage:
 *   bun scripts/capture-screenshots.ts <baseUrl> <outputDir> [authCookie]
 *
 * Example:
 *   bun scripts/capture-screenshots.ts http://localhost:3000 ./screenshots/baseline/frontend
 *   bun scripts/capture-screenshots.ts http://localhost:3001 ./screenshots/baseline/admin
 */
import { chromium, type Browser, type Page } from 'playwright';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';

const FRONTEND_ROUTES = [
  '/', '/auth', '/dashboard', '/profile', '/account', '/account/credits',
  '/analytics', '/chat', '/chat/history', '/chat/sample-1',
  '/contact', '/about', '/news', '/news/welcome-to-epsx',
  '/notifications', '/payment', '/payment/subscription/1', '/permissions',
  '/plans', '/portfolio', '/developer', '/developer/usage', '/developer/docs',
  '/manual', '/access-denied', '/offline', '/privacy', '/terms',
  '/not-a-real-page', // 404
];

const ADMIN_ROUTES = [
  '/', '/analytics', '/audit-log', '/chat', '/chat/sample-1',
  '/developer-portal', '/developer-portal/api-keys/create',
  '/media', '/news', '/news/create', '/news/1/edit',
  '/notifications', '/notifications/create', '/notifications/manage',
  '/payments', '/policies', '/settings', '/unauthorized',
  '/wallet-management', '/wallet-management/wallets',
  '/wallet-management/0x1234', '/wallet-management/0x1234/disable',
  '/wallet-management/credits', '/wallet-management/access',
  '/wallet-management/access/plans', '/wallet-management/access/plans/1',
  '/access-denied', '/auth',
  '/not-a-real-page', // 404
];

async function capture(
  browser: Browser,
  baseUrl: string,
  routes: string[],
  outDir: string,
  authCookie?: { name: string; value: string; domain: string; path: string },
) {
  await mkdir(outDir, { recursive: true });
  const ctx = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    ignoreHTTPSErrors: true,
  });
  if (authCookie) await ctx.addCookies([authCookie]);
  const page = await ctx.newPage();

  for (const route of routes) {
    const url = `${baseUrl}${route}`;
    const safeName = route === '/' ? 'home' : route.replace(/^\/+/, '').replace(/\//g, '__').replace(/[^a-zA-Z0-9_=-]/g, '_');
    const file = join(outDir, `${safeName}.png`);
    try {
      const resp = await page.goto(url, { waitUntil: 'networkidle', timeout: 15000 });
      const status = resp?.status() ?? 0;
      // Wait for either content or error
      await page.waitForLoadState('domcontentloaded');
      await page.waitForTimeout(500);
      await page.screenshot({ path: file, fullPage: true });
      console.log(`  [${status}] ${route} -> ${file}`);
    } catch (e) {
      console.error(`  [ERR] ${route}: ${(e as Error).message}`);
      // Still take a screenshot of whatever rendered
      try { await page.screenshot({ path: file, fullPage: true }); } catch {}
    }
  }

  await ctx.close();
}

async function main() {
  const [, , baseUrl, outDir, cookieHeader] = process.argv;
  if (!baseUrl || !outDir) {
    console.error('Usage: capture-screenshots.ts <baseUrl> <outDir> [cookie]');
    process.exit(1);
  }

  const isAdmin = baseUrl.includes('3001') || baseUrl.includes('admin');
  const routes = isAdmin ? ADMIN_ROUTES : FRONTEND_ROUTES;

  let authCookie: { name: string; value: string; domain: string; path: string } | undefined;
  if (cookieHeader) {
    // cookieHeader = "name=value"
    const [name, ...rest] = cookieHeader.split('=');
    authCookie = {
      name: name!,
      value: rest.join('='),
      domain: new URL(baseUrl).hostname,
      path: '/',
    };
  }

  console.log(`Capturing ${routes.length} routes from ${baseUrl} -> ${outDir}`);
  console.log(`Auth cookie: ${authCookie ? `${authCookie.name}=${authCookie.value.slice(0, 10)}...` : 'none'}`);

  const browser = await chromium.launch();
  try {
    await capture(browser, baseUrl, routes, outDir, authCookie);
  } finally {
    await browser.close();
  }
}

main().catch(e => { console.error(e); process.exit(1); });
