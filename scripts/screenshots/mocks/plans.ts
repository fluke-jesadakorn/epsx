import type { MockHandler } from '../types';

const plans = [
  {
    id: 'plan-free',
    name: 'Free',
    slug: 'free',
    price_monthly: 0,
    price_yearly: 0,
    currency: 'USDT',
    features: ['Basic stock rankings (Top 10)', '1 watchlist (5 stocks)', 'Daily market summary', 'Community access'],
    limits: { api_calls: 100, watchlists: 1, stocks_per_list: 5, exports: 0 },
    is_popular: false,
    active: true,
  },
  {
    id: 'plan-pro',
    name: 'Pro',
    slug: 'pro',
    price_monthly: 29,
    price_yearly: 290,
    currency: 'USDT',
    features: ['Full stock rankings (Top 500)', '10 watchlists (50 stocks each)', 'Real-time alerts', 'API access (10K calls/day)', 'CSV/PDF exports', 'Priority support'],
    limits: { api_calls: 10000, watchlists: 10, stocks_per_list: 50, exports: 100 },
    is_popular: true,
    active: true,
  },
  {
    id: 'plan-enterprise',
    name: 'Enterprise',
    slug: 'enterprise',
    price_monthly: 99,
    price_yearly: 990,
    currency: 'USDT',
    features: ['Unlimited stock rankings', 'Unlimited watchlists', 'Custom alerts & webhooks', 'Unlimited API access', 'White-label reports', 'Dedicated account manager', 'SLA guarantee'],
    limits: { api_calls: -1, watchlists: -1, stocks_per_list: -1, exports: -1 },
    is_popular: false,
    active: true,
  },
];

export const plansMocks: MockHandler[] = [
  {
    pattern: '**/api/plans**',
    handler: () => ({ items: plans, total: 3 }),
  },
  {
    pattern: '**/api/public/plans**',
    handler: () => ({ items: plans, total: 3 }),
  },
  {
    pattern: '**/api/plans/plan-*',
    handler: (url: URL) => {
      const id = url.pathname.split('/').pop();
      return plans.find(p => p.id === id) ?? plans[0];
    },
  },
];
