#!/usr/bin/env bun

import {
  execFileSync,
  spawn,
  type ChildProcess,
  type SpawnOptions,
} from 'node:child_process';
import { createWriteStream, existsSync } from 'node:fs';
import { mkdir, rm } from 'node:fs/promises';
import { createServer } from 'node:net';
import { dirname, resolve } from 'node:path';

import {
  baselineLockPath,
  defaultSourceRoot,
  loadApprovedDifferences,
  loadBaselineLock,
  loadManifest,
  repoRoot,
  runtimeConfig,
} from './lib/config';
import { runBackendContracts } from './lib/backend-contracts';
import { listFiles, readJson, sha256File } from './lib/files';
import { generateReport, verifyArtifactManifest } from './lib/report';
import { RuntimeResetManager } from './lib/runtime-reset';
import type { ScenarioGroup, ScenarioManifest } from './lib/types';

interface ManagedProcess {
  name: string;
  child: ChildProcess;
  logPath: string;
  processGroupId?: number;
}

interface RouteContract {
  applications: {
    frontend: { expectedCount: number; routes: Array<{ path: string }> };
    admin: { expectedCount: number; routes: Array<{ path: string }> };
  };
}

const migrationRoot = resolve(repoRoot, 'e2e/migration');
const composePath = resolve(migrationRoot, 'runtime/compose.yml');
const lockedBunVersion = '1.3.4';
const args = process.argv.slice(2);
const command = args[0] ?? 'doctor';

function option(name: string, fallback?: string): string | undefined {
  const index = args.indexOf(name);
  if (index === -1) {
    return fallback;
  }
  const value = args[index + 1];
  if (!value || value.startsWith('--')) {
    throw new Error(`${name} requires a value`);
  }
  return value;
}

function groupId(): number {
  const parsed = Number(option('--group', process.env.E2E_GROUP ?? '0'));
  if (!Number.isInteger(parsed) || parsed < 0 || parsed > 9) {
    throw new Error(`invalid feature group ${parsed}`);
  }
  return parsed;
}

function contractRouteMatches(
  contractPath: string,
  scenarioPath: string
): boolean {
  const pathname = new URL(scenarioPath, 'http://epsx.invalid').pathname;
  const pattern = contractPath
    .split('/')
    .map(segment =>
      segment.startsWith(':') ? '[^/]+' : escapedGrepLiteral(segment)
    )
    .join('/');
  return new RegExp(`^${pattern}$`).test(pathname);
}

function safeEnvironment(
  additions: Record<string, string | undefined> = {}
): NodeJS.ProcessEnv {
  const allowed = [
    'PATH',
    'HOME',
    'USER',
    'SHELL',
    'TMPDIR',
    'LANG',
    'LC_ALL',
    'TERM',
    'CI',
    'CARGO_HOME',
    'RUSTUP_HOME',
    'BUN_INSTALL',
    'RUSTC_WRAPPER',
    'CARGO_TARGET_DIR',
    'CARGO_INCREMENTAL',
    // Preserve an explicitly selected local Docker daemon. This lets the
    // campaign use a dedicated non-Kubernetes E2E profile without mutating the
    // user's active Docker context.
    'DOCKER_HOST',
    'DOCKER_CONTEXT',
    'DOCKER_TLS_VERIFY',
    'DOCKER_CERT_PATH',
  ];
  const environment: NodeJS.ProcessEnv = {};
  for (const key of allowed) {
    if (process.env[key] !== undefined) {
      environment[key] = process.env[key];
    }
  }
  for (const [key, value] of Object.entries(additions)) {
    if (value !== undefined) {
      environment[key] = value;
    }
  }
  return environment;
}

function runCommand(
  executable: string,
  commandArgs: string[],
  options: {
    cwd?: string;
    env?: NodeJS.ProcessEnv;
  } = {}
): void {
  process.stdout.write(`$ ${executable} ${commandArgs.join(' ')}\n`);
  execFileSync(executable, commandArgs, {
    cwd: options.cwd,
    env: options.env,
    stdio: 'inherit',
  });
}

async function startManagedProcess(options: {
  name: string;
  executable: string;
  commandArgs: string[];
  spawnOptions: SpawnOptions;
  logPath: string;
}): Promise<ManagedProcess> {
  const { commandArgs, executable, logPath, name, spawnOptions } = options;
  await mkdir(dirname(logPath), { recursive: true });
  const output = createWriteStream(logPath, { flags: 'w' });
  const useProcessGroup = process.platform !== 'win32';
  const child = spawn(executable, commandArgs, {
    ...spawnOptions,
    detached: useProcessGroup,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  child.stdout.pipe(output);
  child.stderr.pipe(output);
  child.once('exit', (code, signal) => {
    output.write(
      `\n[process-exit] name=${name} code=${String(code)} signal=${String(signal)}\n`
    );
    output.end();
  });
  return {
    name,
    child,
    logPath,
    processGroupId: useProcessGroup ? child.pid : undefined,
  };
}

async function stopManagedProcess(processInfo: ManagedProcess): Promise<void> {
  const { child, name, processGroupId } = processInfo;
  const groupIsAlive = (): boolean => {
    if (child.exitCode !== null || child.signalCode !== null) {
      return false;
    }
    if (processGroupId === undefined) {
      return true;
    }
    try {
      process.kill(-processGroupId, 0);
      return true;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ESRCH') {
        return false;
      }
      throw error;
    }
  };
  if (!groupIsAlive()) {
    return;
  }
  signalManagedProcess(processInfo, 'SIGTERM');
  const deadline = Date.now() + 10_000;
  while (groupIsAlive() && Date.now() < deadline) {
    await new Promise(resolvePromise => setTimeout(resolvePromise, 100));
  }
  if (groupIsAlive()) {
    process.stderr.write(
      `${name} did not stop after SIGTERM; sending SIGKILL\n`
    );
    signalManagedProcess(processInfo, 'SIGKILL');
  }
}

function signalManagedProcess(
  processInfo: ManagedProcess,
  signal: NodeJS.Signals
): void {
  try {
    if (processInfo.processGroupId === undefined) {
      processInfo.child.kill(signal);
    } else {
      process.kill(-processInfo.processGroupId, signal);
    }
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ESRCH') {
      throw error;
    }
  }
}

async function assertTcpPortAvailable(
  rawUrl: string,
  label: string
): Promise<void> {
  const url = new URL(rawUrl);
  const port = Number(url.port);
  await new Promise<void>((resolvePromise, rejectPromise) => {
    const server = createServer();
    server.unref();
    server.once('error', error => {
      rejectPromise(
        new Error(
          `${label} requires free ${url.hostname}:${port}: ${error.message}`
        )
      );
    });
    server.listen({ host: url.hostname, port, exclusive: true }, () =>
      server.close(() => resolvePromise())
    );
  });
}

async function waitForUrl(
  url: string,
  processInfo: ManagedProcess | undefined,
  timeoutMs = 120_000
): Promise<void> {
  const started = Date.now();
  let lastError = '';
  while (Date.now() - started < timeoutMs) {
    if (
      processInfo &&
      (processInfo.child.exitCode !== null ||
        processInfo.child.signalCode !== null)
    ) {
      throw new Error(
        `${processInfo.name} exited before ${url} became ready; see ${processInfo.logPath}`
      );
    }
    try {
      const response = await fetch(url, { redirect: 'manual' });
      if (response.status < 500) {
        return;
      }
      lastError = `HTTP ${response.status}`;
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
    }
    await new Promise(resolvePromise => setTimeout(resolvePromise, 500));
  }
  throw new Error(`timed out waiting for ${url}: ${lastError}`);
}

async function prepareSource(): Promise<string> {
  const lock = await loadBaselineLock();
  const sourceRoot = resolve(process.env.E2E_SOURCE_ROOT ?? defaultSourceRoot);
  if (!existsSync(sourceRoot)) {
    // A killed prior run can leave a registered worktree whose directory no
    // longer exists. Remove only this deterministic source registration before
    // recreating it; never prune or mutate unrelated user worktrees.
    try {
      execFileSync('git', ['worktree', 'remove', '--force', sourceRoot], {
        cwd: repoRoot,
        stdio: 'ignore',
      });
    } catch {
      // No matching registration is the normal first-run case.
    }
    runCommand(
      'git',
      ['worktree', 'add', '--detach', sourceRoot, lock.commit],
      { cwd: repoRoot }
    );
  }
  const actual = execFileSync(
    'git',
    ['rev-parse', '--verify', 'HEAD^{commit}'],
    { cwd: sourceRoot, encoding: 'utf8' }
  ).trim();
  if (actual !== lock.commit) {
    throw new Error(
      `source checkout ${sourceRoot} is ${actual}; expected immutable ${lock.commit}`
    );
  }
  return sourceRoot;
}

// Validation is intentionally exhaustive and fail-closed because this command
// is the branch/CI policy boundary for all ten cumulative groups.
// eslint-disable-next-line max-lines-per-function, complexity, sonarjs/cognitive-complexity
async function doctor(): Promise<void> {
  const lock = await loadBaselineLock();
  const manifest = await loadManifest();
  const approvedDifferences = await loadApprovedDifferences();
  execFileSync('git', ['cat-file', '-e', `${lock.commit}^{commit}`], {
    cwd: repoRoot,
  });
  execFileSync('git', ['merge-base', '--is-ancestor', lock.commit, lock.ref], {
    cwd: repoRoot,
  });
  if (manifest.baselineLock !== 'e2e/migration/baseline.lock.json') {
    throw new Error('scenario manifest does not reference the baseline lock');
  }
  if (
    manifest.approvedDifferences !== 'e2e/migration/approved-differences.json'
  ) {
    throw new Error(
      'scenario manifest does not reference the approved-difference registry'
    );
  }
  const contract = await readJson<RouteContract>(
    resolve(repoRoot, manifest.routeContract)
  );
  const bypasses = await readJson<{ schemaVersion: number; items: unknown[] }>(
    resolve(migrationRoot, 'bypasses.json')
  );
  if (bypasses.schemaVersion !== 1 || !Array.isArray(bypasses.items)) {
    throw new Error('bypass registry schemaVersion 1 is required');
  }
  if (bypasses.items.length > 0) {
    throw new Error('migration campaign cannot pass with scenario bypasses');
  }
  const expectedCategories = new Set([
    'backend-authority',
    'security',
    'wallet-siwe-legal-accuracy',
    'unsupported-feature-removal',
  ]);
  if (
    approvedDifferences.allowedCategories.length !== expectedCategories.size ||
    approvedDifferences.allowedCategories.some(
      category => !expectedCategories.has(category)
    )
  ) {
    throw new Error(
      'approved differences may use only authority, security, legal-accuracy, or unsupported-feature categories'
    );
  }
  const approvedKeys = new Set<string>();
  for (const item of approvedDifferences.items) {
    if (
      item.reason.trim() === '' ||
      item.sourceEvidence.trim() === '' ||
      item.targetEvidence.trim() === '' ||
      item.matrixIds.length === 0 ||
      item.maximumDifferencePercent <= 1 ||
      item.maximumDifferencePercent > 100 ||
      !approvedDifferences.allowedCategories.includes(item.category)
    ) {
      throw new Error(
        `invalid approved-difference entry for ${item.scenarioId}`
      );
    }
    for (const matrixId of item.matrixIds) {
      const key = `${item.scenarioId}/${matrixId}`;
      if (approvedKeys.has(key)) {
        throw new Error(`duplicate approved-difference entry ${key}`);
      }
      approvedKeys.add(key);
    }
  }
  if (
    manifest.groups.length !== 10 ||
    manifest.groups.some((group, index) => group.id !== index)
  ) {
    throw new Error('scenario groups must be the ordered cumulative range 0-9');
  }
  for (const group of manifest.groups) {
    const matrices = manifest.matrices[group.matrix];
    if (
      !Array.isArray(matrices) ||
      matrices.length === 0 ||
      group.repeat < 2 ||
      group.surfaces.length === 0 ||
      group.states.length === 0 ||
      group.actions.length === 0 ||
      group.outcomes.length === 0 ||
      group.fixtureRequirements.length === 0 ||
      !Array.isArray(group.scenarios) ||
      group.scenarios.length === 0
    ) {
      throw new Error(
        `group ${group.id} must explicitly declare matrices, repeats, surfaces, states, actions, outcomes, fixtures, and scenarios`
      );
    }
    for (const scenario of group.scenarios) {
      if (
        !group.surfaces.includes(scenario.surface) ||
        scenario.state.id.trim() === '' ||
        !Array.isArray(scenario.actions) ||
        !Array.isArray(scenario.outcomes) ||
        scenario.outcomes.length === 0 ||
        !Array.isArray(scenario.fixtureRequirements)
      ) {
        throw new Error(
          `scenario ${scenario.id} is missing explicit state/action/outcome/surface/fixture data`
        );
      }
      if (
        scenario.state.session === 'authenticated' &&
        scenario.state.audience === undefined
      ) {
        throw new Error(
          `authenticated scenario ${scenario.id} is missing its audience`
        );
      }
      if (
        scenario.state.fixtureModeSide !== undefined &&
        scenario.state.fixtureMode === undefined
      ) {
        throw new Error(
          `scenario ${scenario.id} declares fixtureModeSide without fixtureMode`
        );
      }
      if (
        scenario.state.sourceAudience !== undefined &&
        scenario.state.session !== 'authenticated'
      ) {
        throw new Error(
          `scenario ${scenario.id} declares sourceAudience without an authenticated session`
        );
      }
    }
    for (const suite of group.backendContracts ?? []) {
      if (
        suite.id.trim() === '' ||
        suite.title.trim() === '' ||
        String(suite.executable) !== 'cargo' ||
        suite.arguments.length === 0 ||
        suite.claims.length === 0 ||
        suite.sources.length === 0 ||
        suite.claims.some(claim => claim.trim() === '') ||
        suite.sources.some(source => source.trim() === '')
      ) {
        throw new Error(`group ${group.id} has an invalid backend contract`);
      }
    }
  }
  for (const surface of ['frontend', 'admin'] as const) {
    const application = contract.applications[surface];
    if (application.routes.length !== application.expectedCount) {
      throw new Error(
        `${surface} route contract has ${application.routes.length}, expected ${application.expectedCount}`
      );
    }
    const assigned = new Set(
      manifest.groups
        .filter(group => group.id >= 1 && group.id <= 8)
        .flatMap(group => group.routes?.[surface] ?? [])
    );
    const missing = application.routes
      .map(({ path }) => path)
      .filter(path => !assigned.has(path));
    if (missing.length > 0) {
      throw new Error(
        `${surface} routes missing feature-group ownership: ${missing.join(', ')}`
      );
    }
    const finalScenarios = manifest.groups[9].scenarios?.filter(
      scenario => scenario.surface === surface
    );
    const matchedContracts = new Set<string>();
    for (const scenario of finalScenarios ?? []) {
      const scenarioPathname = new URL(scenario.path, 'http://epsx.invalid')
        .pathname;
      const exactMatches = application.routes.filter(
        route => route.path === scenarioPathname
      );
      const matches =
        exactMatches.length > 0
          ? exactMatches
          : application.routes.filter(route =>
              contractRouteMatches(route.path, scenario.path)
            );
      if (matches.length !== 1) {
        throw new Error(
          `PR 9 scenario ${scenario.id} matches ${matches.length} ${surface} route contracts`
        );
      }
      const [matched] = matches;
      if (matchedContracts.has(matched.path)) {
        throw new Error(
          `PR 9 duplicates ${surface} route contract ${matched.path}`
        );
      }
      matchedContracts.add(matched.path);
    }
    const missingFinalRoutes = application.routes
      .map(route => route.path)
      .filter(path => !matchedContracts.has(path));
    if (missingFinalRoutes.length > 0) {
      throw new Error(
        `PR 9 is missing ${surface} route contracts: ${missingFinalRoutes.join(', ')}`
      );
    }
  }
  const finalGroup = manifest.groups[9];
  const finalFrontend = finalGroup.scenarios?.filter(
    scenario => scenario.surface === 'frontend'
  );
  const finalAdmin = finalGroup.scenarios?.filter(
    scenario => scenario.surface === 'admin'
  );
  if (
    finalFrontend?.length !== contract.applications.frontend.expectedCount ||
    finalAdmin?.length !== contract.applications.admin.expectedCount ||
    finalGroup.requiredBypasses !== 0
  ) {
    throw new Error(
      'PR 9 must execute all 28 frontend and 27 admin routes with zero bypasses'
    );
  }
  for (const executable of ['bun', 'cargo', 'docker', 'git']) {
    execFileSync('sh', ['-c', `command -v "$1"`, 'doctor', executable], {
      stdio: 'ignore',
    });
  }
  process.stdout.write(
    `migration doctor: PASS — baseline=${lock.commit}, frontend=28, admin=27, groups=0-9, lock=${baselineLockPath}\n`
  );
}

async function safeCleanArtifactRoot(path: string): Promise<void> {
  const expected = resolve(migrationRoot, 'artifacts');
  if (resolve(path) !== expected) {
    throw new Error(`refusing to clean unexpected artifact root ${path}`);
  }
  await rm(path, { recursive: true, force: true });
  await mkdir(path, { recursive: true });
}

interface CleanupFailure {
  label: string;
  error: unknown;
}

async function cleanupRuntime(options: {
  composeEnvironment: NodeJS.ProcessEnv;
  composeStarted: boolean;
  config: Awaited<ReturnType<typeof runtimeConfig>>;
  managed: ManagedProcess[];
  resetManager: RuntimeResetManager | undefined;
  runtimeBootstrapped: boolean;
  selectedGroup: number;
}): Promise<CleanupFailure[]> {
  const {
    composeEnvironment,
    composeStarted,
    config,
    managed,
    resetManager,
    runtimeBootstrapped,
    selectedGroup,
  } = options;
  const failures: CleanupFailure[] = [];
  const attempt = async (
    label: string,
    operation: () => Promise<void> | void
  ): Promise<void> => {
    try {
      await operation();
    } catch (error) {
      failures.push({ label, error });
    }
  };
  const fixtureProcess = managed.find(
    processInfo => processInfo.name === 'fixture-server'
  );

  for (const processInfo of [...managed].reverse()) {
    if (processInfo !== fixtureProcess) {
      await attempt(`failed to stop ${processInfo.name}`, () =>
        stopManagedProcess(processInfo)
      );
    }
  }
  if (resetManager !== undefined && runtimeBootstrapped) {
    await attempt('final runtime rollback gate failed', async () => {
      // Application polling can race a reset while source/target processes are
      // alive. Stop them first, then prove one final rollback and smoke while
      // the isolated fixture and Compose dependencies remain available.
      await resetManager.reset(
        `group-${selectedGroup}/cli-finalize`,
        'post',
        resolve(config.artifactRoot, 'reset-final.json')
      );
      await resetManager.smoke();
    });
  }
  if (fixtureProcess !== undefined) {
    await attempt('failed to stop fixture-server', () =>
      stopManagedProcess(fixtureProcess)
    );
  }
  if (composeStarted) {
    await attempt('failed to remove isolated Compose runtime', () => {
      runCommand(
        'docker',
        ['compose', '-f', composePath, 'down', '--volumes', '--remove-orphans'],
        { cwd: repoRoot, env: composeEnvironment }
      );
    });
  }
  return failures;
}

function mergeCleanupFailures(
  runError: unknown,
  failures: CleanupFailure[]
): unknown {
  for (const failure of failures) {
    const message =
      failure.error instanceof Error
        ? failure.error.message
        : String(failure.error);
    process.stderr.write(`${failure.label}: ${message}\n`);
  }
  return runError ?? failures[0]?.error;
}

interface PlaywrightShard {
  grep: string;
  project?: string;
}

const maximumScenariosPerPlaywrightShard = 12;

function escapedGrepLiteral(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function scenarioShards(group: ScenarioGroup): PlaywrightShard[] {
  const scenarios = group.scenarios ?? [];
  const shards: PlaywrightShard[] = [];
  for (
    let offset = 0;
    offset < scenarios.length;
    offset += maximumScenariosPerPlaywrightShard
  ) {
    const ids = scenarios
      .slice(offset, offset + maximumScenariosPerPlaywrightShard)
      .map(scenario => escapedGrepLiteral(scenario.id));
    shards.push({
      grep: `group ${group.id}: (?:${ids.join('|')}) \\[`,
      project: 'migration-chromium',
    });
  }
  return shards;
}

function buildPlaywrightShards(
  manifest: ScenarioManifest,
  accumulatedGroups: ScenarioGroup[],
  explicitGrep?: string
): PlaywrightShard[] {
  if (explicitGrep !== undefined && explicitGrep !== '') {
    const exactGroup = /^group ([0-8]):$/.exec(explicitGrep);
    if (exactGroup !== null) {
      const group = accumulatedGroups.find(
        candidate => candidate.id === Number(exactGroup[1])
      );
      if (group !== undefined) {
        return scenarioShards(group);
      }
    }
    const requestedBrowser = process.env.E2E_BROWSER;
    const explicitProject =
      requestedBrowser === 'chromium' ||
      requestedBrowser === 'firefox' ||
      requestedBrowser === 'webkit'
        ? `migration-${requestedBrowser}`
        : 'migration-chromium';
    return [
      {
        grep: explicitGrep,
        // Explicit filters are review-sized Chromium evidence selectors;
        // cross-browser smoke remains owned by the generated PR 9 shards.
        // E2E_BROWSER is an opt-in smoke selector for local browser recovery.
        project: explicitProject,
      },
    ];
  }
  const shards: PlaywrightShard[] = [];
  for (const group of accumulatedGroups) {
    if (group.id !== 9) {
      shards.push(...scenarioShards(group));
      continue;
    }
    const matrices = manifest.matrices[group.matrix] ?? [];
    for (const surface of group.surfaces) {
      for (const matrix of matrices) {
        shards.push({
          grep: `group 9: pr9\\.${surface}\\..* \\[${matrix.id}\\]`,
          project: 'migration-chromium',
        });
      }
      for (const browser of group.browsers ?? []) {
        shards.push({
          grep: `cross-browser pr9\\.${surface}\\.`,
          project: `migration-${browser}`,
        });
      }
    }
  }
  return shards;
}

function playwrightArgumentsForShard(shard: PlaywrightShard): string[] {
  const argumentsForShard = [
    'playwright',
    'test',
    '--config',
    resolve(migrationRoot, 'playwright.config.ts'),
    '--grep',
    shard.grep,
  ];
  if (shard.project !== undefined) {
    argumentsForShard.push('--project', shard.project);
  }
  return argumentsForShard;
}

function selectPlaywrightShards(shards: PlaywrightShard[]): PlaywrightShard[] {
  const rawIndex = process.env.E2E_SHARD_INDEX?.trim();
  const rawCount = process.env.E2E_SHARD_COUNT?.trim();
  if (rawIndex === undefined && rawCount === undefined) {
    return shards;
  }
  if (rawIndex === undefined || rawCount === undefined) {
    throw new Error(
      'E2E_SHARD_INDEX and E2E_SHARD_COUNT must be provided together'
    );
  }
  const index = Number(rawIndex);
  const count = Number(rawCount);
  if (
    !Number.isInteger(index) ||
    !Number.isInteger(count) ||
    count < 1 ||
    index < 0 ||
    index >= count
  ) {
    throw new Error(
      `invalid Playwright shard selection index=${rawIndex} count=${rawCount}`
    );
  }
  return shards.filter((_, shardIndex) => shardIndex % count === index);
}

// Process orchestration is intentionally centralized so cleanup owns every
// child/container handle in one try/finally boundary.
// eslint-disable-next-line max-lines-per-function, complexity
async function run(): Promise<void> {
  const selectedGroup = groupId();
  await doctor();
  await prepareSource();
  const manifest = await loadManifest();
  const accumulatedGroups =
    selectedGroup === 0
      ? manifest.groups.filter(group => group.id === 0)
      : manifest.groups.filter(
          group => group.id >= 0 && group.id <= selectedGroup
        );
  const requiredSurfaces = new Set(
    accumulatedGroups.flatMap(group => group.surfaces)
  );
  const requiresAdmin = requiredSurfaces.has('admin');
  // The command itself owns the isolated Compose graph. Standalone reset
  // manager calls remain guarded unless their caller makes this explicit.
  process.env.E2E_ALLOW_RUNTIME_MUTATION = '1';
  const config = await runtimeConfig(selectedGroup);
  await Promise.all([
    assertTcpPortAvailable(config.fixtureUrl, 'fixture server'),
    assertTcpPortAvailable(config.sourceFrontendUrl, 'Next.js frontend source'),
    assertTcpPortAvailable(config.targetFrontendUrl, 'Rust frontend target'),
    ...(requiresAdmin
      ? [
          assertTcpPortAvailable(config.sourceAdminUrl, 'Next.js admin source'),
          assertTcpPortAvailable(config.targetAdminUrl, 'Rust admin target'),
        ]
      : []),
  ]);
  await safeCleanArtifactRoot(config.artifactRoot);
  await mkdir(config.runRoot, { recursive: true });
  const logsRoot = resolve(config.artifactRoot, 'server-logs');
  await mkdir(logsRoot, { recursive: true });

  const composeEnvironment = safeEnvironment({
    E2E_POSTGRES_PORT: new URL(config.postgresAdminUrl).port,
    E2E_REDIS_PORT: new URL(config.redisUrl).port,
    E2E_ANVIL_PORT: new URL(config.anvilUrl).port,
  });
  const managed: ManagedProcess[] = [];
  let composeStarted = false;
  let testStatus: number | undefined;
  let runError: unknown;
  let evidenceReady = false;
  let resetManager: RuntimeResetManager | undefined;
  let runtimeBootstrapped = false;
  let activePlaywright: ManagedProcess | undefined;
  let interruptedSignal: NodeJS.Signals | undefined;
  const handleInterrupt = (signal: NodeJS.Signals): void => {
    if (interruptedSignal !== undefined) {
      return;
    }
    interruptedSignal = signal;
    process.stderr.write(
      `migration e2e received ${signal}; stopping scoped child process groups\n`
    );
    if (activePlaywright !== undefined) {
      signalManagedProcess(activePlaywright, 'SIGTERM');
    }
    for (const processInfo of [...managed].reverse()) {
      signalManagedProcess(processInfo, 'SIGTERM');
    }
  };
  const handleSigint = (): void => handleInterrupt('SIGINT');
  const handleSigterm = (): void => handleInterrupt('SIGTERM');
  process.on('SIGINT', handleSigint);
  process.on('SIGTERM', handleSigterm);
  try {
    // `compose up` can create a partial project before returning an error, so
    // cleanup owns the project from the moment startup is attempted.
    composeStarted = true;
    runCommand('docker', ['compose', '-f', composePath, 'up', '-d', '--wait'], {
      cwd: repoRoot,
      env: composeEnvironment,
    });

    const fixture = await startManagedProcess({
      name: 'fixture-server',
      executable: 'bun',
      commandArgs: [resolve(migrationRoot, 'fixture-server.ts')],
      spawnOptions: {
        cwd: repoRoot,
        env: safeEnvironment({
          E2E_FIXTURE_PORT: new URL(config.fixtureUrl).port,
          E2E_FIXTURE_TOKEN: config.fixtureToken,
          E2E_FIXTURE_LOG: resolve(logsRoot, 'fixture.log'),
        }),
      },
      logPath: resolve(logsRoot, 'fixture-process.log'),
    });
    managed.push(fixture);
    await waitForUrl(`${config.fixtureUrl}/health`, fixture, 30_000);

    resetManager = new RuntimeResetManager(config);
    await resetManager.bootstrap();
    runtimeBootstrapped = true;

    if (!existsSync(resolve(repoRoot, 'node_modules/.bin/playwright'))) {
      runCommand(
        'bunx',
        [`bun@${lockedBunVersion}`, 'install', '--frozen-lockfile'],
        {
          cwd: repoRoot,
          env: safeEnvironment(),
        }
      );
    }
    const baselineLock = await loadBaselineLock();
    const dependencyLockPath = resolve(
      config.sourceRoot,
      baselineLock.dependencyLock
    );
    const lockHashBefore = await sha256File(dependencyLockPath);
    if (lockHashBefore !== baselineLock.dependencyLockSha256) {
      throw new Error(
        `source dependency lock hash ${lockHashBefore} does not match ${baselineLock.dependencyLockSha256}`
      );
    }
    if (!existsSync(resolve(config.sourceRoot, 'node_modules/.bin/next'))) {
      runCommand('bunx', [`bun@${lockedBunVersion}`, 'install', '--no-save'], {
        cwd: config.sourceRoot,
        env: safeEnvironment(),
      });
    }
    const lockHashAfter = await sha256File(dependencyLockPath);
    if (lockHashAfter !== lockHashBefore) {
      throw new Error(
        `source dependency installation changed the locked dependency graph: ${lockHashBefore} -> ${lockHashAfter}`
      );
    }
    const sourceChanges = execFileSync(
      'git',
      ['status', '--porcelain', '--untracked-files=no'],
      { cwd: config.sourceRoot, encoding: 'utf8' }
    ).trim();
    if (sourceChanges !== '') {
      throw new Error(
        `source dependency installation modified the immutable baseline: ${sourceChanges}`
      );
    }
    const shardIndex = Number(process.env.E2E_SHARD_INDEX ?? '0');
    if (shardIndex === 0) {
      await runBackendContracts({
        config,
        environment: safeEnvironment({
          NOTIFICATION_RUNTIME_DATABASE_URL: databaseUrlForRuntime(config),
          NOTIFICATION_RUNTIME_REDIS_URL: config.redisUrl,
        }),
        groups: accumulatedGroups,
        resetManager,
      });
    } else {
      process.stdout.write(
        `backend contract suites assigned to shard 0; skipping on shard ${shardIndex}\n`
      );
    }
    runCommand(
      'cargo',
      [
        'build',
        '--locked',
        '-p',
        'epsx-frontend',
        ...(requiresAdmin ? ['-p', 'epsx-admin'] : ['--bin', 'bff-frontend']),
      ],
      {
        cwd: repoRoot,
        env: safeEnvironment(),
      }
    );

    const commonAppEnvironment = {
      ENV: 'development',
      EPSX_ENV: 'local',
      DEPLOYMENT_ENV: 'development',
      NODE_ENV: 'development',
      BACKEND_URL: config.fixtureUrl,
      API_URL: config.fixtureUrl,
      OIDC_ISSUER: config.fixtureUrl,
      NOTIFICATION_SERVICE_URL: config.fixtureUrl,
      CONTENT_SERVICE_URL: config.fixtureUrl,
      DATABASE_URL: databaseUrlForRuntime(config),
      REDIS_URL: config.redisUrl,
      RPC_URL: config.anvilUrl,
      CHAIN_RPC_URL: config.anvilUrl,
      BLOCKCHAIN_NETWORK: 'testnet',
      CHAIN_ID: '31337',
      NEXT_PUBLIC_BLOCKCHAIN_NETWORK: 'testnet',
      NEXT_PUBLIC_CHAIN_ID: '31337',
      NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID: '00000000000000000000000000000000',
      NEXT_PUBLIC_OAUTH_CLIENT_ID: 'epsx-frontend',
      NEXT_PUBLIC_BACKEND_URL: config.fixtureUrl,
      NEXT_PUBLIC_APP_URL: config.sourceFrontendUrl,
      NEXT_PUBLIC_ADMIN_URL: config.sourceAdminUrl,
      FRONTEND_URL: config.sourceFrontendUrl,
      ADMIN_FRONTEND_URL: config.sourceAdminUrl,
      WEB3_APP_SECRET: 'epsx-e2e-only-web3-secret-32-characters-minimum',
      NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET:
        '0x0000000000000000000000000000000000000000',
      NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET:
        '0x0000000000000000000000000000000000000000',
      NEXT_PUBLIC_PAYMENT_ESCROW_TESTNET:
        '0x0000000000000000000000000000000000000000',
      NEXT_PUBLIC_PAYMENT_RECEIVER_TESTNET:
        '0x0000000000000000000000000000000000000000',
      MINIO_ENDPOINT: config.fixtureUrl,
      MINIO_PUBLIC_URL: config.fixtureUrl,
      NEXT_PUBLIC_CDN_URL: config.fixtureUrl,
      TZ: 'UTC',
    };
    const cargoTargetRoot = resolve(
      repoRoot,
      process.env.CARGO_TARGET_DIR?.trim() || 'target'
    );
    const source = await startManagedProcess({
      name: 'nextjs-source',
      executable: 'bun',
      commandArgs: ['run', 'dev'],
      spawnOptions: {
        cwd: resolve(config.sourceRoot, 'apps/frontend'),
        env: safeEnvironment({
          ...commonAppEnvironment,
          PORT: new URL(config.sourceFrontendUrl).port,
        }),
      },
      logPath: resolve(logsRoot, 'nextjs-source.log'),
    });
    managed.push(source);

    const target = await startManagedProcess({
      name: 'dioxus-target',
      executable: resolve(cargoTargetRoot, 'debug/bff-frontend'),
      commandArgs: [],
      spawnOptions: {
        cwd: repoRoot,
        env: safeEnvironment({
          ...commonAppEnvironment,
          PORT: new URL(config.targetFrontendUrl).port,
          HOST: '127.0.0.1',
          FRONTEND_URL: config.targetFrontendUrl,
          RUST_LOG: 'info',
        }),
      },
      logPath: resolve(logsRoot, 'dioxus-target.log'),
    });
    managed.push(target);

    const readiness = [
      waitForUrl(config.sourceFrontendUrl, source),
      waitForUrl(config.targetFrontendUrl, target),
    ];
    if (requiresAdmin) {
      const sourceAdmin = await startManagedProcess({
        name: 'nextjs-admin-source',
        executable: 'bun',
        commandArgs: ['run', 'dev'],
        spawnOptions: {
          cwd: resolve(config.sourceRoot, 'apps/admin-frontend'),
          env: safeEnvironment({
            ...commonAppEnvironment,
            PORT: new URL(config.sourceAdminUrl).port,
            NEXT_PUBLIC_APP_URL: config.sourceAdminUrl,
            FRONTEND_URL: config.sourceAdminUrl,
            NEXT_PUBLIC_OAUTH_CLIENT_ID: 'epsx-admin',
          }),
        },
        logPath: resolve(logsRoot, 'nextjs-admin-source.log'),
      });
      managed.push(sourceAdmin);
      const targetAdmin = await startManagedProcess({
        name: 'dioxus-admin-target',
        executable: resolve(cargoTargetRoot, 'debug/bff-admin'),
        commandArgs: [],
        spawnOptions: {
          cwd: repoRoot,
          env: safeEnvironment({
            ...commonAppEnvironment,
            PORT: new URL(config.targetAdminUrl).port,
            HOST: '127.0.0.1',
            FRONTEND_URL: config.targetAdminUrl,
            NEXT_PUBLIC_OAUTH_CLIENT_ID: 'epsx-admin',
            RUST_LOG: 'info',
          }),
        },
        logPath: resolve(logsRoot, 'dioxus-admin-target.log'),
      });
      managed.push(targetAdmin);
      readiness.push(
        waitForUrl(config.sourceAdminUrl, sourceAdmin),
        waitForUrl(config.targetAdminUrl, targetAdmin)
      );
    }
    await Promise.all(readiness);

    const playwrightEnvironment = safeEnvironment({
      E2E_GROUP: String(selectedGroup),
      E2E_SOURCE_ROOT: config.sourceRoot,
      E2E_ARTIFACT_ROOT: config.artifactRoot,
      E2E_RUN_ROOT: config.runRoot,
      E2E_ALLOW_RUNTIME_MUTATION: '1',
      E2E_SOURCE_FRONTEND_URL: config.sourceFrontendUrl,
      E2E_SOURCE_ADMIN_URL: config.sourceAdminUrl,
      E2E_TARGET_FRONTEND_URL: config.targetFrontendUrl,
      E2E_TARGET_ADMIN_URL: config.targetAdminUrl,
      E2E_FIXTURE_URL: config.fixtureUrl,
      E2E_FIXTURE_TOKEN: config.fixtureToken,
      E2E_POSTGRES_ADMIN_URL: config.postgresAdminUrl,
      E2E_POSTGRES_TEMPLATE_DATABASE: config.postgresTemplateDatabase,
      E2E_POSTGRES_RUNTIME_DATABASE: config.postgresRuntimeDatabase,
      E2E_REDIS_URL: config.redisUrl,
      E2E_REDIS_PREFIX: config.redisPrefix,
      E2E_ANVIL_URL: config.anvilUrl,
      ...(process.env.E2E_LAYOUT_SELECTORS !== undefined &&
      process.env.E2E_LAYOUT_SELECTORS !== ''
        ? {
            E2E_LAYOUT_SELECTORS: process.env.E2E_LAYOUT_SELECTORS,
          }
        : {}),
    });
    const playwrightShards = selectPlaywrightShards(
      buildPlaywrightShards(manifest, accumulatedGroups, process.env.E2E_GREP)
    );
    process.stdout.write(
      `selected ${playwrightShards.length} Playwright shard(s) from ` +
        `${process.env.E2E_SHARD_COUNT ?? '1'} campaign worker(s)\n`
    );
    for (const shard of playwrightShards) {
      process.stdout.write(
        `playwright shard: ${shard.project ?? 'all projects'} / ${shard.grep}\n`
      );
      const useProcessGroup = process.platform !== 'win32';
      const playwright = spawn('bunx', playwrightArgumentsForShard(shard), {
        cwd: repoRoot,
        detached: useProcessGroup,
        env: playwrightEnvironment,
        stdio: 'inherit',
      });
      activePlaywright = {
        name: 'playwright',
        child: playwright,
        logPath: resolve(config.artifactRoot, 'playwright-report'),
        processGroupId: useProcessGroup ? playwright.pid : undefined,
      };
      testStatus = await new Promise<number>(resolvePromise => {
        playwright.once('exit', code => resolvePromise(code ?? 1));
      });
      activePlaywright = undefined;
      if (testStatus !== 0) {
        throw new Error(
          `Playwright migration group ${selectedGroup} shard ${shard.grep} failed`
        );
      }
    }

    evidenceReady = (await listFiles(config.artifactRoot)).some(path =>
      path.endsWith('/reproducibility.json')
    );
  } catch (error) {
    runError = error;
  } finally {
    if (interruptedSignal !== undefined && runError === undefined) {
      runError = new Error(`migration e2e interrupted by ${interruptedSignal}`);
    }
    runError = mergeCleanupFailures(
      runError,
      await cleanupRuntime({
        composeEnvironment,
        composeStarted,
        config,
        managed,
        resetManager,
        runtimeBootstrapped,
        selectedGroup,
      })
    );
    process.off('SIGINT', handleSigint);
    process.off('SIGTERM', handleSigterm);
  }
  const isShardedRun = process.env.E2E_SHARD_COUNT !== undefined;
  if (runError === undefined && evidenceReady && !isShardedRun) {
    try {
      await generateReport(config);
      await verifyArtifactManifest(config);
    } catch (error) {
      runError = error;
    }
  }
  if (runError !== undefined) {
    throw runError;
  }
  process.stdout.write(
    `migration group ${selectedGroup}: PASS — reset, capture, repeat, report, and artifact hash verification completed\n`
  );
}

function databaseUrlForRuntime(
  config: Awaited<ReturnType<typeof runtimeConfig>>
): string {
  const url = new URL(config.postgresAdminUrl);
  url.pathname = `/${config.postgresRuntimeDatabase}`;
  return url.toString();
}

async function report(): Promise<void> {
  const config = await runtimeConfig(groupId());
  const result = await generateReport(config);
  process.stdout.write(`wrote ${result.reportPath}\n`);
}

async function verifyArtifacts(): Promise<void> {
  const config = await runtimeConfig(groupId());
  const checked = await verifyArtifactManifest(config);
  process.stdout.write(`artifact verification: PASS — ${checked} files\n`);
}

try {
  if (command === 'doctor') {
    await doctor();
  } else if (command === 'prepare-source') {
    process.stdout.write(`${await prepareSource()}\n`);
  } else if (command === 'run') {
    await run();
  } else if (command === 'report') {
    await report();
  } else if (command === 'verify-artifacts') {
    await verifyArtifacts();
  } else {
    throw new Error(
      `unknown command ${command}; use doctor, prepare-source, run, report, or verify-artifacts`
    );
  }
} catch (error) {
  process.stderr.write(
    `migration e2e: ERROR: ${
      error instanceof Error ? (error.stack ?? error.message) : String(error)
    }\n`
  );
  process.exit(1);
}
