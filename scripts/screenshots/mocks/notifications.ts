import type { MockHandler } from '../types';

const notifications = [
  { id: 'n-001', title: 'Welcome to EPSX', message: 'Your account has been set up successfully. Explore the platform to get started.', type: 'system', priority: 'low', read: true, created_at: '2025-02-14T08:00:00Z' },
  { id: 'n-002', title: 'Security Alert: New Login', message: 'A new login was detected from Chrome on macOS. If this was not you, please secure your account.', type: 'security', priority: 'high', read: false, created_at: '2025-02-14T07:45:00Z' },
  { id: 'n-003', title: 'AAPL Price Target Reached', message: 'Apple Inc. has reached your price target of $190.00. Current price: $189.84.', type: 'alert', priority: 'medium', read: false, created_at: '2025-02-14T07:30:00Z' },
  { id: 'n-004', title: 'Subscription Renewal', message: 'Your Pro plan subscription will renew in 3 days. Ensure sufficient credits.', type: 'billing', priority: 'medium', read: false, created_at: '2025-02-13T16:00:00Z' },
  { id: 'n-005', title: 'API Rate Limit Warning', message: 'You have used 85% of your daily API call limit (8,500/10,000).', type: 'system', priority: 'high', read: false, created_at: '2025-02-13T14:20:00Z' },
  { id: 'n-006', title: 'New Feature: Portfolio Analytics', message: 'We have launched advanced portfolio analytics. Check it out in the Portfolio section.', type: 'announcement', priority: 'low', read: true, created_at: '2025-02-12T10:00:00Z' },
  { id: 'n-007', title: 'Permission Updated', message: 'Your analytics access has been upgraded to include sector-level data.', type: 'system', priority: 'medium', read: true, created_at: '2025-02-11T09:15:00Z' },
  { id: 'n-008', title: 'TSLA Unusual Volume', message: 'Tesla Inc. is experiencing 3x average trading volume. Current volume: 98.2M.', type: 'alert', priority: 'high', read: false, created_at: '2025-02-10T15:30:00Z' },
  { id: 'n-009', title: 'Credit Balance Low', message: 'Your credit balance is below 50 credits. Consider purchasing more to avoid service interruption.', type: 'billing', priority: 'medium', read: true, created_at: '2025-02-09T11:00:00Z' },
  { id: 'n-010', title: 'Weekly Market Summary', message: 'The S&P 500 gained 1.2% this week. Tech sector led gains with 2.1% increase.', type: 'announcement', priority: 'low', read: true, created_at: '2025-02-08T08:00:00Z' },
];

export const notificationsMocks: MockHandler[] = [
  {
    pattern: '**/api/notifications**',
    handler: (url: URL) => {
      const type = url.searchParams.get('type');
      const priority = url.searchParams.get('priority');
      const search = url.searchParams.get('search') ?? '';

      let filtered = [...notifications];
      if (type) filtered = filtered.filter(n => n.type === type);
      if (priority) filtered = filtered.filter(n => n.priority === priority);
      if (search) filtered = filtered.filter(n => n.title.toLowerCase().includes(search.toLowerCase()));

      return {
        items: filtered,
        total: filtered.length,
        unread_count: filtered.filter(n => !n.read).length,
      };
    },
  },
  {
    pattern: '**/api/notifications/unread**',
    handler: () => ({ count: 5 }),
  },
];

export const emptyNotificationsMocks: MockHandler[] = [
  {
    pattern: '**/api/notifications**',
    handler: () => ({ items: [], total: 0, unread_count: 0 }),
  },
  {
    pattern: '**/api/notifications/unread**',
    handler: () => ({ count: 0 }),
  },
];
