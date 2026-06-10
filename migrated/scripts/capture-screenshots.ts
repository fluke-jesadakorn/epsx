#!/usr/bin/env bun
/**
 * Capture screenshots of every route from a running EPSX frontend or admin.
 *
 * Usage:
 *   bun scripts/capture-screenshots.ts <baseUrl> <outputDir> [--admin]
 *
 * Example:
 *   bun scripts/capture-screenshots.ts http://localhost:3000 ./screenshots/baseline/nextjs-frontend
 *   bun scripts/capture-screenshots.ts http://localhost:3001 ./screenshots/baseline/nextjs-admin --admin
 *   bun scripts/capture-screenshots.ts http://localhost:4000 ./screenshots/dioxus-frontend
 *   bun scripts/capture-screenshots.ts http://localhost:4001 ./screenshots/dioxus-admin --admin
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
) {
  await mkdir(outDir, { recursive: true });
  const ctx = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    ignoreHTTPSErrors: true,
  });
  const page = await ctx.newPage();

  for (const route of routes) {
    const url = `${baseUrl}${route}`;
    const safeName = route === '/' ? 'home' : route.replace(/^\/+/, '').replace(/\//g, '__').replace(/[^a-zA-Z0-9_=-]/g, '_');
    const file = join(outDir, `${safeName}.png`);
    try {
      const resp = await page.goto(url, { waitUntil: 'networkidle', timeout: 20000 });
      const status = resp?.status() ?? 0;
      await page.waitForLoadState('domcontentloaded');
      await page.waitForTimeout(2500);
      await page.screenshot({ path: file, fullPage: true });
      console.log(`  [${status}] ${route} -> ${file}`);
    } catch (e) {
      console.error(`  [ERR] ${route}: ${(e as Error).message}`);
      try { await page.screenshot({ path: file, fullPage: true }); } catch {}
    }
  }

  await ctx.close();
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length < 2) {
    console.error('Usage: capture-screenshots.ts <baseUrl> <outputDir> [--admin]');
    process.exit(1);
  }
  const baseUrl = args[0]!;
  const outDir = args[1]!;
  const isAdmin = args.includes('--admin');

  const routes = isAdmin ? ADMIN_ROUTES : FRONTEND_ROUTES;
  console.log(`Capturing ${routes.length} ${isAdmin ? 'admin' : 'frontend'} routes from ${baseUrl} -> ${outDir}`);

  const browser = await chromium.launch();
  try {
    await capture(browser, baseUrl, routes, outDir);
  } finally {
    await browser.close();
  }
}

main().catch(e => { console.error(e); process.exit(1); });
