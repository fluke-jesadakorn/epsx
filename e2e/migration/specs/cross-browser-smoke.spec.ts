import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { expect, test } from '@playwright/test';

import { runtimeConfig } from '../lib/config';
import { writeJson } from '../lib/files';
import { RuntimeResetManager } from '../lib/runtime-reset';
import type { RuntimeConfig, Scenario, ScenarioManifest } from '../lib/types';

const selectedGroupId = Number(process.env.E2E_GROUP ?? '0');
const manifest = JSON.parse(
  readFileSync(resolve(process.cwd(), 'e2e/migration/scenarios.json'), 'utf8')
) as ScenarioManifest;
const finalGroup = manifest.groups.find(group => group.id === 9);
if (!finalGroup?.scenarios) {
  throw new Error('PR 9 scenarios are required');
}

let config: RuntimeConfig;
let runtime: RuntimeResetManager;

async function fixtureSession(scenario: Scenario): Promise<string | undefined> {
  if (scenario.state.session !== 'authenticated') {
    return undefined;
  }
  const audience = scenario.state.audience;
  if (audience === undefined) {
    throw new Error(`scenario ${scenario.id} has no audience`);
  }
  const url = new URL('/__e2e/session', config.fixtureUrl);
  url.searchParams.set('audience', audience);
  url.searchParams.set(
    'permissions',
    (scenario.state.permissions ?? []).join(' ')
  );
  const response = await fetch(url, {
    headers: { 'x-epsx-e2e-token': config.fixtureToken },
  });
  if (!response.ok) {
    throw new Error(`fixture session failed with HTTP ${response.status}`);
  }
  return ((await response.json()) as { accessToken: string }).accessToken;
}

function targetBaseUrl(scenario: Scenario): string {
  return scenario.surface === 'admin'
    ? config.targetAdminUrl
    : config.targetFrontendUrl;
}

test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  config = await runtimeConfig(selectedGroupId);
  runtime = new RuntimeResetManager(config);
});

for (const scenario of finalGroup.scenarios) {
  test(`cross-browser ${scenario.id}`, async ({ browser, browserName }) => {
    test.skip(selectedGroupId !== 9, 'Cross-browser smoke is owned by PR 9');
    const proofRoot = resolve(
      config.artifactRoot,
      'cross-browser',
      browserName,
      scenario.id.replaceAll(/[^a-zA-Z0-9._-]/g, '-')
    );
    for (let repeat = 1; repeat <= 2; repeat += 1) {
      await runtime.reset(
        `${scenario.id}/${browserName}/repeat-${repeat}`,
        'pre',
        resolve(proofRoot, `repeat-${repeat}-reset-pre.json`)
      );
      const context = await browser.newContext();
      const page = await context.newPage();
      try {
        const token = await fixtureSession(scenario);
        if (token !== undefined) {
          await context.addCookies([
            {
              name:
                scenario.surface === 'admin'
                  ? 'epsx.admin.access_token'
                  : 'epsx.frontend.access_token',
              value: token,
              url: targetBaseUrl(scenario),
              httpOnly: true,
              sameSite: 'Lax',
            },
          ]);
        }
        const consoleErrors: string[] = [];
        const pageErrors: string[] = [];
        const hydrationWarnings: string[] = [];
        page.on('console', message => {
          if (message.type() === 'error' || message.type() === 'assert') {
            consoleErrors.push(message.text());
          }
          if (
            /hydration|hydrated|did not match|server rendered html/i.test(
              message.text()
            )
          ) {
            hydrationWarnings.push(message.text());
          }
        });
        page.on('pageerror', error => pageErrors.push(error.message));
        const startedAt = performance.now();
        const response = await page.goto(
          new URL(scenario.path, targetBaseUrl(scenario)).toString(),
          { waitUntil: 'domcontentloaded' }
        );
        const navigationMs = performance.now() - startedAt;
        await page
          .waitForLoadState('networkidle', { timeout: 7_500 })
          .catch(() => undefined);
        const firstText = (await page.locator('body').innerText()).trim();
        const accessibility = await page.locator('body').ariaSnapshot();
        await page.keyboard.press('Tab');
        const focusVisible = await page.evaluate(
          () =>
            document.activeElement !== null &&
            document.activeElement !== document.body
        );
        await page.reload({ waitUntil: 'domcontentloaded' });
        const reloadedText = (await page.locator('body').innerText()).trim();
        const reloadedAccessibility = await page.locator('body').ariaSnapshot();
        let offlineProof:
          | {
              navigatorOnline: boolean;
              bodyTextLength: number;
            }
          | undefined;
        if (scenario.id === 'pr9.frontend.offline') {
          await context.setOffline(true);
          offlineProof = {
            navigatorOnline: await page.evaluate(() => navigator.onLine),
            bodyTextLength: (await page.locator('body').innerText()).trim()
              .length,
          };
          await context.setOffline(false);
          await page.reload({ waitUntil: 'domcontentloaded' });
        }
        const accessibilitySha256 = createHash('sha256')
          .update(accessibility)
          .digest('hex');
        const reloadedAccessibilitySha256 = createHash('sha256')
          .update(reloadedAccessibility)
          .digest('hex');
        const proof = {
          schemaVersion: 1,
          groupId: 9,
          scenarioId: scenario.id,
          browserName,
          repeat,
          status: response?.status() ?? null,
          navigationMs,
          bodyTextLength: firstText.length,
          reloadBodyTextLength: reloadedText.length,
          accessibilityLength: accessibility.length,
          reloadAccessibilityLength: reloadedAccessibility.length,
          accessibilitySha256,
          reloadedAccessibilitySha256,
          focusVisible,
          hydrationWarnings,
          offlineProof: offlineProof ?? null,
          consoleErrors,
          pageErrors,
        };
        await writeJson(
          resolve(proofRoot, `repeat-${repeat}-functional.json`),
          proof
        );
        expect(proof.status).not.toBeNull();
        expect(proof.status ?? 599).toBeLessThan(500);
        expect(proof.bodyTextLength).toBeGreaterThan(50);
        expect(proof.reloadBodyTextLength).toBeGreaterThan(50);
        expect(proof.reloadBodyTextLength).toBe(proof.bodyTextLength);
        expect(proof.accessibilityLength).toBeGreaterThan(20);
        expect(proof.reloadAccessibilityLength).toBe(proof.accessibilityLength);
        expect(proof.focusVisible).toBe(true);
        expect(proof.hydrationWarnings).toEqual([]);
        if (proof.offlineProof !== null) {
          expect(proof.offlineProof.navigatorOnline).toBe(false);
          expect(proof.offlineProof.bodyTextLength).toBeGreaterThan(50);
        }
        expect(proof.consoleErrors).toEqual([]);
        expect(proof.pageErrors).toEqual([]);
        expect(proof.navigationMs).toBeLessThan(15_000);
      } finally {
        await context.close();
        await runtime.reset(
          `${scenario.id}/${browserName}/repeat-${repeat}`,
          'post',
          resolve(proofRoot, `repeat-${repeat}-reset-post.json`)
        );
      }
    }
  });
}
