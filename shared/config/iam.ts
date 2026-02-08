/**
 * CONSOLIDATED IAM CONFIGURATION
 * Unified Identity and Access Management configuration shared across admin-frontend and frontend
 * Supports structured permissions in "platform:resource:action" format
 */

// ============================================================================
// CORE IAM CONFIGURATION
// ============================================================================

// ============================================================================
// STRUCTURED PERMISSION DEFINITIONS
// ============================================================================

export const PERMISSIONS = {
  // Admin permissions (highest privilege level)
  ADMIN_ALL: 'admin:*:*',
  ADMIN_USERS_MANAGE: 'admin:users:manage',
  ADMIN_USERS_CREATE: 'admin:users:create',
  ADMIN_USERS_READ: 'admin:users:read',
  ADMIN_USERS_DELETE: 'admin:users:delete',
  ADMIN_PERMISSIONS_MANAGE: 'admin:permissions:manage',
  ADMIN_PERMISSIONS_VIEW: 'admin:permissions:view',
  ADMIN_SYSTEM_MANAGE: 'admin:system:manage',
  ADMIN_SYSTEM_VIEW: 'admin:system:view',
  ADMIN_AUDIT_READ: 'admin:audit:read',
  ADMIN_AUDIT_MANAGE: 'admin:audit:manage',
  ADMIN_SECURITY_READ: 'admin:security:read',
  ADMIN_SECURITY_MANAGE: 'admin:security:manage',
  ADMIN_NOTIFICATIONS_MANAGE: 'admin:notifications:manage',
  ADMIN_ANALYTICS_VIEW: 'admin:analytics:view',
  ADMIN_ANALYTICS_MANAGE: 'admin:analytics:manage',

  // EPSX platform permissions (user-focused)
  EPSX_ALL: 'epsx:*:*',
  EPSX_ANALYTICS_VIEW: 'epsx:analytics:view',
  EPSX_ANALYTICS_EXPORT: 'epsx:analytics:export',
  EPSX_ANALYTICS_ADVANCED: 'epsx:analytics:advanced',
  EPSX_REALTIME_ACCESS: 'epsx:realtime:access',
  EPSX_PROFILE_MANAGE: 'epsx:profile:manage',
  EPSX_PROFILE_VIEW: 'epsx:profile:view',
  EPSX_NOTIFICATIONS_RECEIVE: 'epsx:notifications:receive',
  EPSX_NOTIFICATIONS_MANAGE: 'epsx:notifications:manage',
  EPSX_NOTIFICATIONS_SEND: 'epsx:notifications:send',
  EPSX_BILLING_MANAGE: 'epsx:billing:manage',
  EPSX_BILLING_VIEW: 'epsx:billing:view',
  EPSX_PAYMENT_CREATE: 'epsx:payment:create',
  EPSX_PAYMENT_VIEW: 'epsx:payment:view',
  EPSX_USERS_MANAGE: 'epsx:users:manage',
  EPSX_USERS_READ: 'epsx:users:read',

  // EPSX Pay platform permissions
  EPSX_PAY_ALL: 'epsx-pay:*:*',
  EPSX_PAY_TRANSACTIONS_CREATE: 'epsx-pay:transactions:create',
  EPSX_PAY_TRANSACTIONS_READ: 'epsx-pay:transactions:read',
  EPSX_PAY_PAYMENTS_PROCESS: 'epsx-pay:payments:process',
  EPSX_PAY_PAYMENTS_REFUND: 'epsx-pay:payments:refund',

  // EPSX Token platform permissions
  EPSX_TOKEN_ALL: 'epsx-token:*:*',
  EPSX_TOKEN_GOVERNANCE_VOTE: 'epsx-token:governance:vote',
  EPSX_TOKEN_GOVERNANCE_PROPOSE: 'epsx-token:governance:propose',
  EPSX_TOKEN_TOKENS_STAKE: 'epsx-token:tokens:stake',
  EPSX_TOKEN_TREASURY_VIEW: 'epsx-token:treasury:view',
} as const;

// ============================================================================
// CORE IAM CONFIGURATION
// ============================================================================

export const IAM_CONFIG = {
  // Default permissions for new users
  defaultPermissions: [
    PERMISSIONS.EPSX_ANALYTICS_VIEW,
    PERMISSIONS.EPSX_PROFILE_MANAGE,
    PERMISSIONS.EPSX_NOTIFICATIONS_RECEIVE
  ],

  // Session configuration
  session: {
    cookieName: 'sess_id',
    maxAge: 60 * 60 * 24 * 7, // 7 days
  },

  // Route protection configuration
  routes: {
    // Public routes that don't require authentication
    public: [
      '/',
      '/login',
      '/register',
      '/api/auth',
      '/api/public',
      '/_next',
      '/favicon.ico',
      '/public',
      '/static',
    ],

    // Protected routes with required permissions (structured format)
    protected: {
      // User-focused routes
      '/dashboard': [PERMISSIONS.EPSX_ANALYTICS_VIEW],
      '/analytics': [PERMISSIONS.EPSX_ANALYTICS_VIEW],
      '/analytics/export': [PERMISSIONS.EPSX_ANALYTICS_EXPORT],
      '/analytics/advanced': [PERMISSIONS.EPSX_ANALYTICS_ADVANCED],
      '/realtime': [PERMISSIONS.EPSX_REALTIME_ACCESS],
      '/profile': [PERMISSIONS.EPSX_PROFILE_MANAGE],
      '/billing': [PERMISSIONS.EPSX_BILLING_MANAGE],
      '/payment': [PERMISSIONS.EPSX_PAYMENT_CREATE],
      '/settings': [PERMISSIONS.EPSX_PROFILE_MANAGE],

      // Admin-focused routes
      '/admin': ['admin:*:*'],
      '/admin/users': ['admin:users:manage'],
      '/admin/users/create': ['admin:users:create'],
      '/admin/users/permissions': ['admin:permissions:manage'],
      '/admin/system': ['admin:system:manage'],
      '/admin/audit': ['admin:audit:read'],
      '/admin/notifications': [PERMISSIONS.ADMIN_NOTIFICATIONS_MANAGE],
      '/admin/analytics': [PERMISSIONS.ADMIN_ANALYTICS_VIEW],
      '/admin/security': [PERMISSIONS.ADMIN_SECURITY_MANAGE],

      // API routes with permissions (user context)
      '/api/analytics/export': [PERMISSIONS.EPSX_ANALYTICS_EXPORT],
      '/api/analytics/advanced': [PERMISSIONS.EPSX_ANALYTICS_ADVANCED],
      '/api/payment/create': [PERMISSIONS.EPSX_PAYMENT_CREATE],
      '/api/profile': [PERMISSIONS.EPSX_PROFILE_MANAGE],

      // API routes with permissions (admin context)
      '/api/admin/users': [PERMISSIONS.ADMIN_USERS_MANAGE],
      '/api/admin/permissions': [PERMISSIONS.ADMIN_PERMISSIONS_MANAGE],
      '/api/admin/system': [PERMISSIONS.ADMIN_SYSTEM_MANAGE],
      '/api/admin/notifications': [PERMISSIONS.ADMIN_NOTIFICATIONS_MANAGE],
      '/api/admin/analytics': [PERMISSIONS.ADMIN_ANALYTICS_MANAGE],
    },

    // Routes that require authentication but no specific permissions
    authenticated: ['/profile', '/settings', '/account', '/admin/profile'],
  },

  // API endpoints - both client-side and server-side configurations
  api: {
    // Client-side uses Next.js API routes
    client: {
      baseUrl: '/api',
      endpoints: {
        auth: '/api/auth',
        permissions: '/api/iam/permissions',
        users: '/api/users',
        analytics: '/api/analytics',
        admin: '/api/admin',
      },
    },

    // Server-side uses direct backend URLs (configured via env)
    server: {
      timeout: 10000, // 10 seconds
      retries: 3,
    }
  },
} as const;

// ============================================================================
// PERMISSION SETS FOR DIFFERENT USER ROLES
// ============================================================================

export const PERMISSION_SETS = {
  // Admin permission sets
  SUPER_ADMIN: [
    'admin:*:*'
  ],

  USER_MANAGER: [
    'admin:users:manage',
    'admin:permissions:view',
    'admin:analytics:view'
  ],

  SYSTEM_ADMIN: [
    'admin:system:manage',
    'admin:audit:read',
    'admin:security:read',
    'admin:analytics:view'
  ],

  CONTENT_MANAGER: [
    'admin:notifications:manage',
    'admin:analytics:view',
    'admin:users:read'
  ],

  // User permission sets
  ENTERPRISE_USER: [
    'epsx:*:*'
  ],

  PREMIUM_USER: [
    PERMISSIONS.EPSX_ANALYTICS_VIEW,
    PERMISSIONS.EPSX_ANALYTICS_EXPORT,
    PERMISSIONS.EPSX_ANALYTICS_ADVANCED,
    PERMISSIONS.EPSX_REALTIME_ACCESS,
    PERMISSIONS.EPSX_PROFILE_MANAGE,
    PERMISSIONS.EPSX_NOTIFICATIONS_RECEIVE,
    PERMISSIONS.EPSX_BILLING_MANAGE,
    PERMISSIONS.EPSX_PAYMENT_CREATE
  ],

  BASIC_USER: [
    PERMISSIONS.EPSX_ANALYTICS_VIEW,
    PERMISSIONS.EPSX_PROFILE_MANAGE,
    PERMISSIONS.EPSX_NOTIFICATIONS_RECEIVE,
    PERMISSIONS.EPSX_BILLING_VIEW
  ],

  FREE_USER: [
    PERMISSIONS.EPSX_ANALYTICS_VIEW,
    PERMISSIONS.EPSX_PROFILE_VIEW,
    PERMISSIONS.EPSX_NOTIFICATIONS_RECEIVE
  ],

  // Platform-specific permission sets
  EPSX_PAY_USER: [
    'epsx-pay:transactions:read',
    'epsx-pay:payments:process'
  ],

  EPSX_TOKEN_VOTER: [
    'epsx-token:governance:vote',
    'epsx-token:tokens:stake'
  ],

  EPSX_TOKEN_GOVERNOR: [
    'epsx-token:governance:vote',
    'epsx-token:governance:propose',
    'epsx-token:tokens:stake',
    'epsx-token:treasury:view'
  ]
} as const;

// ============================================================================
// PLATFORM DEFINITIONS
// ============================================================================

export const PLATFORMS = {
  EPSX: 'epsx',
  EPSX_PAY: 'epsx-pay',
  EPSX_TOKEN: 'epsx-token',
  ADMIN: 'admin'
} as const;

// ============================================================================
// ERROR MESSAGES AND CONSTANTS
// ============================================================================

export const IAM_ERROR_MESSAGES = {
  UNAUTHORIZED: 'You are not authorized to access this resource',
  FORBIDDEN: 'You do not have permission to access this resource',
  SESSION_EXPIRED: 'Your session has expired. Please log in again',
  INVALID_PERMISSION: 'Invalid permission specified',
  PERMISSION_DENIED: 'Permission denied',
  INSUFFICIENT_PERMISSIONS: 'You do not have sufficient permissions',
  INVALID_PLATFORM: 'Invalid platform specified',
  INVALID_RESOURCE: 'Invalid resource specified',
  INVALID_ACTION: 'Invalid action specified',
} as const;

// ============================================================================
// CACHE CONFIGURATION
// ============================================================================

export const CACHE_CONFIG = {
  permissions: {
    ttl: 5 * 60 * 1000, // 5 minutes
    key: 'user_permissions',
  },
  user_claims: {
    ttl: 10 * 60 * 1000, // 10 minutes  
    key: 'user_claims',
  },
  admin_permissions: {
    ttl: 3 * 60 * 1000, // 3 minutes (shorter for admin due to higher security)
    key: 'admin_permissions',
  },
} as const;

// ============================================================================
// PERMISSION UTILITY FUNCTIONS
// ============================================================================

/**
 * Get required permissions for a specific route
 */
export function getRoutePermissions(route: string): string[] | null {
  const permissions = IAM_CONFIG.routes.protected[route as keyof typeof IAM_CONFIG.routes.protected] as readonly string[] | undefined;
  return permissions ? (permissions as string[]) : null;
}

/**
 * Check if a route is public (no authentication required)
 */
export function isPublicRoute(route: string): boolean {
  return IAM_CONFIG.routes.public.some(publicRoute =>
    route.startsWith(publicRoute) || route === publicRoute
  );
}

/**
 * Check if a route requires authentication only (no specific permissions)
 */
export function isAuthenticatedRoute(route: string): boolean {
  return (IAM_CONFIG.routes.authenticated as readonly string[]).includes(route);
}

/**
 * Validate permission format (platform:resource:action)
 */
export function isValidPermission(permission: string): boolean {
  const parts = permission.split(':');
  return parts.length === 3 && parts.every(part => part.length > 0);
}

/**
 * Parse permission into components
 */
export function parsePermission(permission: string): { platform: string; resource: string; action: string } | null {
  const parts = permission.split(':');
  if (parts.length !== 3) { return null; }

  return {
    platform: parts[0] || '',
    resource: parts[1] || '',
    action: parts[2] || ''
  };
}

/**
 * Build permission from components
 */
export function buildPermission(platform: string, resource: string, action: string): string {
  return `${platform}:${resource}:${action}`;
}

/**
 * Get platform from permission
 */
export function getPermissionPlatform(permission: string): string | null {
  const parsed = parsePermission(permission);
  return parsed ? parsed.platform : null;
}

/**
 * Check if user has permission (supports wildcards)
 *
 * PERMISSION REFACTOR: Client-side permission checks are now permissive.
 * Backend (Rust) enforces access control based on user plan/permissions.
 * This is for UI display hints only.
 */
export function hasPermission(userPermissions: string[], _requiredPermission: string): boolean {
  // If user is authenticated (has any permissions), we are permissive on the client
  return userPermissions.length > 0;
}

/**
 * Get user's platform-specific permissions
 */
export function getPlatformPermissions(userPermissions: string[], platform: string): string[] {
  return userPermissions.filter(permission => {
    const permissionPlatform = getPermissionPlatform(permission);
    return permissionPlatform === platform;
  });
}

/**
 * Check if user is admin (has any admin permissions)
 */
export function isAdmin(userPermissions: string[]): boolean {
  // PERMISSION REFACTOR: Client-side is permissive for any authenticated user.
  // Backend validates actual admin role.
  return userPermissions.length > 0;
}

/**
 * Check if user is super admin (has admin:*:* permission)
 */
export function isSuperAdmin(userPermissions: string[]): boolean {
  return userPermissions.length > 0;
}

/**
 * Get user's effective permission set based on their highest tier
 */
export function getUserEffectivePermissions(userPermissions: string[]): string[] {
  // PERMISSION REFACTOR: Return provided permissions as-is.
  // Granular set expansion is now managed by the backend.
  return [...userPermissions];
}