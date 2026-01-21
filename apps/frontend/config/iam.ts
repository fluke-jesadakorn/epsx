/**
 * FRONTEND - IAM CONFIGURATION COMPATIBILITY LAYER
 * Migrated to use consolidated shared/config/iam.ts
 * Provides user-specific configuration and backward compatibility
 */

import {
  getPlatformPermissions,
  getUserEffectivePermissions,
  hasAllPermissions,
  hasAnyPermission,
  hasPermission,
  isAdmin,
  isSuperAdmin,
  PLATFORMS,
  // Core IAM utilities
  IAM_CONFIG as SHARED_IAM_CONFIG,
  buildPermission as sharedBuildPermission,
  // Utility functions
  getRoutePermissions as sharedGetRoutePermissions,
  isAuthenticatedRoute as sharedIsAuthenticatedRoute,
  isPublicRoute as sharedIsPublicRoute,
  isValidPermission as sharedIsValidPermission,
  parsePermission as sharedParsePermission
} from '@/shared/config/iam';

/**
 * Frontend-specific IAM configuration
 * Extends shared IAM config with user/frontend context
 */
export const IAM_CONFIG = {
  ...SHARED_IAM_CONFIG,

  // Frontend-specific route permissions (extends shared config)
  routes: {
    ...SHARED_IAM_CONFIG.routes,

    // Additional frontend-specific routes
    protected: {
      ...SHARED_IAM_CONFIG.routes.protected,

      // Frontend-specific routes
      '/portfolio': ['epsx:analytics:view'],
      '/permissions': ['epsx:profile:manage'],

      // Frontend API routes
      '/api/auth/web3/permissions': ['epsx:profile:view'],
      '/api/auth/session': ['epsx:profile:view'],
    },
  },

  // Frontend-specific session configuration
  session: {
    cookieName: 'user_session',
    maxAge: 60 * 60 * 24 * 7, // 7 days for user sessions
  },

  // Frontend-specific API configuration
  api: {
    ...SHARED_IAM_CONFIG.api,
    client: {
      baseUrl: '/api',
      endpoints: {
        auth: '/api/auth',
        permissions: '/api/iam/permissions',
        users: '/api/users',
        analytics: '/api/analytics',
        payment: '/api/payment',
        web3: '/api/auth/web3',
      },
    },
  },
} as const;

// Re-export all shared permissions for frontend use
export { PERMISSIONS } from '@/shared/config/iam';

// Re-export shared permission sets and add frontend-specific sets
export {
  CACHE_CONFIG,
  IAM_ERROR_MESSAGES, PERMISSION_SETS,
  PLATFORMS
} from '@/shared/config/iam';

/**
 * Frontend-specific permission sets
 * Additional permission sets for frontend user context
 */
export const FRONTEND_PERMISSION_SETS = {
  // User-specific permission sets (non-admin)
  FREE_USER: [
    'epsx:analytics:view',
    'epsx:profile:view',
    'epsx:notifications:receive'
  ],

  TRIAL_USER: [
    'epsx:analytics:view',
    'epsx:analytics:export',
    'epsx:profile:manage',
    'epsx:notifications:receive'
  ],

  // Web3-specific permission sets
  WEB3_USER: [
    'epsx:analytics:view',
    'epsx:profile:manage',
    'epsx:notifications:receive',
    'epsx:web3:connect'
  ],

  WEB3_PREMIUM: [
    'epsx:analytics:view',
    'epsx:analytics:export',
    'epsx:analytics:advanced',
    'epsx:realtime:access',
    'epsx:profile:manage',
    'epsx:notifications:receive',
    'epsx:billing:manage',
    'epsx:web3:connect',
    'epsx:web3:transact'
  ],
} as const;

/**
 * Frontend-specific utility functions
 */

/**
 * Check if user has permissions for frontend context (non-admin)
 */
export function hasUserPermissions(userPermissions: string[]): boolean {
  // Users should have EPSX platform permissions, not admin permissions
  return getPlatformPermissions(userPermissions, PLATFORMS.EPSX).length > 0;
}

/**
 * Check if user can access frontend route
 */
export function canAccessUserRoute(route: string, userPermissions: string[]): boolean {
  // Check if route is public
  if (isPublicRoute(route)) {
    return true;
  }

  // Check if route requires authentication only
  if (isAuthenticatedRoute(route)) {
    return userPermissions.length > 0;
  }

  // Check specific route permissions
  const requiredPermissions = getRoutePermissions(route);
  if (requiredPermissions && requiredPermissions.length > 0) {
    return hasAnyPermission(userPermissions, requiredPermissions);
  }

  // Default to requiring authentication
  return userPermissions.length > 0;
}

/**
 * Get user-specific effective permissions (filter out admin permissions)
 */
export function getUserEffectivePermissionsFiltered(userPermissions: string[]): string[] {
  const effectivePermissions = getUserEffectivePermissions(userPermissions);

  // Filter out admin permissions for user context
  return effectivePermissions.filter(permission =>
    !permission.startsWith('admin:')
  );
}

/**
 * Validate user permission context
 */
export function validateUserPermission(permission: string): {
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

  // User context - should not be admin platform
  if (parsed.platform === PLATFORMS.ADMIN) {
    return {
      valid: false,
      error: 'Admin permissions not allowed in user context'
    };
  }

  return { valid: true };
}

/**
 * Get user's permission tier based on their permissions
 */
export function getUserPermissionTier(userPermissions: string[]): 'free' | 'trial' | 'basic' | 'premium' | 'enterprise' {
  if (!userPermissions || userPermissions.length === 0) {
    return 'free';
  }

  // Check for enterprise permissions
  if (hasPermission(userPermissions, 'epsx:*:*')) {
    return 'enterprise';
  }

  // Check for premium permissions
  const premiumPermissions = [
    'epsx:analytics:advanced',
    'epsx:realtime:access',
    'epsx:analytics:export'
  ];

  if (premiumPermissions.some(permission => hasPermission(userPermissions, permission))) {
    return 'premium';
  }

  // Check for basic permissions
  const basicPermissions = [
    'epsx:analytics:view',
    'epsx:profile:manage'
  ];

  if (basicPermissions.some(permission => hasPermission(userPermissions, permission))) {
    // Check if it's trial (has export but not advanced)
    if (hasPermission(userPermissions, 'epsx:analytics:export') &&
      !hasPermission(userPermissions, 'epsx:analytics:advanced')) {
      return 'trial';
    }
    return 'basic';
  }

  return 'free';
}

// Re-export shared utility functions with frontend context
export const getRoutePermissions = sharedGetRoutePermissions;
export const isPublicRoute = sharedIsPublicRoute;
export const isAuthenticatedRoute = sharedIsAuthenticatedRoute;
export const isValidPermission = sharedIsValidPermission;
export const parsePermission = sharedParsePermission;
export const buildPermission = sharedBuildPermission;

// Legacy function name compatibility
export function getPermissionPlatform(permission: string): string | null {
  const parsed = parsePermission(permission);
  return parsed ? parsed.platform : null;
}

// Re-export all shared IAM utilities for frontend use
export {
  getPlatformPermissions, getUserEffectivePermissions, hasAllPermissions, hasAnyPermission, hasPermission, isAdmin,
  isSuperAdmin
};

// Legacy compatibility - export frontend IAM config as default
export default IAM_CONFIG;