/**
 * ADMIN FRONTEND - IAM CONFIGURATION COMPATIBILITY LAYER
 * Web3 wallet-first system with stub IAM configuration for compatibility
 * Provides admin-specific configuration and backward compatibility
 */

// Stub constants for IAM compatibility
const STUB_IAM_CONFIG = {
  platforms: ['admin', 'epsx', 'epsx-pay', 'epsx-token'],
  defaultPlatform: 'admin',
  wildcardPermissions: true,
  permissionCaching: true,
  routes: {
    protected: {} as Record<string, string[]>
  },
  api: {
    client: {
      baseUrl: '/api',
      endpoints: {}
    }
  }
} as const;

export const IAM_CONFIG = STUB_IAM_CONFIG;

export const PERMISSIONS = {
  admin: {
    users: ['view', 'create', 'update', 'delete', 'manage'],
    system: ['view', 'configure', 'manage'],
    analytics: ['view', 'export', 'manage']
  },
  epsx: {
    analytics: ['view', 'export'],
    data: ['view', 'access']
  }
} as const;

export const PERMISSION_SETS = {
  ADMIN_FULL: 'admin:*:*',
  ADMIN_USERS: 'admin:users:*',
  ADMIN_SYSTEM: 'admin:system:*',
  EPSX_BASIC: 'epsx:analytics:view'
} as const;

export const PLATFORMS = {
  ADMIN: 'admin',
  EPSX: 'epsx',
  EPSX_PAY: 'epsx-pay',
  EPSX_TOKEN: 'epsx-token'
} as const;

export const CACHE_CONFIG = {
  ttl: 300,
  maxSize: 1000,
  enabled: true
} as const;

export const IAM_ERROR_MESSAGES = {
  INVALID_PERMISSION: 'Invalid permission format',
  UNAUTHORIZED: 'Unauthorized access',
  INSUFFICIENT_PERMISSIONS: 'Insufficient permissions',
  PERMISSION_DENIED: 'Permission denied'
} as const;

// Stub utility functions for compatibility
export function getRoutePermissions(route: string): string[] {
  const adminRoutes: Record<string, string[]> = {
    '/admin/users': ['admin:users:view'],
    '/admin/system': ['admin:system:view'],
    '/admin/analytics': ['admin:analytics:view'],
    '/admin': ['admin:*:*']
  };
  return adminRoutes[route] || ['admin:*:*'];
}

export function isPublicRoute(route: string): boolean {
  return route === '/login' || route === '/';
}

export function isAuthenticatedRoute(route: string): boolean {
  return route.startsWith('/admin');
}

export function isValidPermission(permission: string): boolean {
  return /^[a-z-]+:[a-z*]+:[a-z*]+$/.test(permission);
}

export function parsePermission(permission: string): { platform: string; resource: string; action: string } | null {
  const parts = permission.split(':');
  if (parts.length !== 3) return null;
  return { platform: parts[0], resource: parts[1], action: parts[2] };
}

export function buildPermission(platform: string, resource: string, action: string): string {
  return `${platform}:${resource}:${action}`;
}

export function hasPermission(userPermissions: string[], requiredPermission: string): boolean {
  const parsed = parsePermission(requiredPermission);
  if (!parsed) return false;
  
  for (const userPerm of userPermissions) {
    const userParsed = parsePermission(userPerm);
    if (!userParsed) continue;
    
    if (userParsed.platform === parsed.platform) {
      if ((userParsed.resource === '*' && userParsed.action === '*') ||
          (userParsed.resource === parsed.resource && userParsed.action === '*') ||
          (userParsed.resource === parsed.resource && userParsed.action === parsed.action)) {
        return true;
      }
    }
  }
  return false;
}

export function hasAnyPermission(userPermissions: string[], requiredPermissions: string[]): boolean {
  return requiredPermissions.some(perm => hasPermission(userPermissions, perm));
}

export function hasAllPermissions(userPermissions: string[], requiredPermissions: string[]): boolean {
  return requiredPermissions.every(perm => hasPermission(userPermissions, perm));
}

export function getPlatformPermissions(userPermissions: string[], platform: string): string[] {
  return userPermissions.filter(perm => perm.startsWith(`${platform}:`));
}

export function isAdmin(userPermissions: string[]): boolean {
  return hasAnyPermission(userPermissions, ['admin:*:*', 'admin:users:manage', 'admin:system:manage']);
}

export function isSuperAdmin(userPermissions: string[]): boolean {
  return hasPermission(userPermissions, 'admin:*:*');
}

export function getUserEffectivePermissions(userPermissions: string[]): string[] {
  return userPermissions; // Simple stub implementation
}

/**
 * Admin-specific IAM configuration
 * Extends shared IAM config with admin context defaults
 */
export const ADMIN_IAM_CONFIG = {
  ...IAM_CONFIG,
  
  // Admin-specific route permissions (extends shared config)
  routes: {
    ...IAM_CONFIG.routes,
    
    // Additional admin-specific routes not in shared config
    protected: {
      ...IAM_CONFIG.routes.protected,
      
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
    ...IAM_CONFIG.api,
    client: {
      baseUrl: '/api/admin',
      endpoints: {
        auth: '/api/v1/auth',
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

/**
 * Admin permission sets (extends shared permission sets)
 */
export const ADMIN_PERMISSION_SETS = {
  ...PERMISSION_SETS,
  
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
  if (!isValidPermission(permission)) {
    return {
      valid: false,
      error: 'Invalid permission format. Must be platform:resource:action'
    };
  }
  
  const parsed = parsePermission(permission);
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

// All constants and utility functions are already exported above
// No additional re-exports needed

// Legacy compatibility - export admin config as default
export default ADMIN_IAM_CONFIG;