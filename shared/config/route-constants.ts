/**
 * Centralized Route Constants for EPSX Platform
 *
 * This file provides a single source of truth for all API endpoints and routes
 * across the entire EPSX platform (Backend, Frontend, Admin-Frontend).
 *
 * Naming Convention: RESTful /api/v1/{resource}/{action}
 * - All backend endpoints use /api/v1/ prefix
 * - Frontend routes map directly to functionality without conflicts
 * - Admin routes use /admin/ prefix for clear separation
 */

export const API_ROUTES = {
  // System endpoints (no version required)
  HEALTH: '/health',
  DOCS: '/docs',

  // API v1 base prefix
  API_V1: '/api/v1',

  // Public endpoints (no authentication required)
  PUBLIC: {
    PLANS: '/api/v1/public/plans',
    PLANS_FEATURES: '/api/v1/public/plans/features',
    STATUS: '/api/v1/public/status',
    NETWORKS: '/api/v1/public/networks',
  },

  // Authentication endpoints
  AUTH: {
    // Web3 Authentication
    WEB3_CHALLENGE: '/api/auth/web3/challenge',
    WEB3_VERIFY: '/api/auth/web3/verify',
    WEB3_SESSION: '/api/auth/web3/session',
    WEB3_LOGOUT: '/api/auth/web3/logout',

    // Session Management
    SESSION_VERIFY: '/api/v1/auth/session/verify',
    SESSION_REFRESH: '/api/v1/auth/session/refresh',

    // User Authentication
    PROFILE: '/api/v1/auth/users/profile',
    PERMISSIONS: '/api/v1/auth/users/permissions',
    SESSIONS: '/api/v1/auth/users/sessions',
  },

  // User management endpoints
  USERS: {
    PROFILE: '/api/v1/users/profile',
    WATCHLIST: '/api/v1/users/watchlist',
    ALERTS: '/api/v1/users/alerts',
    PUSH_SUBSCRIPTION: '/api/v1/users/push-subscription',
    NOTIFICATIONS: '/api/v1/users/notifications',
    SETTINGS: '/api/v1/users/settings',
    WALLET_CONNECT: '/api/v1/users/wallet/connect',
    WALLET_DISCONNECT: '/api/v1/users/wallet/disconnect',
  },

  // Analytics endpoints
  ANALYTICS: {
    // Core analytics data
    RANKINGS: '/api/v1/analytics/rankings',
    COUNTRIES: '/api/v1/analytics/countries',
    SECTORS: '/api/v1/analytics/sectors',
    FILTERS: '/api/v1/analytics/filters',

    // Market data
    MARKET_OVERVIEW: '/api/v1/analytics/market/overview',
    MARKET_TRENDS: '/api/v1/analytics/market/trends',
    TOP_GAINERS: '/api/v1/analytics/market/top-gainers',
    TOP_LOSERS: '/api/v1/analytics/market/top-losers',

    // Performance metrics
    PERFORMANCE: '/api/v1/analytics/performance',
    PORTFOLIO_ANALYSIS: '/api/v1/analytics/portfolio/analysis',

    // Public analytics (no auth)
    PUBLIC_RANKINGS: '/api/v1/public/analytics/rankings',
    PUBLIC_FILTERS: '/api/v1/public/analytics/filters',

    // Export functionality
    EXPORT: '/api/v1/analytics/export',
  },

  // Notification endpoints
  NOTIFICATIONS: {
    // Real-time streams
    STREAM: '/api/v1/notifications/stream',
    SSE_ENDPOINT: '/api/v1/notifications/sse',

    // Notification management
    PREFERENCES: '/api/v1/notifications/preferences',
    HISTORY: '/api/v1/notifications/history',
  },

  // Admin-only endpoints
  ADMIN: {
    // User management
    USERS: '/api/v1/admin/users',
    USER_DETAILS: '/api/v1/admin/users/:id',
    USER_PERMISSIONS: '/api/v1/admin/users/:id/permissions',

    // Permission management
    PERMISSIONS: '/api/v1/admin/permissions',
    PERMISSION_GROUPS: '/api/v1/admin/permissions/groups',
    PERMISSION_ASSIGNMENTS: '/api/v1/admin/permissions/assignments',

    // Wallet management
    WALLET_MANAGEMENT: '/api/v1/admin/wallets',
    WALLET_DETAILS: '/api/v1/admin/wallets/:address',
    WALLET_TRANSACTIONS: '/api/v1/admin/wallets/:address/transactions',

    // System management
    SYSTEM_STATUS: '/api/v1/admin/system/status',
    SYSTEM_METRICS: '/api/v1/admin/system/metrics',
    SYSTEM_LOGS: '/api/v1/admin/system/logs',

    // Content management
    PLANS: '/api/v1/admin/plans',
    PROMOTIONS: '/api/v1/admin/promotions',
    ANNOUNCEMENTS: '/api/v1/admin/announcements',

    // Analytics and reporting
    ANALYTICS: '/api/v1/admin/analytics',
    ANALYTICS_OVERVIEW: '/api/v1/admin/analytics/overview',
    ANALYTICS_USERS: '/api/v1/admin/analytics/users',
    ANALYTICS_PERMISSIONS: '/api/v1/admin/analytics/permissions',
    ANALYTICS_REVENUE: '/api/v1/admin/analytics/revenue',
    ANALYTICS_PERFORMANCE: '/api/v1/admin/analytics/performance',
    REPORTS: '/api/v1/admin/reports',
    AUDIT_LOGS: '/api/v1/admin/audit-logs',

    // Notification management
    NOTIFICATIONS: '/api/v1/admin/notifications',
    NOTIFICATION_TEMPLATES: '/api/v1/admin/notifications/templates',
    BROADCAST_NOTIFICATIONS: '/api/v1/admin/notifications/broadcast',
  },

  // Permission Authority endpoints (ALL applications use this)
  PERMISSIONS: {
    // Core permission validation
    VALIDATE: '/api/v1/permissions/validate',
    VALIDATE_BULK: '/api/v1/permissions/validate-bulk',
    CHECK: '/api/v1/permissions/check',

    // Wallet-specific permissions
    WALLET_PERMISSIONS: '/api/v1/permissions/wallet/:wallet_address',
    WALLET_GROUPS: '/api/v1/permissions/wallet/:wallet_address/groups',

    // Permission group management
    GROUPS: '/api/v1/permissions/groups',
    GROUP_DETAILS: '/api/v1/permissions/groups/:id',
    GROUP_PERMISSIONS: '/api/v1/permissions/groups/:id/permissions',

    // Assignment management
    ASSIGNMENTS: '/api/v1/permissions/assignments',
    WALLET_ASSIGNMENTS: '/api/v1/permissions/assignments/wallet/:wallet_address',
    GROUP_ASSIGNMENTS: '/api/v1/permissions/assignments/group/:group_id',

    // Permission hierarchy and inheritance
    HIERARCHY: '/api/v1/permissions/hierarchy',
    INHERITANCE: '/api/v1/permissions/inheritance',
    DEPENDENCIES: '/api/v1/permissions/dependencies',

    // Permission analytics and monitoring
    EXPIRING: '/api/v1/admin/permissions/expiring',
    EXPIRED: '/api/v1/admin/permissions/expired',
    STATS: '/api/v1/admin/permissions/stats',
    GRANT_HISTORY: '/api/v1/admin/permissions/history/grants',
    REVOKE_HISTORY: '/api/v1/admin/permissions/history/revocations',

    // Bulk operations
    BULK_GRANT: '/api/v1/admin/users/bulk/permissions/grant',
    BULK_REVOKE: '/api/v1/admin/users/bulk/permissions/revoke',

    // Permission management
    ADMIN_PERMISSIONS: '/api/v1/admin/permissions',
    GRANT: '/api/v1/admin/permissions/grant',
    REVOKE: '/api/v1/admin/permissions/revoke',
    UPDATE_EXPIRY: '/api/v1/admin/permissions/expiry',

    // Web3 permission management
    WEB3_PERMISSIONS: '/api/v1/admin/web3/permissions',
    WEB3_GRANT: '/api/v1/admin/web3/permissions/grant',
    WEB3_RECENT_WALLETS: '/api/v1/admin/web3/recent-wallets',
  },

  // Plan and billing endpoints
  PLANS: {
    SUBSCRIPTION: '/api/v1/plans/subscription',
    SUBSCRIPTION_STATUS: '/api/v1/plans/subscription/status',
    SUBSCRIPTION_HISTORY: '/api/v1/plans/subscription/history',
    USAGE: '/api/v1/plans/usage',
    BILLING: '/api/v1/plans/billing',
    INVOICES: '/api/v1/plans/invoices',
  },
} as const;

// Frontend route constants (for Next.js App Router)
export const FRONTEND_ROUTES = {
  // Public routes
  HOME: '/',
  ABOUT: '/about',

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
  SETTINGS: '/settings',
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
    PERMISSION_GROUPS: '/admin/permissions/groups',
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
  acknowledgeNotification: (notificationId: string) => constructRoute('/api/v1/notifications/{id}/acknowledge', { id: notificationId }),

  // Frontend route examples
  adminUserDetails: (userId: string) => constructRoute(ADMIN_ROUTES.ADMIN.USER_DETAILS, { id: userId }),
  adminWalletDetails: (walletAddress: string) => constructRoute(ADMIN_ROUTES.ADMIN.WALLET_DETAILS, { address: walletAddress }),
} as const;