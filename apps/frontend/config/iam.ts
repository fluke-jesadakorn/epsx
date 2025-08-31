// Permission-only IAM Configuration for the frontend application
// Using structured permissions in "platform:resource:action" format

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
      '/forgot-password',
      '/reset-password',
      '/api/v1/auth',
      '/api/public',
      '/_next',
      '/favicon.ico',
      '/public',
      '/static',
    ],

    // Protected routes with required permissions (structured format)
    protected: {
      '/dashboard': ['epsx:analytics:view'],
      '/analytics/export': ['epsx:analytics:export'],
      '/analytics/advanced': ['epsx:analytics:advanced'],
      '/realtime': ['epsx:realtime:access'],
      '/profile': ['epsx:profile:manage'],
      '/billing': ['epsx:billing:manage'],
      '/admin': ['admin:*:*'],
      '/admin/users': ['admin:users:manage', 'epsx:users:manage'],
      '/admin/system': ['admin:system:manage'],
      '/admin/audit': ['admin:audit:read'],
      
      // API routes with permissions
      '/api/v1/admin/users': ['admin:users:manage', 'epsx:users:manage'],
      '/api/v1/admin/system': ['admin:system:manage'],
      '/api/v1/analytics/export': ['epsx:analytics:export'],
      '/api/v1/analytics/advanced': ['epsx:analytics:advanced'],
    },

    // Routes that require authentication but no specific permissions
    authenticated: ['/profile', '/settings', '/account'],
  },

  // API endpoints - client-side uses relative paths
  api: {
    baseUrl: '/api', // Use Next.js API routes for client-side
    endpoints: {
      auth: '/api/v1/auth',
      permissions: '/api/v1/iam/permissions',
      users: '/api/v1/users',
      analytics: '/api/v1/analytics',
    },
  },

  // Backend API configuration - server-side only
  backend: {
    timeout: 10000, // 10 seconds
  },
};

// Structured permission definitions using "platform:resource:action" format
export const PERMISSIONS = {
  // Admin permissions
  ADMIN_ALL: 'admin:*:*',
  ADMIN_USERS_MANAGE: 'admin:users:manage',
  ADMIN_USERS_READ: 'admin:users:read',
  ADMIN_SYSTEM_MANAGE: 'admin:system:manage',
  ADMIN_AUDIT_READ: 'admin:audit:read',
  ADMIN_SECURITY_READ: 'admin:security:read',

  // EPSX platform permissions
  EPSX_ANALYTICS_VIEW: 'epsx:analytics:view',
  EPSX_ANALYTICS_EXPORT: 'epsx:analytics:export',
  EPSX_ANALYTICS_ADVANCED: 'epsx:analytics:advanced',
  EPSX_REALTIME_ACCESS: 'epsx:realtime:access',
  EPSX_PROFILE_MANAGE: 'epsx:profile:manage',
  EPSX_NOTIFICATIONS_RECEIVE: 'epsx:notifications:receive',
  EPSX_BILLING_MANAGE: 'epsx:billing:manage',
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

// Permission sets (replace role-based logic)
export const PERMISSION_SETS = {
  ADMIN: [
    'admin:*:*'
  ],
  
  PREMIUM_USER: [
    'epsx:analytics:view',
    'epsx:analytics:export', 
    'epsx:analytics:advanced',
    'epsx:realtime:access',
    'epsx:profile:manage',
    'epsx:notifications:receive',
    'epsx:billing:manage'
  ],
  
  BASIC_USER: [
    'epsx:analytics:view',
    'epsx:profile:manage',
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

// Platform definitions
export const PLATFORMS = {
  EPSX: 'epsx',
  EPSX_PAY: 'epsx-pay', 
  EPSX_TOKEN: 'epsx-token',
  ADMIN: 'admin'
} as const;

// Error messages
export const IAM_ERROR_MESSAGES = {
  UNAUTHORIZED: 'You are not authorized to access this resource',
  FORBIDDEN: 'You do not have permission to access this resource',
  SESSION_EXPIRED: 'Your session has expired. Please log in again',
  INVALID_PERMISSION: 'Invalid permission specified',
  PERMISSION_DENIED: 'Permission denied',
  INSUFFICIENT_PERMISSIONS: 'You do not have sufficient permissions',
} as const;

// Cache configuration
export const CACHE_CONFIG = {
  permissions: {
    ttl: 5 * 60 * 1000, // 5 minutes
    key: 'user_permissions',
  },
  user_claims: {
    ttl: 10 * 60 * 1000, // 10 minutes  
    key: 'user_claims',
  },
} as const;

// Route permission mapping helper
export function getRoutePermissions(route: string): string[] | null {
  return IAM_CONFIG.routes.protected[route as keyof typeof IAM_CONFIG.routes.protected] || null;
}

// Check if route is public
export function isPublicRoute(route: string): boolean {
  return IAM_CONFIG.routes.public.some(publicRoute => 
    route.startsWith(publicRoute) || route === publicRoute
  );
}

// Check if route requires authentication only (no specific permissions)
export function isAuthenticatedRoute(route: string): boolean {
  return IAM_CONFIG.routes.authenticated.includes(route);
}

// Permission validation helper
export function isValidPermission(permission: string): boolean {
  const parts = permission.split(':');
  return parts.length === 3 && parts.every(part => part.length > 0);
}

// Parse permission into components
export function parsePermission(permission: string): { platform: string; resource: string; action: string } | null {
  const parts = permission.split(':');
  if (parts.length !== 3) return null;
  
  return {
    platform: parts[0],
    resource: parts[1], 
    action: parts[2]
  };
}

// Build permission from components
export function buildPermission(platform: string, resource: string, action: string): string {
  return `${platform}:${resource}:${action}`;
}

// Get platform from permission
export function getPermissionPlatform(permission: string): string | null {
  const parsed = parsePermission(permission);
  return parsed ? parsed.platform : null;
}