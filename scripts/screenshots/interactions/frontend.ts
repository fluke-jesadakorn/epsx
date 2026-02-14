import type { Objective } from '../types';
import { waitFor, fillField, tryClick } from '../helpers';

// ===== PUBLIC PAGES =====

const publicObjectives: Objective[] = [
  {
    id: 'home', name: 'Home Page', route: '/', auth: false,
    steps: [{ name: 'home', desc: 'Landing page', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
  {
    id: 'about', name: 'About Page', route: '/about', auth: false,
    steps: [{ name: 'about', desc: 'About page', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
  {
    id: 'terms', name: 'Terms', route: '/terms', auth: false,
    steps: [{ name: 'terms', desc: 'Terms of service', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
  {
    id: 'privacy', name: 'Privacy', route: '/privacy', auth: false,
    steps: [{ name: 'privacy', desc: 'Privacy policy', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
  {
    id: 'offline', name: 'Offline', route: '/offline', auth: false,
    steps: [{ name: 'offline', desc: 'Offline fallback', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
  {
    id: 'access-denied', name: 'Access Denied', route: '/access-denied', auth: false,
    steps: [{ name: 'access-denied', desc: 'Access denied page', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
  {
    id: 'auth', name: 'Auth Page', route: '/auth', auth: false,
    steps: [{ name: 'auth', desc: 'Authentication page', action: async (page) => { await page.waitForTimeout(1000); } }],
  },
];

// ===== ANALYTICS =====

const analyticsObjectives: Objective[] = [
  {
    id: 'analytics-default', name: 'Stock Rankings', route: '/analytics', auth: true,
    steps: [
      { name: 'analytics-default', desc: 'Default rankings table', action: async (page) => {
        await waitFor(page, 'table, [class*="table"], [class*="ranking"]');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'analytics-search', name: 'Search Stocks', route: '/analytics', auth: true,
    steps: [
      { name: 'analytics-search', desc: 'Search for AAPL', action: async (page) => {
        await waitFor(page, 'table, [class*="table"], [class*="ranking"]');
        await fillField(page, 'input[type="search"], input[placeholder*="earch"], input[placeholder*="stock"], input[name="search"]', 'AAPL');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'analytics-filter-country', name: 'Filter by Country', route: '/analytics', auth: true,
    steps: [
      { name: 'analytics-filter-country', desc: 'Open country filter', action: async (page) => {
        await waitFor(page, 'table, [class*="table"]');
        await tryClick(page, 'button:has-text("Country"), select[name*="country"], [class*="country"] button, [data-filter="country"]');
        await page.waitForTimeout(800);
      }},
    ],
  },
  {
    id: 'analytics-filter-sector', name: 'Filter by Sector', route: '/analytics', auth: true,
    steps: [
      { name: 'analytics-filter-sector', desc: 'Open sector filter', action: async (page) => {
        await waitFor(page, 'table, [class*="table"]');
        await tryClick(page, 'button:has-text("Sector"), select[name*="sector"], [class*="sector"] button, [data-filter="sector"]');
        await page.waitForTimeout(800);
      }},
    ],
  },
  {
    id: 'analytics-sort', name: 'Sort Column', route: '/analytics', auth: true,
    steps: [
      { name: 'analytics-sort', desc: 'Click column header to sort', action: async (page) => {
        await waitFor(page, 'table, [class*="table"]');
        await tryClick(page, 'th:has-text("Price"), th:has-text("Change"), th:has-text("Score"), [role="columnheader"]');
        await page.waitForTimeout(800);
      }},
    ],
  },
  {
    id: 'analytics-pagination', name: 'Paginate', route: '/analytics', auth: true,
    steps: [
      { name: 'analytics-pagination', desc: 'Navigate to next page', action: async (page) => {
        await waitFor(page, 'table, [class*="table"]');
        await tryClick(page, 'button:has-text("Next"), button:has-text("2"), [aria-label="Next page"], [class*="pagination"] button:nth-child(2)');
        await page.waitForTimeout(800);
      }},
    ],
  },
];

// ===== DASHBOARD & ACCOUNT =====

const dashboardObjectives: Objective[] = [
  {
    id: 'dashboard', name: 'Dashboard', route: '/dashboard', auth: true,
    steps: [
      { name: 'dashboard', desc: 'User dashboard', action: async (page) => {
        await waitFor(page, '[class*="dashboard"], [class*="stats"], main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'account', name: 'Account Overview', route: '/account', auth: true,
    steps: [
      { name: 'account', desc: 'Account overview', action: async (page) => {
        await waitFor(page, '[class*="account"], main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'account-payments', name: 'Payment History', route: '/account', auth: true,
    steps: [
      { name: 'account-payments', desc: 'Payment history tab', action: async (page) => {
        await waitFor(page, 'main');
        await tryClick(page, 'button:has-text("Payment"), button:has-text("History"), [role="tab"]:has-text("Payment"), a:has-text("Payment")');
        await page.waitForTimeout(800);
      }},
    ],
  },
  {
    id: 'account-prefs', name: 'Notification Prefs', route: '/account', auth: true,
    steps: [
      { name: 'account-prefs', desc: 'Toggle notification preferences', action: async (page) => {
        await waitFor(page, 'main');
        await tryClick(page, 'button:has-text("Preferences"), button:has-text("Notification"), [role="tab"]:has-text("Pref"), a:has-text("Setting")');
        await page.waitForTimeout(500);
        await tryClick(page, 'input[type="checkbox"], [role="switch"], button[role="switch"]');
        await page.waitForTimeout(500);
      }},
    ],
  },
  {
    id: 'account-credits', name: 'Credit Balance', route: '/account/credits', auth: true,
    steps: [
      { name: 'account-credits', desc: 'Credits page', action: async (page) => {
        await waitFor(page, 'main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'profile', name: 'Profile', route: '/profile', auth: true,
    steps: [
      { name: 'profile', desc: 'Profile page', action: async (page) => {
        await waitFor(page, 'main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'profile-edit', name: 'Edit Profile', route: '/profile', auth: true,
    steps: [
      { name: 'profile-edit', desc: 'Fill profile name field', action: async (page) => {
        await waitFor(page, 'main');
        await tryClick(page, 'button:has-text("Edit"), button:has-text("Update"), a:has-text("Edit")');
        await fillField(page, 'input[name="name"], input[placeholder*="name"], input[placeholder*="Name"]', 'Updated User Name');
        await page.waitForTimeout(500);
      }},
    ],
  },
];

// ===== PLANS & PAYMENT =====

const plansObjectives: Objective[] = [
  {
    id: 'plans', name: 'Compare Plans', route: '/plans', auth: false,
    steps: [
      { name: 'plans', desc: 'Plan cards with pricing', action: async (page) => {
        await waitFor(page, '[class*="plan"], [class*="pricing"], main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'payment', name: 'Payment Checkout', route: '/payment', auth: true,
    steps: [
      { name: 'payment', desc: 'Checkout page', action: async (page) => {
        await waitFor(page, 'main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'payment-detail', name: 'Plan Payment', route: '/payment', auth: true,
    steps: [
      { name: 'payment-detail', desc: 'Specific plan payment', action: async (page) => {
        await waitFor(page, 'main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
];

// ===== PORTFOLIO & PERMISSIONS =====

const portfolioObjectives: Objective[] = [
  {
    id: 'portfolio', name: 'Portfolio', route: '/portfolio', auth: true,
    steps: [
      { name: 'portfolio', desc: 'Holdings table', action: async (page) => {
        await waitFor(page, 'table, [class*="portfolio"], main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'portfolio-search', name: 'Search Portfolio', route: '/portfolio', auth: true,
    steps: [
      { name: 'portfolio-search', desc: 'Search holdings', action: async (page) => {
        await waitFor(page, 'main');
        await fillField(page, 'input[type="search"], input[placeholder*="earch"], input[name="search"]', 'AAPL');
        await page.waitForTimeout(800);
      }},
    ],
  },
  {
    id: 'permissions', name: 'Permissions', route: '/permissions', auth: true,
    steps: [
      { name: 'permissions', desc: 'Entitlements list', action: async (page) => {
        await waitFor(page, 'main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
];

// ===== NOTIFICATIONS =====

const notificationsObjectives: Objective[] = [
  {
    id: 'notifications-default', name: 'Notifications', route: '/notifications', auth: true,
    steps: [
      { name: 'notifications-default', desc: 'Notification list', action: async (page) => {
        await waitFor(page, '[class*="notification"], main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'notifications-filter-type', name: 'Filter by Type', route: '/notifications', auth: true,
    steps: [
      { name: 'notifications-filter-type', desc: 'Select Security type', action: async (page) => {
        await waitFor(page, 'main');
        await tryClick(page, 'button:has-text("Type"), select[name*="type"], [class*="type"] button, button:has-text("Security"), [data-filter="type"]');
        await page.waitForTimeout(800);
      }},
    ],
  },
  {
    id: 'notifications-filter-priority', name: 'Filter by Priority', route: '/notifications', auth: true,
    steps: [
      { name: 'notifications-filter-priority', desc: 'Select High priority', action: async (page) => {
        await waitFor(page, 'main');
        await tryClick(page, 'button:has-text("Priority"), select[name*="priority"], button:has-text("High"), [data-filter="priority"]');
        await page.waitForTimeout(800);
      }},
    ],
  },
  {
    id: 'notifications-search', name: 'Search Notifications', route: '/notifications', auth: true,
    steps: [
      { name: 'notifications-search', desc: 'Search query', action: async (page) => {
        await waitFor(page, 'main');
        await fillField(page, 'input[type="search"], input[placeholder*="earch"], input[name="search"]', 'security');
        await page.waitForTimeout(800);
      }},
    ],
  },
  {
    id: 'notifications-empty', name: 'Empty State', route: '/notifications', auth: true,
    mockOverrides: { emptyNotifications: true },
    steps: [
      { name: 'notifications-empty', desc: 'No notifications', action: async (page) => {
        await waitFor(page, 'main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
];

// ===== DEVELOPER =====

const developerObjectives: Objective[] = [
  {
    id: 'developer', name: 'Developer Overview', route: '/developer', auth: true,
    steps: [
      { name: 'developer', desc: 'Developer portal', action: async (page) => {
        await waitFor(page, 'main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'developer-docs', name: 'API Docs', route: '/developer/docs', auth: true,
    steps: [
      { name: 'developer-docs', desc: 'API documentation', action: async (page) => {
        await waitFor(page, 'main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'developer-usage', name: 'API Usage', route: '/developer/usage', auth: true,
    steps: [
      { name: 'developer-usage', desc: 'Usage charts', action: async (page) => {
        await waitFor(page, 'main');
        await page.waitForTimeout(1000);
      }},
    ],
  },
  {
    id: 'developer-create-key', name: 'Create API Key', route: '/developer', auth: true,
    steps: [
      { name: 'developer-create-key', desc: 'Open key creation', action: async (page) => {
        await waitFor(page, 'main');
        await tryClick(page, 'button:has-text("Create"), button:has-text("New"), button:has-text("Add"), a:has-text("Create")');
        await page.waitForTimeout(800);
      }},
    ],
  },
];

export const frontendObjectives: Objective[] = [
  ...publicObjectives,
  ...analyticsObjectives,
  ...dashboardObjectives,
  ...plansObjectives,
  ...portfolioObjectives,
  ...notificationsObjectives,
  ...developerObjectives,
];
