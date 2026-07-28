import { appendFile, mkdir } from 'node:fs/promises';
import { dirname } from 'node:path';

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
// eslint-disable-next-line complexity
async function handler(request: Request): Promise<Response> {
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
    return json({ keys: [] });
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
  return json(
    {
      success: false,
      error: 'fixture_route_not_implemented',
      path: url.pathname,
    },
    404
  );
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
