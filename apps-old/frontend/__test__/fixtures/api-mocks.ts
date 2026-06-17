/**
 * API Mock Fixtures for Frontend E2E Tests
 * Provides mock data for all API endpoints used by the main frontend app.
 */

export const MOCK_WALLET = '0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68';

export const MOCK_USER = {
  id: 'usr-e2e-001',
  wallet_address: MOCK_WALLET,
  name: 'E2E Test User',
  email: 'e2e@epsx.test',
  role: 'user',
  created_at: '2024-06-01T00:00:00Z',
  updated_at: '2024-06-01T00:00:00Z',
};

export const MOCK_TOKEN = 'eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c3ItZTJlLTAwMSIsImlhdCI6MTcwMDAwMDAwMH0.e2e-test-token';

export const MOCK_PERMISSIONS = [
  'epsx:analytics:view',
  'epsx:portfolio:view',
  'epsx:plans:view',
  'epsx:notifications:view',
  'epsx:developer:view',
  'epsx:profile:manage',
  'epsx:account:manage',
];

const ok = <T>(data: T) => ({ status: 200, data, success: true });
const list = <T>(items: T[], total?: number) => ok({
  items,
  pagination: { page: 1, limit: 20, total: total ?? items.length, totalPages: 1 },
});

export const API_MOCKS = {
  // Auth endpoints
  'POST /api/auth/web3/challenge': ok({ challenge: 'mock-siwe-challenge-nonce', nonce: 'abc123' }),
  'POST /api/auth/web3/verify': ok({ token: MOCK_TOKEN, user: MOCK_USER }),
  'GET /api/auth/web3/session': ok({ authenticated: true, user: MOCK_USER, token: MOCK_TOKEN }),
  'GET /api/auth/session/verify': ok({ valid: true, user: MOCK_USER }),
  'POST /api/auth/session/refresh': ok({ token: MOCK_TOKEN }),
  'GET /api/auth/users/profile': ok(MOCK_USER),
  'GET /api/auth/users/permissions': ok({ permissions: MOCK_PERMISSIONS }),
  'GET /api/auth/users/sessions': ok({ sessions: [{ id: 'sess-001', active: true, created_at: '2024-06-01T00:00:00Z' }] }),
  'POST /api/auth/web3/logout': ok({ success: true }),

  // User endpoints
  'GET /api/users/profile': ok(MOCK_USER),
  'PUT /api/users/profile': ok(MOCK_USER),
  'GET /api/users/watchlist': ok({ stocks: [{ symbol: 'AAPL', name: 'Apple', price: 185.5 }, { symbol: 'GOOGL', name: 'Alphabet', price: 142.3 }] }),
  'GET /api/users/settings': ok({ theme: 'dark', notifications: true, language: 'en' }),
  'PUT /api/users/settings': ok({ theme: 'dark', notifications: true, language: 'en' }),
  'GET /api/users/access-overview': ok({ plan: 'pro', permissions: MOCK_PERMISSIONS, expires_at: '2025-12-31T00:00:00Z' }),
  'GET /api/notifications': list([
    { id: 'n1', title: 'Welcome', message: 'Welcome to EPSX', read: false, created_at: '2024-06-01T00:00:00Z' },
    { id: 'n2', title: 'Plan Activated', message: 'Your Pro plan is active', read: true, created_at: '2024-05-28T00:00:00Z' },
  ]),

  // Analytics endpoints
  'GET /api/analytics/rankings': list([
    { rank: 1, symbol: 'AAPL', name: 'Apple Inc', score: 95.2, change: 2.1, sector: 'Technology', country: 'US' },
    { rank: 2, symbol: 'MSFT', name: 'Microsoft', score: 93.8, change: -0.5, sector: 'Technology', country: 'US' },
    { rank: 3, symbol: 'GOOGL', name: 'Alphabet', score: 91.5, change: 1.3, sector: 'Technology', country: 'US' },
    { rank: 4, symbol: 'AMZN', name: 'Amazon', score: 89.1, change: 0.8, sector: 'Consumer', country: 'US' },
    { rank: 5, symbol: 'NVDA', name: 'NVIDIA', score: 88.7, change: 3.2, sector: 'Technology', country: 'US' },
  ], 50),
  'GET /api/analytics/countries': ok([
    { code: 'US', name: 'United States', count: 120 },
    { code: 'GB', name: 'United Kingdom', count: 45 },
    { code: 'JP', name: 'Japan', count: 38 },
  ]),
  'GET /api/analytics/sectors': ok([
    { id: 'tech', name: 'Technology', count: 85 },
    { id: 'finance', name: 'Finance', count: 62 },
    { id: 'health', name: 'Healthcare', count: 41 },
  ]),
  'GET /api/analytics/filters': ok({ countries: ['US', 'GB', 'JP'], sectors: ['Technology', 'Finance', 'Healthcare'], sortOptions: ['score', 'change', 'name'] }),
  'GET /api/analytics/market/overview': ok({ totalMarketCap: 45_000_000_000_000, dailyVolume: 280_000_000_000, activePairs: 12500, lastUpdated: '2024-06-01T00:00:00Z' }),
  'GET /api/analytics/market/trends': ok([{ date: '2024-06-01', value: 45000 }, { date: '2024-05-31', value: 44800 }]),
  'GET /api/analytics/market/top-gainers': list([{ symbol: 'NVDA', change: 5.2 }, { symbol: 'TSLA', change: 3.8 }]),
  'GET /api/analytics/market/top-losers': list([{ symbol: 'META', change: -2.1 }, { symbol: 'NFLX', change: -1.5 }]),
  'GET /api/analytics/performance': ok({ daily: 2.1, weekly: 5.4, monthly: 12.3, yearly: 28.7 }),
  'GET /api/analytics/portfolio/analysis': ok({ totalValue: 125000, holdings: 12, topSector: 'Technology', diversification: 0.72 }),
  'GET /api/analytics/export': ok({ url: '/exports/analytics-2024.csv' }),
  'GET /api/public/analytics/rankings': list([
    { rank: 1, symbol: 'AAPL', name: 'Apple Inc', score: 95.2, change: 2.1 },
    { rank: 2, symbol: 'MSFT', name: 'Microsoft', score: 93.8, change: -0.5 },
  ], 50),
  'GET /api/public/analytics/filters': ok({ countries: ['US', 'GB'], sectors: ['Technology'], sortOptions: ['score'] }),

  // Notification endpoints
  'GET /api/notifications/stream': ok({ type: 'connected' }),
  'GET /api/notifications/sse': ok({ type: 'connected' }),
  'GET /api/notifications/preferences': ok({ email: true, push: true, inApp: true }),
  'PUT /api/notifications/preferences': ok({ email: true, push: true, inApp: true }),
  'GET /api/notifications/history': list([
    { id: 'nh1', title: 'Price Alert', message: 'AAPL crossed $185', created_at: '2024-06-01T00:00:00Z' },
  ]),

  // Plans & Payments
  'GET /api/public/plans': ok([
    { id: 'free', name: 'Free', price: 0, features: ['Basic Analytics', '5 Watchlist Items'] },
    { id: 'pro', name: 'Pro', price: 29, features: ['Advanced Analytics', 'Unlimited Watchlist', 'Export Data'] },
    { id: 'enterprise', name: 'Enterprise', price: 99, features: ['Everything in Pro', 'API Access', 'Priority Support'] },
  ]),
  'GET /api/public/plans/features': ok([
    { id: 'analytics', name: 'Analytics', tiers: { free: 'basic', pro: 'advanced', enterprise: 'advanced' } },
    { id: 'watchlist', name: 'Watchlist', tiers: { free: '5', pro: 'unlimited', enterprise: 'unlimited' } },
  ]),
  'GET /api/plans/subscription': ok({ planId: 'pro', status: 'active', startDate: '2024-01-01', endDate: '2025-01-01' }),
  'GET /api/plans/subscription/status': ok({ active: true, plan: 'pro', daysRemaining: 180 }),
  'GET /api/plans/subscription/history': list([{ id: 'sub1', plan: 'pro', status: 'active', date: '2024-01-01' }]),
  'GET /api/plans/usage': ok({ apiCalls: 1200, limit: 10000, period: 'monthly' }),
  'GET /api/plans/billing': ok({ nextBilling: '2024-07-01', amount: 29, method: 'crypto' }),
  'GET /api/plans/invoices': list([{ id: 'inv1', amount: 29, date: '2024-06-01', status: 'paid' }]),
  'GET /api/payments/history': list([{ id: 'pay1', amount: 29, method: 'USDT', date: '2024-06-01', status: 'completed' }]),

  // Permissions
  'POST /api/permissions/validate': ok({ valid: true }),
  'POST /api/permissions/validate-bulk': ok({ results: MOCK_PERMISSIONS.map(p => ({ permission: p, valid: true })) }),
  'GET /api/permissions/check': ok({ allowed: true }),

  // Public
  'GET /api/public/status': ok({ status: 'operational', version: '2.1.0', uptime: 99.99 }),
  'GET /api/public/networks': ok([{ id: 56, name: 'BSC Mainnet', rpc: 'https://bsc-dataseed.binance.org' }, { id: 97, name: 'BSC Testnet', rpc: 'https://data-seed-prebsc-1-s1.binance.org:8545' }]),
} as const;
