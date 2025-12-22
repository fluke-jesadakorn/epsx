/**
 * CONSOLIDATED IAM CONFIGURATION
 * Unified Identity and Access Management configuration shared across admin-frontend and frontend
 * Supports structured permissions in "platform:resource:action" format
 */

// ============================================================================
// CORE IAM CONFIGURATION
// ============================================================================

export const IAM_CONFIG = {
  // Default permissions for new users
  defaultPermissions: [
    'epsx:analytics:view',
    'epsx:profile:manage',
    'epsx:notifications:receive'
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
      '/api/v1/auth',
      '/api/public',
      '/_next',
      '/favicon.ico',
      '/public',
      '/static',
    ],

    // Protected routes with required permissions (structured format)
    protected: {
      // User-focused routes
      '/dashboard': ['epsx:analytics:view'],
      '/analytics': ['epsx:analytics:view'],
      '/analytics/export': ['epsx:analytics:export'],
      '/analytics/advanced': ['epsx:analytics:advanced'],
      '/realtime': ['epsx:realtime:access'],
      '/profile': ['epsx:profile:manage'],
      '/billing': ['epsx:billing:manage'],
      '/payment': ['epsx:payment:create'],
      '/settings': ['epsx:profile:manage'],

      // Admin-focused routes
      '/admin': ['admin:*:*'],
      '/admin/users': ['admin:users:manage'],
      '/admin/users/create': ['admin:users:create'],
      '/admin/users/permissions': ['admin:permissions:manage'],
      '/admin/system': ['admin:system:manage'],
      '/admin/audit': ['admin:audit:read'],
      '/admin/notifications': ['admin:notifications:manage'],
      '/admin/analytics': ['admin:analytics:view'],
      '/admin/security': ['admin:security:manage'],

      // API routes with permissions (user context)
      '/api/v1/analytics/export': ['epsx:analytics:export'],
      '/api/v1/analytics/advanced': ['epsx:analytics:advanced'],
      '/api/v1/payment/create': ['epsx:payment:create'],
      '/api/v1/profile': ['epsx:profile:manage'],

      // API routes with permissions (admin context)
      '/api/admin/users': ['admin:users:manage'],
      '/api/admin/permissions': ['admin:permissions:manage'],
      '/api/admin/system': ['admin:system:manage'],
      '/api/admin/notifications': ['admin:notifications:manage'],
      '/api/admin/analytics': ['admin:analytics:manage'],
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
        auth: '/api/v1/auth',
        permissions: '/api/v1/iam/permissions',
        users: '/api/v1/users',
        analytics: '/api/v1/analytics',
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
    'epsx:analytics:view',
    'epsx:analytics:export',
    'epsx:analytics:advanced',
    'epsx:realtime:access',
    'epsx:profile:manage',
    'epsx:notifications:receive',
    'epsx:billing:manage',
    'epsx:payment:create'
  ],

  BASIC_USER: [
    'epsx:analytics:view',
    'epsx:profile:manage',
    'epsx:notifications:receive',
    'epsx:billing:view'
  ],

  FREE_USER: [
    'epsx:analytics:view',
    'epsx:profile:view',
    'epsx:notifications:receive'
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
  const permissions = IAM_CONFIG.routes.protected[route as keyof typeof IAM_CONFIG.routes.protected];
  return permissions ? [...permissions] : null; // Convert readonly array to mutable array
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
  if (parts.length !== 3) return null;

  return {
    platform: parts[0],
    resource: parts[1],
    action: parts[2]
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
 * ⚠️ IMPORTANT: FOR UI DISPLAY ONLY - DO NOT USE FOR AUTHORIZATION
 * Backend validates all permissions via Rust middleware.
 * Use hasPermissionForDisplay() from SharedWeb3AuthClient for UI display.
 */
export function hasPermission(userPermissions: string[], requiredPermission: string): boolean {
  // @deprecated - This is for UI display only. Backend enforces all permissions.
  if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
    console.warn('[DEPRECATED] hasPermission() is for UI display only. Backend enforces all permissions via JWT middleware.');
  }
  if (!userPermissions || userPermissions.length === 0) return false;

  // Check for exact match first
  if (userPermissions.includes(requiredPermission)) return true;

  // Check for wildcard permissions
  const { platform, resource, action } = parsePermission(requiredPermission) || {};
  if (!platform || !resource || !action) return false;

  // Check various wildcard patterns
  const wildcardPatterns = [
    `${platform}:*:*`, // Platform wildcard (e.g., admin:*:*)
    `${platform}:${resource}:*`, // Resource wildcard (e.g., admin:users:*)
  ];

  return wildcardPatterns.some(pattern => userPermissions.includes(pattern));
}

/**
 * Check if user has any of the required permissions
 *
 * ⚠️ IMPORTANT: FOR UI DISPLAY ONLY - DO NOT USE FOR AUTHORIZATION
 * Backend validates all permissions via Rust middleware.
 */
export function hasAnyPermission(userPermissions: string[], requiredPermissions: string[]): boolean {
  // @deprecated - This is for UI display only. Backend enforces all permissions.
  if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
    console.warn('[DEPRECATED] hasAnyPermission() is for UI display only. Backend enforces all permissions via JWT middleware.');
  }
  return requiredPermissions.some(permission => hasPermission(userPermissions, permission));
}

/**
 * Check if user has all required permissions
 *
 * ⚠️ IMPORTANT: FOR UI DISPLAY ONLY - DO NOT USE FOR AUTHORIZATION
 * Backend validates all permissions via Rust middleware.
 */
export function hasAllPermissions(userPermissions: string[], requiredPermissions: string[]): boolean {
  // @deprecated - This is for UI display only. Backend enforces all permissions.
  if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
    console.warn('[DEPRECATED] hasAllPermissions() is for UI display only. Backend enforces all permissions via JWT middleware.');
  }
  return requiredPermissions.every(permission => hasPermission(userPermissions, permission));
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
  return getPlatformPermissions(userPermissions, PLATFORMS.ADMIN).length > 0;
}

/**
 * Check if user is super admin (has admin:*:* permission)
 */
export function isSuperAdmin(userPermissions: string[]): boolean {
  return hasPermission(userPermissions, PERMISSIONS.ADMIN_ALL);
}

/**
 * Get user's effective permission set based on their highest tier
 */
export function getUserEffectivePermissions(userPermissions: string[]): string[] {
  // If user has any wildcard permission, include the base permissions for that platform
  const effectivePermissions = new Set(userPermissions);

  for (const permission of userPermissions) {
    if (permission.endsWith(':*:*')) {
      // User has platform-wide access
      const platform = getPermissionPlatform(permission);
      if (platform === PLATFORMS.ADMIN) {
        Object.values(PERMISSIONS)
          .filter(p => p.startsWith('admin:'))
          .forEach(p => effectivePermissions.add(p));
      } else if (platform === PLATFORMS.EPSX) {
        Object.values(PERMISSIONS)
          .filter(p => p.startsWith('epsx:'))
          .forEach(p => effectivePermissions.add(p));
      }
    }
  }

  return Array.from(effectivePermissions);
}