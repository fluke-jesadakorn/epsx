import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';

import { readJson } from './files';
import type {
  BaselineLock,
  RuntimeConfig,
  ScenarioGroup,
  ScenarioManifest,
} from './types';

export const repoRoot = execFileSync(
  'git',
  ['rev-parse', '--show-toplevel'],
  {
    cwd: process.cwd(),
    encoding: 'utf8',
  }
).trim();
const migrationRoot = resolve(repoRoot, 'e2e/migration');
export const defaultSourceRoot = resolve(
  tmpdir(),
  'epsx-migration-e2e-source-373bd231'
);
export const manifestPath = resolve(migrationRoot, 'scenarios.json');
export const baselineLockPath = resolve(migrationRoot, 'baseline.lock.json');

export async function loadManifest(): Promise<ScenarioManifest> {
  const manifest = await readJson<ScenarioManifest>(manifestPath);
  if (manifest.schemaVersion !== 1 || !Array.isArray(manifest.groups)) {
    throw new Error('scenario manifest schemaVersion 1 is required');
  }
  return manifest;
}

export async function loadBaselineLock(): Promise<BaselineLock> {
  const lock = await readJson<BaselineLock>(baselineLockPath);
  if (
    lock.schemaVersion !== 1 ||
    lock.immutable !== true ||
    !/^[0-9a-f]{40}$/.test(lock.commit) ||
    lock.packageManager !== 'bun@1.3.4' ||
    lock.dependencyLock !== 'bun.lock' ||
    !/^[0-9a-f]{64}$/.test(lock.dependencyLockSha256)
  ) {
    throw new Error(
      'immutable baseline lock with a full 40-character SHA is required'
    );
  }
  return lock;
}

function gitRevision(cwd: string, revision = 'HEAD'): string {
  return execFileSync(
    'git',
    ['rev-parse', '--verify', `${revision}^{commit}`],
    {
      cwd,
      encoding: 'utf8',
    }
  ).trim();
}

function requireLoopbackUrl(raw: string, label: string): URL {
  const url = new URL(raw);
  if (!['127.0.0.1', 'localhost', '::1'].includes(url.hostname)) {
    throw new Error(
      `${label} must use a loopback host, received ${url.hostname}`
    );
  }
  return url;
}

function requiredSourceRoot(): string {
  const sourceRoot = resolve(
    process.env.E2E_SOURCE_ROOT ?? defaultSourceRoot
  );
  if (!existsSync(sourceRoot)) {
    throw new Error(
      `pinned source checkout is missing at ${sourceRoot}; run the prepare-source command`
    );
  }
  return sourceRoot;
}

// Environment validation is intentionally exhaustive: every mutating
// dependency must independently prove it is loopback and run-scoped.
// eslint-disable-next-line complexity
export async function runtimeConfig(groupId = 0): Promise<RuntimeConfig> {
  const lock = await loadBaselineLock();
  const sourceRoot = requiredSourceRoot();
  const sourceCommit = gitRevision(sourceRoot);
  if (sourceCommit !== lock.commit) {
    throw new Error(
      `source checkout ${sourceRoot} is ${sourceCommit}, expected ${lock.commit}`
    );
  }
  const targetCommit = gitRevision(repoRoot);
  const runRoot = resolve(
    process.env.E2E_RUN_ROOT ?? resolve(migrationRoot, '.runtime')
  );
  const runId = process.env.E2E_RUN_ID ?? `group-${groupId}`;
  if (!/^[a-zA-Z0-9._-]+$/.test(runId)) {
    throw new Error(
      'E2E_RUN_ID may contain only letters, digits, dot, underscore, or dash'
    );
  }

  const config: RuntimeConfig = {
    repoRoot,
    sourceRoot,
    runRoot,
    artifactRoot: resolve(
      process.env.E2E_ARTIFACT_ROOT ?? resolve(migrationRoot, 'artifacts')
    ),
    evidenceRoot: resolve(
      process.env.E2E_EVIDENCE_ROOT ??
        resolve(repoRoot, `docs/e2e/pr${groupId}/evidence`)
    ),
    groupId,
    sourceCommit,
    targetCommit,
    sourceFrontendUrl:
      process.env.E2E_SOURCE_FRONTEND_URL ?? 'http://127.0.0.1:4100',
    sourceAdminUrl: process.env.E2E_SOURCE_ADMIN_URL ?? 'http://127.0.0.1:4101',
    targetFrontendUrl:
      process.env.E2E_TARGET_FRONTEND_URL ?? 'http://127.0.0.1:4200',
    targetAdminUrl: process.env.E2E_TARGET_ADMIN_URL ?? 'http://127.0.0.1:4201',
    fixtureUrl: process.env.E2E_FIXTURE_URL ?? 'http://127.0.0.1:48080',
    fixtureToken: process.env.E2E_FIXTURE_TOKEN ?? 'epsx-e2e-local-reset-token',
    postgresAdminUrl:
      process.env.E2E_POSTGRES_ADMIN_URL ??
      'postgresql://epsx_e2e:epsx_e2e@127.0.0.1:15432/postgres',
    postgresTemplateDatabase:
      process.env.E2E_POSTGRES_TEMPLATE_DATABASE ??
      `epsx_e2e_pr${groupId}_template`,
    postgresRuntimeDatabase:
      process.env.E2E_POSTGRES_RUNTIME_DATABASE ??
      `epsx_e2e_pr${groupId}_runtime`,
    redisUrl: process.env.E2E_REDIS_URL ?? 'redis://127.0.0.1:16379/0',
    redisPrefix: process.env.E2E_REDIS_PREFIX ?? `epsx:e2e:${runId}:`,
    anvilUrl: process.env.E2E_ANVIL_URL ?? 'http://127.0.0.1:18545',
    allowRuntimeMutation: process.env.E2E_ALLOW_RUNTIME_MUTATION === '1',
  };

  for (const [label, value] of [
    ['source frontend URL', config.sourceFrontendUrl],
    ['source admin URL', config.sourceAdminUrl],
    ['target frontend URL', config.targetFrontendUrl],
    ['target admin URL', config.targetAdminUrl],
    ['fixture URL', config.fixtureUrl],
    ['PostgreSQL admin URL', config.postgresAdminUrl],
    ['Redis URL', config.redisUrl],
    ['Anvil URL', config.anvilUrl],
  ]) {
    requireLoopbackUrl(value, label);
  }
  for (const name of [
    config.postgresTemplateDatabase,
    config.postgresRuntimeDatabase,
  ]) {
    if (!/^epsx_e2e_[a-zA-Z0-9_]+$/.test(name)) {
      throw new Error(`scratch database name is not safely scoped: ${name}`);
    }
  }
  if (!config.redisPrefix.startsWith('epsx:e2e:')) {
    throw new Error('Redis prefix must start with epsx:e2e:');
  }
  return config;
}

export async function selectedGroup(groupId: number): Promise<ScenarioGroup> {
  const manifest = await loadManifest();
  const group = manifest.groups.find(candidate => candidate.id === groupId);
  if (!group) {
    throw new Error(`unknown scenario group ${groupId}`);
  }
  return group;
}
