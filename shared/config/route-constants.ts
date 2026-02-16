/**
 * Centralized Route Constants for EPSX Platform
 *
 * This file provides a single source of truth for all API endpoints and routes
 * across the entire EPSX platform (Backend, Frontend, Admin-Frontend).
 *
 * Naming Convention: RESTful /api/{resource}/{action}
 * - All backend endpoints use /api/ prefix
 * - Frontend routes map directly to functionality without conflicts
 * - Admin routes use /admin/ prefix for clear separation
 */

export const API_ROUTES = {
  // System endpoints
  HEALTH: '/health',
  DOCS: '/docs',

  // Public endpoints (no authentication required)
  PUBLIC: {
    PLANS: '/api/public/plans',
    PLANS_FEATURES: '/api/public/plans/features',
    STATUS: '/api/public/status',
    NETWORKS: '/api/public/networks',
  },

  // Authentication endpoints
  AUTH: {
    // Web3 Authentication
    WEB3_CHALLENGE: '/api/auth/web3/challenge',
    WEB3_VERIFY: '/api/auth/web3/verify',
    WEB3_SESSION: '/api/auth/web3/session',
    WEB3_LOGOUT: '/api/auth/web3/logout',

    // Session Management
    SESSION_VERIFY: '/api/auth/session/verify',
    SESSION_REFRESH: '/api/auth/session/refresh',

    // User Authentication
    PROFILE: '/api/auth/users/profile',
    PERMISSIONS: '/api/auth/users/permissions',
    SESSIONS: '/api/auth/users/sessions',
  },

  // User management endpoints
  USERS: {
    PROFILE: '/api/users/profile',
    WATCHLIST: '/api/users/watchlist',
    ALERTS: '/api/users/alerts',
    PUSH_SUBSCRIPTION: '/api/users/push-subscription',
    NOTIFICATIONS: '/api/notifications',
    SETTINGS: '/api/users/settings',
    ACCESS_OVERVIEW: '/api/users/access-overview', // NEW
    WALLET_CONNECT: '/api/users/wallet/connect',
    WALLET_DISCONNECT: '/api/users/wallet/disconnect',
  },

  // Analytics endpoints
  ANALYTICS: {
    // Core analytics data
    RANKINGS: '/api/analytics/rankings',
    COUNTRIES: '/api/analytics/countries',
    SECTORS: '/api/analytics/sectors',
    FILTERS: '/api/analytics/filters',

    // Market data
    MARKET_OVERVIEW: '/api/analytics/market/overview',
    MARKET_TRENDS: '/api/analytics/market/trends',
    TOP_GAINERS: '/api/analytics/market/top-gainers',
    TOP_LOSERS: '/api/analytics/market/top-losers',

    // Performance metrics
    PERFORMANCE: '/api/analytics/performance',
    PORTFOLIO_ANALYSIS: '/api/analytics/portfolio/analysis',

    // Public analytics (no auth)
    PUBLIC_RANKINGS: '/api/public/analytics/rankings',
    PUBLIC_FILTERS: '/api/public/analytics/filters',

    // Export functionality
    EXPORT: '/api/analytics/export',
  },

  // Notification endpoints
  NOTIFICATIONS: {
    // Real-time streams
    STREAM: '/api/notifications/stream',
    SSE_ENDPOINT: '/api/notifications/sse',

    // Notification management
    PREFERENCES: '/api/notifications/preferences',
    HISTORY: '/api/notifications/history',
  },

  // Admin-only endpoints
  ADMIN: {
    // User management
    USERS: '/api/admin/users',
    USER_DETAILS: '/api/admin/users/:id',
    USER_PERMISSIONS: '/api/admin/users/:id/permissions',

    // Permission management
    PERMISSIONS: '/api/admin/permissions',
    PERMISSION_PLANS: '/api/permissions/plans',
    PERMISSION_ASSIGNMENTS: '/api/permissions/assignments',

    // Wallet management
    WALLET_MANAGEMENT: '/api/admin/wallets',
    WALLET_DETAILS: '/api/admin/wallets/:address',
    WALLET_TRANSACTIONS: '/api/admin/wallets/:address/transactions',

    // System management
    SYSTEM_STATUS: '/api/admin/system/status',
    SYSTEM_METRICS: '/api/admin/system/metrics',
    SYSTEM_LOGS: '/api/admin/system/logs',

    // Content management
    PLANS: '/api/admin/plans',
    FEATURES: '/api/admin/features',
    PROMOTIONS: '/api/admin/promotions',
    ANNOUNCEMENTS: '/api/admin/announcements',

    // Analytics and reporting
    ANALYTICS: '/api/admin/analytics',
    ANALYTICS_OVERVIEW: '/api/admin/analytics/overview',
    ANALYTICS_USERS: '/api/admin/analytics/users',
    ANALYTICS_PERMISSIONS: '/api/admin/analytics/permissions',
    ANALYTICS_REVENUE: '/api/admin/analytics/revenue',
    ANALYTICS_PERFORMANCE: '/api/admin/analytics/performance',
    REPORTS: '/api/admin/reports',
    AUDIT_LOGS: '/api/admin/audit-logs',

    // Notification management
    NOTIFICATIONS: '/api/admin/notifications',
    NOTIFICATION_TEMPLATES: '/api/admin/notifications/templates',
    BROADCAST_NOTIFICATIONS: '/api/admin/notifications/broadcast',

    // System settings (global admin console settings)
    SETTINGS: '/api/admin/settings',
    SETTINGS_BY_CATEGORY: '/api/admin/settings/:category',
    SETTINGS_RESET: '/api/admin/settings/reset',

    // Developer Portal (API key management)
    DEVELOPER_PORTAL: {
      API_KEYS: '/api/admin/developer-portal/api-keys',
      API_KEY_DETAILS: '/api/admin/developer-portal/api-keys/:id',
      API_KEY_REVOKE: '/api/admin/developer-portal/api-keys/:id/revoke',
      MODULES: '/api/admin/developer-portal/modules',
      MODULE_DETAILS: '/api/admin/developer-portal/modules/:id',
      STATS: '/api/admin/developer-portal/stats',
    },
  },

  // Permission Authority endpoints (ALL applications use this)
  PERMISSIONS: {
    // Core permission validation
    VALIDATE: '/api/permissions/validate',
    VALIDATE_BULK: '/api/permissions/validate-bulk',
    CHECK: '/api/permissions/check',

    // Wallet-specific permissions
    WALLET_PERMISSIONS: '/api/permissions/wallet/:wallet_address',
    WALLET_PLANS: '/api/permissions/wallet/:wallet_address/plans',

    // Permission plan management
    PLANS: '/api/permissions/plans',
    PLAN_DETAILS: '/api/permissions/plans/:id',
    PLAN_PERMISSIONS: '/api/permissions/plans/:id/permissions',

    // Assignment management
    ASSIGNMENTS: '/api/permissions/assignments',
    WALLET_ASSIGNMENTS: '/api/permissions/assignments/wallet/:wallet_address',
    PLAN_ASSIGNMENTS: '/api/permissions/assignments/plan/:plan_id',

    // Permission hierarchy and inheritance
    HIERARCHY: '/api/permissions/hierarchy',
    INHERITANCE: '/api/permissions/inheritance',
    DEPENDENCIES: '/api/permissions/dependencies',

    // Permission analytics and monitoring
    EXPIRING: '/api/admin/permissions/expiring',
    EXPIRED: '/api/admin/permissions/expired',
    STATS: '/api/admin/permissions/system/stats',
    GRANT_HISTORY: '/api/admin/permissions/history/grants',
    REVOKE_HISTORY: '/api/admin/permissions/history/revocations',

    // Bulk operations
    BULK_GRANT: '/api/admin/permissions/bulk/grant',
    BULK_REVOKE: '/api/admin/permissions/bulk/revoke',
    BULK_ASSIGN_PLANS: '/api/admin/permissions/bulk/assign-plans',
    BULK_APPLY_TEMPLATE: '/api/admin/permissions/bulk/apply-template',
    BULK_VALIDATE: '/api/admin/permissions/bulk/validate',

    // Permission management
    ADMIN_PERMISSIONS: '/api/admin/permissions',
    GRANT: '/api/admin/permissions/grant',
    REVOKE: '/api/admin/permissions/revoke',
    UPDATE_EXPIRY: '/api/admin/permissions/expiry',

    // Web3 permission management
    WEB3_PERMISSIONS: '/api/admin/web3/permissions',
    WEB3_GRANT: '/api/admin/web3/permissions/grant',
    WEB3_RECENT_WALLETS: '/api/admin/web3/recent-wallets',
  },

  // Plan and billing endpoints
  PLANS: {
    SUBSCRIPTION: '/api/plans/subscription',
    SUBSCRIPTION_STATUS: '/api/plans/subscription/status',
    SUBSCRIPTION_HISTORY: '/api/plans/subscription/history',
    USAGE: '/api/plans/usage',
    BILLING: '/api/plans/billing',
    INVOICES: '/api/plans/invoices',
  },

  // Payment endpoints
  PAYMENTS: {
    HISTORY: '/api/payments/history',
  },
} as const;

// Frontend route constants (for Next.js App Router)
export const FRONTEND_ROUTES = {
  // Public routes
  HOME: '/',
  ABOUT: '/about',
  CONTACT: '/contact',

  // Authentication routes
  AUTH: {
    LOGIN: '/auth/login',
    SIGNIN: '/auth/signin',
    SIGNOUT: '/auth/signout',
  },

  // Main application routes
  DASHBOARD: '/dashboard',
  ANALYTICS: '/analytics',
  NOTIFICATIONS: '/notifications',
  PROFILE: '/profile',
  ACCOUNT: '/account',
  PLANS: '/plans',

  // Feature-specific routes
  WATCHLIST: '/watchlist',
  ALERTS: '/alerts',
  MARKET: '/market',
  PORTFOLIO: '/portfolio',
} as const;

// Admin Frontend route constants
export const ADMIN_ROUTES = {
  // Admin authentication
  AUTH: '/auth',

  // Admin dashboard
  DASHBOARD: '/',

  // Admin management routes (all under /admin/)
  ADMIN: {
    USERS: '/admin/users',
    USER_DETAILS: '/admin/users/:id',
    PERMISSIONS: '/admin/permissions',
    PERMISSION_PLANS: '/admin/permissions/plans',
    WALLET_MANAGEMENT: '/admin/wallets',
    WALLET_DETAILS: '/admin/wallets/:address',

    // System management
    SYSTEM: '/admin/system',
    SYSTEM_METRICS: '/admin/system/metrics',
    SYSTEM_LOGS: '/admin/system/logs',

    // Content management
    PLANS: '/admin/plans',
    PROMOTIONS: '/admin/promotions',
    ANNOUNCEMENTS: '/admin/announcements',

    // Analytics and reporting
    ANALYTICS: '/admin/analytics',
    REPORTS: '/admin/reports',
    AUDIT_LOGS: '/admin/audit-logs',

    // Notification management
    NOTIFICATIONS: '/admin/notifications',
    NOTIFICATION_TEMPLATES: '/admin/notifications/templates',

    // Developer portal
    DEVELOPER_PORTAL: '/admin/developer-portal',
    API_KEYS: '/admin/developer-portal/api-keys',
    API_DOCUMENTATION: '/admin/developer-portal/docs',
    WEBHOOKS: '/admin/developer-portal/webhooks',
  },
} as const;

// HTTP methods for type safety
export const HTTP_METHODS = {
  GET: 'GET',
  POST: 'POST',
  PUT: 'PUT',
  PATCH: 'PATCH',
  DELETE: 'DELETE',
} as const;

// Route types for TypeScript integration
export type ApiRoute = keyof typeof API_ROUTES | string;
export type FrontendRoute = keyof typeof FRONTEND_ROUTES | string;
export type AdminRoute = keyof typeof ADMIN_ROUTES | string;
export type HttpMethod = typeof HTTP_METHODS[keyof typeof HTTP_METHODS];

// Utility functions for route construction
export const constructRoute = (baseRoute: string, params: Record<string, string>): string => {
  let route = baseRoute;
  Object.entries(params).forEach(([key, value]) => {
    route = route.replace(`:${key}`, encodeURIComponent(value));
  });
  return route;
};

// Example usage helpers
export const routeExamples = {
  // API route examples
  getUserProfile: (userId: string) => constructRoute(API_ROUTES.ADMIN.USER_DETAILS, { id: userId }),
  getWalletPermissions: (walletAddress: string) => constructRoute(API_ROUTES.PERMISSIONS.WALLET_PERMISSIONS, { wallet_address: walletAddress }),
  acknowledgeNotification: (notificationId: string) => constructRoute('/api/notifications/{id}/acknowledge', { id: notificationId }),

  // Frontend route examples
  adminUserDetails: (userId: string) => constructRoute(ADMIN_ROUTES.ADMIN.USER_DETAILS, { id: userId }),
  adminWalletDetails: (walletAddress: string) => constructRoute(ADMIN_ROUTES.ADMIN.WALLET_DETAILS, { address: walletAddress }),
} as const;