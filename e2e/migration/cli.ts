#!/usr/bin/env bun

import {
  execFileSync,
  spawn,
  type ChildProcess,
  type SpawnOptions,
} from 'node:child_process';
import { createWriteStream, existsSync } from 'node:fs';
import { mkdir, rm } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';

import {
  baselineLockPath,
  defaultSourceRoot,
  loadBaselineLock,
  loadManifest,
  repoRoot,
  runtimeConfig,
} from './lib/config';
import { listFiles, readJson, sha256File } from './lib/files';
import { generateReport, verifyArtifactManifest } from './lib/report';
import { RuntimeResetManager } from './lib/runtime-reset';

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
    if (processGroupId === undefined) {
      return child.exitCode === null && child.signalCode === null;
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
  const sendSignal = (signal: NodeJS.Signals): void => {
    try {
      if (processGroupId === undefined) {
        child.kill(signal);
      } else {
        process.kill(-processGroupId, signal);
      }
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ESRCH') {
        throw error;
      }
    }
  };
  if (!groupIsAlive()) {
    return;
  }
  sendSignal('SIGTERM');
  const deadline = Date.now() + 10_000;
  while (groupIsAlive() && Date.now() < deadline) {
    await new Promise(resolvePromise => setTimeout(resolvePromise, 100));
  }
  if (groupIsAlive()) {
    process.stderr.write(
      `${name} did not stop after SIGTERM; sending SIGKILL\n`
    );
    sendSignal('SIGKILL');
  }
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

async function doctor(): Promise<void> {
  const lock = await loadBaselineLock();
  const manifest = await loadManifest();
  execFileSync('git', ['cat-file', '-e', `${lock.commit}^{commit}`], {
    cwd: repoRoot,
  });
  execFileSync('git', ['merge-base', '--is-ancestor', lock.commit, lock.ref], {
    cwd: repoRoot,
  });
  if (manifest.baselineLock !== 'e2e/migration/baseline.lock.json') {
    throw new Error('scenario manifest does not reference the baseline lock');
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
    throw new Error('PR 0 cannot pass with required bypasses');
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

// Process orchestration is intentionally centralized so cleanup owns every
// child/container handle in one try/finally boundary.
// eslint-disable-next-line max-lines-per-function, complexity
async function run(): Promise<void> {
  const selectedGroup = groupId();
  if (selectedGroup !== 0) {
    throw new Error(
      `PR ${selectedGroup} scenarios are not executable on the PR 0 branch`
    );
  }
  await doctor();
  await prepareSource();
  // The command itself owns the isolated Compose graph. Standalone reset
  // manager calls remain guarded unless their caller makes this explicit.
  process.env.E2E_ALLOW_RUNTIME_MUTATION = '1';
  const config = await runtimeConfig(selectedGroup);
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

    const resetManager = new RuntimeResetManager(config);
    await resetManager.bootstrap();

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
    runCommand(
      'cargo',
      ['build', '--locked', '-p', 'epsx-frontend', '--bin', 'bff-frontend'],
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
      executable: resolve(repoRoot, 'target/debug/bff-frontend'),
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

    await Promise.all([
      waitForUrl(config.sourceFrontendUrl, source),
      waitForUrl(config.targetFrontendUrl, target),
    ]);

    const playwright = spawn(
      'bunx',
      [
        'playwright',
        'test',
        '--config',
        resolve(migrationRoot, 'playwright.config.ts'),
      ],
      {
        cwd: repoRoot,
        env: safeEnvironment({
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
        }),
        stdio: 'inherit',
      }
    );
    testStatus = await new Promise<number>(resolvePromise => {
      playwright.once('exit', code => resolvePromise(code ?? 1));
    });
    await resetManager.smoke();

    evidenceReady = (await listFiles(config.artifactRoot)).some(path =>
      path.endsWith('/reproducibility.json')
    );
    if (testStatus !== 0) {
      throw new Error(`Playwright migration group ${selectedGroup} failed`);
    }
  } catch (error) {
    runError = error;
  } finally {
    for (const processInfo of managed.reverse()) {
      await stopManagedProcess(processInfo);
    }
    if (composeStarted) {
      try {
        runCommand(
          'docker',
          [
            'compose',
            '-f',
            composePath,
            'down',
            '--volumes',
            '--remove-orphans',
          ],
          { cwd: repoRoot, env: composeEnvironment }
        );
      } catch (error) {
        runError ??= error;
      }
    }
  }
  if (runError === undefined && evidenceReady) {
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
