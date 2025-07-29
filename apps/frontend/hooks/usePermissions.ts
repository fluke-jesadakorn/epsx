import { useState, useEffect, useCallback } from 'react';
import { logger } from '@/lib/logger';
import { usePermissionContext } from '@epsx/server-providers';

interface PermissionHookState {
  permissions: string[];
  profiles: string[];
  role: string;
  loading: boolean;
  error: string | null;
  hasWildcardAccess: boolean;
}

interface PermissionCheckResult {
  allowed: boolean;
  reason?: string;
  loading: boolean;
}

/**
 * React hook for permission-based component rendering and access control
 */
export function usePermissions() {
  const { permissions: serverPermissions, error: contextError } = usePermissionContext();
  const [state, setState] = useState<PermissionHookState>({
    permissions: [],
    profiles: [],
    role: 'user',
    loading: true,
    error: null,
    hasWildcardAccess: false,
  });

  const [cache, setCache] = useState<Map<string, { result: boolean; timestamp: number }>>(new Map());
  const CACHE_TTL = 2 * 60 * 1000; // 2 minutes

  // Load user permissions from context
  useEffect(() => {
    if (contextError) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: contextError
      }));
      return;
    }

    if (serverPermissions) {
      try {
        const permissionNames = Array.isArray(serverPermissions) 
          ? serverPermissions.map(p => p.name || p.permission || p)
          : [];
        const profileNames = []; // Will be enhanced when profiles are implemented
        const hasWildcard = permissionNames.some(p => p === '*' || p.endsWith(':*') || p.endsWith('.*'));
        
        setState({
          permissions: permissionNames,
          profiles: profileNames,
          role: 'user', // Default role, could be enhanced with server data
          loading: false,
          error: null,
          hasWildcardAccess: hasWildcard,
        });
        
        logger.debug('User permissions loaded from context', {
          permissionCount: permissionNames.length,
          profileCount: profileNames.length,
          role: 'user'
        });
      } catch (error) {
        logger.error('Error processing permissions from context', { error: error instanceof Error ? error.message : error });
        setState(prev => ({
          ...prev,
          loading: false,
          error: error instanceof Error ? error.message : 'Unknown error'
        }));
      }
    }
  }, [serverPermissions, contextError]);

  /**
   * Check if user has a specific permission
   */
  const hasPermission = useCallback((permission: string): PermissionCheckResult => {
    if (state.loading) {
      return { allowed: false, loading: true };
    }

    if (state.error) {
      return { allowed: false, loading: false, reason: state.error };
    }

    // Check cache first
    const cacheKey = `perm_${permission}`;
    const cached = cache.get(cacheKey);
    if (cached && (Date.now() - cached.timestamp) < CACHE_TTL) {
      return { allowed: cached.result, loading: false };
    }

    // Check for wildcard access
    if (state.hasWildcardAccess || state.permissions.includes('*')) {
      const result = { allowed: true, loading: false, reason: 'Wildcard access' };
      setCache(prev => new Map(prev.set(cacheKey, { result: true, timestamp: Date.now() })));
      return result;
    }

    // Check exact permission
    if (state.permissions.includes(permission)) {
      const result = { allowed: true, loading: false, reason: 'Direct permission' };
      setCache(prev => new Map(prev.set(cacheKey, { result: true, timestamp: Date.now() })));
      return result;
    }

    // Check wildcard permissions
    const hasWildcard = state.permissions.some(userPerm => {
      if (userPerm.endsWith(':*') || userPerm.endsWith('.*')) {
        const prefix = userPerm.slice(0, -2);
        return permission.startsWith(prefix + ':') || permission.startsWith(prefix + '.');
      }
      return false;
    });

    if (hasWildcard) {
      const result = { allowed: true, loading: false, reason: 'Wildcard permission match' };
      setCache(prev => new Map(prev.set(cacheKey, { result: true, timestamp: Date.now() })));
      return result;
    }

    const result = { allowed: false, loading: false, reason: `Missing permission: ${permission}` };
    setCache(prev => new Map(prev.set(cacheKey, { result: false, timestamp: Date.now() })));
    return result;
  }, [state, cache]);

  /**
   * Check if user has any of the specified permissions
   */
  const hasAnyPermission = useCallback((permissions: string[]): PermissionCheckResult => {
    if (state.loading) {
      return { allowed: false, loading: true };
    }

    for (const permission of permissions) {
      const result = hasPermission(permission);
      if (result.allowed) {
        return { allowed: true, loading: false, reason: `Permission granted: ${permission}` };
      }
    }

    return { 
      allowed: false, 
      loading: false, 
      reason: `Missing all required permissions: ${permissions.join(', ')}` 
    };
  }, [hasPermission, state.loading]);

  /**
   * Check if user has all of the specified permissions
   */
  const hasAllPermissions = useCallback((permissions: string[]): PermissionCheckResult => {
    if (state.loading) {
      return { allowed: false, loading: true };
    }

    for (const permission of permissions) {
      const result = hasPermission(permission);
      if (!result.allowed) {
        return { 
          allowed: false, 
          loading: false, 
          reason: `Missing required permission: ${permission}` 
        };
      }
    }

    return { allowed: true, loading: false, reason: 'All permissions granted' };
  }, [hasPermission, state.loading]);

  /**
   * Check if user has a specific permission profile
   */
  const hasProfile = useCallback((profileName: string): PermissionCheckResult => {
    if (state.loading) {
      return { allowed: false, loading: true };
    }

    const hasProfileAccess = state.profiles.includes(profileName);
    return {
      allowed: hasProfileAccess,
      loading: false,
      reason: hasProfileAccess ? `Profile access: ${profileName}` : `Missing profile: ${profileName}`
    };
  }, [state]);

  /**
   * Check role-based access with hierarchy
   */
  const hasRole = useCallback((requiredRole: string): PermissionCheckResult => {
    if (state.loading) {
      return { allowed: false, loading: true };
    }

    const roleHierarchy: Record<string, number> = {
      'user': 1,
      'premium': 2,
      'moderator': 3,
      'admin': 4,
      'super_admin': 5
    };

    const userLevel = roleHierarchy[state.role.toLowerCase()] || 0;
    const requiredLevel = roleHierarchy[requiredRole.toLowerCase()] || 1;

    const hasAccess = userLevel >= requiredLevel;
    return {
      allowed: hasAccess,
      loading: false,
      reason: hasAccess 
        ? `Role hierarchy access: ${state.role} >= ${requiredRole}` 
        : `Insufficient role: ${state.role} < ${requiredRole}`
    };
  }, [state]);

  /**
   * Check access to a specific route
   */
  const canAccessRoute = useCallback((route: string): PermissionCheckResult => {
    if (state.loading) {
      return { allowed: false, loading: true };
    }

    // Basic route permissions mapping (simplified)
    const routePermissions: Record<string, string> = {
      '/dashboard': 'route:/dashboard',
      '/analytics': 'route:/analytics/*',
      '/premium': 'route:/premium/*',
      '/admin': 'route:/admin/*',
      '/reports': 'route:/reports/*',
      '/trading': 'route:/trading/*'
    };

    const requiredPermission = routePermissions[route];
    if (!requiredPermission) {
      return { allowed: true, loading: false, reason: 'Public route' };
    }

    return hasPermission(requiredPermission);
  }, [hasPermission, state.loading]);

  /**
   * Refresh permissions from server
   */
  const refreshPermissions = useCallback(async () => {
    setCache(new Map()); // Clear cache
    // Note: Refreshing now requires parent component to refetch server data
    // This maintains the server/client separation
  }, []);

  return {
    // State
    permissions: state.permissions,
    profiles: state.profiles,
    role: state.role,
    loading: state.loading,
    error: state.error,
    hasWildcardAccess: state.hasWildcardAccess,

    // Permission checks
    hasPermission,
    hasAnyPermission,
    hasAllPermissions,
    hasProfile,
    hasRole,
    canAccessRoute,

    // Actions
    refreshPermissions,
  };
}

/**
 * Higher-order component for permission-based rendering
 */
export interface PermissionGuardProps {
  permission?: string;
  permissions?: string[];
  profile?: string;
  role?: string;
  requireAll?: boolean;
  fallback?: React.ReactNode;
  children: React.ReactNode;
}

export function PermissionGuard({
  permission,
  permissions,
  profile,
  role,
  requireAll = false,
  fallback = null,
  children
}: PermissionGuardProps) {
  const {
    hasPermission,
    hasAnyPermission,
    hasAllPermissions,
    hasProfile,
    hasRole,
    loading
  } = usePermissions();

  if (loading) {
    return <div>Loading permissions...</div>;
  }

  // Check permission profile
  if (profile) {
    const profileCheck = hasProfile(profile);
    if (!profileCheck.allowed) {
      return <>{fallback}</>;
    }
  }

  // Check role
  if (role) {
    const roleCheck = hasRole(role);
    if (!roleCheck.allowed) {
      return <>{fallback}</>;
    }
  }

  // Check single permission
  if (permission) {
    const permCheck = hasPermission(permission);
    if (!permCheck.allowed) {
      return <>{fallback}</>;
    }
  }

  // Check multiple permissions
  if (permissions && permissions.length > 0) {
    const permCheck = requireAll 
      ? hasAllPermissions(permissions)
      : hasAnyPermission(permissions);
    
    if (!permCheck.allowed) {
      return <>{fallback}</>;
    }
  }

  return <>{children}</>;
}