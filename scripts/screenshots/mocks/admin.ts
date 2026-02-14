import type { MockHandler } from '../types';

const auditLog = [
  { id: 'al-001', action: 'user.login', actor: '0x742d...bD68', target: null, details: 'Web3 SIWE login from Chrome/macOS', category: 'auth', ip: '192.168.1.42', timestamp: '2025-02-14T08:30:00Z' },
  { id: 'al-002', action: 'wallet.disable', actor: '0xAb58...eC9B', target: '0xDead...Beef', details: 'Wallet disabled: policy violation', category: 'admin', ip: '10.0.0.15', timestamp: '2025-02-14T08:15:00Z' },
  { id: 'al-003', action: 'permission.grant', actor: '0xAb58...eC9B', target: '0x742d...bD68', details: 'Granted epsx:developer:view', category: 'permissions', ip: '10.0.0.15', timestamp: '2025-02-13T16:00:00Z' },
  { id: 'al-004', action: 'plan.create', actor: '0xAb58...eC9B', target: 'plan-enterprise', details: 'Created Enterprise plan ($99/mo)', category: 'billing', ip: '10.0.0.15', timestamp: '2025-02-13T14:00:00Z' },
  { id: 'al-005', action: 'api_key.create', actor: '0x1234...5678', target: null, details: 'New API key: ak_test_...xyz', category: 'developer', ip: '203.0.113.50', timestamp: '2025-02-13T12:00:00Z' },
  { id: 'al-006', action: 'notification.send', actor: '0xAb58...eC9B', target: 'all_users', details: 'Broadcast: New Feature Launch', category: 'admin', ip: '10.0.0.15', timestamp: '2025-02-12T10:00:00Z' },
  { id: 'al-007', action: 'subscription.upgrade', actor: '0x742d...bD68', target: 'plan-pro', details: 'Upgraded Free -> Pro', category: 'billing', ip: '192.168.1.42', timestamp: '2025-02-10T10:00:00Z' },
  { id: 'al-008', action: 'wallet.suspend', actor: '0xAb58...eC9B', target: '0xFeed...Face', details: 'Suspended: suspicious activity', category: 'admin', ip: '10.0.0.15', timestamp: '2025-01-20T14:00:00Z' },
  { id: 'al-009', action: 'settings.update', actor: '0xAb58...eC9B', target: null, details: 'Updated maintenance window', category: 'system', ip: '10.0.0.15', timestamp: '2025-01-15T09:00:00Z' },
  { id: 'al-010', action: 'user.login', actor: '0xCafe...Babe', target: null, details: 'Web3 SIWE login from Firefox/Linux', category: 'auth', ip: '172.16.0.1', timestamp: '2025-01-14T16:00:00Z' },
  { id: 'al-011', action: 'credit.grant', actor: '0xAb58...eC9B', target: '0xAb58...eC9B', details: 'Admin credit grant: 500 credits', category: 'billing', ip: '10.0.0.15', timestamp: '2025-01-10T09:00:00Z' },
  { id: 'al-012', action: 'permission.revoke', actor: '0xAb58...eC9B', target: '0xFeed...Face', details: 'Revoked all permissions', category: 'permissions', ip: '10.0.0.15', timestamp: '2025-01-20T14:05:00Z' },
];

const settings = {
  maintenance_mode: false,
  registration_open: true,
  max_api_calls_per_day: 10000,
  max_watchlists: 10,
  default_plan: 'free',
  supported_chains: ['BSC Mainnet', 'BSC Testnet'],
  feature_flags: { portfolio_analytics: true, webhook_notifications: true, csv_exports: true },
};

const devStats = {
  total_api_keys: 24,
  active_api_keys: 18,
  total_api_calls_today: 45230,
  total_api_calls_month: 1234567,
  avg_response_time_ms: 142,
  error_rate_pct: 0.3,
};

const apiKeys = [
  { id: 'ak-001', name: 'Production API', key_prefix: 'ak_prod_...xyz', status: 'active', permissions: ['analytics:read', 'portfolio:read'], created_at: '2024-06-01T10:00:00Z', last_used: '2025-02-14T08:00:00Z', calls_today: 1250 },
  { id: 'ak-002', name: 'Development API', key_prefix: 'ak_dev_...abc', status: 'active', permissions: ['analytics:read'], created_at: '2024-08-15T12:00:00Z', last_used: '2025-02-13T22:00:00Z', calls_today: 340 },
  { id: 'ak-003', name: 'Webhook Handler', key_prefix: 'ak_hook_...def', status: 'revoked', permissions: ['notifications:write'], created_at: '2024-03-10T08:00:00Z', last_used: '2024-09-01T10:00:00Z', calls_today: 0 },
];

const subscriptions = [
  { id: 'sub-001', user_wallet: '0x742d...bD68', plan: 'Pro', status: 'active', amount: 29.00, currency: 'USDT', billing_cycle: 'monthly', started_at: '2025-02-01T00:00:00Z', expires_at: '2025-03-01T00:00:00Z', auto_renew: true },
  { id: 'sub-002', user_wallet: '0x1234...5678', plan: 'Enterprise', status: 'active', amount: 99.00, currency: 'USDT', billing_cycle: 'monthly', started_at: '2025-01-15T00:00:00Z', expires_at: '2025-02-15T00:00:00Z', auto_renew: true },
  { id: 'sub-003', user_wallet: '0xCafe...Babe', plan: 'Free', status: 'active', amount: 0, currency: 'USDT', billing_cycle: 'monthly', started_at: '2024-08-15T00:00:00Z', expires_at: null, auto_renew: false },
];

const affiliates = [
  { id: 'aff-001', wallet: '0x742d...bD68', code: 'EPSX2024', referrals: 12, earnings: 145.50, status: 'active', created_at: '2024-06-01T10:00:00Z' },
  { id: 'aff-002', wallet: '0x1234...5678', code: 'TRADER99', referrals: 5, earnings: 62.00, status: 'active', created_at: '2024-09-15T12:00:00Z' },
  { id: 'aff-003', wallet: '0xCafe...Babe', code: 'APIDEV', referrals: 0, earnings: 0, status: 'pending', created_at: '2025-01-10T08:00:00Z' },
];

const payments = [
  { id: 'pay-001', wallet: '0x742d...bD68', type: 'subscription', amount: 29.00, currency: 'USDT', status: 'completed', tx_hash: '0xabc123...', created_at: '2025-02-01T00:00:00Z' },
  { id: 'pay-002', wallet: '0x1234...5678', type: 'subscription', amount: 99.00, currency: 'USDT', status: 'completed', tx_hash: '0xdef456...', created_at: '2025-01-15T00:00:00Z' },
  { id: 'pay-003', wallet: '0x742d...bD68', type: 'credits', amount: 29.00, currency: 'USDT', status: 'completed', tx_hash: '0x789abc...', created_at: '2025-02-13T14:00:00Z' },
  { id: 'pay-004', wallet: '0xCafe...Babe', type: 'credits', amount: 14.50, currency: 'USDT', status: 'pending', tx_hash: null, created_at: '2025-02-14T10:00:00Z' },
  { id: 'pay-005', wallet: '0x742d...bD68', type: 'subscription', amount: 29.00, currency: 'USDT', status: 'completed', tx_hash: '0xaaa111...', created_at: '2025-01-01T00:00:00Z' },
];

const accessPlans = [
  { id: 'plan-1', name: 'Basic Analytics', permissions: ['epsx:analytics:view'], users_count: 45, status: 'active' },
  { id: 'plan-2', name: 'Full Access', permissions: ['epsx:analytics:view', 'epsx:portfolio:view', 'epsx:developer:view'], users_count: 18, status: 'active' },
  { id: 'plan-3', name: 'Developer Only', permissions: ['epsx:developer:view'], users_count: 7, status: 'active' },
];

const accessPermissions = [
  { id: 'perm-1', resource: 'epsx:analytics', action: 'view', description: 'View stock analytics and rankings', assigned_users: 63 },
  { id: 'perm-2', resource: 'epsx:portfolio', action: 'view', description: 'View portfolio and holdings', assigned_users: 18 },
  { id: 'perm-3', resource: 'epsx:developer', action: 'view', description: 'Access developer portal and API', assigned_users: 25 },
  { id: 'perm-4', resource: 'epsx:notifications', action: 'view', description: 'View and manage notifications', assigned_users: 70 },
  { id: 'perm-5', resource: 'epsx:plans', action: 'view', description: 'View subscription plans', assigned_users: 70 },
  { id: 'perm-6', resource: 'epsx:profile', action: 'manage', description: 'Manage user profile settings', assigned_users: 70 },
];

const dashboardStats = {
  total_users: 1247,
  active_users_today: 342,
  total_revenue: 45230.50,
  monthly_revenue: 12450.00,
  active_wallets: 892,
  api_calls_today: 145230,
  new_users_week: 38,
  churn_rate: 2.1,
};

export const adminMocks: MockHandler[] = [
  {
    pattern: '**/api/admin/dashboard**',
    handler: () => dashboardStats,
  },
  {
    pattern: '**/api/admin/audit**',
    handler: (url: URL) => {
      const search = url.searchParams.get('search') ?? '';
      const category = url.searchParams.get('category');
      let filtered = [...auditLog];
      if (search) filtered = filtered.filter(e => e.action.includes(search) || e.details.toLowerCase().includes(search.toLowerCase()));
      if (category) filtered = filtered.filter(e => e.category === category);
      return { items: filtered, total: filtered.length };
    },
  },
  {
    pattern: '**/api/admin/settings**',
    handler: () => settings,
  },
  {
    pattern: '**/api/admin/developer/stats**',
    handler: () => devStats,
  },
  {
    pattern: '**/api/admin/developer/api-keys**',
    handler: () => ({ items: apiKeys, total: apiKeys.length }),
  },
  {
    pattern: '**/api/admin/subscriptions/sub-*',
    handler: () => subscriptions[0],
  },
  {
    pattern: '**/api/admin/subscriptions**',
    handler: () => ({ items: subscriptions, total: subscriptions.length }),
  },
  {
    pattern: '**/api/admin/affiliates**',
    handler: () => ({ items: affiliates, total: affiliates.length }),
  },
  {
    pattern: '**/api/admin/payments**',
    handler: () => ({ items: payments, total: payments.length }),
  },
  {
    pattern: '**/api/admin/access/plans/plan-*',
    handler: () => accessPlans[0],
  },
  {
    pattern: '**/api/admin/access/plans**',
    handler: () => ({ items: accessPlans, total: accessPlans.length }),
  },
  {
    pattern: '**/api/admin/access/permissions**',
    handler: () => ({ items: accessPermissions, total: accessPermissions.length }),
  },
  {
    pattern: '**/api/admin/access**',
    handler: () => ({ plans: accessPlans, permissions: accessPermissions, total_users: 70 }),
  },
  {
    pattern: '**/api/admin/users**',
    handler: () => ({
      items: [
        { id: 'usr-e2e-001', wallet_address: '0x742d...bD68', name: 'E2E Test User', role: 'user', plan: 'Pro', status: 'active', last_login: '2025-02-14T08:30:00Z' },
        { id: 'usr-002', wallet_address: '0x1234...5678', name: 'Trading Bot', role: 'user', plan: 'Enterprise', status: 'active', last_login: '2025-02-13T22:15:00Z' },
        { id: 'usr-003', wallet_address: '0xCafe...Babe', name: 'API Test', role: 'user', plan: 'Free', status: 'active', last_login: '2025-02-12T16:00:00Z' },
      ],
      total: 3,
    }),
  },
];
