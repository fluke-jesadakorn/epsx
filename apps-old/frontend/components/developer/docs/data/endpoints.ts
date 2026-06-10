export interface ParamDef {
  name: string;
  type: string;
  required: boolean;
  desc: string;
  default?: string;
}

export interface HeaderDef {
  name: string;
  required: boolean;
  desc: string;
}

export type Tier = 'free' | 'basic' | 'premium' | 'enterprise';
export type Method = 'GET' | 'POST' | 'DELETE';

export interface EndpointDef {
  method: Method;
  path: string;
  title: string;
  desc: string;
  tier: Tier;
  params?: ParamDef[];
  headers: HeaderDef[];
  responseExample: Record<string, unknown>;
  rateLimits: Record<Tier, string>;
}

export interface EndpointCategory {
  id: string;
  title: string;
  desc: string;
  endpoints: EndpointDef[];
}

const bearerHeader: HeaderDef = {
  name: 'Authorization',
  required: true,
  desc: 'Bearer <api_key>',
};

const optionalBearerHeader: HeaderDef = {
  name: 'Authorization',
  required: false,
  desc: 'Bearer <api_key> — optional, unlocks premium columns',
};

const defaultRateLimits: Record<Tier, string> = {
  free: '30/min',
  basic: '60/min',
  premium: '120/min',
  enterprise: '600/min',
};

export const ENDPOINT_CATEGORIES: EndpointCategory[] = [
  {
    id: 'auth',
    title: 'Authentication',
    desc: 'API keys use the same Authorization header as JWT tokens. Pass your key as a Bearer token.',
    endpoints: [
      {
        method: 'GET',
        path: '/api/auth/session/verify',
        title: 'Verify session',
        desc: 'Verify that your API key is valid and return associated permissions.',
        tier: 'free',
        headers: [bearerHeader],
        responseExample: {
          success: true,
          data: {
            wallet_address: '0x1234...abcd',
            permissions: ['epsx:analytics:read', 'epsx:export:csv'],
            auth_method: 'api_key',
          },
        },
        rateLimits: defaultRateLimits,
      },
    ],
  },
  {
    id: 'analytics',
    title: 'Analytics',
    desc: 'Market data, stock rankings, filters, countries, and sector breakdowns.',
    endpoints: [
      {
        method: 'GET',
        path: '/api/analytics/rankings',
        title: 'Get stock rankings',
        desc: 'Returns paginated EPS rankings with optional filters. Free tier gets limited columns; premium unlocks all fields.',
        tier: 'free',
        params: [
          { name: 'page', type: 'number', required: false, desc: 'Page number', default: '1' },
          { name: 'per_page', type: 'number', required: false, desc: 'Results per page (max 100)', default: '20' },
          { name: 'sort_by', type: 'string', required: false, desc: 'Sort column (e.g. eps_growth, market_cap)', default: 'eps_growth' },
          { name: 'sort_dir', type: 'string', required: false, desc: 'asc or desc', default: 'desc' },
          { name: 'country', type: 'string', required: false, desc: 'ISO country code filter (e.g. US, TH)' },
          { name: 'sector', type: 'string', required: false, desc: 'Sector filter' },
          { name: 'search', type: 'string', required: false, desc: 'Search by ticker or company name' },
        ],
        headers: [optionalBearerHeader],
        responseExample: {
          success: true,
          data: {
            items: [
              {
                ticker: 'AAPL',
                name: 'Apple Inc.',
                country: 'US',
                sector: 'Technology',
                eps_growth: 12.5,
                market_cap: 3200000000000,
                rank: 1,
              },
            ],
            pagination: { page: 1, per_page: 20, total: 5420, total_pages: 271 },
          },
        },
        rateLimits: { free: '10/min', basic: '60/min', premium: '120/min', enterprise: '600/min' },
      },
      {
        method: 'GET',
        path: '/api/analytics/filters',
        title: 'Get filter options',
        desc: 'Returns available filter values for countries, sectors, and sort columns.',
        tier: 'free',
        headers: [optionalBearerHeader],
        responseExample: {
          success: true,
          data: {
            countries: [{ code: 'US', name: 'United States', count: 2100 }],
            sectors: [{ name: 'Technology', count: 450 }],
            sort_options: ['eps_growth', 'market_cap', 'revenue'],
          },
        },
        rateLimits: defaultRateLimits,
      },
      {
        method: 'GET',
        path: '/api/analytics/countries',
        title: 'Get countries',
        desc: 'Returns list of countries with stock data available.',
        tier: 'free',
        headers: [optionalBearerHeader],
        responseExample: {
          success: true,
          data: [{ code: 'US', name: 'United States', count: 2100 }],
        },
        rateLimits: defaultRateLimits,
      },
      {
        method: 'GET',
        path: '/api/analytics/sectors',
        title: 'Get sectors',
        desc: 'Returns available sector categories.',
        tier: 'free',
        headers: [optionalBearerHeader],
        responseExample: {
          success: true,
          data: [{ name: 'Technology', count: 450 }],
        },
        rateLimits: defaultRateLimits,
      },
    ],
  },
  {
    id: 'portfolio',
    title: 'Portfolio & Watchlist',
    desc: 'Manage your stock watchlist. Requires authentication.',
    endpoints: [
      {
        method: 'GET',
        path: '/api/users/watchlist',
        title: 'Get watchlist',
        desc: 'Returns current user watchlist with stock data.',
        tier: 'basic',
        headers: [bearerHeader],
        responseExample: {
          success: true,
          data: {
            items: [
              { ticker: 'AAPL', name: 'Apple Inc.', added_at: '2025-01-15T10:30:00Z' },
            ],
            count: 1,
          },
        },
        rateLimits: defaultRateLimits,
      },
      {
        method: 'POST',
        path: '/api/users/watchlist',
        title: 'Add to watchlist',
        desc: 'Add a stock ticker to your watchlist.',
        tier: 'basic',
        params: [
          { name: 'ticker', type: 'string', required: true, desc: 'Stock ticker symbol (e.g. AAPL)' },
        ],
        headers: [bearerHeader],
        responseExample: {
          success: true,
          data: { ticker: 'AAPL', added_at: '2025-01-15T10:30:00Z' },
        },
        rateLimits: defaultRateLimits,
      },
      {
        method: 'DELETE',
        path: '/api/users/watchlist',
        title: 'Remove from watchlist',
        desc: 'Remove a stock ticker from your watchlist.',
        tier: 'basic',
        params: [
          { name: 'ticker', type: 'string', required: true, desc: 'Stock ticker symbol to remove' },
        ],
        headers: [bearerHeader],
        responseExample: {
          success: true,
          data: { removed: true },
        },
        rateLimits: defaultRateLimits,
      },
    ],
  },
  {
    id: 'user',
    title: 'User',
    desc: 'User profile and access information.',
    endpoints: [
      {
        method: 'GET',
        path: '/api/users/profile',
        title: 'Get profile',
        desc: 'Returns the authenticated user profile including wallet address and plan info.',
        tier: 'free',
        headers: [bearerHeader],
        responseExample: {
          success: true,
          data: {
            wallet_address: '0x1234...abcd',
            plans: [{ name: 'Premium', slug: 'premium' }],
            created_at: '2025-01-01T00:00:00Z',
          },
        },
        rateLimits: defaultRateLimits,
      },
      {
        method: 'GET',
        path: '/api/users/access-overview',
        title: 'Get access overview',
        desc: 'Returns a summary of permissions and plan features available to the user.',
        tier: 'free',
        headers: [bearerHeader],
        responseExample: {
          success: true,
          data: {
            permissions: ['epsx:analytics:read'],
            plans: [{ name: 'Premium', features: ['Full rankings', 'CSV export'] }],
          },
        },
        rateLimits: defaultRateLimits,
      },
    ],
  },
];
