import { appendFile, mkdir } from 'node:fs/promises';
import { dirname } from 'node:path';
import { createPrivateKey, sign } from 'node:crypto';

import { permissionAllows } from './lib/fixture-permissions';

interface FixtureRequest {
  sequence: number;
  method: string;
  path: string;
  query: string;
  bodySha256?: string;
}

const port = Number(process.env.E2E_FIXTURE_PORT ?? '48080');
const token = process.env.E2E_FIXTURE_TOKEN ?? 'epsx-e2e-local-reset-token';
const logPath = process.env.E2E_FIXTURE_LOG;

let requests: FixtureRequest[] = [];
let mutations: FixtureRequest[] = [];
let sequence = 0;
let fixtureMode = 'healthy';

const signingKeyId = 'epsx-e2e-rs256-v1';
const fixtureTier = 'migration-e2e';
const fixtureTimestamp = '2026-01-01T00:00:00.000Z';
const fixtureWalletAddress = '0xea6400000000000000000000000000000000e3df';
const fixturePlanId = '00000000-0000-0000-0000-000000000001';
const fixtureMerchantId = '00000000-0000-0000-0000-000000000002';
const fixtureNewsId = '00000000-0000-0000-0000-000000000006';
const fixtureNotificationId = 'idem_notification_e2e_1';
const fixtureConversationId = '550e8400-e29b-41d4-a716-446655440000';
const fixtureTopicId = '550e8400-e29b-41d4-a716-446655440001';
const fixtureMessageId = '550e8400-e29b-41d4-a716-446655440002';
const signingKey = createPrivateKey(`-----BEGIN PRIVATE KEY-----
MIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQC3Zucb7soDltXU
G5e/am1A1dC6zZyXA6TBse5ktX70zTTfIEsro7LoYF44UgWmM3iyrNAK5kVijIr4
hURnmaiPfxf6KO1XmRq4J4zav27yV7+LkHHX9EmFSokpZkAikhCjV2fW3acpkWBD
Yei+v5wWiSrXJdcccQr0BQieC+fP1a35jErN95VVdQ3sT+KvmBm0djGqMan4gGeW
6Zd1wXVJM2a/hPf+AcPtKfGN4MVU3l38nupiPnmcN5FN7A/f75IyXFLdd4sA51FR
QKdnnNXj3jJFuaE7k7O9eRdfFZsgOMN5lykz5aDYoBm+ju0a1RVXAyNm1DpURoi/
c3GrJlJ3AgMBAAECggEANnjIwaIdvA0ru1DqtD6e7nfTA/iXvr6lS6ZWYPELIRhl
0LOdv/th4uTkdyPda6yz95WeQO59wzRs/j1OwNqBlwUvkOxg+fiOWA3fJwVepXns
eT5Qocx7nawyquokuF/bszf9rnKs+IqmJb1JzIXKjWL2J2qkxlzI3Qs1sQNmOXMD
hUJZy0IEbTXs5Ix5r7dRWA2qUPLrHnWT8vm7oaGNYhJRaFTqTaauGRLVJ403zSoI
KXpVtU6k8MX4LlQlTpQC3ej0UnMqZewFf0aHDW1fv2cqab+el2V3I/EekMyDx68z
9EsZdult/wIZOP8BBCzWIyQE56OY+A1hvFL7w00qSQKBgQDsJ6FPkMYdAqLkEyjn
brNZqprkqpOXHluUZuwO5vOH9ragtIIIQoJKv5cxlwmY9dD/KPBjC+1MBFkzk3CM
ATNLgBqxA4/ZHwFCtPZr002IX3QtoZjM6pHUn8CN24Jp6QeBz+5Xw6c7YoMvclTb
GRhvhpTexzpWyeGNXobUDcPP2wKBgQDG0Gg5s78DkmfDktMfVuw7lOx8PrDC788R
3JlSYXe62bs9CDS1LFB8OCXxIj/vnjj6P4888SzPYZ5bW3F1cjc2o5TQe3E9DcEi
aclkrekmf649LpBTcQ66Gf7XDuC9qUIfMs5Kcre5FoY7XUFlTtkQjfj6x9AQ+Lhr
ebFdmeaIlQKBgQDIkkgxebaqAQk0SQmetqjhaUMxH6dG3GPPwTKQ3ZrNSb+G8ojW
VxauQdc6KRvfrDgb3zt8BC9BNxhD89/NKV/VqjIBUhMkx26cp3H71nWtc9UKxIsw
z7GYMy6pzVwQc/kKSf4W0HgCugLNk396ru/QGS/rnq5v8/r7xOMiy6YZrQKBgQCp
uk/QOwhuNzXIe/crAR0JvJirdSWYNfw0RnzKHJWHecvkTbYZmWxYr+KMWm301cHU
uiBBqa9UmAUF/yn8VvaV+c7YsRm6QpzIEUGyZtntWQFaD/98jL9C12B9HqF0qSPe
2JPOcOMx6u3LjlB++XJMNLgC+ERDyOJANpLZ0sJBhQKBgQCgpBL4H+ycCK4npGMG
SCZiOro0+J52I9plnzcnpT92bg3GrH5Wa72cMrfOTYg4T1KTQ2NbnCWIsvzNFV+W
v8c5kgY8YwO5hfBbV1VfoIMo3nu2rasMHbUzX9xnBxUB7PZD3bfbs3uBn29vdkmn
Wjh7jLLxLl6Tu7Awh6UNeNJ29w==
-----END PRIVATE KEY-----
`);
const signingJwk = {
  kty: 'RSA',
  use: 'sig',
  alg: 'RS256',
  kid: signingKeyId,
  n: 't2bnG-7KA5bV1BuXv2ptQNXQus2clwOkwbHuZLV-9M003yBLK6Oy6GBeOFIFpjN4sqzQCuZFYoyK-IVEZ5moj38X-ijtV5kauCeM2r9u8le_i5Bx1_RJhUqJKWZAIpIQo1dn1t2nKZFgQ2Hovr-cFokq1yXXHHEK9AUIngvnz9Wt-YxKzfeVVXUN7E_ir5gZtHYxqjGp-IBnlumXdcF1STNmv4T3_gHD7SnxjeDFVN5d_J7qYj55nDeRTewP3--SMlxS3XeLAOdRUUCnZ5zV494yRbmhO5OzvXkXXxWbIDjDeZcpM-Wg2KAZvo7tGtUVVwMjZtQ6VEaIv3NxqyZSdw',
  e: 'AQAB',
};

function base64Url(value: string | Buffer): string {
  return Buffer.from(value).toString('base64url');
}

function fixtureAccessToken(options: {
  issuer: string;
  audience: string;
  permissions: string;
  keyId: string;
}): string {
  const { audience, issuer, keyId, permissions } = options;
  const address = fixtureWalletAddress;
  const header = base64Url(
    JSON.stringify({ alg: 'RS256', typ: 'JWT', kid: keyId })
  );
  const payload = base64Url(
    JSON.stringify({
      iss: issuer,
      sub: address,
      aud: [audience],
      exp: 2524608000,
      iat: 1785283200,
      nbf: 1785283170,
      jti: `epsx-e2e-${audience}`,
      scope: permissions,
      wallet_address: address,
      auth_method: 'web3_siwe',
      auth_time: 1785283200,
    })
  );
  const message = `${header}.${payload}`;
  const signature = sign('RSA-SHA256', Buffer.from(message), signingKey);
  return `${message}.${signature.toString('base64url')}`;
}

function fixtureWallet(
  status = 'active',
  version = 3
): {
  address: string;
  chain_id: string;
  label: string;
  role: string;
  status: string;
  metadata: Record<string, never>;
  version: number;
  created_at: string;
} {
  return {
    address: fixtureWalletAddress,
    chain_id: '31337',
    label: 'Migration owner',
    role: 'user',
    status,
    metadata: {},
    version,
    created_at: fixtureTimestamp,
  };
}

function fixturePlan(): {
  id: string;
  merchant_id: string;
  name: string;
  description: string;
  amount: string;
  currency: string;
  chain_id: string;
  interval: number;
  active: boolean;
  created_at: string;
  version: number;
} {
  return {
    id: fixturePlanId,
    merchant_id: fixtureMerchantId,
    name: 'Migration Professional',
    description: 'Deterministic backend-authoritative access plan.',
    amount: '2900',
    currency: 'USD',
    chain_id: '31337',
    interval: 30,
    active: true,
    created_at: fixtureTimestamp,
    version: 7,
  };
}

function requiredAdminPermission(path: string, method: string): string | null {
  const mutation = !['GET', 'HEAD'].includes(method);
  if (path.startsWith('/api/v1/admin/wallets')) {
    return mutation ? 'admin:wallets:manage' : 'admin:wallets:read';
  }
  if (path.startsWith('/api/v1/admin/credits')) {
    return mutation ? 'admin:credits:manage' : 'admin:credits:read';
  }
  if (path.startsWith('/api/v1/admin/subscription/access')) {
    return mutation ? 'admin:access:manage' : 'admin:access:read';
  }
  if (path.startsWith('/api/v1/admin/subscription/plans')) {
    return mutation ? 'admin:plans:manage' : 'admin:plans:read';
  }
  if (path.startsWith('/api/v1/analytics/admin/audit-log')) {
    return 'admin:audit:read';
  }
  if (
    path.startsWith('/api/admin/dashboard/') ||
    path.startsWith('/api/admin/web3/recent-wallets')
  ) {
    return 'admin:dashboard:view';
  }
  if (path.startsWith('/api/admin/analytics/')) {
    return 'admin:analytics:view';
  }
  if (
    path.startsWith('/api/admin/news') ||
    path.startsWith('/api/admin/media') ||
    path.startsWith('/api/admin/files')
  ) {
    return 'admin:content:manage';
  }
  if (path === '/api/v1/notification/send') {
    return 'admin:notifications:create';
  }
  if (path.startsWith('/api/v1/notification/admin')) {
    return mutation ? 'admin:notifications:manage' : 'admin:notifications:read';
  }
  if (path.startsWith('/api/admin/chat/conversations')) {
    return mutation ? 'admin:chat:send' : 'admin:chat:read';
  }
  return null;
}

function fixtureAudience(value: unknown): string | null {
  return Array.isArray(value) &&
    value.length === 1 &&
    typeof value[0] === 'string'
    ? value[0]
    : null;
}

function fixtureSubject(
  subject: unknown,
  walletAddress: unknown
): string | null {
  return typeof subject === 'string' &&
    typeof walletAddress === 'string' &&
    subject === walletAddress
    ? subject
    : null;
}

function fixturePrincipal(request: Request): {
  subject: string;
  permissions: string[];
  audience: string;
} | null {
  const authorization = request.headers.get('authorization');
  if (authorization?.startsWith('Bearer ') !== true) {
    return null;
  }
  const segments = authorization.slice('Bearer '.length).split('.');
  if (segments.length !== 3) {
    return null;
  }
  try {
    const claims = JSON.parse(
      Buffer.from(segments[1], 'base64url').toString('utf8')
    ) as {
      aud?: unknown;
      sub?: unknown;
      wallet_address?: unknown;
      scope?: unknown;
    };
    const subject = fixtureSubject(claims.sub, claims.wallet_address);
    const audience = fixtureAudience(claims.aud);
    if (
      subject === null ||
      audience === null ||
      typeof claims.scope !== 'string'
    ) {
      return null;
    }
    return {
      subject,
      audience,
      permissions: claims.scope.split(/\s+/).filter(Boolean),
    };
  } catch {
    return null;
  }
}

if (!Number.isInteger(port) || port < 1024 || port > 65535) {
  throw new Error(`invalid E2E fixture port ${port}`);
}

async function log(message: string): Promise<void> {
  const line = `${message}\n`;
  process.stdout.write(line);
  if (logPath !== undefined) {
    await mkdir(dirname(logPath), { recursive: true });
    await appendFile(logPath, line, 'utf8');
  }
}

function json(value: unknown, status = 200): Response {
  return Response.json(value, {
    status,
    headers: {
      'access-control-allow-origin': '*',
      'access-control-allow-headers':
        'authorization,content-type,x-api-version,x-access-level,x-request-id',
      'access-control-allow-methods': 'GET,HEAD,POST,PUT,PATCH,DELETE,OPTIONS',
      'cache-control': 'no-store',
      'x-epsx-e2e-fixture': '1',
    },
  });
}

const fixtureQuarterEnd = '2026-06-30';

const rankings = [
  {
    rank: 1,
    symbol: 'NVDA',
    company_name: 'NVIDIA Corporation',
    latest_date: fixtureQuarterEnd,
    value: 4.12,
    active_status: 'active',
    quarterly_performance: [
      {
        quarter: 'Q2 2026',
        date: fixtureQuarterEnd,
        price: 184.25,
        eps: 4.12,
        eps_growth: 42.5,
        price_growth: 18.2,
        announcement_date: '2026-07-15',
        announcement_timestamp: 1784073600,
        is_estimated: false,
      },
    ],
    next_quarter_estimate: {
      quarter: 'Q3 2026',
      estimated_eps: 4.48,
      announcement_date: '2026-10-15',
      announcement_timestamp: 1792022400,
      days_until_announcement: 79,
      estimated_price_target: 196.0,
      confidence: 'High',
    },
    next_earnings_date: 1792022400,
    last_earnings_date: 1784073600,
    next_earnings_date_formatted: 'October 15, 2026',
    days_until_next_earnings: 79,
    progress_percentage: 35.0,
    current_eps: 4.12,
    growth_factor: 1.425,
    price_current: 184.25,
  },
  {
    rank: 2,
    symbol: 'MSFT',
    company_name: 'Microsoft Corporation',
    latest_date: fixtureQuarterEnd,
    value: 3.88,
    active_status: 'active',
    quarterly_performance: [
      {
        quarter: 'Q2 2026',
        date: fixtureQuarterEnd,
        price: 512.4,
        eps: 3.88,
        eps_growth: 24.1,
        price_growth: 11.4,
        announcement_date: '2026-07-21',
        announcement_timestamp: 1784592000,
        is_estimated: false,
      },
    ],
    next_quarter_estimate: null,
    next_earnings_date: null,
    last_earnings_date: 1784592000,
    next_earnings_date_formatted: null,
    days_until_next_earnings: null,
    progress_percentage: null,
    current_eps: 3.88,
    growth_factor: 1.241,
    price_current: 512.4,
  },
  {
    rank: 3,
    symbol: 'AAPL',
    company_name: 'Apple Inc.',
    latest_date: fixtureQuarterEnd,
    value: 2.34,
    active_status: 'active',
    quarterly_performance: [
      {
        quarter: 'Q2 2026',
        date: fixtureQuarterEnd,
        price: 228.7,
        eps: 2.34,
        eps_growth: 16.8,
        price_growth: 7.9,
        announcement_date: '2026-07-29',
        announcement_timestamp: 1785283200,
        is_estimated: false,
      },
    ],
    next_quarter_estimate: null,
    next_earnings_date: null,
    last_earnings_date: 1785283200,
    next_earnings_date_formatted: null,
    days_until_next_earnings: null,
    progress_percentage: null,
    current_eps: 2.34,
    growth_factor: 1.168,
    price_current: 228.7,
  },
];

function rankingResponse(limit: number, page: number): Record<string, unknown> {
  const limited = fixtureMode === 'analytics-limited';
  const available =
    fixtureMode === 'analytics-empty'
      ? []
      : limited
        ? rankings.slice(1)
        : rankings;
  const totalPages =
    available.length === 0 ? 0 : Math.ceil(available.length / limit);
  const selected = available.slice((page - 1) * limit, page * limit);
  const stale = fixtureMode === 'analytics-stale';
  return {
    success: true,
    data: selected,
    pagination: {
      page,
      limit,
      total: available.length,
      totalPages,
      hasNext: page < totalPages,
      hasPrev: page > 1 && totalPages > 0,
    },
    metadata: {
      available_countries: ['United States'],
      available_sectors: ['Technology'],
      request_timestamp: stale
        ? '2020-01-01T00:00:00Z'
        : '2026-07-28T00:00:00Z',
      data_source: stale ? 'stale-cache' : 'epsx-e2e-fixture-v1',
    },
    access_info: {
      min_accessible_rank: limited ? 2 : 1,
      locked_ranks_count: limited ? 1 : 0,
    },
    message: null,
    processing_time_ms: 1,
  };
}

const publicPlans = [
  {
    id: 'plan-free',
    name: 'Free',
    plan_type: 'free',
    plan_group: 'personal',
    current_price: '0',
    effective_price: 0,
    currency: 'USD',
    display_order: 1,
    is_active: true,
    is_highlighted: false,
    features: ['Public rankings', 'Weekly market digest'],
  },
  {
    id: 'plan-pro',
    name: 'Professional',
    plan_type: 'premium',
    plan_group: 'personal',
    current_price: '29',
    effective_price: 29,
    currency: 'USD',
    display_order: 2,
    is_active: true,
    is_highlighted: true,
    features: ['Full rankings', 'Portfolio analytics'],
  },
];

const publicNews = [
  {
    id: 'news-e2e-1',
    slug: 'deterministic-market-brief',
    title: 'Deterministic Market Brief',
    summary: 'A fixed local article used by the migration evidence harness.',
    content: 'This fixture is intentionally stable across repeated E2E runs.',
    status: 'published',
    published_at: '2026-07-01T00:00:00Z',
    author: 'EPSX Research',
    tags: ['Research'],
    featured: true,
    read_time: 3,
  },
];

const targetPublicNews = [
  {
    id: fixtureNewsId,
    slug: 'deterministic-market-brief',
    title: 'Deterministic Market Brief',
    summary: 'A fixed local article used by the migration evidence harness.',
    content:
      'This fixture is intentionally stable across repeated E2E runs.\n\nIt verifies the published content boundary.',
    cover_image_url: null,
    author: 'EPSX Research',
    status: 'published',
    published_at: '2026-07-01T00:00:00Z',
    tags: ['Research', 'engineering'],
    featured: true,
  },
];

const legacyPublicNews = [
  {
    id: fixtureNewsId,
    slug: 'deterministic-market-brief',
    title: 'Deterministic Market Brief',
    summary: 'A fixed local article used by the migration evidence harness.',
    content:
      'This fixture is intentionally stable across repeated E2E runs.\n\nIt verifies the published content boundary.',
    cover_image_url: null,
    author_wallet: 'EPSX Research',
    status: 'published',
    tags: ['Research', 'engineering'],
    published_at: '2026-07-01T00:00:00Z',
    created_at: '2026-06-30T00:00:00Z',
    updated_at: fixtureTimestamp,
    is_pinned: false,
    pinned_at: null,
  },
];

function adminNewsArticle(
  options: {
    status?: 'draft' | 'published';
    pinned?: boolean;
  } = {}
): Record<string, unknown> {
  const status = options.status ?? 'published';
  return {
    id: fixtureNewsId,
    title: 'Deterministic Market Brief',
    slug: 'deterministic-market-brief',
    summary: 'A fixed local article used by the migration evidence harness.',
    content: 'This fixture is intentionally stable across repeated E2E runs.',
    cover_image_url: null,
    author_wallet: fixtureWalletAddress,
    status,
    tags: ['Research'],
    published_at: status === 'published' ? '2026-07-01T00:00:00Z' : null,
    created_at: '2026-06-30T00:00:00Z',
    updated_at: fixtureTimestamp,
    is_pinned: options.pinned ?? false,
    pinned_at: options.pinned === true ? fixtureTimestamp : null,
  };
}

function fixtureResponseMeta(): Record<string, string> {
  return {
    timestamp: fixtureTimestamp,
    version: 'v1',
  };
}

function adminNewsEnvelope(data: unknown, status = 200): Response {
  return json(
    {
      success: true,
      data,
      error: null,
      meta: fixtureResponseMeta(),
    },
    status
  );
}

function mediaItems(bucket: string, origin: string): unknown[] {
  const key =
    bucket === 'public'
      ? 'guides/getting-started.pdf'
      : 'news/release-notes.pdf';
  return [
    {
      key,
      url: `${origin}/__e2e/media/${bucket}/fixture.pdf`,
      size: 4096,
      last_modified: fixtureTimestamp,
    },
  ];
}

function adminAnalyticsData(empty = false): Record<string, unknown> {
  return {
    user_stats: empty
      ? null
      : {
          total: 12,
          active: 10,
          today_connections: 2,
          total_users: 12,
          active_users: 10,
        },
    permission_analytics: empty
      ? null
      : {
          total: 8,
          total_plans: 2,
          total_permissions: 8,
          active_permissions: 7,
        },
    plan_stats: empty
      ? null
      : {
          total_plans: 2,
          active_plans: 2,
          total_memberships: 6,
          active_memberships: 5,
          recent_assignments: 1,
        },
    system_metrics: null,
    developer_portal: empty
      ? null
      : {
          total_api_keys: 3,
          active_api_keys: 2,
        },
  };
}

function adminAnalyticsEnvelope(data: Record<string, unknown>): unknown {
  return {
    success: true,
    data,
    error: null,
    message: 'Analytics dashboard retrieved',
    timestamp: fixtureTimestamp,
    admin_meta: {
      operation: 'get_admin_analytics_dashboard',
      performed_by: fixtureWalletAddress,
    },
  };
}

function adminDashboardEnvelope(): unknown {
  return {
    success: true,
    data: {
      observed_at: fixtureTimestamp,
      total_users: 12,
      active_users: 10,
    },
    error: null,
    message: 'Dashboard user status retrieved successfully',
    timestamp: fixtureTimestamp,
    admin_meta: {
      operation: 'get_dashboard_user_status',
      performed_by: fixtureWalletAddress,
    },
  };
}

function authorizedControlRequest(request: Request): boolean {
  return request.headers.get('x-epsx-e2e-token') === token;
}

// The fixture router stays explicit so every supported dependency path is
// reviewable in one place.
// eslint-disable-next-line max-lines-per-function, complexity, sonarjs/cognitive-complexity
async function routeRequest(request: Request): Promise<Response> {
  const url = new URL(request.url);
  if (request.method === 'OPTIONS') {
    return json({});
  }
  if (url.pathname === '/__e2e/reset') {
    if (!authorizedControlRequest(request)) {
      return json({ error: 'forbidden' }, 403);
    }
    requests = [];
    mutations = [];
    sequence = 0;
    fixtureMode = 'healthy';
    await log('fixture reset');
    return json({ reset: true });
  }
  if (url.pathname === '/__e2e/state') {
    if (!authorizedControlRequest(request)) {
      return json({ error: 'forbidden' }, 403);
    }
    return json({
      requestCount: requests.length,
      requests,
      mutations,
      mode: fixtureMode,
    });
  }
  if (url.pathname === '/__e2e/mode') {
    if (!authorizedControlRequest(request) || request.method !== 'PUT') {
      return json({ error: 'forbidden' }, 403);
    }
    const body = (await request.json()) as { mode?: unknown };
    if (typeof body.mode !== 'string' || body.mode.trim() === '') {
      return json({ error: 'invalid_mode' }, 400);
    }
    fixtureMode = body.mode;
    return json({ mode: fixtureMode });
  }
  if (url.pathname === '/__e2e/session') {
    if (!authorizedControlRequest(request)) {
      return json({ error: 'forbidden' }, 403);
    }
    const audience = url.searchParams.get('audience') ?? '';
    if (!['epsx-frontend', 'epsx-admin'].includes(audience)) {
      return json({ error: 'invalid_audience' }, 400);
    }
    const keyId = url.searchParams.get('key_id') ?? signingKeyId;
    if (!/^epsx-e2e-rs256-[a-z0-9-]{1,64}$/.test(keyId)) {
      return json({ error: 'invalid_key_id' }, 400);
    }
    return json({
      accessToken: fixtureAccessToken({
        issuer: url.origin,
        audience,
        permissions: url.searchParams.get('permissions') ?? '',
        keyId,
      }),
    });
  }

  sequence += 1;
  const entry: FixtureRequest = {
    sequence,
    method: request.method,
    path: url.pathname,
    query: [...url.searchParams.entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, value]) => `${key}=${value}`)
      .join('&'),
  };
  requests.push(entry);
  if (!['GET', 'HEAD'].includes(request.method)) {
    mutations.push(entry);
  }
  await log(
    `fixture request ${entry.sequence} ${entry.method} ${entry.path}${url.search}`
  );

  if (url.pathname === '/health' || url.pathname === '/api/health') {
    return json({ status: 'ok', service: 'epsx-e2e-fixture' });
  }
  if (
    fixtureMode === 'dependency-unavailable' &&
    url.pathname.includes('jwks')
  ) {
    return json({ success: false, error: 'dependency_unavailable' }, 503);
  }
  if (url.pathname.includes('jwks')) {
    return json({ keys: [signingJwk] });
  }
  const requiredPermission = requiredAdminPermission(
    url.pathname,
    request.method
  );
  if (requiredPermission !== null) {
    const principal = fixturePrincipal(request);
    if (
      principal?.audience !== 'epsx-admin' ||
      !permissionAllows(principal.permissions, requiredPermission)
    ) {
      return json({ success: false, error: 'forbidden' }, 403);
    }
  }
  const adminContentPath =
    url.pathname.startsWith('/api/admin/news') ||
    url.pathname.startsWith('/api/admin/media') ||
    url.pathname.startsWith('/api/admin/files');
  const publicContentPath =
    url.pathname === '/api/v1/content/news' ||
    url.pathname.startsWith('/api/v1/content/news/') ||
    url.pathname === '/api/public/news' ||
    (url.pathname.startsWith('/api/public/news/') &&
      url.pathname !== '/api/public/news/featured');
  if (fixtureMode === 'content-forbidden' && adminContentPath) {
    return json({ success: false, error: 'forbidden' }, 403);
  }
  if (
    fixtureMode === 'content-unavailable' &&
    (adminContentPath || publicContentPath)
  ) {
    return json({ success: false, error: 'dependency_unavailable' }, 503);
  }
  if (
    fixtureMode === 'content-malformed' &&
    (adminContentPath || publicContentPath)
  ) {
    return json({ malformed: true });
  }
  if (
    fixtureMode === 'forbidden' &&
    (url.pathname.startsWith('/api/v1/admin/') ||
      url.pathname.startsWith('/api/v1/analytics/admin/'))
  ) {
    return json({ success: false, error: 'forbidden' }, 403);
  }
  if (fixtureMode === 'conflict' && !['GET', 'HEAD'].includes(request.method)) {
    return json({ success: false, error: 'optimistic_conflict' }, 409);
  }
  if (fixtureMode === 'malformed') {
    return json({ malformed: true });
  }
  if (url.pathname === '/api/admin/dashboard/user-status') {
    if (fixtureMode === 'admin-dashboard-forbidden') {
      return json({ success: false, error: 'forbidden' }, 403);
    }
    if (fixtureMode === 'admin-dashboard-unavailable') {
      return json({ success: false, error: 'dependency_unavailable' }, 503);
    }
    if (fixtureMode === 'admin-dashboard-malformed') {
      const malformed = adminDashboardEnvelope() as {
        data: { observed_at: string };
      };
      malformed.data.observed_at = 'not-an-rfc3339-timestamp';
      return json(malformed);
    }
    return json(adminDashboardEnvelope());
  }
  if (url.pathname === '/api/admin/analytics/dashboard') {
    if (fixtureMode === 'admin-analytics-forbidden') {
      return json({ success: false, error: 'forbidden' }, 403);
    }
    if (fixtureMode === 'admin-analytics-unavailable') {
      return json({ success: false, error: 'dependency_unavailable' }, 503);
    }
    if (fixtureMode === 'admin-analytics-malformed') {
      return json(
        adminAnalyticsEnvelope({
          ...adminAnalyticsData(),
          observed_at: fixtureTimestamp,
        })
      );
    }
    return json(
      adminAnalyticsEnvelope(
        adminAnalyticsData(fixtureMode === 'admin-analytics-empty')
      )
    );
  }
  if (
    (url.pathname === '/api/analytics/rankings' ||
      url.pathname === '/api/public/analytics/rankings') &&
    fixtureMode === 'analytics-unavailable'
  ) {
    return json({ success: false, error: 'dependency_unavailable' }, 503);
  }
  if (
    (url.pathname === '/api/analytics/rankings' ||
      url.pathname === '/api/public/analytics/rankings') &&
    fixtureMode === 'analytics-malformed'
  ) {
    return json({
      ...rankingResponse(3, 1),
      metadata: {
        available_countries: [],
        available_sectors: [],
        request_timestamp: 'not-an-rfc3339-timestamp',
        data_source: 'unverified',
      },
    });
  }
  if (url.pathname === '/api/v1/admin/wallets/stats') {
    return json({
      total: 1,
      active: 1,
      disabled: 0,
      new_30_days: 1,
      correlation_id: 'e2e-wallet-stats',
    });
  }
  if (url.pathname === '/api/v1/admin/wallets') {
    return json({
      items: [fixtureWallet()],
      total: 1,
      limit: 100,
      offset: 0,
      correlation_id: 'e2e-wallet-list',
    });
  }
  const targetWalletPath = `/api/v1/admin/wallets/${fixtureWalletAddress}`;
  if (url.pathname === `${targetWalletPath}/disable`) {
    return json({
      wallet: fixtureWallet('disabled', 4),
      evidence: {
        operation_id: '00000000-0000-0000-0000-000000000003',
        version: 4,
        observed_at: fixtureTimestamp,
      },
      correlation_id: 'e2e-wallet-disable',
    });
  }
  if (url.pathname === targetWalletPath) {
    return json(fixtureWallet());
  }
  if (url.pathname === '/api/v1/admin/credits') {
    return json({
      outstanding_minor: 12_000,
      granted_today_minor: 2_000,
      revoked_today_minor: 500,
      active_accounts: 1,
      correlation_id: 'e2e-credit-stats',
    });
  }
  if (url.pathname.startsWith('/api/v1/admin/credits/')) {
    return json({
      transaction_id: '00000000-0000-0000-0000-000000000004',
      version: 4,
      correlation_id: 'e2e-credit-mutation',
    });
  }
  if (url.pathname === '/api/v1/admin/subscription/access') {
    return json({
      items: [
        {
          wallet_address: fixtureWalletAddress,
          plan_id: fixturePlanId,
          plan_name: 'Migration Professional',
          permission: 'epsx:analytics:read',
          expires_at: null,
          version: 2,
          assigned_by: fixtureWalletAddress,
          updated_at: fixtureTimestamp,
        },
      ],
      correlation_id: 'e2e-access-list',
    });
  }
  if (url.pathname.startsWith('/api/v1/admin/subscription/access/')) {
    return json({
      success: true,
      correlation_id: 'e2e-access-mutation',
    });
  }
  if (url.pathname === '/api/v1/admin/subscription/plans') {
    if (!['GET', 'HEAD'].includes(request.method)) {
      return json({ success: true, id: fixturePlanId });
    }
    return json({
      items: [fixturePlan()],
      total: 1,
      limit: 100,
      offset: 0,
      correlation_id: 'e2e-plan-list',
    });
  }
  if (url.pathname === `/api/v1/admin/subscription/plans/${fixturePlanId}`) {
    return ['GET', 'HEAD'].includes(request.method)
      ? json(fixturePlan())
      : json({ success: true, id: fixturePlanId, version: 8 });
  }
  if (url.pathname === '/api/v1/analytics/admin/audit-log') {
    const walletDisabled = mutations.some(
      mutation => mutation.path === `${targetWalletPath}/disable`
    );
    return json({
      items: [
        {
          id: '00000000-0000-0000-0000-000000000005',
          category: 'wallet',
          action: walletDisabled ? 'wallet.disabled' : 'wallet.reviewed',
          resource_type: 'wallet',
          effect: 'success',
          occurred_at: fixtureTimestamp,
        },
      ],
      next_cursor: null,
      has_more: false,
    });
  }
  if (
    url.pathname === '/api/analytics/rankings' ||
    url.pathname === '/api/public/analytics/rankings'
  ) {
    const limit = Math.max(
      1,
      Math.min(10, Number(url.searchParams.get('limit') ?? '3'))
    );
    const page = Math.max(
      1,
      Math.min(1_000_000, Number(url.searchParams.get('page') ?? '1'))
    );
    return json(rankingResponse(limit, page));
  }
  if (url.pathname === '/api/analytics/filters') {
    const data = {
      countries: [{ value: 'america', label: 'United States' }],
      sectors: ['Technology'],
      exchanges: ['NASDAQ'],
      stock_types: ['common'],
    };
    return request.headers.has('x-api-version') ||
      request.headers.has('x-access-level')
      ? json({ success: true, data })
      : json(data);
  }
  if (url.pathname === '/api/public/analytics/filters') {
    return json({
      success: true,
      data: {
        countries: [{ value: 'america', label: 'United States' }],
        sectors: ['Technology'],
        exchanges: ['NASDAQ'],
        stock_types: ['common'],
      },
    });
  }
  if (url.pathname === '/api/users/watchlist') {
    return json({ success: true, data: { symbols: ['NVDA'] } });
  }
  if (url.pathname === '/api/users/portfolio/overview') {
    return json({
      success: true,
      data: {
        watchlist: ['NVDA'],
        rankings,
      },
    });
  }
  if (url.pathname === '/api/payments/plans/my-plan-access') {
    return json({
      success: true,
      data: {
        wallet_address: fixtureWalletAddress,
        plan_name: 'Migration Professional',
        plan_id: fixturePlanId,
        plan_expires_at: null,
        days_remaining: 30,
        status: 'active',
        ranking_offset: 1,
        can_upgrade: false,
        tier_level: 2,
        proration_credit: null,
        current_plan_price: '29',
      },
    });
  }
  if (url.pathname === '/api/admin/dashboard/summary') {
    return json({
      success: true,
      data: {
        wallet_stats: {
          total: 12,
          active: 10,
          today_connections: 2,
        },
        permission_stats: {
          total: 8,
          pending_notifications: 0,
        },
        system_health: null,
      },
    });
  }
  if (url.pathname === '/api/admin/web3/recent-wallets') {
    return json({
      success: true,
      data: {
        recent_wallets: [
          {
            wallet_address: fixtureWalletAddress,
            metadata: {},
            created_at: fixtureTimestamp,
            last_auth_at: fixtureTimestamp,
            is_active: true,
            active_permissions_count: 2,
            connection_info: {
              is_new: false,
              last_seen: 1_788_278_400,
            },
          },
        ],
        analytics: {
          total_in_period: 1,
          daily_breakdown: [
            {
              date: '2026-07-01',
              connections: 1,
            },
          ],
          period_days: 30,
          avg_daily: 0.03,
        },
        metadata: {
          limit: 10,
          total_count: 1,
          has_more: false,
          generated_at: fixtureTimestamp,
        },
      },
    });
  }
  if (url.pathname === '/api/public/plans') {
    return json({ success: true, data: publicPlans });
  }
  if (url.pathname === '/api/public/news/featured') {
    return json({ success: true, data: publicNews });
  }
  if (url.pathname === '/api/public/news') {
    return json({
      success: true,
      data: {
        articles: legacyPublicNews,
        total: legacyPublicNews.length,
        page: 1,
        limit: Number(url.searchParams.get('limit') ?? '10'),
      },
    });
  }
  if (url.pathname.startsWith('/api/public/news/')) {
    const slug = url.pathname.split('/').at(-1);
    const article = legacyPublicNews.find(candidate => candidate.slug === slug);
    return article
      ? json({ success: true, data: article })
      : json({ success: false, error: 'not_found' }, 404);
  }
  if (url.pathname === '/api/admin/news') {
    const status = url.searchParams.get('status');
    const article = adminNewsArticle({
      status: status === 'draft' ? 'draft' : 'published',
    });
    if (['GET', 'HEAD'].includes(request.method)) {
      const articles = fixtureMode === 'content-empty' ? [] : [article];
      return adminNewsEnvelope({
        articles,
        total: articles.length,
        page: Number(url.searchParams.get('page') ?? '1'),
        limit: 20,
      });
    }
    return adminNewsEnvelope(article, 201);
  }
  if (url.pathname === '/api/admin/news/upload-image') {
    return adminNewsEnvelope({
      url: 'https://assets.epsx.invalid/news/cover.png',
      thumb_url: null,
      filename: 'migration-cover.png',
      mime: 'image/png',
      size: 68,
    });
  }
  if (url.pathname.startsWith('/api/admin/news/')) {
    const segments = url.pathname.split('/').filter(Boolean);
    const id = segments[3];
    if (id !== fixtureNewsId) {
      return json({ success: false, error: 'not_found' }, 404);
    }
    const operation = segments[4];
    if (request.method === 'DELETE' && operation === undefined) {
      return adminNewsEnvelope({ id: fixtureNewsId, deleted: true });
    }
    return adminNewsEnvelope(
      adminNewsArticle({
        status: operation === 'unpublish' ? 'draft' : 'published',
        pinned: operation === 'pin',
      })
    );
  }
  if (url.pathname === '/api/admin/files/upload') {
    return adminNewsEnvelope({
      bucket: 'public',
      key: 'uploads/migration-proof.txt',
      url: `${url.origin}/__e2e/media/public/migration-proof.txt`,
      thumb_url: null,
      mime: 'text/plain',
      size: 21,
      deleted: false,
    });
  }
  if (url.pathname.startsWith('/api/admin/media/')) {
    const segments = url.pathname.split('/').filter(Boolean);
    const bucket = segments[3];
    if (!['news', 'public'].includes(bucket ?? '')) {
      return json({ success: false, error: 'invalid_bucket' }, 400);
    }
    const key = segments.slice(4).join('/');
    if (request.method === 'DELETE') {
      return adminNewsEnvelope({
        bucket,
        key,
        url: null,
        thumb_url: null,
        mime: null,
        size: null,
        deleted: true,
      });
    }
    return adminNewsEnvelope(
      fixtureMode === 'content-empty'
        ? []
        : mediaItems(bucket ?? 'news', url.origin)
    );
  }
  const legacyWallet = {
    wallet_address: fixtureWalletAddress,
    is_active: true,
    created_at: fixtureTimestamp,
    last_auth_at: fixtureTimestamp,
    metadata: {
      label: 'Migration owner',
      note: 'Deterministic wallet fixture',
    },
    platforms: ['epsx'],
    permissions: [
      {
        permission: 'epsx:analytics:read',
        platform: 'epsx',
        is_active: true,
        source: 'migration-e2e',
        created_at: fixtureTimestamp,
      },
    ],
    groups: [],
    plan_name: 'Migration Professional',
    plans: [
      {
        plan_id: fixturePlanId,
        plan_name: 'Migration Professional',
        plan_type: 'premium',
        assigned_at: fixtureTimestamp,
        is_active: true,
      },
    ],
    subscriptions: [],
  };
  if (url.pathname === '/api/admin/wallets/stats') {
    return json({
      success: true,
      data: {
        total_users: 1,
        active_users: 1,
        inactive_users: 0,
        new_users_30_days: 1,
        active_users_30_days: 1,
        growth_rate: 0,
      },
    });
  }
  if (url.pathname === '/api/admin/wallets') {
    return json({
      success: true,
      data: {
        wallets: [legacyWallet],
        pagination: {
          page: 1,
          limit: 50,
          total: 1,
          total_pages: 1,
          has_next_page: false,
          has_previous_page: false,
        },
      },
    });
  }
  if (
    url.pathname === `/api/admin/wallets/${fixtureWalletAddress}` ||
    url.pathname === `/api/admin/wallets/${fixtureWalletAddress}/access-summary`
  ) {
    if (url.pathname.endsWith('/access-summary')) {
      return json({
        success: true,
        data: {
          wallet_address: fixtureWalletAddress,
          plans: legacyWallet.plans,
          permissions: legacyWallet.permissions,
        },
      });
    }
    return json({
      success: true,
      data: { ...legacyWallet, wallet: legacyWallet },
    });
  }
  if (
    url.pathname === '/api/permissions/plans' ||
    url.pathname === '/api/permissions/assignments' ||
    url.pathname === '/api/admin/permissions/available' ||
    url.pathname === '/api/admin/permissions/assignments'
  ) {
    return json({
      success: true,
      data:
        url.pathname === '/api/admin/permissions/available'
          ? ['epsx:analytics:read']
          : [],
    });
  }
  if (url.pathname === `/api/permissions/plans/${fixturePlanId}`) {
    return json({
      success: true,
      data: {
        id: fixturePlanId,
        name: 'Migration Professional',
        slug: 'migration-professional',
        description: 'Deterministic legacy access plan.',
        plan_type: 'premium',
        permissions: ['epsx:analytics:read'],
        is_active: true,
        created_at: fixtureTimestamp,
        updated_at: fixtureTimestamp,
        default_expiry_days: 30,
        tier_level: 1,
      },
    });
  }
  if (url.pathname === '/api/admin/subscriptions') {
    return json({
      success: true,
      data: { subscriptions: [], total: 0 },
    });
  }
  if (url.pathname === '/api/payments/admin/credits/stats') {
    return json({
      success: true,
      data: {
        total_credits_outstanding: 12_000,
        total_credits_granted_today: 2_000,
        total_credits_used_today: 500,
        active_users_with_credits: 1,
        total_transactions_today: 2,
        average_balance: 12_000,
      },
    });
  }
  if (url.pathname === '/api/admin/audit-logs') {
    return json({
      success: true,
      data: {
        entries: [
          {
            id: 'legacy-audit-e2e-1',
            action: 'wallet.reviewed',
            wallet_address: fixtureWalletAddress,
            resource_type: 'wallet',
            resource_id: fixtureWalletAddress,
            result: 'success',
            details: null,
            timestamp: fixtureTimestamp,
            category: 'wallet',
          },
        ],
        total_pages: 1,
      },
    });
  }
  if (url.pathname === '/api/admin/settings') {
    return json({
      data: {
        general: {
          systemName: 'EPSX Admin',
          adminEmail: 'admin@epsx.io',
          maintenanceMode: false,
        },
        notifications: {
          emailNotifications: true,
          pushNotifications: false,
          smsNotifications: true,
          securityAlerts: true,
        },
        security: { sessionTimeout: 30 },
        appearance: { theme: 'auto', primaryColor: '#3b82f6' },
      },
    });
  }
  if (
    url.pathname === '/api/auth/session' ||
    url.pathname === '/api/auth/web3/session'
  ) {
    const principal = fixturePrincipal(request);
    const expectedAudience =
      request.headers.get('x-app-type') === 'admin'
        ? 'epsx-admin'
        : principal?.audience;
    if (
      principal === null ||
      expectedAudience === undefined ||
      principal.audience !== expectedAudience
    ) {
      return json({ error: 'authentication_required' }, 401);
    }
    if (url.pathname === '/api/auth/web3/session') {
      return json({
        authenticated: true,
        wallet_address: principal.subject,
        permissions: principal.permissions,
      });
    }
    return json({
      user: {
        id: principal.subject,
        wallet_address: principal.subject,
        permissions: principal.permissions,
        tier: fixtureTier,
      },
      expiresAt: 2524608000000,
    });
  }
  if (url.pathname === '/api/auth/web3/logout') {
    return json({ success: true, revoked: true });
  }
  if (url.pathname === '/api/permissions/definitions') {
    const principal = fixturePrincipal(request);
    if (principal === null) {
      return json({ success: false, error: 'authentication_required' }, 401);
    }
    return json({
      success: true,
      data: principal.permissions.map((permission, index) => ({
        id: `fixture-permission-${index + 1}`,
        permission_string: permission,
        name: permission,
        description: 'Backend-issued migration contract permission',
        platform: permission.split(':')[0] ?? 'epsx',
        category: permission.split(':')[1] ?? null,
        is_system: true,
        is_active: true,
        created_at: fixtureTimestamp,
      })),
    });
  }
  if (url.pathname === '/api/users/permissions/status') {
    const principal = fixturePrincipal(request);
    if (principal === null) {
      return json({ success: false, error: 'authentication_required' }, 401);
    }
    const permissions = principal.permissions.map(permission => ({
      permission,
      expires_at: null,
      source: 'session',
      granted_by: null,
      granted_at: fixtureTimestamp,
      is_active: true,
      expires_soon: false,
      time_until_expiry: null,
      metadata: null,
    }));
    return json({
      success: true,
      data: {
        wallet_address: principal.subject,
        permissions,
        permission_version: 1,
        last_updated: fixtureTimestamp,
        total_permissions: permissions.length,
        active_permissions: permissions.length,
        expired_permissions: 0,
        expiring_soon: 0,
        has_admin_access: principal.permissions.some(permission =>
          permission.startsWith('admin:')
        ),
        platform_permissions: {
          epsx: principal.permissions.filter(permission =>
            permission.startsWith('epsx:')
          ),
          admin: principal.permissions.filter(permission =>
            permission.startsWith('admin:')
          ),
        },
      },
    });
  }
  if (url.pathname === '/api/users/access-overview') {
    const principal = fixturePrincipal(request);
    if (principal === null) {
      return json({ success: false, error: 'authentication_required' }, 401);
    }
    return json({
      success: true,
      data: {
        plan: fixtureTier,
        permissions: principal.permissions,
        expires_at: null,
      },
    });
  }
  if (url.pathname === '/api/payments/credits/balance') {
    return json({
      success: true,
      data: {
        wallet_address: fixtureWalletAddress,
        balance: 120,
        pending_balance: 0,
        available_balance: 120,
        lifetime_earned: 160,
        lifetime_spent: 40,
        last_transaction_at: fixtureTimestamp,
      },
    });
  }
  if (url.pathname === '/api/payments/credits/history') {
    return json({
      success: true,
      data: {
        success: true,
        data: [
          {
            id: 'credit-e2e-1',
            wallet_address: fixtureWalletAddress,
            amount: 120,
            balance_after: 120,
            tx_type: 'grant',
            reference_id: null,
            reference_type: null,
            reason: 'Migration baseline',
            granted_by: fixtureWalletAddress,
            expires_at: null,
            created_at: fixtureTimestamp,
          },
        ],
        count: 1,
      },
    });
  }
  if (url.pathname === '/api/payments/history') {
    return json({
      success: true,
      data: {
        payments: [],
        pagination: {
          page: 1,
          per_page: 10,
          total: 0,
          total_pages: 1,
        },
      },
    });
  }
  const notificationPath =
    url.pathname.startsWith('/api/v1/notification/') ||
    url.pathname.startsWith('/api/v1/notifications/');
  const adminNotificationPath =
    url.pathname.startsWith('/api/v1/notification/admin') ||
    url.pathname === '/api/v1/notification/send';
  const chatPath = url.pathname.startsWith('/api/admin/chat/');
  if (fixtureMode === 'notification-forbidden' && adminNotificationPath) {
    return json({ success: false, error: 'forbidden' }, 403);
  }
  if (fixtureMode === 'notification-unavailable' && notificationPath) {
    return json({ success: false, error: 'dependency_unavailable' }, 503);
  }
  if (fixtureMode === 'notification-malformed' && notificationPath) {
    return json({ malformed: true });
  }
  if (
    fixtureMode === 'notification-mutation-unavailable' &&
    notificationPath &&
    !['GET', 'HEAD'].includes(request.method)
  ) {
    return json({ success: false, error: 'dependency_unavailable' }, 503);
  }
  if (
    fixtureMode === 'notification-send-conflict' &&
    url.pathname === '/api/v1/notification/send'
  ) {
    return json({ success: false, error: 'idempotency_conflict' }, 409);
  }
  if (fixtureMode === 'chat-forbidden' && chatPath) {
    return json({ success: false, error: 'forbidden' }, 403);
  }
  if (fixtureMode === 'chat-unavailable' && chatPath) {
    return json({ success: false, error: 'dependency_unavailable' }, 503);
  }
  if (
    fixtureMode === 'chat-malformed' &&
    chatPath &&
    ['GET', 'HEAD'].includes(request.method)
  ) {
    return json({ success: true, data: { malformed: true }, error: null });
  }
  if (
    fixtureMode === 'chat-mutation-conflict' &&
    chatPath &&
    !['GET', 'HEAD'].includes(request.method)
  ) {
    return json({ success: false, error: 'idempotency_conflict' }, 409);
  }
  if (
    fixtureMode === 'chat-mutation-unavailable' &&
    chatPath &&
    !['GET', 'HEAD'].includes(request.method)
  ) {
    return json({ success: false, error: 'dependency_unavailable' }, 503);
  }
  if (url.pathname === '/api/notifications/preferences') {
    return json({
      success: true,
      data: {
        preferences: {
          analytics: true,
          security: true,
          account: true,
          system: false,
          marketing: false,
        },
      },
    });
  }
  if (url.pathname === '/api/v1/notification/preferences') {
    const principal = fixturePrincipal(request);
    const requiredPermission =
      request.method === 'GET'
        ? 'epsx:notifications:read'
        : 'epsx:notifications:update';
    if (
      principal?.audience !== 'epsx-frontend' ||
      !permissionAllows(principal.permissions, requiredPermission)
    ) {
      return json({ success: false, error: 'forbidden' }, 403);
    }
    return json({
      channels: { email: true, in_app: true, push: false },
      quiet_hours: {
        enabled: true,
        start: '22:00',
        end: '07:00',
      },
      timezone: 'Asia/Bangkok',
      updated_at: fixtureTimestamp,
    });
  }
  if (
    url.pathname === '/api/v1/notification/push' ||
    url.pathname === '/api/v1/notification/push/unsubscribe' ||
    url.pathname === '/api/v1/notifications/push' ||
    url.pathname === '/api/v1/notifications/push/unsubscribe'
  ) {
    return json({
      enabled: false,
      subscribed: false,
      public_key: null,
    });
  }
  if (
    url.pathname === '/api/notifications/stream' ||
    url.pathname === '/api/v1/notification/stream' ||
    url.pathname === '/api/v1/notifications/stream' ||
    url.pathname === '/api/chat/stream' ||
    url.pathname === '/api/chat/admin/stream'
  ) {
    return new Response(': deterministic fixture stream connected\n\n', {
      headers: {
        'cache-control': 'no-store',
        'content-type': 'text/event-stream',
        'x-epsx-e2e-fixture': '1',
      },
    });
  }
  if (
    url.pathname === '/api/v1/notification/stream/ack' ||
    url.pathname === '/api/v1/notifications/stream/ack'
  ) {
    return json({ acknowledged: true });
  }
  if (url.pathname === '/api/admin/notifications') {
    return json({
      success: true,
      data: {
        notifications: [],
        pagination: { page: 1, limit: 5, total: 0, total_pages: 1 },
      },
    });
  }
  if (url.pathname === '/api/v1/notification/list') {
    if (fixtureMode === 'notification-empty') {
      return json({ items: [], total: 0 });
    }
    return json({
      items: [
        {
          id: fixtureNotificationId,
          user_id: fixtureWalletAddress,
          channel: 'in_app',
          recipient: fixtureWalletAddress,
          template_id: null,
          subject: 'Security notice',
          body: 'Your deterministic migration notification is ready.',
          data: null,
          status: 'sent',
          error: null,
          sent_at: fixtureTimestamp,
          created_at: fixtureTimestamp,
          read_at: null,
          clicked_at: null,
          title: 'Migration notification',
          notification_type: 'security',
          priority: 'high',
          action_url: null,
          expires_at: null,
        },
      ],
      total: 1,
    });
  }
  if (url.pathname === '/api/v1/notification/unread-count') {
    const principal = fixturePrincipal(request);
    const canReadNotifications =
      principal?.audience === 'epsx-frontend' &&
      permissionAllows(principal.permissions, 'epsx:notifications:read');
    return json({
      count:
        canReadNotifications && fixtureMode !== 'notification-empty' ? 1 : 0,
    });
  }
  if (
    url.pathname === '/api/v1/notification/mark-all-read' ||
    url.pathname === '/api/v1/notification/clear-all' ||
    /^\/api\/v1\/notification\/[A-Za-z0-9_-]+(?:\/(?:read|unread|acknowledge|dismiss|click))?$/.test(
      url.pathname
    )
  ) {
    return json({ success: true, updated_count: 1, deleted_count: 1 });
  }
  if (url.pathname === '/api/v1/notification/send') {
    const status =
      fixtureMode === 'notification-send-pending'
        ? 'pending'
        : fixtureMode === 'notification-send-failed'
          ? 'failed'
          : 'sent';
    return json({
      id: 'idem_notification_send_e2e',
      status,
      delivered: status === 'sent',
      error: status === 'failed' ? 'provider_failed' : null,
      request_id: 'epsx-e2e-notification-send',
    });
  }
  if (url.pathname === '/api/v1/notification/admin/list') {
    const items =
      fixtureMode === 'notification-empty'
        ? []
        : [
            {
              id: fixtureNotificationId,
              title: 'Migration notification',
              subject: 'Security notice',
              channel: 'in_app',
              status: 'sent',
              notification_type: 'security',
              priority: 'high',
              sent_at: fixtureTimestamp,
              created_at: fixtureTimestamp,
            },
          ];
    return json({
      items,
      total: items.length,
      limit: 20,
      offset: Number(url.searchParams.get('offset') ?? '0'),
    });
  }
  if (url.pathname === '/api/v1/notification/admin/metrics') {
    return json({
      queue_depth: 1,
      queue_age_seconds: 5,
      suppressed: 1,
      retry_wait: 1,
      terminal_failed: 1,
      dead_lettered: 1,
      provider_accepted: 3,
      attempting: 1,
      channel_outcomes: { email: 1, in_app: 2, push: 0 },
      provider_events: 4,
      delivery_attempts: 5,
      replay_cursors: 1,
      replay_cursor_age_seconds: 2,
      active_streams: 1,
      stream_connections_total: 3,
      stream_reconnects_total: 1,
      stream_replayed_events_total: 2,
      stream_lag_seconds: 1,
      stream_query_failures_total: 0,
    });
  }
  if (
    /^\/api\/v1\/notification\/admin\/[A-Za-z0-9_-]+(?:\/read)?$/.test(
      url.pathname
    )
  ) {
    return json({ success: true });
  }
  if (url.pathname === '/api/admin/chat/conversations') {
    const items =
      fixtureMode === 'chat-empty'
        ? []
        : [
            {
              id: fixtureConversationId,
              topic_id: fixtureTopicId,
              wallet_address: fixtureWalletAddress,
              subject: 'Migration support conversation',
              status: 'open',
              assigned_agent: fixtureWalletAddress,
              last_message_at: fixtureTimestamp,
              unread_user: 0,
              unread_agent: 1,
              created_at: fixtureTimestamp,
              updated_at: fixtureTimestamp,
            },
          ];
    return json({
      success: true,
      data: {
        items,
        total: items.length,
        page: Number(url.searchParams.get('page') ?? '1'),
        limit: Number(url.searchParams.get('limit') ?? '20'),
        has_next: false,
      },
      error: null,
      meta: {
        timestamp: fixtureTimestamp,
        request_id: 'epsx-e2e-chat-list',
      },
    });
  }
  if (
    url.pathname === `/api/admin/chat/conversations/${fixtureConversationId}`
  ) {
    return json({
      success: true,
      data: {
        id: fixtureConversationId,
        topic_id: fixtureTopicId,
        wallet_address: fixtureWalletAddress,
        subject: 'Migration support conversation',
        status: 'open',
        assigned_agent: fixtureWalletAddress,
        last_message_at: fixtureTimestamp,
        unread_user: 0,
        unread_agent: 1,
        created_at: fixtureTimestamp,
        updated_at: fixtureTimestamp,
      },
      error: null,
      meta: {
        timestamp: fixtureTimestamp,
        request_id: 'epsx-e2e-chat-detail',
      },
    });
  }
  if (
    url.pathname ===
    `/api/admin/chat/conversations/${fixtureConversationId}/messages`
  ) {
    if (['GET', 'HEAD'].includes(request.method)) {
      return json({
        success: true,
        data: [
          {
            id: fixtureMessageId,
            conversation_id: fixtureConversationId,
            sender_type: 'user',
            sender_address: fixtureWalletAddress,
            content: 'Please verify my migration notification.',
            is_read: false,
            created_at: fixtureTimestamp,
          },
        ],
        error: null,
        meta: {
          timestamp: fixtureTimestamp,
          request_id: 'epsx-e2e-chat-messages',
        },
      });
    }
    return json({ success: true });
  }
  if (
    new RegExp(
      `^/api/admin/chat/conversations/${fixtureConversationId}/(?:status|assign|read)$`
    ).test(url.pathname)
  ) {
    return json({ success: true });
  }
  if (url.pathname === '/api/v1/content/news') {
    const articles = fixtureMode === 'content-empty' ? [] : targetPublicNews;
    return json({
      success: true,
      data: {
        articles,
        total: articles.length,
        page: 1,
        limit: 100,
      },
      error: null,
    });
  }
  if (url.pathname.startsWith('/api/v1/content/news/')) {
    const slug = url.pathname.split('/').at(-1);
    const article = targetPublicNews.find(candidate => candidate.slug === slug);
    return article
      ? json({ success: true, data: article, error: null })
      : json({ success: false, error: 'not_found' }, 404);
  }
  if (
    url.pathname.includes('/auth/me') ||
    url.pathname.includes('/auth/session') ||
    url.pathname.includes('/oauth/userinfo')
  ) {
    return json({ success: false, error: 'authentication_required' }, 401);
  }
  if (
    url.pathname === '/api/admin/me' ||
    url.pathname === '/api/users/profile'
  ) {
    const principal = fixturePrincipal(request);
    if (principal === null) {
      return json({ success: false, error: 'authentication_required' }, 401);
    }
    return json({
      success: true,
      data: {
        id: principal.subject,
        subject: principal.subject,
        wallet_address: principal.subject,
        permissions: principal.permissions,
        capabilities: [fixtureTier],
        auth_method: 'web3_siwe',
        tier: fixtureTier,
        status: 'active',
        created_at: fixtureTimestamp,
        last_login: fixtureTimestamp,
      },
    });
  }
  return json(
    {
      success: false,
      error: 'fixture_route_not_implemented',
      path: url.pathname,
    },
    404
  );
}

async function handler(request: Request): Promise<Response> {
  const response = await routeRequest(request);
  const origin = request.headers.get('origin');
  if (origin !== null && origin !== '') {
    response.headers.set('access-control-allow-origin', origin);
    response.headers.set('access-control-allow-credentials', 'true');
    response.headers.set('vary', 'origin');
  }
  return response;
}

const server = Bun.serve({
  hostname: '127.0.0.1',
  port,
  fetch: handler,
});

await log(`fixture listening on ${server.url}`);

for (const signal of ['SIGINT', 'SIGTERM'] as const) {
  process.on(signal, () => {
    void server.stop(true);
    process.exit(0);
  });
}
