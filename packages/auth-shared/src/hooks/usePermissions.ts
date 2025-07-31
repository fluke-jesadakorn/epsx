import { useMemo, useCallback } from 'react';
import type { AuthenticatedUser } from '@epsx/types';
import type { PermissionCheckResult } from '../types/index.js';

export interface UsePermissionsOptions {
  /** Enable caching of permission results */
  enableCaching?: boolean;
  /** Debug permission checks */
  debug?: boolean;
}

export interface PermissionsContext {
  user: AuthenticatedUser | null;
  permissions: string[];
  roles: string[];
  profiles: string[];
}

/**
 * Unified permissions hook with advanced permission checking
 * Consolidates duplicate permission logic from across the codebase
 */
export function usePermissions(
  context: PermissionsContext,
  options: UsePermissionsOptions = {}
) {
  const { enableCaching = true, debug = false } = options;
  const { user, permissions = [], roles = [], profiles = [] } = context;

  // Permission cache (when enabled)
  const permissionCache = useMemo(() => new Map<string, boolean>(), [enableCaching]);

  // Core permission checking function
  const checkPermission = useCallback((
    permission: string,
    options: { bypassCache?: boolean } = {}
  ): PermissionCheckResult => {
    const cacheKey = permission;
    
    // Check cache first (if enabled and not bypassed)
    if (enableCaching && !options.bypassCache && permissionCache.has(cacheKey)) {
      const cached = permissionCache.get(cacheKey)!;
      return {
        allowed: cached,
        reason: cached ? 'Permission granted (cached)' : 'Permission denied (cached)',
        requiredPermission: permission,
        userPermissions: permissions,
        userRole: roles[0],
        userProfiles: profiles,
      };
    }

    let allowed = false;
    let reason = 'Permission denied';

    if (!user) {
      reason = 'User not authenticated';
    } else if (permission === '*' || permissions.includes('*')) {
      allowed = true;
      reason = 'Wildcard permission granted';
    } else if (permission.endsWith('.*')) {
      // Handle namespace wildcards (e.g., "admin.*")
      const basePermission = permission.slice(0, -2);
      allowed = permissions.some(p => p.startsWith(basePermission));
      reason = allowed 
        ? `Namespace permission granted for ${basePermission}` 
        : `No permissions found for namespace ${basePermission}`;
    } else if (permission.endsWith(':*')) {
      // Handle action wildcards (e.g., "users:*")
      const basePermission = permission.slice(0, -2);
      allowed = permissions.some(p => p.startsWith(basePermission + ':'));
      reason = allowed 
        ? `Action wildcard permission granted for ${basePermission}` 
        : `No action permissions found for ${basePermission}`;
    } else if (permission.includes('|')) {
      // Handle OR logic (e.g., "admin:read|user:read")
      const orPermissions = permission.split('|');
      allowed = orPermissions.some(p => permissions.includes(p.trim()));
      reason = allowed 
        ? `OR permission granted for one of: ${permission}` 
        : `None of the OR permissions granted: ${permission}`;
    } else if (permission.includes('&')) {
      // Handle AND logic (e.g., "admin:read&admin:write")
      const andPermissions = permission.split('&');
      allowed = andPermissions.every(p => permissions.includes(p.trim()));
      reason = allowed 
        ? `AND permission granted for all of: ${permission}` 
        : `Not all AND permissions granted: ${permission}`;
    } else {
      // Exact permission match
      allowed = permissions.includes(permission);
      reason = allowed 
        ? 'Exact permission match' 
        : `Permission ${permission} not found in user permissions`;
    }

    // Cache result (if enabled)
    if (enableCaching) {
      permissionCache.set(cacheKey, allowed);
    }

    // Debug logging
    if (debug) {
      console.log('Permission Check:', {
        permission,
        allowed,
        reason,
        userPermissions: permissions,
        userRoles: roles,
      });
    }

    return {
      allowed,
      reason,
      requiredPermission: permission,
      userPermissions: permissions,
      userRole: roles[0],
      userProfiles: profiles,
    };
  }, [user, permissions, roles, profiles, enableCaching, debug, permissionCache]);

  // Convenience methods
  const hasPermission = useCallback((permission: string) => {
    return checkPermission(permission).allowed;
  }, [checkPermission]);

  const hasAnyPermission = useCallback((requiredPermissions: string[]) => {
    return requiredPermissions.some(permission => hasPermission(permission));
  }, [hasPermission]);

  const hasAllPermissions = useCallback((requiredPermissions: string[]) => {
    return requiredPermissions.every(permission => hasPermission(permission));
  }, [hasPermission]);

  // Role checking methods
  const hasRole = useCallback((role: string) => {
    return roles.includes(role);
  }, [roles]);

  const hasAnyRole = useCallback((requiredRoles: string[]) => {
    return requiredRoles.some(role => roles.includes(role));
  }, [roles]);

  const hasAllRoles = useCallback((requiredRoles: string[]) => {
    return requiredRoles.every(role => roles.includes(role));
  }, [roles]);

  // Profile checking methods
  const hasProfile = useCallback((profile: string) => {
    return profiles.includes(profile);
  }, [profiles]);

  const hasAnyProfile = useCallback((requiredProfiles: string[]) => {
    return requiredProfiles.some(profile => profiles.includes(profile));
  }, [profiles]);

  // Route access checking
  const canAccessRoute = useCallback((
    routeConfig: {
      permissions?: string[];
      roles?: string[];
      profiles?: string[];
      requireAuth?: boolean;
    } = {}
  ) => {
    const { 
      permissions: routePermissions, 
      roles: routeRoles, 
      profiles: routeProfiles,
      requireAuth = true 
    } = routeConfig;

    // Check authentication requirement
    if (requireAuth && !user) {
      return {
        allowed: false,
        reason: 'Authentication required',
      };
    }

    // Check permissions
    if (routePermissions?.length) {
      const hasRequiredPermissions = hasAnyPermission(routePermissions);
      if (!hasRequiredPermissions) {
        return {
          allowed: false,
          reason: `Missing required permissions: ${routePermissions.join(', ')}`,
        };
      }
    }

    // Check roles
    if (routeRoles?.length) {
      const hasRequiredRoles = hasAnyRole(routeRoles);
      if (!hasRequiredRoles) {
        return {
          allowed: false,
          reason: `Missing required roles: ${routeRoles.join(', ')}`,
        };
      }
    }

    // Check profiles
    if (routeProfiles?.length) {
      const hasRequiredProfiles = hasAnyProfile(routeProfiles);
      if (!hasRequiredProfiles) {
        return {
          allowed: false,
          reason: `Missing required profiles: ${routeProfiles.join(', ')}`,
        };
      }
    }

    return {
      allowed: true,
      reason: 'Route access granted',
    };
  }, [user, hasAnyPermission, hasAnyRole, hasAnyProfile]);

  // Clear permission cache
  const clearCache = useCallback(() => {
    if (enableCaching) {
      permissionCache.clear();
    }
  }, [enableCaching, permissionCache]);

  return {
    // Core permission methods
    checkPermission,
    hasPermission,
    hasAnyPermission,
    hasAllPermissions,
    
    // Role methods
    hasRole,
    hasAnyRole,
    hasAllRoles,
    
    // Profile methods
    hasProfile,
    hasAnyProfile,
    
    // Route access
    canAccessRoute,
    
    // Cache management
    clearCache,
    
    // State
    permissions,
    roles,
    profiles,
    isAuthenticated: !!user,
  };
}