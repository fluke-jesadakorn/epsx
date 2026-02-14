import type { MockHandler } from '../types';

const profile = {
  id: 'usr-e2e-001',
  wallet_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68',
  name: 'E2E Test User',
  email: 'e2e@epsx.test',
  avatar: null,
  role: 'user',
  bio: 'Crypto enthusiast and quantitative trader. Using EPSX for market analytics and portfolio management.',
  created_at: '2024-01-15T10:00:00Z',
  last_login: '2025-02-14T08:30:00Z',
  subscription: { plan: 'Pro', status: 'active', expires_at: '2025-03-15T00:00:00Z' },
  preferences: { notifications_email: true, notifications_push: true, theme: 'dark', language: 'en' },
};

const creditBalance = {
  balance: 250,
  total_purchased: 500,
  total_used: 250,
  last_purchase: '2025-02-13T14:00:00Z',
};

const paymentHistory = [
  { id: 'pay-001', type: 'subscription', amount: 29.00, currency: 'USDT', status: 'completed', description: 'Pro Plan - Monthly', tx_hash: '0xabc123...def456', created_at: '2025-02-01T00:00:00Z' },
  { id: 'pay-002', type: 'credits', amount: 29.00, currency: 'USDT', status: 'completed', description: '100 Credits Purchase', tx_hash: '0x789abc...123def', created_at: '2025-02-13T14:00:00Z' },
  { id: 'pay-003', type: 'subscription', amount: 29.00, currency: 'USDT', status: 'completed', description: 'Pro Plan - Monthly', tx_hash: '0xdef789...abc123', created_at: '2025-01-01T00:00:00Z' },
  { id: 'pay-004', type: 'credits', amount: 14.50, currency: 'USDT', status: 'completed', description: '50 Credits Purchase', tx_hash: '0x456def...789abc', created_at: '2024-12-20T10:00:00Z' },
  { id: 'pay-005', type: 'subscription', amount: 0, currency: 'USDT', status: 'completed', description: 'Free Plan - Activation', tx_hash: null, created_at: '2024-01-15T10:00:00Z' },
];

const accessOverview = {
  plan: 'Pro',
  permissions: ['epsx:analytics:view', 'epsx:portfolio:view', 'epsx:plans:view', 'epsx:notifications:view', 'epsx:developer:view', 'epsx:profile:manage'],
  api_keys: 2,
  active_sessions: 1,
  subscription_status: 'active',
  expires_at: '2025-03-15T00:00:00Z',
};

const portfolio = {
  holdings: [
    { symbol: 'AAPL', name: 'Apple Inc.', quantity: 50, avg_price: 175.20, current_price: 189.84, value: 9492.00, pnl: 732.00, pnl_pct: 8.35 },
    { symbol: 'TSLA', name: 'Tesla Inc.', quantity: 20, avg_price: 235.00, current_price: 248.42, value: 4968.40, pnl: 268.40, pnl_pct: 5.71 },
    { symbol: 'MSFT', name: 'Microsoft Corp.', quantity: 15, avg_price: 380.50, current_price: 415.60, value: 6234.00, pnl: 526.50, pnl_pct: 9.22 },
    { symbol: 'NVDA', name: 'NVIDIA Corp.', quantity: 10, avg_price: 720.00, current_price: 878.35, value: 8783.50, pnl: 1583.50, pnl_pct: 22.00 },
    { symbol: 'AMZN', name: 'Amazon.com Inc.', quantity: 30, avg_price: 165.80, current_price: 178.25, value: 5347.50, pnl: 373.50, pnl_pct: 7.51 },
  ],
  total_value: 34825.40,
  total_pnl: 3483.90,
  total_pnl_pct: 11.11,
};

const permissions = [
  { id: 'p-001', resource: 'epsx:analytics', action: 'view', granted_at: '2024-01-15T10:00:00Z', source: 'Pro Plan' },
  { id: 'p-002', resource: 'epsx:portfolio', action: 'view', granted_at: '2024-01-15T10:00:00Z', source: 'Pro Plan' },
  { id: 'p-003', resource: 'epsx:plans', action: 'view', granted_at: '2024-01-15T10:00:00Z', source: 'Free Plan' },
  { id: 'p-004', resource: 'epsx:notifications', action: 'view', granted_at: '2024-01-15T10:00:00Z', source: 'Free Plan' },
  { id: 'p-005', resource: 'epsx:developer', action: 'view', granted_at: '2025-02-01T00:00:00Z', source: 'Pro Plan' },
  { id: 'p-006', resource: 'epsx:profile', action: 'manage', granted_at: '2024-01-15T10:00:00Z', source: 'Free Plan' },
];

export const usersMocks: MockHandler[] = [
  {
    pattern: '**/api/users/profile**',
    handler: () => profile,
  },
  {
    pattern: '**/api/users/credits**',
    handler: () => creditBalance,
  },
  {
    pattern: '**/api/users/payments**',
    handler: () => ({ items: paymentHistory, total: paymentHistory.length }),
  },
  {
    pattern: '**/api/users/access**',
    handler: () => accessOverview,
  },
  {
    pattern: '**/api/users/portfolio**',
    handler: () => portfolio,
  },
  {
    pattern: '**/api/users/permissions**',
    handler: () => ({ items: permissions, total: permissions.length }),
  },
  {
    pattern: '**/api/permissions/**',
    handler: () => ({ items: permissions, total: permissions.length }),
  },
  {
    pattern: '**/api/portfolio**',
    handler: () => portfolio,
  },
];
