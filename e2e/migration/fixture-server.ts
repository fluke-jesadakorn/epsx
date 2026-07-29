import { appendFile, mkdir } from 'node:fs/promises';
import { dirname } from 'node:path';
import { createPrivateKey, sign } from 'node:crypto';

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

function fixtureAccessToken(
  issuer: string,
  audience: string,
  permissions: string
): string {
  const address = '0xea6400000000000000000000000000000000e3df';
  const header = base64Url(
    JSON.stringify({ alg: 'RS256', typ: 'JWT', kid: signingKeyId })
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

function fixturePrincipal(request: Request): {
  subject: string;
  permissions: string[];
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
    ) as { sub?: unknown; wallet_address?: unknown; scope?: unknown };
    const subject =
      typeof claims.sub === 'string' &&
      typeof claims.wallet_address === 'string' &&
      claims.sub === claims.wallet_address
        ? claims.sub
        : null;
    if (subject === null || typeof claims.scope !== 'string') {
      return null;
    }
    return {
      subject,
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

function rankingResponse(limit: number): unknown {
  const selected = rankings.slice(0, limit);
  return {
    success: true,
    data: selected,
    pagination: {
      page: 1,
      limit,
      total: rankings.length,
      totalPages: 1,
      hasNext: false,
      hasPrev: false,
    },
    metadata: {
      available_countries: ['United States'],
      available_sectors: ['Technology'],
      request_timestamp: '2026-07-28T00:00:00Z',
      data_source: 'epsx-e2e-fixture-v1',
    },
    access_info: {
      min_accessible_rank: 1,
      locked_ranks_count: 0,
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
    return json({
      accessToken: fixtureAccessToken(
        url.origin,
        audience,
        url.searchParams.get('permissions') ?? ''
      ),
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
  if (url.pathname.includes('jwks')) {
    return json({ keys: [signingJwk] });
  }
  if (fixtureMode === 'dependency-unavailable') {
    return json({ success: false, error: 'dependency_unavailable' }, 503);
  }
  if (fixtureMode === 'malformed') {
    return json({ malformed: true });
  }
  if (
    url.pathname === '/api/analytics/rankings' ||
    url.pathname === '/api/public/analytics/rankings'
  ) {
    const limit = Math.max(
      1,
      Math.min(10, Number(url.searchParams.get('limit') ?? '3'))
    );
    return json(rankingResponse(limit));
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
  if (url.pathname === '/api/public/plans') {
    return json({ success: true, data: publicPlans });
  }
  if (url.pathname === '/api/public/news/featured') {
    return json({ success: true, data: publicNews });
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
  if (url.pathname === '/api/v1/notification/list') {
    return json({ items: [], total: 0 });
  }
  if (url.pathname === '/api/v1/notification/unread-count') {
    return json({ count: 0 });
  }
  if (url.pathname === '/api/v1/content/news') {
    return json({ success: true, data: publicNews });
  }
  if (url.pathname.startsWith('/api/v1/content/news/')) {
    const slug = url.pathname.split('/').at(-1);
    const article = publicNews.find(candidate => candidate.slug === slug);
    return article
      ? json({ success: true, data: article })
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
        subject: principal.subject,
        wallet_address: principal.subject,
        permissions: principal.permissions,
        capabilities: ['migration-e2e'],
        auth_method: 'web3_siwe',
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
