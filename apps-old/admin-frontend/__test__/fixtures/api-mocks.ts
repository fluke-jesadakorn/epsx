/**
 * API Mock Fixtures for Admin Frontend E2E Tests
 * Provides mock data for all API endpoints used by the admin portal.
 */

import { TEST_SESSIONS, TEST_USERS } from './admin-test-fixtures';

export const MOCK_WALLET = '0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B';

export const MOCK_ADMIN = {
  id: 'admin-e2e-001',
  wallet_address: MOCK_WALLET,
  name: 'E2E Admin',
  email: 'admin-e2e@epsx.test',
  role: 'admin',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-06-01T00:00:00Z',
};

export const MOCK_TOKEN = TEST_SESSIONS.VALID_ADMIN?.token ?? 'mock-admin-token';

export const ADMIN_PERMISSIONS = [
  'admin:users:manage',
  'admin:wallets:manage',
  'admin:permissions:manage',
  'admin:analytics:view',
  'admin:notifications:manage',
  'admin:plans:manage',
  'admin:settings:manage',
  'admin:audit:view',
  'admin:developer:manage',
  'admin:system:manage',
];

const ok = <T>(data: T) => ({ status: 200, data, success: true });
const list = <T>(items: T[], total?: number) => ok({
  items,
  pagination: { page: 1, limit: 20, total: total ?? items.length, totalPages: 1 },
});

export const API_MOCKS = {
  // Auth
  'POST /api/auth/web3/challenge': ok({ challenge: 'mock-admin-siwe-challenge', nonce: 'adm123' }),
  'POST /api/auth/web3/verify': ok({ token: MOCK_TOKEN, user: MOCK_ADMIN }),
  'GET /api/auth/web3/session': ok({ authenticated: true, user: MOCK_ADMIN, token: MOCK_TOKEN }),
  'GET /api/auth/session/verify': ok({ valid: true, user: MOCK_ADMIN }),
  'POST /api/auth/session/refresh': ok({ token: MOCK_TOKEN }),
  'GET /api/auth/users/profile': ok(MOCK_ADMIN),
  'GET /api/auth/users/permissions': ok({ permissions: ADMIN_PERMISSIONS }),
  'POST /api/auth/web3/logout': ok({ success: true }),

  // Admin Users
  'GET /api/admin/users': list(Object.values(TEST_USERS).map(u => ({
    ...u, wallet_address: `0x${u.id.replace(/[^a-f0-9]/gi, '').padEnd(40, '0')}`, created_at: '2024-01-01T00:00:00Z',
  })), 8),
  'GET /api/admin/users/:id': ok({ ...TEST_USERS.ADMIN, wallet_address: MOCK_WALLET }),

  // Admin Wallets
  'GET /api/admin/wallets': list([
    { address: MOCK_WALLET, label: 'Main Admin', balance: '12.5', status: 'active', lastActivity: '2024-06-01T00:00:00Z' },
    { address: '0x1234567890abcdef1234567890abcdef12345678', label: 'User Wallet', balance: '3.2', status: 'active', lastActivity: '2024-05-28T00:00:00Z' },
    { address: '0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef', label: 'Disabled', balance: '0', status: 'disabled', lastActivity: '2024-03-15T00:00:00Z' },
  ]),
  'GET /api/admin/wallets/:address': ok({ address: MOCK_WALLET, label: 'Main Admin', balance: '12.5', status: 'active', transactions: 145, permissions: ADMIN_PERMISSIONS }),
  'GET /api/admin/wallets/:address/transactions': list([
    { id: 'tx1', hash: '0xabc123', amount: '1.5', type: 'transfer', status: 'confirmed', date: '2024-06-01T00:00:00Z' },
    { id: 'tx2', hash: '0xdef456', amount: '0.8', type: 'payment', status: 'confirmed', date: '2024-05-30T00:00:00Z' },
  ]),

  // Permissions
  'GET /api/admin/permissions': ok({ permissions: ADMIN_PERMISSIONS, total: ADMIN_PERMISSIONS.length }),
  'GET /api/permissions/plans': list([
    { id: 'plan-1', name: 'Basic Access', permissions: ['epsx:analytics:view'], wallets: 12 },
    { id: 'plan-2', name: 'Pro Access', permissions: ['epsx:analytics:view', 'epsx:portfolio:view', 'epsx:developer:view'], wallets: 45 },
  ]),
  'GET /api/permissions/assignments': list([
    { id: 'asgn-1', wallet: MOCK_WALLET, plan: 'Pro Access', grantedAt: '2024-01-15T00:00:00Z', expiresAt: '2025-01-15T00:00:00Z' },
  ]),
  'POST /api/admin/permissions/grant': ok({ success: true }),
  'POST /api/admin/permissions/revoke': ok({ success: true }),

  // Analytics
  'GET /api/admin/analytics': ok({ users: 150, revenue: 45000, activeWallets: 89, apiCalls: 125000 }),
  'GET /api/admin/analytics/overview': ok({ totalUsers: 150, activeToday: 42, newThisWeek: 12, revenue: { monthly: 15000, total: 45000 } }),
  'GET /api/admin/analytics/users': ok({ signups: [{ date: '2024-06-01', count: 5 }], retention: 0.85, churn: 0.02 }),
  'GET /api/admin/analytics/revenue': ok({ monthly: [{ month: '2024-06', amount: 15000 }], mrr: 15000, arr: 180000 }),
  'GET /api/admin/analytics/performance': ok({ avgResponseTime: 120, p95: 450, p99: 890, uptime: 99.99 }),

  // Notifications
  'GET /api/admin/notifications': list([
    { id: 'an1', title: 'System Update', message: 'Maintenance scheduled', type: 'system', created_at: '2024-06-01T00:00:00Z', recipients: 150 },
    { id: 'an2', title: 'New Feature', message: 'Portfolio tracking launched', type: 'feature', created_at: '2024-05-28T00:00:00Z', recipients: 150 },
  ]),
  'POST /api/admin/notifications': ok({ id: 'an3', success: true }),
  'GET /api/admin/notifications/templates': ok([
    { id: 'tpl1', name: 'Welcome', subject: 'Welcome to EPSX', body: 'Hello {{name}}!' },
    { id: 'tpl2', name: 'Maintenance', subject: 'Scheduled Maintenance', body: 'Maintenance on {{date}}' },
  ]),
  'POST /api/admin/notifications/broadcast': ok({ sent: 150, failed: 0 }),

  // Plans
  'GET /api/admin/plans': ok([
    { id: 'free', name: 'Free', price: 0, subscribers: 60, features: ['Basic Analytics'] },
    { id: 'pro', name: 'Pro', price: 29, subscribers: 75, features: ['Advanced Analytics', 'Unlimited Watchlist'] },
    { id: 'enterprise', name: 'Enterprise', price: 99, subscribers: 15, features: ['Everything'] },
  ]),
  'GET /api/plans/subscription': ok({ planId: 'enterprise', status: 'active', startDate: '2024-01-01' }),

  // Audit
  'GET /api/admin/audit-logs': list([
    { id: 'al1', action: 'user.login', actor: MOCK_WALLET, target: 'session', timestamp: '2024-06-01T10:00:00Z', ip: '192.168.1.1' },
    { id: 'al2', action: 'permission.grant', actor: MOCK_WALLET, target: 'wallet-xyz', timestamp: '2024-06-01T09:30:00Z', ip: '192.168.1.1' },
    { id: 'al3', action: 'plan.update', actor: MOCK_WALLET, target: 'plan-pro', timestamp: '2024-06-01T09:00:00Z', ip: '192.168.1.1' },
  ]),

  // System
  'GET /api/admin/system/status': ok({ status: 'operational', services: { api: 'up', db: 'up', redis: 'up', blockchain: 'up' } }),
  'GET /api/admin/system/metrics': ok({ cpu: 35, memory: 62, disk: 45, connections: 89 }),
  'GET /api/admin/settings': ok({ maintenanceMode: false, registrationOpen: true, maxApiKeys: 5 }),

  // Developer Portal
  'GET /api/admin/developer-portal/api-keys': list([
    { id: 'key1', name: 'Production Key', prefix: 'epsx_pk_', created_at: '2024-01-15T00:00:00Z', lastUsed: '2024-06-01T00:00:00Z', status: 'active' },
    { id: 'key2', name: 'Test Key', prefix: 'epsx_tk_', created_at: '2024-03-01T00:00:00Z', lastUsed: '2024-05-15T00:00:00Z', status: 'active' },
  ]),
  'POST /api/admin/developer-portal/api-keys': ok({ id: 'key3', key: 'epsx_pk_new123', name: 'New Key' }),
  'GET /api/admin/developer-portal/stats': ok({ totalKeys: 2, activeKeys: 2, totalCalls: 125000, callsToday: 3400 }),

  // Payments
  'GET /api/payments/history': list([
    { id: 'p1', wallet: MOCK_WALLET, amount: 99, plan: 'Enterprise', method: 'USDT', date: '2024-06-01T00:00:00Z', status: 'completed' },
    { id: 'p2', wallet: '0x1234...5678', amount: 29, plan: 'Pro', method: 'BNB', date: '2024-05-28T00:00:00Z', status: 'completed' },
  ]),

  // Public
  'GET /api/public/plans': ok([
    { id: 'free', name: 'Free', price: 0, features: ['Basic Analytics'] },
    { id: 'pro', name: 'Pro', price: 29, features: ['Advanced Analytics'] },
    { id: 'enterprise', name: 'Enterprise', price: 99, features: ['Everything'] },
  ]),
  'GET /api/public/status': ok({ status: 'operational', version: '2.1.0' }),
  'GET /api/public/networks': ok([{ id: 56, name: 'BSC Mainnet' }, { id: 97, name: 'BSC Testnet' }]),
} as const;
