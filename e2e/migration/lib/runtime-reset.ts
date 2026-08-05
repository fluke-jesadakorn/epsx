import { createConnection } from 'node:net';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

import { Client } from 'pg';

import {
  ensureDirectory,
  readJson,
  sha256,
  stableJson,
  writeJson,
} from './files';
import type { ResetProof, RuntimeConfig, StateDigest } from './types';

interface RuntimeState {
  schemaVersion: number;
  anvilSnapshotId: string;
  baseline: StateDigest;
}

const transientTables = new Set([
  'infra_logs.outbox_events',
  'public.notification_outbox',
  'public.notification_channel_jobs',
  'public.provider_callback_fixtures',
  'public.sse_cursors',
  'public.worker_leases',
]);

function quoteIdentifier(value: string): string {
  if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(value)) {
    throw new Error(`unsafe PostgreSQL identifier: ${value}`);
  }
  return `"${value.replaceAll('"', '""')}"`;
}

function databaseUrl(adminUrl: string, database: string): string {
  const url = new URL(adminUrl);
  url.pathname = `/${database}`;
  return url.toString();
}

async function withPg<T>(
  url: string,
  callback: (client: Client) => Promise<T>
): Promise<T> {
  const client = new Client({ connectionString: url });
  await client.connect();
  try {
    return await callback(client);
  } finally {
    await client.end();
  }
}

async function databaseExists(
  adminUrl: string,
  database: string
): Promise<boolean> {
  return withPg(adminUrl, async client => {
    const result = await client.query(
      'SELECT 1 FROM pg_database WHERE datname = $1',
      [database]
    );
    return result.rowCount === 1;
  });
}

async function terminateDatabaseConnections(
  adminUrl: string,
  database: string
): Promise<void> {
  await withPg(adminUrl, async client => {
    await client.query(
      `SELECT pg_terminate_backend(pid)
       FROM pg_stat_activity
       WHERE datname = $1 AND pid <> pg_backend_pid()`,
      [database]
    );
  });
}

async function dropDatabase(adminUrl: string, database: string): Promise<void> {
  if (!(await databaseExists(adminUrl, database))) {
    return;
  }
  await terminateDatabaseConnections(adminUrl, database);
  await withPg(adminUrl, async client => {
    await client.query(`DROP DATABASE ${quoteIdentifier(database)}`);
  });
}

async function createDatabase(
  adminUrl: string,
  database: string,
  template?: string
): Promise<void> {
  await withPg(adminUrl, async client => {
    const fromTemplate =
      template !== undefined ? ` TEMPLATE ${quoteIdentifier(template)}` : '';
    await client.query(
      `CREATE DATABASE ${quoteIdentifier(database)}${fromTemplate}`
    );
  });
}

async function initializeTemplate(config: RuntimeConfig): Promise<void> {
  await dropDatabase(config.postgresAdminUrl, config.postgresRuntimeDatabase);
  await dropDatabase(config.postgresAdminUrl, config.postgresTemplateDatabase);
  await createDatabase(
    config.postgresAdminUrl,
    config.postgresTemplateDatabase
  );
  const baselineSql = await readFile(
    resolve(config.repoRoot, 'e2e/migration/runtime/baseline.sql'),
    'utf8'
  );
  await withPg(
    databaseUrl(config.postgresAdminUrl, config.postgresTemplateDatabase),
    async client => {
      await client.query(baselineSql);
    }
  );
  await createDatabase(
    config.postgresAdminUrl,
    config.postgresRuntimeDatabase,
    config.postgresTemplateDatabase
  );
}

async function restoreRuntimeDatabase(config: RuntimeConfig): Promise<void> {
  await dropDatabase(config.postgresAdminUrl, config.postgresRuntimeDatabase);
  await createDatabase(
    config.postgresAdminUrl,
    config.postgresRuntimeDatabase,
    config.postgresTemplateDatabase
  );
}

async function postgresDigest(
  config: RuntimeConfig,
  database: string
): Promise<StateDigest['postgres']> {
  return withPg(
    databaseUrl(config.postgresAdminUrl, database),
    async client => {
      const tablesResult = await client.query<{
        schemaname: string;
        tablename: string;
      }>(
        `SELECT schemaname, tablename
         FROM pg_tables
         WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
         ORDER BY schemaname, tablename`
      );
      const tables: StateDigest['postgres']['tables'] = [];
      for (const row of tablesResult.rows) {
        const table = `${row.schemaname}.${row.tablename}`;
        const qualified = `${quoteIdentifier(row.schemaname)}.${quoteIdentifier(row.tablename)}`;
        const result = await client.query<{ rows: string; checksum: string }>(
          `SELECT count(*)::text AS rows,
                  COALESCE(md5(string_agg(row_hash, '' ORDER BY row_hash)), md5('')) AS checksum
           FROM (
             SELECT md5(row_to_json(source_row)::text) AS row_hash
             FROM ${qualified} AS source_row
           ) AS row_hashes`
        );
        tables.push({
          table,
          rows: Number(result.rows[0]?.rows ?? '0'),
          checksum: result.rows[0]?.checksum ?? sha256(''),
        });
      }
      const checksum = sha256(
        stableJson(
          tables.map(({ table, rows, checksum: tableChecksum }) => ({
            table,
            rows,
            checksum: tableChecksum,
          }))
        )
      );
      return {
        database,
        tables,
        checksum,
        transientTablesEmpty: tables
          .filter(({ table }) => transientTables.has(table))
          .every(({ rows }) => rows === 0),
      };
    }
  );
}

type RedisValue = string | number | Buffer | null | RedisValue[];

function encodeRedisCommand(parts: string[]): Buffer {
  const chunks = [Buffer.from(`*${parts.length}\r\n`)];
  for (const part of parts) {
    const value = Buffer.from(part);
    chunks.push(
      Buffer.from(`$${value.length}\r\n`),
      value,
      Buffer.from('\r\n')
    );
  }
  return Buffer.concat(chunks);
}

// RESP has one branch per wire type; keeping the parser local avoids granting
// the harness a broad Redis client with unsafe flush operations.
// eslint-disable-next-line complexity
function parseRedisValue(
  buffer: Buffer,
  offset: number
): { value: RedisValue; offset: number } | null {
  if (offset >= buffer.length) {
    return null;
  }
  const type = String.fromCharCode(buffer[offset]);
  const lineEnd = buffer.indexOf('\r\n', offset);
  if (lineEnd === -1) {
    return null;
  }
  const line = buffer.subarray(offset + 1, lineEnd).toString();
  if (type === '+' || type === ':') {
    return {
      value: type === ':' ? Number(line) : line,
      offset: lineEnd + 2,
    };
  }
  if (type === '-') {
    throw new Error(`Redis error: ${line}`);
  }
  if (type === '$') {
    const length = Number(line);
    if (length === -1) {
      return { value: null, offset: lineEnd + 2 };
    }
    const valueStart = lineEnd + 2;
    const valueEnd = valueStart + length;
    if (buffer.length < valueEnd + 2) {
      return null;
    }
    return {
      value: buffer.subarray(valueStart, valueEnd),
      offset: valueEnd + 2,
    };
  }
  if (type === '*') {
    const count = Number(line);
    const values: RedisValue[] = [];
    let nextOffset = lineEnd + 2;
    for (let index = 0; index < count; index += 1) {
      const parsed = parseRedisValue(buffer, nextOffset);
      if (!parsed) {
        return null;
      }
      values.push(parsed.value);
      nextOffset = parsed.offset;
    }
    return { value: values, offset: nextOffset };
  }
  throw new Error(`unsupported Redis response type ${type}`);
}

async function redisCommands(
  redisUrl: string,
  commands: string[][]
): Promise<RedisValue[]> {
  const url = new URL(redisUrl);
  const port = Number(url.port || '6379');
  const authCommands: string[][] = [];
  if (url.password !== '') {
    authCommands.push(
      url.username === ''
        ? ['AUTH', decodeURIComponent(url.password)]
        : [
            'AUTH',
            decodeURIComponent(url.username),
            decodeURIComponent(url.password),
          ]
    );
  }
  const allCommands = [...authCommands, ...commands];
  return new Promise((resolvePromise, reject) => {
    const socket = createConnection({ host: url.hostname, port });
    let received = Buffer.alloc(0);
    socket.setTimeout(10_000);
    socket.on('connect', () => {
      socket.write(
        Buffer.concat(allCommands.map(command => encodeRedisCommand(command)))
      );
    });
    socket.on('data', (chunk: Buffer) => {
      received = Buffer.concat([received, chunk]);
      try {
        const values: RedisValue[] = [];
        let offset = 0;
        while (values.length < allCommands.length) {
          const parsed = parseRedisValue(received, offset);
          if (!parsed) {
            return;
          }
          values.push(parsed.value);
          offset = parsed.offset;
        }
        socket.end();
        resolvePromise(values.slice(authCommands.length));
      } catch (error) {
        socket.destroy();
        reject(error);
      }
    });
    socket.on('timeout', () => {
      socket.destroy(new Error('Redis command timed out'));
    });
    socket.on('error', reject);
  });
}

function redisString(value: RedisValue): string {
  if (Buffer.isBuffer(value)) {
    return value.toString();
  }
  if (typeof value === 'string' || typeof value === 'number') {
    return String(value);
  }
  throw new Error(`unexpected Redis value: ${stableJson(value)}`);
}

async function redisKeys(redisUrl: string, prefix: string): Promise<string[]> {
  let cursor = '0';
  const keys: string[] = [];
  do {
    const [response] = await redisCommands(redisUrl, [
      ['SCAN', cursor, 'MATCH', `${prefix}*`, 'COUNT', '1000'],
    ]);
    if (!Array.isArray(response) || response.length !== 2) {
      throw new Error('Redis SCAN returned an unexpected response');
    }
    cursor = redisString(response[0]);
    const batch = response[1];
    if (!Array.isArray(batch)) {
      throw new Error('Redis SCAN key list was not an array');
    }
    keys.push(...batch.map(redisString));
  } while (cursor !== '0');
  return [...new Set(keys)].sort();
}

async function resetRedis(config: RuntimeConfig): Promise<void> {
  const keys = await redisKeys(config.redisUrl, config.redisPrefix);
  for (let index = 0; index < keys.length; index += 200) {
    await redisCommands(config.redisUrl, [
      ['DEL', ...keys.slice(index, index + 200)],
    ]);
  }
}

async function redisDigest(
  config: RuntimeConfig
): Promise<StateDigest['redis']> {
  const keys = await redisKeys(config.redisUrl, config.redisPrefix);
  const entries = [];
  for (const key of keys) {
    const [dump] = await redisCommands(config.redisUrl, [['DUMP', key]]);
    const value =
      dump === null
        ? Buffer.alloc(0)
        : Buffer.isBuffer(dump)
          ? dump
          : Buffer.from(String(dump));
    entries.push({ key, valueHash: sha256(value) });
  }
  return {
    prefix: config.redisPrefix,
    keys: entries,
    checksum: sha256(stableJson(entries)),
  };
}

async function rpc<T>(
  url: string,
  method: string,
  params: unknown[] = []
): Promise<T> {
  const response = await fetch(url, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method,
      params,
    }),
  });
  if (!response.ok) {
    throw new Error(`Anvil ${method} returned HTTP ${response.status}`);
  }
  const payload = (await response.json()) as {
    result?: T;
    error?: { message?: string };
  };
  if (payload.error || payload.result === undefined) {
    throw new Error(
      `Anvil ${method} failed: ${payload.error?.message ?? 'missing result'}`
    );
  }
  return payload.result;
}

async function anvilDigest(
  config: RuntimeConfig
): Promise<StateDigest['anvil']> {
  const chainId = await rpc<string>(config.anvilUrl, 'eth_chainId');
  if (chainId.toLowerCase() !== '0x7a69') {
    throw new Error(
      `Anvil safety check expected chain 31337, received ${chainId}`
    );
  }
  const blockNumber = await rpc<string>(config.anvilUrl, 'eth_blockNumber');
  const addresses = await rpc<string[]>(config.anvilUrl, 'eth_accounts');
  const accounts = [];
  for (const address of addresses) {
    const [balance, nonce, code] = await Promise.all([
      rpc<string>(config.anvilUrl, 'eth_getBalance', [address, 'latest']),
      rpc<string>(config.anvilUrl, 'eth_getTransactionCount', [
        address,
        'latest',
      ]),
      rpc<string>(config.anvilUrl, 'eth_getCode', [address, 'latest']),
    ]);
    accounts.push({
      address: address.toLowerCase(),
      balance,
      nonce,
      codeHash: sha256(code),
    });
  }
  accounts.sort((left, right) => left.address.localeCompare(right.address));
  return {
    chainId,
    blockNumber,
    accounts,
    checksum: sha256(stableJson({ chainId, blockNumber, accounts })),
  };
}

async function fixtureRequest<T>(
  config: RuntimeConfig,
  path: string,
  method = 'GET'
): Promise<T> {
  const response = await fetch(`${config.fixtureUrl}${path}`, {
    method,
    headers: { 'x-epsx-e2e-token': config.fixtureToken },
  });
  if (!response.ok) {
    throw new Error(`fixture ${method} ${path} returned ${response.status}`);
  }
  return (await response.json()) as T;
}

async function resetFixture(config: RuntimeConfig): Promise<void> {
  await fixtureRequest(config, '/__e2e/reset', 'POST');
}

async function fixtureDigest(
  config: RuntimeConfig
): Promise<StateDigest['fixture']> {
  const state = await fixtureRequest<{
    requestCount: number;
    requests: unknown[];
    mutations: unknown[];
  }>(config, '/__e2e/state');
  const digestable = {
    requestCount: state.requestCount,
    requests: state.requests,
    mutations: state.mutations,
  };
  return { ...digestable, checksum: sha256(stableJson(digestable)) };
}

function statePath(config: RuntimeConfig): string {
  return resolve(config.runRoot, 'runtime-state.json');
}

async function captureState(
  config: RuntimeConfig,
  database = config.postgresRuntimeDatabase
): Promise<StateDigest> {
  const [postgres, redis, anvil, fixture] = await Promise.all([
    postgresDigest(config, database),
    redisDigest(config),
    anvilDigest(config),
    fixtureDigest(config),
  ]);
  return {
    capturedAt: new Date().toISOString(),
    postgres,
    redis,
    anvil,
    fixture,
  };
}

function compareState(
  baseline: StateDigest,
  actual: StateDigest
): ResetProof['checks'] {
  return {
    postgresMatchesBaseline:
      baseline.postgres.checksum === actual.postgres.checksum,
    transientTablesEmpty: actual.postgres.transientTablesEmpty,
    redisMatchesBaseline: baseline.redis.checksum === actual.redis.checksum,
    anvilMatchesBaseline: baseline.anvil.checksum === actual.anvil.checksum,
    fixtureMatchesBaseline:
      baseline.fixture.checksum === actual.fixture.checksum,
  };
}

export class RuntimeResetManager {
  constructor(private readonly config: RuntimeConfig) {}

  private assertMutationAllowed(): void {
    if (!this.config.allowRuntimeMutation) {
      throw new Error(
        'runtime reset is guarded; set E2E_ALLOW_RUNTIME_MUTATION=1 for the isolated E2E services'
      );
    }
  }

  async bootstrap(): Promise<StateDigest> {
    this.assertMutationAllowed();
    await ensureDirectory(this.config.runRoot);
    await initializeTemplate(this.config);
    await resetRedis(this.config);
    await resetFixture(this.config);

    const chainId = await rpc<string>(this.config.anvilUrl, 'eth_chainId');
    if (chainId.toLowerCase() !== '0x7a69') {
      throw new Error(`refusing non-Anvil chain ${chainId}`);
    }
    const anvilSnapshotId = await rpc<string>(
      this.config.anvilUrl,
      'evm_snapshot'
    );
    const runtimeBaseline = await captureState(this.config);
    const templatePostgres = await postgresDigest(
      this.config,
      this.config.postgresTemplateDatabase
    );
    const baseline: StateDigest = {
      ...runtimeBaseline,
      postgres: templatePostgres,
    };
    const checks = compareState(baseline, runtimeBaseline);
    if (!Object.values(checks).every(Boolean)) {
      throw new Error(
        `runtime bootstrap did not match its template: ${stableJson(checks)}`
      );
    }
    await writeJson(statePath(this.config), {
      schemaVersion: 1,
      anvilSnapshotId,
      baseline,
    } satisfies RuntimeState);
    return baseline;
  }

  async reset(
    scenarioId: string,
    phase: ResetProof['phase'],
    proofPath: string
  ): Promise<ResetProof> {
    this.assertMutationAllowed();
    const startedAt = new Date().toISOString();
    const state = await readJson<RuntimeState>(statePath(this.config));
    if (state.schemaVersion !== 1) {
      throw new Error('unsupported runtime state schema');
    }
    const beforeReset = await captureState(this.config);

    await restoreRuntimeDatabase(this.config);
    await resetRedis(this.config);
    const reverted = await rpc<boolean>(this.config.anvilUrl, 'evm_revert', [
      state.anvilSnapshotId,
    ]);
    if (!reverted) {
      throw new Error(
        `Anvil could not revert snapshot ${state.anvilSnapshotId}; runtime restart is required`
      );
    }
    const anvilSnapshotId = await rpc<string>(
      this.config.anvilUrl,
      'evm_snapshot'
    );
    await resetFixture(this.config);

    const afterReset = await captureState(this.config);
    const checks = compareState(state.baseline, afterReset);
    const passed = Object.values(checks).every(Boolean);
    const proof: ResetProof = {
      schemaVersion: 1,
      scenarioId,
      phase,
      startedAt,
      completedAt: new Date().toISOString(),
      beforeReset,
      afterReset,
      baseline: state.baseline,
      checks,
      passed,
    };
    await writeJson(proofPath, proof);
    await writeJson(statePath(this.config), {
      ...state,
      anvilSnapshotId,
    } satisfies RuntimeState);
    if (!passed) {
      throw new Error(
        `runtime rollback failed for ${scenarioId}/${phase}: ${stableJson(checks)}`
      );
    }
    return proof;
  }

  async smoke(): Promise<StateDigest> {
    const state = await readJson<RuntimeState>(statePath(this.config));
    const current = await captureState(this.config);
    const checks = compareState(state.baseline, current);
    if (!Object.values(checks).every(Boolean)) {
      throw new Error(`post-reset smoke failed: ${stableJson(checks)}`);
    }
    return current;
  }
}
