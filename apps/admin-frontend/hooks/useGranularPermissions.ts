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
  AdminPermissionHookResult,
  GrantPermissionRequest,
  RevokePermissionRequest,
  BulkPermissionRequest,
  ExtendPermissionRequest,
  BulkOperationResult,
  PermissionAuditEntry,
  AdminPermissionDashboard,
  UserPermissionOverview,
  PermissionSearchFilters,
  PermissionTemplate
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
// API CLIENT FUNCTIONS
// ============================================================================

const getAuthToken = () => localStorage.getItem('access_token');

const apiCall = async (endpoint: string, options: RequestInit = {}) => {
  const token = getAuthToken();
  
  const response = await fetch(`/api/v1/admin${endpoint}`, {
    ...options,
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
      ...options.headers
    }
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Unknown error' }));
    throw new Error(error.message || `API call failed: ${response.status}`);
  }

  return response.json();
};

// ============================================================================
// MAIN ADMIN PERMISSION HOOK
// ============================================================================

export function useAdminGranularPermissions(): AdminPermissionHookResult {
  const { user, isAuthenticated } = useAuth.getState();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<any>(null);

  // Cast user to EnhancedUserClaims if available
  const enhancedUser = user as EnhancedUserClaims | null;
  const permissions = enhancedUser?.permissions || {};

  // Check if user has admin permissions
  const isAdmin = hasPermissionGranular(permissions, 'admin:*:*');
  const canManagePermissions = hasPermissionGranular(permissions, 'admin:permissions:manage') || isAdmin;

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

  // Admin API functions
  const getUserPermissions = useCallback(async (userId: string): Promise<PermissionStatusResponse> => {
    setLoading(true);
    try {
      const response = await apiCall(`/permissions/users/${userId}`);
      return response;
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const getAllUsersWithPermissions = useCallback(async (filters?: PermissionSearchFilters): Promise<UserPermissionOverview[]> => {
    setLoading(true);
    try {
      const query = filters ? `?${new URLSearchParams(filters as any).toString()}` : '';
      const response = await apiCall(`/permissions/users${query}`);
      return response;
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const grantPermission = useCallback(async (request: GrantPermissionRequest): Promise<void> => {
    if (!canManagePermissions) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED');
    }

    setLoading(true);
    try {
      await apiCall('/permissions/grant', {
        method: 'POST',
        body: JSON.stringify(request)
      });
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [canManagePermissions]);

  const revokePermission = useCallback(async (request: RevokePermissionRequest): Promise<void> => {
    if (!canManagePermissions) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED');
    }

    setLoading(true);
    try {
      await apiCall('/permissions/revoke', {
        method: 'POST',
        body: JSON.stringify(request)
      });
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [canManagePermissions]);

  const bulkGrantPermissions = useCallback(async (request: BulkPermissionRequest): Promise<BulkOperationResult> => {
    if (!canManagePermissions) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED');
    }

    setLoading(true);
    try {
      const response = await apiCall('/permissions/bulk/grant', {
        method: 'POST',
        body: JSON.stringify(request)
      });
      return response;
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [canManagePermissions]);

  const bulkRevokePermissions = useCallback(async (
    request: Omit<BulkPermissionRequest, 'expires_at' | 'source'>
  ): Promise<BulkOperationResult> => {
    if (!canManagePermissions) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED');
    }

    setLoading(true);
    try {
      const response = await apiCall('/permissions/bulk/revoke', {
        method: 'POST',
        body: JSON.stringify(request)
      });
      return response;
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [canManagePermissions]);

  const extendPermission = useCallback(async (request: ExtendPermissionRequest): Promise<void> => {
    if (!canManagePermissions) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED');
    }

    setLoading(true);
    try {
      await apiCall('/permissions/extend', {
        method: 'POST',
        body: JSON.stringify(request)
      });
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [canManagePermissions]);

  const createPermissionTemplate = useCallback(async (
    template: Omit<PermissionTemplate, 'id' | 'created_at' | 'created_by'>
  ): Promise<PermissionTemplate> => {
    if (!canManagePermissions) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED');
    }

    setLoading(true);
    try {
      const response = await apiCall('/permissions/templates', {
        method: 'POST',
        body: JSON.stringify(template)
      });
      return response;
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [canManagePermissions]);

  const applyPermissionTemplate = useCallback(async (
    templateId: string, 
    userIds: string[]
  ): Promise<BulkOperationResult> => {
    if (!canManagePermissions) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED');
    }

    setLoading(true);
    try {
      const response = await apiCall(`/permissions/templates/${templateId}/apply`, {
        method: 'POST',
        body: JSON.stringify({ user_ids: userIds })
      });
      return response;
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [canManagePermissions]);

  const getDashboard = useCallback(async (): Promise<AdminPermissionDashboard> => {
    if (!isAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED');
    }

    setLoading(true);
    try {
      const response = await apiCall('/permissions/dashboard');
      return response;
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [isAdmin]);

  const getPermissionAudit = useCallback(async (
    userId?: string, 
    limit: number = 100
  ): Promise<PermissionAuditEntry[]> => {
    if (!hasPermission('admin:audit:read')) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED');
    }

    setLoading(true);
    try {
      const query = new URLSearchParams({ limit: limit.toString() });
      if (userId) query.append('user_id', userId);
      
      const response = await apiCall(`/permissions/audit?${query.toString()}`);
      return response;
    } catch (err) {
      setError(err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [hasPermission]);

  return {
    // User permission queries
    getUserPermissions,
    getAllUsersWithPermissions,
    
    // Permission management
    grantPermission,
    revokePermission,
    bulkGrantPermissions,
    bulkRevokePermissions,
    extendPermission,
    
    // Templates
    createPermissionTemplate,
    applyPermissionTemplate,
    
    // Monitoring
    getDashboard,
    getPermissionAudit,
    
    // State
    loading,
    error: error ? {
      code: 'ADMIN_REQUIRED',
      message: error.message,
      details: error.toString()
    } : null
  };
}

// ============================================================================
// SPECIALIZED ADMIN HOOKS
// ============================================================================

export function useAdminPermissionDashboard() {
  const adminPermissions = useAdminGranularPermissions();
  const [dashboard, setDashboard] = useState<AdminPermissionDashboard | null>(null);

  useEffect(() => {
    const loadDashboard = async () => {
      try {
        const data = await adminPermissions.getDashboard();
        setDashboard(data);
      } catch (err) {
        console.error('Failed to load admin dashboard:', err);
      }
    };

    loadDashboard();
  }, []);

  return {
    dashboard,
    refreshDashboard: () => adminPermissions.getDashboard().then(setDashboard),
    ...adminPermissions
  };
}

export function useUserPermissionManagement(userId: string) {
  const adminPermissions = useAdminGranularPermissions();
  const [userPermissions, setUserPermissions] = useState<PermissionStatusResponse | null>(null);

  useEffect(() => {
    const loadUserPermissions = async () => {
      try {
        const data = await adminPermissions.getUserPermissions(userId);
        setUserPermissions(data);
      } catch (err) {
        console.error('Failed to load user permissions:', err);
      }
    };

    if (userId) {
      loadUserPermissions();
    }
  }, [userId]);

  const refreshUserPermissions = useCallback(async () => {
    const data = await adminPermissions.getUserPermissions(userId);
    setUserPermissions(data);
    return data;
  }, [userId, adminPermissions]);

  return {
    userPermissions,
    refreshUserPermissions,
    ...adminPermissions
  };
}

// Export the main hook as default
export { useAdminGranularPermissions as default };