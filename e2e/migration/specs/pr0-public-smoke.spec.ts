import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { expect, test } from '@playwright/test';

import { runtimeConfig } from '../lib/config';
import { captureSide } from '../lib/capture';
import { compareCaptures, compareRepeatScreenshots } from '../lib/compare';
import { slugify, writeJson } from '../lib/files';
import { RuntimeResetManager } from '../lib/runtime-reset';
import type {
  CaptureResult,
  RuntimeConfig,
  ScenarioManifest,
} from '../lib/types';

const groupId = Number(process.env.E2E_GROUP ?? '0');
const manifest = JSON.parse(
  readFileSync(
    resolve(process.cwd(), 'e2e/migration/scenarios.json'),
    'utf8'
  )
) as ScenarioManifest;
const group = manifest.groups.find(candidate => candidate.id === groupId);
if (!group) {
  throw new Error(`unknown scenario group ${groupId}`);
}
const matrices = manifest.matrices[group.matrix ?? 'full'];
let config: RuntimeConfig;
let runtime: RuntimeResetManager;

if (!group.scenarios || group.scenarios.length === 0) {
  throw new Error(`group ${groupId} has no executable scenarios`);
}
if (matrices.length === 0) {
  throw new Error(`group ${groupId} has no scenario matrix`);
}

const baseUrl = (
  side: 'source' | 'target',
  surface: 'frontend' | 'admin'
): string => {
  if (side === 'source') {
    return surface === 'frontend'
      ? config.sourceFrontendUrl
      : config.sourceAdminUrl;
  }
  return surface === 'frontend'
    ? config.targetFrontendUrl
    : config.targetAdminUrl;
};

test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  config = await runtimeConfig(groupId);
  runtime = new RuntimeResetManager(config);
});

for (const scenario of group.scenarios) {
  for (const matrix of matrices) {
    test(`${scenario.id} [${matrix.id}]`, async ({ browser }, testInfo) => {
      const testRoot = resolve(
        config.artifactRoot,
        slugify(scenario.id),
        slugify(matrix.id)
      );
      const captures: Record<'source' | 'target', CaptureResult[]> = {
        source: [],
        target: [],
      };
      const repeats = group.repeat ?? 1;

      for (let repeat = 1; repeat <= repeats; repeat += 1) {
        const repeatRoot = resolve(testRoot, `repeat-${repeat}`);
        await runtime.reset(
          `${scenario.id}/${matrix.id}/repeat-${repeat}`,
          'pre',
          resolve(repeatRoot, 'reset-pre.json')
        );
        try {
          const source = await captureSide({
            browser,
            side: 'source',
            scenario,
            matrixId: matrix.id,
            repeat,
            baseUrl: baseUrl('source', scenario.surface),
            artifactDirectory: repeatRoot,
            viewport: matrix.viewport,
            colorScheme: matrix.colorScheme,
          });
          const target = await captureSide({
            browser,
            side: 'target',
            scenario,
            matrixId: matrix.id,
            repeat,
            baseUrl: baseUrl('target', scenario.surface),
            artifactDirectory: repeatRoot,
            viewport: matrix.viewport,
            colorScheme: matrix.colorScheme,
          });
          captures.source.push(source);
          captures.target.push(target);

          for (const capture of [source, target]) {
            expect(
              capture.status,
              `${capture.side} document must return a status`
            ).not.toBeNull();
            expect(
              capture.status ?? 599,
              `${capture.side} document must not return a server error`
            ).toBeLessThan(500);
            expect(
              capture.bodyTextLength,
              `${capture.side} page must render meaningful text`
            ).toBeGreaterThan(50);
            expect(capture.pageErrors, `${capture.side} page errors`).toEqual(
              []
            );
            expect(
              capture.consoleErrors,
              `${capture.side} console errors`
            ).toEqual([]);
            expect(
              capture.failedRequests,
              `${capture.side} failed network requests`
            ).toEqual([]);
          }

          if (scenario.expectedSourcePath !== undefined) {
            expect(new URL(source.finalUrl).pathname).toBe(
              scenario.expectedSourcePath
            );
          }
          if (scenario.expectedTargetPath !== undefined) {
            expect(new URL(target.finalUrl).pathname).toBe(
              scenario.expectedTargetPath
            );
          }

          const comparison = await compareCaptures({
            source,
            target,
            artifactDirectory: repeatRoot,
            captureOnly: group.comparisonGate === 'capture-only',
          });
          await testInfo.attach(
            `${scenario.id}-${matrix.id}-repeat-${repeat}-contact-sheet`,
            {
              path: comparison.contactSheet,
              contentType: 'image/png',
            }
          );
        } finally {
          // Rollback is a failure-path invariant: assertion, capture, or
          // comparison failures must still clear every scenario mutation and
          // fixture request counter before Playwright advances or exits.
          await runtime.reset(
            `${scenario.id}/${matrix.id}/repeat-${repeat}`,
            'post',
            resolve(repeatRoot, 'reset-post.json')
          );
        }
      }

      const screenshotEquivalence = {
        source: await compareRepeatScreenshots({
          captures: captures.source,
          artifactDirectory: testRoot,
          side: 'source',
        }),
        target: await compareRepeatScreenshots({
          captures: captures.target,
          artifactDirectory: testRoot,
          side: 'target',
        }),
      };
      const checks = {
        sourceScreenshot: screenshotEquivalence.source.equivalent,
        sourceDom:
          new Set(captures.source.map(capture => capture.domSha256)).size === 1,
        sourceAccessibility:
          new Set(captures.source.map(capture => capture.accessibilitySha256))
            .size === 1,
        targetScreenshot: screenshotEquivalence.target.equivalent,
        targetDom:
          new Set(captures.target.map(capture => capture.domSha256)).size === 1,
        targetAccessibility:
          new Set(captures.target.map(capture => capture.accessibilitySha256))
            .size === 1,
      };
      const reproducibility = {
        schemaVersion: 1,
        scenarioId: scenario.id,
        matrixId: matrix.id,
        repeats,
        checks,
        screenshotEquivalence,
        passed: Object.values(checks).every(Boolean),
        source: captures.source.map(capture => ({
          repeat: capture.repeat,
          screenshotSha256: capture.screenshotSha256,
          domSha256: capture.domSha256,
          accessibilitySha256: capture.accessibilitySha256,
        })),
        target: captures.target.map(capture => ({
          repeat: capture.repeat,
          screenshotSha256: capture.screenshotSha256,
          domSha256: capture.domSha256,
          accessibilitySha256: capture.accessibilitySha256,
        })),
      };
      await writeJson(
        resolve(testRoot, 'reproducibility.json'),
        reproducibility
      );
      expect(
        reproducibility.passed,
        `captures must be reproducible across ${repeats} clean repeats`
      ).toBe(true);
    });
  }
}
