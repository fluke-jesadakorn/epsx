'use client'

import { useAuth } from '@/lib/auth'
import { useCallback, useState, useEffect } from 'react'
import { 
  EnhancedUserClaims, 
  GranularPermissionClaim,
  GranularPermissionSet,
  PermissionStatusResponse,
  PermissionHealthInfo,
  PermissionExpiryDetails,
  TokenValidationResult,
  HashValidationResult,
  GranularPermissionError,
  UsePermissionHookResult
} from '@/types/granular-permissions'

// ============================================================================
// GRANULAR PERMISSION UTILITIES
// ============================================================================

/**
 * Parse permission string into components
 */
const parsePermission = (permission: string) => {
  const parts = permission.split(':');
  if (parts.length < 3) return null;
  
  return {
    platform: parts[0],
    resource: parts[1], 
    action: parts[2],
    full: permission
  };
};

/**
 * Check if permission claim is still valid (not expired)
 */
const isClaimValid = (claim: GranularPermissionClaim): boolean => {
  if (!claim.expires_at) return true; // Permanent permission
  const now = Math.floor(Date.now() / 1000);
  return claim.expires_at > now;
};

/**
 * Filter valid permissions from user claims
 */
const getValidPermissions = (permissions: Record<string, GranularPermissionClaim>): string[] => {
  return Object.entries(permissions)
    .filter(([_, claim]) => isClaimValid(claim))
    .map(([permission, _]) => permission);
};

/**
 * Check if user has a specific permission
 */
const hasPermissionGranular = (
  permissions: Record<string, GranularPermissionClaim>,
  requiredPermission: string
): boolean => {
  // Check exact match first
  if (permissions[requiredPermission] && isClaimValid(permissions[requiredPermission])) {
    return true;
  }

  const required = parsePermission(requiredPermission);
  if (!required) return false;

  // Check wildcard permissions
  for (const [perm, claim] of Object.entries(permissions)) {
    if (!isClaimValid(claim)) continue;

    const userPerm = parsePermission(perm);
    if (!userPerm) continue;

    // Check for exact match
    if (userPerm.platform === required.platform && 
        userPerm.resource === required.resource && 
        userPerm.action === required.action) {
      return true;
    }

    // Check for wildcard matches
    if (userPerm.platform === required.platform) {
      // Platform-level wildcard: "epsx:*:*"
      if (userPerm.resource === '*' && userPerm.action === '*') {
        return true;
      }
      
      // Resource-level wildcard: "epsx:analytics:*"
      if (userPerm.resource === required.resource && userPerm.action === '*') {
        return true;
      }
    }

    // Global admin permission: "admin:*:*"
    if (userPerm.platform === 'admin' && userPerm.resource === '*' && userPerm.action === '*') {
      return true;
    }
  }

  return false;
};

/**
 * Get permission expiry details
 */
const getPermissionExpiryDetails = (
  permissions: Record<string, GranularPermissionClaim>,
  permission: string
): PermissionExpiryDetails | null => {
  const claim = permissions[permission];
  if (!claim) return null;

  const now = Date.now();
  const isExpired = claim.expires_at ? (claim.expires_at * 1000) <= now : false;
  const expiresInMs = claim.expires_at ? (claim.expires_at * 1000) - now : undefined;
  
  let expiresInHuman: string | undefined;
  if (expiresInMs && expiresInMs > 0) {
    const hours = Math.floor(expiresInMs / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);
    
    if (days > 0) {
      expiresInHuman = `${days} day${days > 1 ? 's' : ''}`;
    } else if (hours > 0) {
      expiresInHuman = `${hours} hour${hours > 1 ? 's' : ''}`;
    } else {
      const minutes = Math.floor(expiresInMs / (1000 * 60));
      expiresInHuman = `${minutes} minute${minutes > 1 ? 's' : ''}`;
    }
  }

  return {
    permission,
    base_permission: permission,
    claim,
    is_expired: isExpired,
    expires_in_ms: expiresInMs,
    expires_in_human: expiresInHuman,
    is_permanent: !claim.expires_at
  };
};

/**
 * Calculate permission health info
 */
const calculatePermissionHealth = (permissions: Record<string, GranularPermissionClaim>): PermissionHealthInfo => {
  const now = Date.now();
  const twentyFourHoursFromNow = now + (24 * 60 * 60 * 1000);

  const allPermissions = Object.entries(permissions);
  const activePermissions = allPermissions.filter(([_, claim]) => isClaimValid(claim));
  const expiredPermissions = allPermissions.filter(([_, claim]) => !isClaimValid(claim));
  const expiringSoonPermissions = activePermissions.filter(([_, claim]) => 
    claim.expires_at && (claim.expires_at * 1000) <= twentyFourHoursFromNow
  );

  const nextExpiry = activePermissions
    .filter(([_, claim]) => claim.expires_at)
    .sort(([_, a], [__, b]) => (a.expires_at || 0) - (b.expires_at || 0))[0];

  const timeUntilNextExpiry = nextExpiry?.[1].expires_at ? 
    (nextExpiry[1].expires_at * 1000) - now : undefined;

  // Calculate health score (0-100)
  const totalPermissions = allPermissions.length;
  const activeCount = activePermissions.length;
  const expiringSoonCount = expiringSoonPermissions.length;
  
  let healthScore = 100;
  if (totalPermissions > 0) {
    const activeRatio = activeCount / totalPermissions;
    const expiringSoonRatio = expiringSoonCount / activeCount;
    
    healthScore = Math.floor(activeRatio * 100);
    if (expiringSoonRatio > 0.3) healthScore -= 20; // Penalty for many expiring permissions
    if (expiringSoonRatio > 0.5) healthScore -= 20; // Additional penalty
  }

  return {
    total_permissions: totalPermissions,
    active_permissions: activeCount,
    expired_permissions: expiredPermissions.length,
    expiring_soon_permissions: expiringSoonPermissions.length,
    next_expiry: nextExpiry?.[1].expires_at,
    time_until_next_expiry: timeUntilNextExpiry,
    health_score: Math.max(0, healthScore)
  };
};

// ============================================================================
// MAIN HOOK
// ============================================================================

export function useGranularPermissions(): UsePermissionHookResult {
  const { user, isAuthenticated } = useAuth.getState();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<any>(null);

  // Cast user to EnhancedUserClaims if available
  const enhancedUser = user as EnhancedUserClaims | null;
  const permissions = enhancedUser?.permissions || {};

  // Core permission checking functions
  const hasPermission = useCallback((permission: string): boolean => {
    if (!enhancedUser || !isAuthenticated) return false;
    return hasPermissionGranular(permissions, permission);
  }, [enhancedUser, permissions, isAuthenticated]);

  const hasAnyPermission = useCallback((permissionList: string[]): boolean => {
    if (!enhancedUser || !isAuthenticated) return false;
    return permissionList.some(permission => hasPermissionGranular(permissions, permission));
  }, [enhancedUser, permissions, isAuthenticated]);

  const hasAllPermissions = useCallback((permissionList: string[]): boolean => {
    if (!enhancedUser || !isAuthenticated) return false;
    return permissionList.every(permission => hasPermissionGranular(permissions, permission));
  }, [enhancedUser, permissions, isAuthenticated]);

  const getPermissionExpiry = useCallback((permission: string): PermissionExpiryDetails | null => {
    if (!enhancedUser) return null;
    return getPermissionExpiryDetails(permissions, permission);
  }, [enhancedUser, permissions]);

  const getPermissionHealth = useCallback((): PermissionHealthInfo | null => {
    if (!enhancedUser) return null;
    return calculatePermissionHealth(permissions);
  }, [enhancedUser, permissions]);

  const isPermissionExpiring = useCallback((permission: string, withinHours: number = 24): boolean => {
    const expiry = getPermissionExpiry(permission);
    if (!expiry || expiry.is_permanent) return false;
    
    const withinMs = withinHours * 60 * 60 * 1000;
    return expiry.expires_in_ms !== undefined && expiry.expires_in_ms <= withinMs;
  }, [getPermissionExpiry]);

  const refreshPermissions = useCallback(async (): Promise<void> => {
    if (!enhancedUser) return;

    setLoading(true);
    setError(null);

    try {
      // In a real implementation, this would call the backend API
      // For now, this is a placeholder that would refresh the user's token
      // and get updated permissions
      
      const response = await fetch('/api/v1/permissions/status', {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
          'Content-Type': 'application/json'
        }
      });

      if (!response.ok) {
        throw new Error('Failed to refresh permissions');
      }

      const permissionStatus: PermissionStatusResponse = await response.json();
      
      // Update user claims in auth state
      // This would need to be implemented in the auth system
      console.log('Refreshed permissions:', permissionStatus);
      
    } catch (err) {
      setError(err);
      console.error('Failed to refresh permissions:', err);
    } finally {
      setLoading(false);
    }
  }, [enhancedUser]);

  // Permission health and expiry info
  const permissionHealth = getPermissionHealth();
  const validPermissions = getValidPermissions(permissions);

  return {
    hasPermission,
    hasAnyPermission, 
    hasAllPermissions,
    getPermissionExpiry,
    getPermissionHealth,
    isPermissionExpiring,
    refreshPermissions,
    loading,
    error: error ? {
      code: 'VALIDATION_ERROR',
      message: error.message,
      details: error.toString()
    } : null
  };
}

// ============================================================================
// SPECIALIZED PERMISSION HOOKS
// ============================================================================

export function useAdminPermissions() {
  const permissions = useGranularPermissions();

  return {
    ...permissions,
    isAdmin: permissions.hasPermission('admin:*:*'),
    canManageUsers: permissions.hasAnyPermission(['admin:users:manage', 'epsx:users:manage']),
    canViewUsers: permissions.hasAnyPermission(['admin:users:read', 'admin:users:manage']),
    canManagePermissions: permissions.hasPermission('admin:permissions:manage'),
    canViewAuditLogs: permissions.hasAnyPermission(['admin:audit:read', 'admin:*:*']),
    canManageSystem: permissions.hasPermission('admin:system:manage')
  };
}

export function useAnalyticsPermissions() {
  const permissions = useGranularPermissions();

  return {
    ...permissions,
    canViewAnalytics: permissions.hasPermission('epsx:analytics:view'),
    canExportData: permissions.hasPermission('epsx:analytics:export'),
    canAccessRealtime: permissions.hasPermission('epsx:realtime:access'),
    canUseAdvancedFilters: permissions.hasPermission('epsx:analytics:advanced'),
    canManageAnalytics: permissions.hasPermission('epsx:analytics:manage')
  };
}

export function useProfilePermissions() {
  const permissions = useGranularPermissions();

  return {
    ...permissions,
    canManageProfile: permissions.hasPermission('epsx:profile:manage'),
    canViewProfile: permissions.hasPermission('epsx:profile:view'),
    canManageBilling: permissions.hasPermission('epsx:billing:manage'),
    canReceiveNotifications: permissions.hasPermission('epsx:notifications:receive')
  };
}

// ============================================================================
// PERMISSION VALIDATION HOOKS
// ============================================================================

export function useRequirePermission(permission: string) {
  const { hasPermission, loading, error } = useGranularPermissions();
  
  return {
    hasPermission: hasPermission(permission),
    loading,
    error
  };
}

export function useRequireAnyPermission(permissionList: string[]) {
  const { hasAnyPermission, loading, error } = useGranularPermissions();
  
  return {
    hasPermission: hasAnyPermission(permissionList),
    loading,
    error
  };
}

export function useRequireAdmin() {
  const { hasPermission, loading, error } = useGranularPermissions();
  
  return {
    hasPermission: hasPermission('admin:*:*'),
    loading,
    error
  };
}

// Export the main hook as default
export { useGranularPermissions as default };