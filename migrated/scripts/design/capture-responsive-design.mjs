#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
import { chromium } from 'playwright';

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
const OUTPUT_ROOT = path.join(ROOT, '.generated', 'design-captures');
const FIXTURE_WALLET = '0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68';
const MAX_CAPTURE_DIMENSION = Number(process.env.DESIGN_CAPTURE_MAX_DIMENSION ?? '2048');

const VIEWPORTS = {
  tablet: {
    width: 834,
    height: 1194,
    isMobile: false,
    hasTouch: true,
    deviceScaleFactor: 2,
  },
  mobile: {
    width: 390,
    height: 844,
    isMobile: true,
    hasTouch: true,
    deviceScaleFactor: 3,
  },
};

const MODES = ['light', 'dark'];
const TARGET_VIEWPORTS = (process.env.DESIGN_CAPTURE_VIEWPORTS ?? 'tablet,mobile')
  .split(',')
  .map((value) => value.trim())
  .filter((value) => value in VIEWPORTS);
const TARGET_STATES = new Set(
  (process.env.DESIGN_CAPTURE_STATES ?? '')
    .split(',')
    .map((value) => value.trim())
    .filter(Boolean)
);

const FRONTEND_BASE_URL = process.env.FRONTEND_BASE_URL ?? 'http://localhost:3000';
const ADMIN_BASE_URL = process.env.ADMIN_BASE_URL ?? 'http://localhost:3001';

function createRoute(route, { auth = false } = {}) {
  return { route, auth };
}

const FRONTEND_STATES = {
  about: createRoute('/about'),
  'access-denied': createRoute('/access-denied'),
  account: createRoute('/account', { auth: true }),
  'account-credits': createRoute('/account/credits', { auth: true }),
  'account-payments': createRoute('/account', { auth: true }),
  'account-prefs': createRoute('/account', { auth: true }),
  analytics: createRoute('/analytics'),
  'analytics-auth': createRoute('/analytics', { auth: true }),
  'analytics-default': createRoute('/analytics?sort_by=growth_factor', { auth: true }),
  'analytics-filter-country': createRoute('/analytics?country=US', { auth: true }),
  'analytics-filter-sector': createRoute('/analytics?sector=Technology', { auth: true }),
  'analytics-pagination': createRoute('/analytics?page=2', { auth: true }),
  'analytics-search': createRoute('/analytics?search=AAPL', { auth: true }),
  'analytics-sort': createRoute('/analytics?sort_by=ranking_position', { auth: true }),
  auth: createRoute('/auth'),
  dashboard: createRoute('/dashboard', { auth: true }),
  developer: createRoute('/developer', { auth: true }),
  'developer-create-key': createRoute('/developer', { auth: true }),
  'developer-docs': createRoute('/developer/docs', { auth: true }),
  'developer-usage': createRoute('/developer/usage', { auth: true }),
  home: createRoute('/'),
  notifications: createRoute('/notifications', { auth: true }),
  'notifications-default': createRoute('/notifications', { auth: true }),
  'notifications-empty': createRoute('/notifications?page=999', { auth: true }),
  'notifications-filter-priority': createRoute('/notifications?priority=high', { auth: true }),
  'notifications-filter-type': createRoute('/notifications?type=security', { auth: true }),
  'notifications-search': createRoute('/notifications', { auth: true }),
  offline: createRoute('/offline'),
  payment: createRoute('/payment?planId=1', { auth: true }),
  'payment-detail': createRoute('/payment/link/1', { auth: true }),
  permissions: createRoute('/permissions', { auth: true }),
  plans: createRoute('/plans'),
  portfolio: createRoute('/portfolio', { auth: true }),
  'portfolio-search': createRoute('/portfolio', { auth: true }),
  privacy: createRoute('/privacy'),
  profile: createRoute('/profile', { auth: true }),
  'profile-edit': createRoute('/profile', { auth: true }),
  terms: createRoute('/terms'),
};

const ADMIN_STATES = {
  'admin-access-denied': createRoute('/access-denied'),
  'admin-access-overview': createRoute('/wallet-management/access', { auth: true }),
  'admin-access-permissions': createRoute('/wallet-management/access', { auth: true }),
  'admin-access-plan-detail': createRoute('/wallet-management/access/plans', { auth: true }),
  'admin-access-plans': createRoute('/wallet-management/access/plans', { auth: true }),
  'admin-analytics': createRoute('/analytics', { auth: true }),
  'admin-api-docs': createRoute('/developer-portal?tab=docs', { auth: true }),
  'admin-api-key-create': createRoute('/developer-portal/api-keys/create', { auth: true }),
  'admin-audit-log': createRoute('/audit-log', { auth: true }),
  'admin-audit-log-category': createRoute('/audit-log', { auth: true }),
  'admin-audit-log-search': createRoute('/audit-log', { auth: true }),
  'admin-auth': createRoute('/'),
  'admin-dashboard': createRoute('/', { auth: true }),
  'admin-dashboard-page': createRoute('/', { auth: true }),
  'admin-developer-portal': createRoute('/developer-portal', { auth: true }),
  'admin-notification-create': createRoute('/notifications/create', { auth: true }),
  'admin-notification-create-filled': createRoute('/notifications/create', { auth: true }),
  'admin-notification-manage': createRoute('/notifications/manage', { auth: true }),
  'admin-notifications': createRoute('/notifications', { auth: true }),
  'admin-payments': createRoute('/payments', { auth: true }),
  'admin-plan-edit': createRoute('/wallet-management/access/plans', { auth: true }),
  'admin-plan-new': createRoute('/wallet-management/access/plans', { auth: true }),
  'admin-profile': createRoute('/settings', { auth: true }),
  'admin-request-access': createRoute('/'),
  'admin-settings': createRoute('/settings', { auth: true }),
  'admin-settings-appearance': createRoute('/settings?tab=appearance', { auth: true }),
  'admin-settings-notifications': createRoute('/settings?tab=notifications', { auth: true }),
  'admin-settings-security': createRoute('/settings?tab=security', { auth: true }),
  'admin-subscription-detail': createRoute('/payments?tab=user-access', { auth: true }),
  'admin-subscription-new': createRoute('/payments?tab=payment-links', { auth: true }),
  'admin-subscription-new-external': createRoute('/payments?tab=payment-links', { auth: true }),
  'admin-subscription-new-filled': createRoute('/payments?tab=payment-links', { auth: true }),
  'admin-unauthorized': createRoute('/unauthorized'),
  'admin-wallet-activity': createRoute('/wallet-management/credits', { auth: true }),
  'admin-wallet-credits': createRoute('/wallet-management/credits', { auth: true }),
  'admin-wallet-detail': createRoute(`/wallet-management/${FIXTURE_WALLET}`, { auth: true }),
  'admin-wallet-disable': createRoute(`/wallet-management/wallets/${FIXTURE_WALLET}/disable`, { auth: true }),
  'admin-wallet-management': createRoute('/wallet-management/wallets', { auth: true }),
  'admin-wallets-default': createRoute('/wallet-management/wallets', { auth: true }),
  'admin-wallets-edit': createRoute('/wallet-management/wallets', { auth: true }),
  'admin-wallets-filter-platform': createRoute('/wallet-management/wallets', { auth: true }),
  'admin-wallets-filter-status': createRoute('/wallet-management/wallets', { auth: true }),
  'admin-wallets-list': createRoute('/wallet-management/wallets', { auth: true }),
  'admin-wallets-search': createRoute('/wallet-management/wallets', { auth: true }),
};

const CAPTURE_MATRIX = {
  frontend: {
    baseUrl: FRONTEND_BASE_URL,
    states: FRONTEND_STATES,
  },
  admin: {
    baseUrl: ADMIN_BASE_URL,
    states: ADMIN_STATES,
  },
};

function ensureDir(target) {
  fs.mkdirSync(target, { recursive: true });
}

function normalizeCaptureSize(filePath) {
  execFileSync('sips', ['-Z', String(MAX_CAPTURE_DIMENSION), filePath], {
    stdio: 'ignore',
  });
}

function withBypass(baseUrl, route, auth) {
  const url = new URL(route, baseUrl);
  if (auth) {
    url.searchParams.set('__design_bypass', '1');
  }
  return url.toString();
}

async function navigate(page, url) {
  await page.goto(url, { waitUntil: 'networkidle' });
  await page.waitForTimeout(600);
}

async function firstVisible(locator) {
  try {
    if (await locator.first().isVisible({ timeout: 1500 })) {
      return locator.first();
    }
  } catch {}
  return null;
}

async function clickIfVisible(page, locatorFactory) {
  const locator = await firstVisible(locatorFactory(page));
  if (locator !== null) {
    await locator.click();
    await page.waitForTimeout(500);
    return true;
  }
  return false;
}

async function fillIfVisible(page, locatorFactory, value) {
  const locator = await firstVisible(locatorFactory(page));
  if (locator !== null) {
    await locator.fill(value);
    await page.waitForTimeout(500);
    return true;
  }
  return false;
}

async function selectOption(page, labelText, optionText) {
  const labels = page.locator('label').filter({ hasText: new RegExp(`^${labelText}$`, 'i') });
  const label = await firstVisible(() => labels);
  if (label !== null) {
    const parent = label.locator('..');
    const combobox = await firstVisible(() => parent.locator('[role="combobox"]'));
    if (combobox !== null) {
      await combobox.click();
      await page.waitForTimeout(300);
      const option = await firstVisible(() => page.getByRole('option', { name: new RegExp(optionText, 'i') }));
      if (option !== null) {
        await option.click();
        await page.waitForTimeout(500);
        return true;
      }
    }

    const inputId = await label.getAttribute('for');
    if (inputId !== null) {
      const select = page.locator(`#${inputId}`);
      if ((await select.count()) > 0) {
        try {
          await select.selectOption({ label: optionText });
          await page.waitForTimeout(500);
          return true;
        } catch {}
        try {
          await select.selectOption(optionText);
          await page.waitForTimeout(500);
          return true;
        } catch {}
      }
    }
  }

  const select = await firstVisible(() => page.getByRole('combobox', { name: new RegExp(labelText, 'i') }));
  if (select !== null) {
    await select.click();
    await page.waitForTimeout(300);
    const option = await firstVisible(() => page.getByRole('option', { name: new RegExp(optionText, 'i') }));
    if (option !== null) {
      await option.click();
      await page.waitForTimeout(500);
      return true;
    }
  }

  return false;
}

async function runFrontendAction(page, stateName) {
  switch (stateName) {
    case 'portfolio-search':
      await fillIfVisible(
        page,
        (currentPage) => currentPage.getByPlaceholder(/search stocks to add to watchlist/i),
        'AAPL'
      );
      break;
    case 'profile-edit':
      await clickIfVisible(page, (currentPage) => currentPage.getByRole('tab', { name: /email/i }));
      await clickIfVisible(page, (currentPage) => currentPage.getByRole('button', { name: /change email/i }));
      break;
    default:
      break;
  }
}

async function runAdminAction(page, stateName) {
  switch (stateName) {
    case 'admin-audit-log-search':
      await fillIfVisible(
        page,
        (currentPage) => currentPage.getByPlaceholder(/search by actor, action, or target/i),
        'wallet'
      );
      break;
    case 'admin-audit-log-category':
      await clickIfVisible(page, (currentPage) => currentPage.getByRole('button', { name: /security/i }));
      break;
    case 'admin-notification-create-filled':
      await fillIfVisible(
        page,
        (currentPage) => currentPage.getByPlaceholder(/0x\.\.\./i),
        FIXTURE_WALLET
      );
      await fillIfVisible(
        page,
        (currentPage) => currentPage.getByPlaceholder(/payload designation/i),
        'Design capture alert'
      );
      await fillIfVisible(
        page,
        (currentPage) => currentPage.getByPlaceholder(/enter transmission data/i),
        'Responsive mobile and tablet preview state.'
      );
      break;
    case 'admin-plan-new':
      await clickIfVisible(page, (currentPage) => currentPage.locator('button').filter({ has: currentPage.locator('svg.lucide-plus') }));
      break;
    case 'admin-plan-edit':
    case 'admin-access-plan-detail':
      await clickIfVisible(page, (currentPage) => currentPage.locator('a[href*="/wallet-management/access/plans/"], button').first());
      break;
    case 'admin-wallet-activity':
      await clickIfVisible(page, (currentPage) => currentPage.getByRole('button', { name: /credit history/i }));
      break;
    case 'admin-wallets-filter-platform':
      await selectOption(page, 'Platform', 'Analytics');
      break;
    case 'admin-wallets-filter-status':
      await selectOption(page, 'Status', 'Disabled');
      break;
    case 'admin-wallets-search':
      await fillIfVisible(
        page,
        (currentPage) => currentPage.getByPlaceholder(/search address, label, or note/i),
        FIXTURE_WALLET.slice(0, 10)
      );
      break;
    default:
      break;
  }
}

async function runStateAction(page, app, stateName) {
  if (app === 'frontend') {
    await runFrontendAction(page, stateName);
    return;
  }
  await runAdminAction(page, stateName);
}

async function createContext(browser, baseUrl, mode, viewport) {
  const context = await browser.newContext({
    viewport: {
      width: viewport.width,
      height: viewport.height,
    },
    deviceScaleFactor: viewport.deviceScaleFactor,
    isMobile: viewport.isMobile,
    hasTouch: viewport.hasTouch,
    colorScheme: mode,
  });

  await context.addInitScript((theme) => {
    window.localStorage.setItem('theme', theme);
    document.cookie = `theme=${theme}; path=/; SameSite=Lax`;
    document.cookie = `__theme_mode=${theme}; path=/; SameSite=Lax`;
  }, mode);

  await context.addCookies([
    { url: baseUrl, name: 'theme', value: mode },
    { url: baseUrl, name: '__theme_mode', value: mode },
  ]);

  return context;
}

async function captureState(page, app, baseUrl, mode, viewportName, stateName, routeConfig, results) {
  const targetUrl = withBypass(baseUrl, routeConfig.route, routeConfig.auth);
  const outputDir = path.join(OUTPUT_ROOT, app, mode, viewportName);
  const outputFile = path.join(outputDir, `${stateName}.jpg`);

  ensureDir(outputDir);

  try {
    await navigate(page, targetUrl);
    await runStateAction(page, app, stateName);
    await page.screenshot({
      path: outputFile,
      fullPage: true,
      type: 'jpeg',
      quality: 85,
    });
    normalizeCaptureSize(outputFile);
    results.generated.push({
      app,
      mode,
      viewport: viewportName,
      state: stateName,
      file: outputFile,
    });
    console.log(`[capture] ok ${app}/${mode}/${viewportName}/${stateName}`);
  } catch (error) {
    results.failed.push({
      app,
      mode,
      viewport: viewportName,
      state: stateName,
      file: outputFile,
      error: error instanceof Error ? error.message : String(error),
    });
    console.error(`[capture] failed ${app}/${mode}/${viewportName}/${stateName}: ${results.failed.at(-1)?.error}`);
  }
}

async function main() {
  ensureDir(OUTPUT_ROOT);

  const browser = await chromium.launch({ headless: true });
  const results = {
    generated: [],
    failed: [],
    skipped: [],
  };

  try {
    for (const [app, config] of Object.entries(CAPTURE_MATRIX)) {
      for (const mode of MODES) {
        for (const viewportName of TARGET_VIEWPORTS) {
          const viewport = VIEWPORTS[viewportName];
          const context = await createContext(browser, config.baseUrl, mode, viewport);
          const page = await context.newPage();

          for (const [stateName, routeConfig] of Object.entries(config.states)) {
            if (TARGET_STATES.size > 0 && !TARGET_STATES.has(stateName)) {
              results.skipped.push({
                app,
                mode,
                viewport: viewportName,
                state: stateName,
                reason: 'state-filter',
              });
              continue;
            }
            await captureState(page, app, config.baseUrl, mode, viewportName, stateName, routeConfig, results);
          }

          await context.close();
        }
      }
    }
  } finally {
    await browser.close();
  }

  const reportPath = path.join(OUTPUT_ROOT, 'capture-report.json');
  fs.writeFileSync(reportPath, JSON.stringify(results, null, 2));
  console.log(`[capture] report ${reportPath}`);

  if (results.failed.length > 0) {
    process.exitCode = 1;
  }
}

await main();
