import type { MockHandler } from '../types';

const wallets = [
  { address: '0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68', name: 'Primary Wallet', status: 'active', platform: 'epsx', chain: 'BSC', balance: '12,450.00 USDT', credits: 250, last_active: '2025-02-14T08:30:00Z', created_at: '2024-01-15T10:00:00Z', transactions: 142 },
  { address: '0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B', name: 'Admin Wallet', status: 'active', platform: 'admin', chain: 'BSC', balance: '45,200.00 USDT', credits: 1000, last_active: '2025-02-14T09:00:00Z', created_at: '2024-01-01T00:00:00Z', transactions: 89 },
  { address: '0x1234567890AbCdEf1234567890aBcDeF12345678', name: 'Trading Bot', status: 'active', platform: 'epsx-pay', chain: 'BSC', balance: '3,100.50 USDT', credits: 75, last_active: '2025-02-13T22:15:00Z', created_at: '2024-06-20T14:30:00Z', transactions: 1245 },
  { address: '0xDeadBeefDeadBeefDeadBeefDeadBeefDeadBeef', name: 'Legacy Wallet', status: 'disabled', platform: 'epsx', chain: 'BSC', balance: '0.00 USDT', credits: 0, last_active: '2024-09-01T10:00:00Z', created_at: '2024-03-10T08:00:00Z', transactions: 23 },
  { address: '0xCafeBabeCafeBabeCafeBabeCafeBabeCafeBabe', name: 'API Test Wallet', status: 'active', platform: 'epsx-token', chain: 'BSC Testnet', balance: '500.00 USDT', credits: 150, last_active: '2025-02-12T16:00:00Z', created_at: '2024-08-15T12:00:00Z', transactions: 67 },
  { address: '0xFeedFaceFeedFaceFeedFaceFeedFaceFeedFace', name: 'Suspended Account', status: 'suspended', platform: 'epsx', chain: 'BSC', balance: '8,900.25 USDT', credits: 0, last_active: '2025-01-20T14:00:00Z', created_at: '2024-05-05T09:30:00Z', transactions: 312 },
];

const activity = [
  { id: 'a-001', wallet: '0x742d...bD68', action: 'Login', details: 'Web3 SIWE authentication', ip: '192.168.1.42', timestamp: '2025-02-14T08:30:00Z' },
  { id: 'a-002', wallet: '0xAb58...eC9B', action: 'Permission Grant', details: 'Granted admin:wallets:manage', ip: '10.0.0.15', timestamp: '2025-02-14T08:15:00Z' },
  { id: 'a-003', wallet: '0x1234...5678', action: 'API Call', details: 'GET /api/analytics/rankings', ip: '203.0.113.50', timestamp: '2025-02-13T22:15:00Z' },
  { id: 'a-004', wallet: '0xDead...Beef', action: 'Wallet Disabled', details: 'Disabled by admin for policy violation', ip: '10.0.0.15', timestamp: '2024-09-01T10:00:00Z' },
  { id: 'a-005', wallet: '0x742d...bD68', action: 'Credit Purchase', details: 'Purchased 100 credits ($29.00)', ip: '192.168.1.42', timestamp: '2025-02-13T14:00:00Z' },
  { id: 'a-006', wallet: '0xCafe...Babe', action: 'API Key Created', details: 'New API key for testing', ip: '172.16.0.1', timestamp: '2025-02-12T16:00:00Z' },
  { id: 'a-007', wallet: '0xFeed...Face', action: 'Wallet Suspended', details: 'Suspicious activity detected', ip: '10.0.0.15', timestamp: '2025-01-20T14:00:00Z' },
  { id: 'a-008', wallet: '0x742d...bD68', action: 'Plan Upgrade', details: 'Upgraded from Free to Pro plan', ip: '192.168.1.42', timestamp: '2025-02-10T10:00:00Z' },
];

const credits = [
  { id: 'c-001', wallet: '0x742d...bD68', type: 'purchase', amount: 100, balance_after: 250, description: 'Credit purchase', timestamp: '2025-02-13T14:00:00Z' },
  { id: 'c-002', wallet: '0x742d...bD68', type: 'usage', amount: -5, balance_after: 150, description: 'API usage (500 calls)', timestamp: '2025-02-12T23:59:00Z' },
  { id: 'c-003', wallet: '0xAb58...eC9B', type: 'grant', amount: 500, balance_after: 1000, description: 'Admin credit grant', timestamp: '2025-02-10T09:00:00Z' },
  { id: 'c-004', wallet: '0xCafe...Babe', type: 'purchase', amount: 50, balance_after: 150, description: 'Credit purchase', timestamp: '2025-02-08T12:00:00Z' },
  { id: 'c-005', wallet: '0x1234...5678', type: 'usage', amount: -25, balance_after: 75, description: 'API usage (2,500 calls)', timestamp: '2025-02-07T23:59:00Z' },
];

export const walletsMocks: MockHandler[] = [
  {
    pattern: '**/api/admin/wallets**',
    handler: (url: URL) => {
      const search = url.searchParams.get('search') ?? '';
      const status = url.searchParams.get('status');
      const platform = url.searchParams.get('platform');

      let filtered = [...wallets];
      if (search) filtered = filtered.filter(w => w.address.toLowerCase().includes(search.toLowerCase()) || w.name.toLowerCase().includes(search.toLowerCase()));
      if (status) filtered = filtered.filter(w => w.status === status);
      if (platform) filtered = filtered.filter(w => w.platform === platform);

      return { items: filtered, total: filtered.length };
    },
  },
  {
    pattern: '**/api/admin/wallets/0x*/activity**',
    handler: () => ({ items: activity, total: activity.length }),
  },
  {
    pattern: '**/api/admin/wallets/0x*/credits**',
    handler: () => ({ items: credits, total: credits.length }),
  },
  {
    pattern: '**/api/admin/wallets/0x*',
    handler: () => wallets[0],
  },
  {
    pattern: '**/api/admin/activity**',
    handler: () => ({ items: activity, total: activity.length }),
  },
  {
    pattern: '**/api/admin/credits**',
    handler: () => ({ items: credits, total: credits.length }),
  },
];
