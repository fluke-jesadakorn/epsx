/**
 * ADMIN FRONTEND - IAM CONFIGURATION COMPATIBILITY LAYER
 * Web3 wallet-first system using consolidated shared configuration
 * Provides admin-specific configuration and backward compatibility
 */

import {
  CACHE_CONFIG,
  getPlatformPermissions,
  getUserEffectivePermissions,
  hasAllPermissions,
  hasAnyPermission,
  hasPermission,
  // Error messages
  IAM_ERROR_MESSAGES,
  isAdmin,
  isSuperAdmin,
  PLATFORMS,
  // Core IAM utilities
  IAM_CONFIG as SHARED_IAM_CONFIG,
  PERMISSION_SETS as SHARED_PERMISSION_SETS,
  PERMISSIONS as SHARED_PERMISSIONS,
  buildPermission as sharedBuildPermission,
  // Utility functions
  getRoutePermissions as sharedGetRoutePermissions,
  isAuthenticatedRoute as sharedIsAuthenticatedRoute,
  isPublicRoute as sharedIsPublicRoute,
  isValidPermission as sharedIsValidPermission,
  parsePermission as sharedParsePermission
} from '@/shared/config/iam';

/**
 * Admin-specific IAM configuration
 * Extends shared IAM config with admin context defaults
 */
export const ADMIN_IAM_CONFIG = {
  ...SHARED_IAM_CONFIG,

  // Admin-specific route permissions (extends shared config)
  routes: {
    ...SHARED_IAM_CONFIG.routes,

    // Additional admin-specific routes not in shared config
    protected: {
      ...SHARED_IAM_CONFIG.routes.protected,

      // Admin-specific routes
      '/admin/bulk-operations': ['admin:users:manage'],
      '/admin/permission-templates': ['admin:permissions:manage'],
      '/admin/developer-portal': ['admin:system:manage'],
      '/admin/compliance': ['admin:security:manage'],
      '/admin/dao': ['admin:governance:manage'],
      '/admin/enterprise': ['admin:enterprise:manage'],

      // Admin API routes
      '/api/admin/bulk': ['admin:users:manage'],
      '/api/admin/templates': ['admin:permissions:manage'],
      '/api/admin/compliance': ['admin:security:manage'],
      '/api/admin/dao': ['admin:governance:manage'],
      '/api/admin/enterprise': ['admin:enterprise:manage'],
    },
  },

  // Admin-specific session configuration
  session: {
    cookieName: 'admin_session',
    maxAge: 60 * 60 * 4, // 4 hours for admin sessions (shorter for security)
  },

  // Admin-specific API configuration
  api: {
    ...SHARED_IAM_CONFIG.api,
    client: {
      baseUrl: '/api/admin',
      endpoints: {
        auth: '/api/auth',
        permissions: '/api/admin/permissions',
        users: '/api/admin/users',
        analytics: '/api/admin/analytics',
        system: '/api/admin/system',
        bulk: '/api/admin/bulk',
        templates: '/api/admin/templates',
        compliance: '/api/admin/compliance',
        dao: '/api/admin/dao',
        enterprise: '/api/admin/enterprise',
      },
    },
  },
} as const;

// Use ADMIN_IAM_CONFIG as the default for this file
export const IAM_CONFIG = ADMIN_IAM_CONFIG;

/**
 * Admin permission sets (extends shared permission sets)
 */
export const ADMIN_PERMISSION_SETS = {
  ...SHARED_PERMISSION_SETS,

  // Additional admin-specific permission sets
  COMPLIANCE_MANAGER: [
    'admin:security:manage',
    'admin:audit:read',
    'admin:users:read',
    'admin:analytics:view'
  ],

  GOVERNANCE_MANAGER: [
    'admin:governance:manage',
    'admin:dao:manage',
    'admin:users:read',
    'admin:analytics:view'
  ],

  ENTERPRISE_MANAGER: [
    'admin:enterprise:manage',
    'admin:users:manage',
    'admin:permissions:manage',
    'admin:analytics:view'
  ],

  DEVELOPER_ADMIN: [
    'admin:system:manage',
    'admin:analytics:manage',
    'admin:audit:read'
  ],
} as const;

// Re-export shared permissions and sets
export const PERMISSIONS = SHARED_PERMISSIONS;
export const PERMISSION_SETS = ADMIN_PERMISSION_SETS;

// Re-export constants
export { CACHE_CONFIG, IAM_ERROR_MESSAGES, PLATFORMS };

/**
 * Admin-specific utility functions
 */

/**
 * Check if user has admin permissions
 */
export function hasAdminPermissions(userPermissions: string[]): boolean {
  return isAdmin(userPermissions);
}

/**
 * Check if user can access admin route
 */
export function canAccessAdminRoute(route: string, userPermissions: string[]): boolean {
  // All admin routes require admin permissions
  if (!hasAdminPermissions(userPermissions)) {
    return false;
  }

  // Check specific route permissions
  const requiredPermissions = getRoutePermissions(route);
  if (requiredPermissions && requiredPermissions.length > 0) {
    return hasAnyPermission(userPermissions, requiredPermissions);
  }

  // If no specific permissions required, admin access is sufficient
  return true;
}

/**
 * Get admin-specific effective permissions
 */
export function getAdminEffectivePermissions(userPermissions: string[]): string[] {
  const effectivePermissions = getUserEffectivePermissions(userPermissions);

  // Filter to only admin permissions for admin context
  return effectivePermissions.filter(permission =>
    permission.startsWith('admin:')
  );
}

/**
 * Validate admin permission context
 */
export function validateAdminPermission(permission: string): {
  valid: boolean;
  error?: string;
} {
  if (!sharedIsValidPermission(permission)) {
    return {
      valid: false,
      error: 'Invalid permission format. Must be platform:resource:action'
    };
  }

  const parsed = sharedParsePermission(permission);
  if (!parsed) {
    return {
      valid: false,
      error: 'Failed to parse permission structure'
    };
  }

  // Admin context - should be admin platform
  if (parsed.platform !== PLATFORMS.ADMIN) {
    return {
      valid: false,
      error: 'Permission must be for admin platform in admin context'
    };
  }

  return { valid: true };
}

/**
 * Get user's admin permission tier
 */
export function getAdminPermissionTier(userPermissions: string[]): 'none' | 'basic' | 'manager' | 'super' {
  if (!hasAdminPermissions(userPermissions)) {
    return 'none';
  }

  if (isSuperAdmin(userPermissions)) {
    return 'super';
  }

  const adminPermissions = getPlatformPermissions(userPermissions, PLATFORMS.ADMIN);

  // Check for manager-level permissions
  const managerPermissions = [
    'admin:users:manage',
    'admin:permissions:manage',
    'admin:system:manage'
  ];

  if (managerPermissions.some(permission => hasPermission(userPermissions, permission))) {
    return 'manager';
  }

  return 'basic';
}

// Re-export shared utility functions
export const getRoutePermissions = sharedGetRoutePermissions;
export const isPublicRoute = sharedIsPublicRoute;
export const isAuthenticatedRoute = sharedIsAuthenticatedRoute;
export const isValidPermission = sharedIsValidPermission;
export const parsePermission = sharedParsePermission;
export const buildPermission = sharedBuildPermission;

// Re-export authentication helpers
export {
  getPlatformPermissions, getUserEffectivePermissions, hasAllPermissions, hasAnyPermission, hasPermission, isAdmin,
  isSuperAdmin
};

// Legacy compatibility - export admin config as default
export default ADMIN_IAM_CONFIG;