import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { expect, test } from '@playwright/test';

import { loadApprovedDifferences, runtimeConfig } from '../lib/config';
import { captureSide } from '../lib/capture';
import { compareCaptures, compareRepeatScreenshots } from '../lib/compare';
import { slugify, writeJson } from '../lib/files';
import { RuntimeResetManager } from '../lib/runtime-reset';
import type {
  ApprovedDifferenceRegistry,
  CaptureResult,
  RuntimeConfig,
  ScenarioManifest,
} from '../lib/types';

const selectedGroupId = Number(process.env.E2E_GROUP ?? '0');
const manifest = JSON.parse(
  readFileSync(resolve(process.cwd(), 'e2e/migration/scenarios.json'), 'utf8')
) as ScenarioManifest;
const selectedGroup = manifest.groups.find(
  candidate => candidate.id === selectedGroupId
);
if (!selectedGroup) {
  throw new Error(`unknown scenario group ${selectedGroupId}`);
}
const campaignGroups =
  selectedGroupId === 0
    ? [selectedGroup]
    : manifest.groups.filter(
        candidate => candidate.id >= 0 && candidate.id <= selectedGroupId
      );
let approvedDifferences: ApprovedDifferenceRegistry;
let config: RuntimeConfig;
let runtime: RuntimeResetManager;

for (const group of campaignGroups) {
  if (!group.scenarios || group.scenarios.length === 0) {
    throw new Error(`group ${group.id} has no executable scenarios`);
  }
  if ((manifest.matrices[group.matrix] ?? []).length === 0) {
    throw new Error(`group ${group.id} has no scenario matrix`);
  }
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

test.beforeAll(async () => {
  config = await runtimeConfig(selectedGroupId);
  runtime = new RuntimeResetManager(config);
  approvedDifferences = await loadApprovedDifferences();
});

for (const group of campaignGroups) {
  const scenarios = group.scenarios ?? [];
  const matrices = manifest.matrices[group.matrix];
  for (const scenario of scenarios) {
    for (const matrix of matrices) {
      test(`group ${group.id}: ${scenario.id} [${matrix.id}]`, async ({
        browser,
        browserName,
      }, testInfo) => {
        test.skip(
          browserName !== 'chromium',
          'Review-sized visual evidence is captured in Chromium; PR 9 has a separate cross-browser functional gate.'
        );
        const testRoot = resolve(
          config.artifactRoot,
          slugify(scenario.id),
          slugify(matrix.id)
        );
        const captures: Record<'source' | 'target', CaptureResult[]> = {
          source: [],
          target: [],
        };

        for (let repeat = 1; repeat <= group.repeat; repeat += 1) {
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
              fixtureUrl: config.fixtureUrl,
              fixtureToken: config.fixtureToken,
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
              fixtureUrl: config.fixtureUrl,
              fixtureToken: config.fixtureToken,
            });
            captures.source.push(source);
            captures.target.push(target);

            for (const capture of [source, target]) {
              const declaredStatus = scenario.outcomes.find(
                outcome =>
                  outcome.type === 'status' &&
                  (outcome.side === undefined ||
                    outcome.side === 'both' ||
                    outcome.side === capture.side)
              );
              expect(
                capture.status,
                `${capture.side} document must return a status`
              ).not.toBeNull();
              if (declaredStatus?.type === 'status') {
                expect(
                  capture.status,
                  `${capture.side} document must return its declared dependency/error status`
                ).toBe(declaredStatus.value);
              } else {
                expect(
                  capture.status ?? 599,
                  `${capture.side} document must not return an unexplained server error`
                ).toBeLessThan(500);
              }
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
              expect(
                capture.outcomeChecks.filter(check => !check.passed),
                `${capture.side} declarative outcomes`
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
              approvedDifferences,
            });
            expect(
              comparison.approvedDifference,
              `${scenario.id}/${matrix.id} visual delta ${comparison.differencePercent}% exceeds its allowed ${comparison.maximumAllowedDifferencePercent}%`
            ).toBe(true);
            await testInfo.attach(
              `${scenario.id}-${matrix.id}-repeat-${repeat}-contact-sheet`,
              {
                path: comparison.contactSheet,
                contentType: 'image/png',
              }
            );
          } finally {
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
            new Set(captures.source.map(capture => capture.domSha256)).size ===
            1,
          sourceAccessibility:
            new Set(captures.source.map(capture => capture.accessibilitySha256))
              .size === 1,
          targetScreenshot: screenshotEquivalence.target.equivalent,
          targetDom:
            new Set(captures.target.map(capture => capture.domSha256)).size ===
            1,
          targetAccessibility:
            new Set(captures.target.map(capture => capture.accessibilitySha256))
              .size === 1,
        };
        const reproducibility = {
          schemaVersion: 1,
          groupId: group.id,
          scenarioId: scenario.id,
          matrixId: matrix.id,
          repeats: group.repeat,
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
          `captures must be reproducible across ${group.repeat} clean repeats`
        ).toBe(true);
      });
    }
  }
}
