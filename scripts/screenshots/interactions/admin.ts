import type { Objective } from '../types';
import { waitFor, fillField, tryClick } from '../helpers';

// ===== DASHBOARD =====

const dashboardObjectives: Objective[] = [
  {
    id: 'admin-dashboard', name: 'Admin Dashboard', route: '/', auth: true,
    steps: [{ name: 'admin-dashboard', desc: 'Admin home', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-dashboard-page', name: 'Dashboard Page', route: '/dashboard', auth: true,
    steps: [{ name: 'admin-dashboard-page', desc: 'Detailed dashboard', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-auth', name: 'Admin Auth', route: '/auth', auth: false,
    steps: [{ name: 'admin-auth', desc: 'Admin auth page', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
  {
    id: 'admin-access-denied', name: 'Access Denied', route: '/access-denied', auth: false,
    steps: [{ name: 'admin-access-denied', desc: 'Access denied', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
  {
    id: 'admin-unauthorized', name: 'Unauthorized', route: '/unauthorized', auth: false,
    steps: [{ name: 'admin-unauthorized', desc: 'Unauthorized page', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
  {
    id: 'admin-request-access', name: 'Request Access', route: '/request-access', auth: false,
    steps: [{ name: 'admin-request-access', desc: 'Request access form', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
];

// ===== WALLET MANAGEMENT =====

const walletObjectives: Objective[] = [
  {
    id: 'admin-wallets-default', name: 'Wallets List', route: '/wallet-management/wallets', auth: true,
    steps: [{ name: 'admin-wallets-default', desc: 'Wallet cards', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-wallets-search', name: 'Search Wallets', route: '/wallet-management/wallets', auth: true,
    steps: [{ name: 'admin-wallets-search', desc: 'Search wallets', action: async (page) => {
      await waitFor(page, 'main');
      await fillField(page, 'input[type="search"], input[placeholder*="earch"], input[name="search"]', '0x742');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-wallets-filter-status', name: 'Filter by Status', route: '/wallet-management/wallets', auth: true,
    steps: [{ name: 'admin-wallets-filter-status', desc: 'Filter active wallets', action: async (page) => {
      await waitFor(page, 'main');
      await tryClick(page, 'button:has-text("Status"), select[name*="status"], button:has-text("Active"), [data-filter="status"]');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-wallets-filter-platform', name: 'Filter by Platform', route: '/wallet-management/wallets', auth: true,
    steps: [{ name: 'admin-wallets-filter-platform', desc: 'Filter by platform', action: async (page) => {
      await waitFor(page, 'main');
      await tryClick(page, 'button:has-text("Platform"), select[name*="platform"], [data-filter="platform"]');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-wallet-detail', name: 'Wallet Detail', route: '/wallet-management/0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68', auth: true,
    steps: [{ name: 'admin-wallet-detail', desc: 'Wallet info', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-wallet-disable', name: 'Disable Wallet', route: '/wallet-management/wallets/0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68/disable', auth: true,
    steps: [{ name: 'admin-wallet-disable', desc: 'Disable confirmation', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-wallet-activity', name: 'Wallet Activity', route: '/wallet-management/activity', auth: true,
    steps: [{ name: 'admin-wallet-activity', desc: 'Activity log', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-wallet-credits', name: 'Wallet Credits', route: '/wallet-management/credits', auth: true,
    steps: [{ name: 'admin-wallet-credits', desc: 'Credit balances', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-wallet-management', name: 'Wallet Hub', route: '/wallet-management', auth: true,
    steps: [{ name: 'admin-wallet-management', desc: 'Wallet hub', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-wallets-edit', name: 'Edit Wallet', route: '/wallet-management/wallets', auth: true,
    steps: [{ name: 'admin-wallets-edit', desc: 'Open edit modal', action: async (page) => {
      await waitFor(page, 'main');
      await tryClick(page, 'button:has-text("Edit"), button[aria-label="Edit"], [class*="edit"]');
      await page.waitForTimeout(800);
    }}],
  },
];

// ===== ACCESS CONTROL =====

const accessObjectives: Objective[] = [
  {
    id: 'admin-access-overview', name: 'Access Overview', route: '/wallet-management/access', auth: true,
    steps: [{ name: 'admin-access-overview', desc: 'Access overview', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-access-permissions', name: 'Access Permissions', route: '/wallet-management/access/permissions', auth: true,
    steps: [{ name: 'admin-access-permissions', desc: 'Permissions list', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-access-plans', name: 'Access Plans', route: '/wallet-management/access/plans', auth: true,
    steps: [{ name: 'admin-access-plans', desc: 'Plans list', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-access-plan-detail', name: 'Plan Detail', route: '/wallet-management/access/plans/plan-1', auth: true,
    steps: [{ name: 'admin-access-plan-detail', desc: 'Plan configuration', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
];

// ===== SUBSCRIPTIONS =====

const subscriptionObjectives: Objective[] = [
  {
    id: 'admin-subscription-detail', name: 'Subscription Detail', route: '/subscriptions/sub-001', auth: true,
    steps: [{ name: 'admin-subscription-detail', desc: 'Subscription info', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-subscription-new', name: 'New Subscription', route: '/subscriptions/new', auth: true,
    steps: [{ name: 'admin-subscription-new', desc: 'Empty form', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-subscription-new-filled', name: 'Filled Subscription', route: '/subscriptions/new', auth: true,
    steps: [{ name: 'admin-subscription-new-filled', desc: 'Form with data', action: async (page) => {
      await waitFor(page, 'main');
      await fillField(page, 'input[name*="wallet"], input[name*="user"], input[placeholder*="wallet"], input[placeholder*="address"]', '0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68');
      await tryClick(page, 'select[name*="plan"], [class*="plan"] button, button:has-text("Select Plan")');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-subscription-new-external', name: 'External Access', route: '/subscriptions/new', auth: true,
    steps: [{ name: 'admin-subscription-new-external', desc: 'External access context', action: async (page) => {
      await waitFor(page, 'main');
      await tryClick(page, 'button:has-text("External"), select option:has-text("External"), label:has-text("External"), input[value="external"]');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-plan-new', name: 'New Plan', route: '/subscriptions/plans/new', auth: true,
    steps: [{ name: 'admin-plan-new', desc: 'Plan creation form', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-plan-edit', name: 'Edit Plan', route: '/subscriptions/plans/plan-1/edit', auth: true,
    steps: [{ name: 'admin-plan-edit', desc: 'Plan edit form', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
];

// ===== NOTIFICATIONS =====

const notificationObjectives: Objective[] = [
  {
    id: 'admin-notifications', name: 'Notifications', route: '/notifications', auth: true,
    steps: [{ name: 'admin-notifications', desc: 'Notification list', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-notification-create', name: 'Create Notification', route: '/notifications/create', auth: true,
    steps: [{ name: 'admin-notification-create', desc: 'Empty create form', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-notification-create-filled', name: 'Filled Notification', route: '/notifications/create', auth: true,
    steps: [{ name: 'admin-notification-create-filled', desc: 'Form with data', action: async (page) => {
      await waitFor(page, 'main');
      await fillField(page, 'input[name*="title"], input[placeholder*="itle"]', 'System Maintenance Notice');
      await fillField(page, 'textarea[name*="message"], textarea[placeholder*="essage"]', 'Scheduled maintenance will occur on Feb 15, 2025 from 2:00 AM to 4:00 AM UTC. Some services may be temporarily unavailable.');
      await tryClick(page, 'select[name*="type"], button:has-text("Type")');
      await tryClick(page, 'select[name*="priority"], button:has-text("Priority")');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-notification-manage', name: 'Manage Notifications', route: '/notifications/manage', auth: true,
    steps: [{ name: 'admin-notification-manage', desc: 'Manage page', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
];

// ===== DEVELOPER PORTAL =====

const developerObjectives: Objective[] = [
  {
    id: 'admin-developer-portal', name: 'Developer Portal', route: '/developer-portal', auth: true,
    steps: [{ name: 'admin-developer-portal', desc: 'Portal overview', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-api-key-create', name: 'Create API Key', route: '/developer-portal/api-keys/create', auth: true,
    steps: [{ name: 'admin-api-key-create', desc: 'API key form', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-api-docs', name: 'API Docs', route: '/docs/api', auth: true,
    steps: [{ name: 'admin-api-docs', desc: 'API documentation', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
];

// ===== ANALYTICS =====

const analyticsObjectives: Objective[] = [
  {
    id: 'admin-analytics', name: 'Analytics', route: '/analytics', auth: true,
    steps: [{ name: 'admin-analytics', desc: 'Admin analytics', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
];

// ===== SYSTEM =====

const systemObjectives: Objective[] = [
  {
    id: 'admin-profile', name: 'Profile', route: '/profile', auth: true,
    steps: [{ name: 'admin-profile', desc: 'Admin profile', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-settings', name: 'Settings', route: '/settings', auth: true,
    steps: [{ name: 'admin-settings', desc: 'General settings', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-settings-appearance', name: 'Appearance Tab', route: '/settings', auth: true,
    steps: [{ name: 'admin-settings-appearance', desc: 'Appearance settings', action: async (page) => {
      await waitFor(page, 'main');
      await tryClick(page, 'button:has-text("Appearance"), [role="tab"]:has-text("Appearance"), a:has-text("Appearance")');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-settings-notifications', name: 'Notifications Tab', route: '/settings', auth: true,
    steps: [{ name: 'admin-settings-notifications', desc: 'Notification settings', action: async (page) => {
      await waitFor(page, 'main');
      await tryClick(page, 'button:has-text("Notification"), [role="tab"]:has-text("Notification"), a:has-text("Notification")');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-settings-security', name: 'Security Tab', route: '/settings', auth: true,
    steps: [{ name: 'admin-settings-security', desc: 'Security settings', action: async (page) => {
      await waitFor(page, 'main');
      await tryClick(page, 'button:has-text("Security"), [role="tab"]:has-text("Security"), a:has-text("Security")');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-audit-log', name: 'Audit Log', route: '/audit-log', auth: true,
    steps: [{ name: 'admin-audit-log', desc: 'Audit trail', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-audit-log-search', name: 'Search Audit', route: '/audit-log', auth: true,
    steps: [{ name: 'admin-audit-log-search', desc: 'Search audit log', action: async (page) => {
      await waitFor(page, 'main');
      await fillField(page, 'input[type="search"], input[placeholder*="earch"], input[name="search"]', 'wallet');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-audit-log-category', name: 'Audit Category', route: '/audit-log', auth: true,
    steps: [{ name: 'admin-audit-log-category', desc: 'Filter by category', action: async (page) => {
      await waitFor(page, 'main');
      await tryClick(page, 'button:has-text("Auth"), button:has-text("Admin"), [role="tab"]:nth-child(2), [class*="category"] button');
      await page.waitForTimeout(800);
    }}],
  },
  {
    id: 'admin-payments', name: 'Payments', route: '/payments', auth: true,
    steps: [{ name: 'admin-payments', desc: 'Payment history', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-affiliates', name: 'Affiliates', route: '/affiliates', auth: true,
    steps: [{ name: 'admin-affiliates', desc: 'Affiliate management', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
  {
    id: 'admin-debug-auth', name: 'Debug Auth', route: '/debug/auth', auth: true,
    steps: [{ name: 'admin-debug-auth', desc: 'Auth debug info', action: async (page) => {
      await waitFor(page, 'main');
      await page.waitForTimeout(1500);
    }}],
  },
];

export const adminObjectives: Objective[] = [
  ...dashboardObjectives,
  ...walletObjectives,
  ...accessObjectives,
  ...subscriptionObjectives,
  ...notificationObjectives,
  ...developerObjectives,
  ...analyticsObjectives,
  ...systemObjectives,
];
