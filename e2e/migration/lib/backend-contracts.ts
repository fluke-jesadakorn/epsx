import { spawnSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

import { sha256File, slugify, writeJson } from './files';
import type {
  BackendContractRepeat,
  BackendContractReproducibility,
  RuntimeConfig,
  ScenarioGroup,
} from './types';
import type { RuntimeResetManager } from './runtime-reset';

interface TestSummary {
  passedTests: number;
  failedTests: number;
  ignoredTests: number;
  allSuitesPassed: boolean;
}

function parseTestSummary(output: string): TestSummary {
  const matches = [
    ...output.matchAll(
      /test result: (ok|FAILED)\.\s+(\d+) passed;\s+(\d+) failed;\s+(\d+) ignored;/g
    ),
  ];
  if (matches.length === 0) {
    throw new Error('backend contract command emitted no Rust test summary');
  }
  return matches.reduce<TestSummary>(
    (summary, match) => ({
      passedTests: summary.passedTests + Number(match[2]),
      failedTests: summary.failedTests + Number(match[3]),
      ignoredTests: summary.ignoredTests + Number(match[4]),
      allSuitesPassed: summary.allSuitesPassed && match[1] === 'ok',
    }),
    {
      passedTests: 0,
      failedTests: 0,
      ignoredTests: 0,
      allSuitesPassed: true,
    }
  );
}

async function runContractRepeat(options: {
  config: RuntimeConfig;
  environment: NodeJS.ProcessEnv;
  group: ScenarioGroup;
  resetManager: RuntimeResetManager;
  suite: NonNullable<ScenarioGroup['backendContracts']>[number];
  repeat: number;
}): Promise<BackendContractRepeat> {
  const { config, environment, group, repeat, resetManager, suite } = options;
  const repeatRoot = resolve(
    config.artifactRoot,
    'backend-contracts',
    slugify(suite.id),
    `repeat-${repeat}`
  );
  await mkdir(repeatRoot, { recursive: true });
  const preResetPath = resolve(repeatRoot, 'reset-pre.json');
  const postResetPath = resolve(repeatRoot, 'reset-post.json');
  const outputPath = resolve(repeatRoot, 'cargo-test.log');
  await resetManager.reset(
    `${suite.id}/repeat-${repeat}`,
    'pre',
    preResetPath
  );

  const startedAt = new Date().toISOString();
  const started = performance.now();
  const command = [suite.executable, ...suite.arguments];
  process.stdout.write(`$ ${command.join(' ')}\n`);
  let result: ReturnType<typeof spawnSync>;
  try {
    result = spawnSync(suite.executable, suite.arguments, {
      cwd: config.repoRoot,
      env: environment,
      encoding: 'utf8',
      maxBuffer: 64 * 1024 * 1024,
    });
  } finally {
    await resetManager.reset(
      `${suite.id}/repeat-${repeat}`,
      'post',
      postResetPath
    );
  }
  const completedAt = new Date().toISOString();
  const output = [
    `$ ${command.join(' ')}`,
    '',
    '--- stdout ---',
    result.stdout,
    '',
    '--- stderr ---',
    result.stderr,
    '',
  ].join('\n');
  await writeFile(outputPath, output, 'utf8');
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
  if (result.error !== undefined) {
    throw result.error;
  }

  const summary = parseTestSummary(output);
  const exitCode = result.status ?? 1;
  const contractResult: BackendContractRepeat = {
    schemaVersion: 1,
    groupId: group.id,
    suiteId: suite.id,
    repeat,
    command,
    startedAt,
    completedAt,
    durationMs: Math.round(performance.now() - started),
    exitCode,
    passedTests: summary.passedTests,
    failedTests: summary.failedTests,
    ignoredTests: summary.ignoredTests,
    outputPath,
    outputSha256: await sha256File(outputPath),
    preResetPath,
    postResetPath,
    passed:
      exitCode === 0 &&
      summary.allSuitesPassed &&
      summary.passedTests > 0 &&
      summary.failedTests === 0 &&
      summary.ignoredTests === 0,
  };
  await writeJson(resolve(repeatRoot, 'result.json'), contractResult);
  if (!contractResult.passed) {
    throw new Error(
      `backend contract ${suite.id} repeat ${repeat} failed; see ${outputPath}`
    );
  }
  return contractResult;
}

export async function runBackendContracts(options: {
  config: RuntimeConfig;
  environment: NodeJS.ProcessEnv;
  groups: ScenarioGroup[];
  resetManager: RuntimeResetManager;
}): Promise<void> {
  const { config, environment, groups, resetManager } = options;
  for (const group of groups) {
    for (const suite of group.backendContracts ?? []) {
      const results: BackendContractRepeat[] = [];
      for (let repeat = 1; repeat <= group.repeat; repeat += 1) {
        results.push(
          await runContractRepeat({
            config,
            environment,
            group,
            resetManager,
            suite,
            repeat,
          })
        );
      }
      const checks = {
        allRunsPassed: results.every(result => result.passed),
        stablePassedTestCount:
          new Set(results.map(result => result.passedTests)).size === 1,
        stableIgnoredTestCount:
          new Set(results.map(result => result.ignoredTests)).size === 1,
        noIgnoredTests: results.every(result => result.ignoredTests === 0),
      };
      const reproducibility: BackendContractReproducibility = {
        schemaVersion: 1,
        groupId: group.id,
        suiteId: suite.id,
        title: suite.title,
        repeats: group.repeat,
        claims: suite.claims,
        sources: suite.sources,
        results,
        checks,
        passed: Object.values(checks).every(Boolean),
      };
      await writeJson(
        resolve(
          config.artifactRoot,
          'backend-contracts',
          slugify(suite.id),
          'reproducibility.json'
        ),
        reproducibility
      );
      if (!reproducibility.passed) {
        throw new Error(
          `backend contract ${suite.id} was not reproducible across ${group.repeat} clean runs`
        );
      }
    }
  }
}
