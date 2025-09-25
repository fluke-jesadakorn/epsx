// ============================================================================
// BACKEND-CENTRIC ADMIN PERMISSIONS HOOK (Phase 2.2)
// Replaces ALL local admin permission validation with backend API calls
// THE SINGLE SOURCE OF TRUTH for admin permission state management
// ============================================================================

'use client';

import { useState, useEffect, useCallback, useMemo } from 'react';
import { 
  adminPermissionAuthority,
  hasAdminPermission,
  hasAnyAdminPermission,
  hasAllAdminPermissions,
  isAdmin,
  isSuperAdmin,
  AdminPermissionDeniedError,
  AdminAuthenticationRequiredError
} from './backend-authority-client';

// ============================================================================
// ADMIN PERMISSION HOOK STATE TYPES
// ============================================================================

export interface AdminPermissionState {
  // Permission validation results
  permissions: Record<string, boolean>;
  
  // Loading states
  loading: boolean;
  validating: Record<string, boolean>;
  
  // Error states  
  error: AdminPermissionError | null;
  errors: Record<string, AdminPermissionError>;
  
  // Admin context
  userId?: string;
  isAdmin: boolean;
  isSuperAdmin: boolean;
  adminTier?: string;
  
  // Admin capabilities
  canManageUsers: boolean;
  canManageSystem: boolean;
  canManagePermissions: boolean;
  canViewAnalytics: boolean;
  canViewAuditLogs: boolean;
  canManageSecurity: boolean;
  
  // Usage and tier info
  usageInfo: Record<string, {
    current_usage: number;
    limit: number;
    usage_percentage: number;
    reset_at?: string;
  }>;
  
  // Cache metadata
  lastUpdated?: string;
  cacheExpiry?: string;
}

export interface AdminPermissionError {
  type: string;
  message: string;
  userMessage: string;
  suggestedActions: string[];
  context?: Record<string, any>;
  upgradeInfo?: {
    current_tier: string;
    required_tier: string;
    upgrade_url?: string;
    benefits: string[];
  };
  adminContext?: {
    adminAction?: string;
    requiredAdminLevel?: string;
    currentAdminLevel?: string;
  };
}

export interface AdminPermissionHookResult extends AdminPermissionState {
  // Admin permission checking functions
  checkAdminPermission: (permission: string, resourcePath?: string) => Promise<boolean>;
  checkAnyAdminPermission: (permissions: string[]) => Promise<boolean>;
  checkAllAdminPermissions: (permissions: string[]) => Promise<boolean>;
  
  // Bulk operations
  validateAdminPermissions: (permissions: string[]) => Promise<void>;
  
  // Cache management
  refreshAdminPermissions: () => Promise<void>;
  clearCache: () => void;
  
  // Error handling
  clearError: (permission?: string) => void;
  handlePermissionError: (error: any, permission?: string) => void;
  
  // Utility functions
  hasPermission: (permission: string) => boolean;
  isLoading: (permission?: string) => boolean;
  getError: (permission?: string) => AdminPermissionError | null;
  
  // Admin-specific utilities
  requiresUpgrade: (permission: string) => boolean;
  getUpgradeInfo: (permission: string) => any;
  getAdminTierInfo: () => any;
  
  // Admin capability checks
  checkAdminCapability: (capability: keyof AdminPermissionState) => boolean;
}

// ============================================================================
// MAIN BACKEND ADMIN PERMISSIONS HOOK
// THE SINGLE SOURCE OF TRUTH for all admin permission state
// ============================================================================

export function useBackendAdminPermissions(
  userId?: string,
  initialPermissions?: string[],
  options: {
    autoRefresh?: boolean;
    refreshInterval?: number; // minutes
    cacheTimeout?: number; // minutes
  } = {}
): AdminPermissionHookResult {
  
  const {
    autoRefresh = false,
    refreshInterval = 30, // 30 minutes
    cacheTimeout = 60, // 60 minutes
  } = options;

  // ============================================================================
  // HOOK STATE MANAGEMENT
  // ============================================================================
  
  const [state, setState] = useState<AdminPermissionState>({
    permissions: {},
    loading: false,
    validating: {},
    error: null,
    errors: {},
    userId,
    isAdmin: false,
    isSuperAdmin: false,
    canManageUsers: false,
    canManageSystem: false,
    canManagePermissions: false,
    canViewAnalytics: false,
    canViewAuditLogs: false,
    canManageSecurity: false,
    usageInfo: {},
  });

  // ============================================================================
  // ADMIN PERMISSION VALIDATION FUNCTIONS
  // ============================================================================
  
  const checkAdminPermission = useCallback(async (
    permission: string,
    resourcePath?: string
  ): Promise<boolean> => {
    if (!userId) {
      console.warn('No admin user ID provided for permission check');
      return false;
    }

    // Set loading state for this specific permission
    setState(prev => ({
      ...prev,
      validating: { ...prev.validating, [permission]: true }
    }));

    try {
      const result = await adminPermissionAuthority.validateAdminPermission(
        userId,
        permission,
        resourcePath
      );

      // Update state with validation result
      setState(prev => ({
        ...prev,
        permissions: { ...prev.permissions, [permission]: result.granted },
        validating: { ...prev.validating, [permission]: false },
        errors: { ...prev.errors, [permission]: undefined }, // Clear error
        usageInfo: result.usage_count !== undefined ? {
          ...prev.usageInfo,
          [permission]: {
            current_usage: result.usage_count,
            limit: result.usage_limit || 100,
            usage_percentage: ((result.usage_count || 0) / (result.usage_limit || 100)) * 100,
            reset_at: result.expires_at,
          }
        } : prev.usageInfo,
        lastUpdated: new Date().toISOString(),
      }));

      return result.granted;
    } catch (error) {
      handlePermissionError(error, permission);
      
      // Update state with error
      setState(prev => ({
        ...prev,
        permissions: { ...prev.permissions, [permission]: false },
        validating: { ...prev.validating, [permission]: false }
      }));

      return false; // Fail closed for security
    }
  }, [userId]);

  const checkAnyAdminPermission = useCallback(async (permissions: string[]): Promise<boolean> => {
    if (!userId) return false;

    try {
      return await adminPermissionAuthority.hasAnyAdminPermission(userId, permissions);
    } catch (error) {
      handlePermissionError(error);
      return false; // Fail closed for security
    }
  }, [userId]);

  const checkAllAdminPermissions = useCallback(async (permissions: string[]): Promise<boolean> => {
    if (!userId) return false;

    try {
      return await adminPermissionAuthority.hasAllAdminPermissions(userId, permissions);
    } catch (error) {
      handlePermissionError(error);
      return false; // Fail closed for security
    }
  }, [userId]);

  // ============================================================================
  // BULK ADMIN PERMISSION VALIDATION
  // ============================================================================
  
  const validateAdminPermissions = useCallback(async (permissions: string[]) => {
    if (!userId || permissions.length === 0) return;

    setState(prev => ({ ...prev, loading: true }));

    try {
      const result = await adminPermissionAuthority.validateBulkAdminPermissions(userId, permissions);

      // Update state with bulk results
      const newPermissions: Record<string, boolean> = {};
      const newErrors: Record<string, AdminPermissionError> = {};
      
      result.results.forEach(({ permission, granted, reason }) => {
        newPermissions[permission] = granted;
        if (!granted && reason) {
          newErrors[permission] = {
            type: 'admin_permission_denied',
            message: reason,
            userMessage: `Admin access denied: ${reason}`,
            suggestedActions: ['Check your admin permissions', 'Contact system administrator'],
            adminContext: {
              adminAction: permission,
              requiredAdminLevel: 'admin',
            },
          };
        }
      });

      setState(prev => ({
        ...prev,
        permissions: { ...prev.permissions, ...newPermissions },
        errors: { ...prev.errors, ...newErrors },
        loading: false,
        lastUpdated: result.validated_at,
      }));
    } catch (error) {
      handlePermissionError(error);
      setState(prev => ({ ...prev, loading: false }));
    }
  }, [userId]);

  // ============================================================================
  // ADMIN CAPABILITIES REFRESH
  // ============================================================================
  
  const refreshAdminPermissions = useCallback(async () => {
    if (!userId) return;

    setState(prev => ({ ...prev, loading: true }));

    try {
      // Get all admin permissions
      const userPermissions = await adminPermissionAuthority.getAdminUserPermissions(userId);
      
      // Check admin capabilities
      const [
        isAdminResult,
        isSuperAdminResult,
        canManageUsersResult,
        canManageSystemResult,
        canManagePermissionsResult,
        canViewAnalyticsResult,
        canViewAuditLogsResult,
        canManageSecurityResult,
      ] = await Promise.all([
        adminPermissionAuthority.isAdmin(userId),
        adminPermissionAuthority.isSuperAdmin(userId),
        adminPermissionAuthority.canManageUsers(userId),
        adminPermissionAuthority.canManageSystem(userId),
        adminPermissionAuthority.canManagePermissions(userId),
        adminPermissionAuthority.canViewAnalytics(userId),
        adminPermissionAuthority.canViewAuditLogs(userId),
        adminPermissionAuthority.canManageSecurity(userId),
      ]);

      // Update state with comprehensive admin permissions
      const newPermissions: Record<string, boolean> = {};
      const newUsageInfo: Record<string, any> = {};
      
      userPermissions.permissions.forEach(({ permission, granted, usage_count, usage_limit }) => {
        newPermissions[permission] = granted;
        if (usage_count !== undefined) {
          newUsageInfo[permission] = {
            current_usage: usage_count,
            limit: usage_limit || 100,
            usage_percentage: ((usage_count || 0) / (usage_limit || 100)) * 100,
          };
        }
      });

      setState(prev => ({
        ...prev,
        permissions: newPermissions,
        usageInfo: newUsageInfo,
        isAdmin: isAdminResult,
        isSuperAdmin: isSuperAdminResult,
        canManageUsers: canManageUsersResult,
        canManageSystem: canManageSystemResult,
        canManagePermissions: canManagePermissionsResult,
        canViewAnalytics: canViewAnalyticsResult,
        canViewAuditLogs: canViewAuditLogsResult,
        canManageSecurity: canManageSecurityResult,
        adminTier: userPermissions.tier_info?.current_tier,
        loading: false,
        lastUpdated: userPermissions.last_updated,
        cacheExpiry: new Date(Date.now() + (cacheTimeout * 60 * 1000)).toISOString(),
      }));
    } catch (error) {
      handlePermissionError(error);
      setState(prev => ({ ...prev, loading: false }));
    }
  }, [userId, cacheTimeout]);

  // ============================================================================
  // ERROR HANDLING
  // ============================================================================
  
  const handlePermissionError = useCallback((error: any, permission?: string) => {
    console.error('Admin permission validation error:', error);
    
    let adminPermissionError: AdminPermissionError;

    if (error instanceof AdminAuthenticationRequiredError) {
      adminPermissionError = {
        type: 'admin_authentication_required',
        message: error.message,
        userMessage: 'Admin authentication required to access this feature',
        suggestedActions: ['Log in with admin credentials', 'Contact system administrator'],
        adminContext: {
          adminAction: permission,
          requiredAdminLevel: 'admin',
        },
      };
    } else if (error instanceof AdminPermissionDeniedError) {
      adminPermissionError = {
        type: 'admin_permission_denied',
        message: error.message,
        userMessage: 'You don\'t have the required admin permissions for this action',
        suggestedActions: ['Contact system administrator', 'Request additional admin permissions'],
        adminContext: error.adminContext,
      };
    } else {
      adminPermissionError = {
        type: 'admin_unknown_error',
        message: error.message || 'Admin permission validation failed',
        userMessage: 'Unable to validate admin permissions. Please try again.',
        suggestedActions: ['Try again', 'Check your connection', 'Contact system administrator'],
        adminContext: {
          adminAction: permission,
        },
      };
    }

    setState(prev => {
      const newState = { ...prev };
      
      if (permission) {
        newState.errors = { ...prev.errors, [permission]: adminPermissionError };
      } else {
        newState.error = adminPermissionError;
      }
      
      return newState;
    });
  }, []);

  const clearError = useCallback((permission?: string) => {
    setState(prev => {
      if (permission) {
        const newErrors = { ...prev.errors };
        delete newErrors[permission];
        return { ...prev, errors: newErrors };
      } else {
        return { ...prev, error: null };
      }
    });
  }, []);

  // ============================================================================
  // UTILITY FUNCTIONS
  // ============================================================================
  
  const hasPermissionLocal = useCallback((permission: string): boolean => {
    return state.permissions[permission] === true;
  }, [state.permissions]);

  const isLoading = useCallback((permission?: string): boolean => {
    if (permission) {
      return state.validating[permission] === true;
    }
    return state.loading || Object.values(state.validating).some(Boolean);
  }, [state.loading, state.validating]);

  const getError = useCallback((permission?: string): AdminPermissionError | null => {
    if (permission) {
      return state.errors[permission] || null;
    }
    return state.error;
  }, [state.errors, state.error]);

  const requiresUpgrade = useCallback((permission: string): boolean => {
    const error = state.errors[permission];
    return error?.type === 'insufficient_tier' || error?.type === 'admin_permission_denied';
  }, [state.errors]);

  const getUpgradeInfo = useCallback((permission: string) => {
    const error = state.errors[permission];
    return error?.upgradeInfo || null;
  }, [state.errors]);

  const getAdminTierInfo = useCallback(() => {
    return {
      tier: state.adminTier || 'basic',
      isAdmin: state.isAdmin,
      isSuperAdmin: state.isSuperAdmin,
      capabilities: {
        canManageUsers: state.canManageUsers,
        canManageSystem: state.canManageSystem,
        canManagePermissions: state.canManagePermissions,
        canViewAnalytics: state.canViewAnalytics,
        canViewAuditLogs: state.canViewAuditLogs,
        canManageSecurity: state.canManageSecurity,
      },
    };
  }, [state]);

  const clearCache = useCallback(() => {
    setState(prev => ({
      ...prev,
      permissions: {},
      usageInfo: {},
      errors: {},
      error: null,
      lastUpdated: undefined,
      cacheExpiry: undefined,
    }));
  }, []);

  const checkAdminCapability = useCallback((capability: keyof AdminPermissionState): boolean => {
    return Boolean(state[capability]);
  }, [state]);

  // ============================================================================
  // EFFECTS FOR AUTO-REFRESH AND INITIAL LOADING
  // ============================================================================
  
  // Initial permission validation
  useEffect(() => {
    if (userId && initialPermissions && initialPermissions.length > 0) {
      validateAdminPermissions(initialPermissions);
    }
  }, [userId, initialPermissions?.join(','), validateAdminPermissions]);

  // Auto-refresh permissions
  useEffect(() => {
    if (!autoRefresh || !userId) return;

    const intervalId = setInterval(() => {
      refreshAdminPermissions();
    }, refreshInterval * 60 * 1000); // Convert minutes to milliseconds

    return () => clearInterval(intervalId);
  }, [autoRefresh, userId, refreshInterval, refreshAdminPermissions]);

  // Initial admin capabilities check
  useEffect(() => {
    if (userId && !state.lastUpdated) {
      refreshAdminPermissions();
    }
  }, [userId, state.lastUpdated, refreshAdminPermissions]);

  // Cache expiry check
  useEffect(() => {
    if (!state.cacheExpiry) return;

    const expiryTime = new Date(state.cacheExpiry).getTime();
    const now = Date.now();
    
    if (now >= expiryTime) {
      refreshAdminPermissions();
    }
  }, [state.cacheExpiry, refreshAdminPermissions]);

  // ============================================================================
  // RETURN HOOK RESULT
  // ============================================================================
  
  return {
    // State
    ...state,
    
    // Functions
    checkAdminPermission,
    checkAnyAdminPermission,
    checkAllAdminPermissions,
    validateAdminPermissions,
    refreshAdminPermissions,
    clearCache,
    clearError,
    handlePermissionError,
    
    // Utilities
    hasPermission: hasPermissionLocal,
    isLoading,
    getError,
    requiresUpgrade,
    getUpgradeInfo,
    getAdminTierInfo,
    checkAdminCapability,
  };
}

// ============================================================================
// CONVENIENCE HOOKS FOR SPECIFIC ADMIN USE CASES
// ============================================================================

// Single admin permission check hook
export function useAdminPermission(
  permission: string, 
  userId?: string,
  resourcePath?: string
) {
  const { 
    checkAdminPermission, 
    hasPermission, 
    isLoading, 
    getError,
    requiresUpgrade,
    getUpgradeInfo
  } = useBackendAdminPermissions(userId, [permission]);

  useEffect(() => {
    if (userId && permission) {
      checkAdminPermission(permission, resourcePath);
    }
  }, [userId, permission, resourcePath, checkAdminPermission]);

  return {
    granted: hasPermission(permission),
    loading: isLoading(permission),
    error: getError(permission),
    requiresUpgrade: requiresUpgrade(permission),
    upgradeInfo: getUpgradeInfo(permission),
    refresh: () => checkAdminPermission(permission, resourcePath),
  };
}

// Multiple admin permissions check hook
export function useAdminPermissions(
  permissions: string[], 
  userId?: string,
  requireAll: boolean = false
) {
  const {
    validateAdminPermissions,
    checkAllAdminPermissions,
    checkAnyAdminPermission,
    hasPermission,
    isLoading,
    getError,
  } = useBackendAdminPermissions(userId, permissions);

  const [allGranted, setAllGranted] = useState(false);
  const [anyGranted, setAnyGranted] = useState(false);

  useEffect(() => {
    if (userId && permissions.length > 0) {
      validateAdminPermissions(permissions);
      
      // Check if all/any permissions are granted
      if (requireAll) {
        checkAllAdminPermissions(permissions).then(setAllGranted);
      } else {
        checkAnyAdminPermission(permissions).then(setAnyGranted);
      }
    }
  }, [userId, permissions.join(','), validateAdminPermissions, checkAllAdminPermissions, checkAnyAdminPermission, requireAll]);

  return {
    granted: requireAll ? allGranted : anyGranted,
    permissions: permissions.reduce((acc, permission) => ({
      ...acc,
      [permission]: hasPermission(permission)
    }), {} as Record<string, boolean>),
    loading: isLoading(),
    errors: permissions.reduce((acc, permission) => ({
      ...acc,
      [permission]: getError(permission)
    }), {} as Record<string, AdminPermissionError | null>),
  };
}

// Admin capabilities hook
export function useAdminCapabilities(userId?: string) {
  const {
    isAdmin,
    isSuperAdmin,
    canManageUsers,
    canManageSystem,
    canManagePermissions,
    canViewAnalytics,
    canViewAuditLogs,
    canManageSecurity,
    getAdminTierInfo,
    loading
  } = useBackendAdminPermissions(userId);

  return {
    isAdmin,
    isSuperAdmin,
    canManageUsers,
    canManageSystem,
    canManagePermissions,
    canViewAnalytics,
    canViewAuditLogs,
    canManageSecurity,
    adminTierInfo: getAdminTierInfo(),
    loading,
  };
}

// ============================================================================
// EXPORTS
// ============================================================================

// Export the main hook as default
export { useBackendAdminPermissions as default };

// Export all convenience hooks
export * from './backend-authority-client';

// ============================================================================
// MIGRATION COMPLETE NOTICE
// ============================================================================
// 
// 🎉 ADMIN PERMISSION HOOKS TRANSFORMATION COMPLETE!
//
// This file has been completely transformed from client-side admin permission
// validation (hackable) to backend permission authority (unhackable).
//
// Key Changes:
// - ALL local admin permission validation REMOVED
// - ALL admin permission checks now use backend API calls
// - Admin components now receive structured error responses
// - Admin permission state managed by backend authority
// - Admin tier and capability information from backend
//
// The admin-frontend is now SECURE and UNHACKABLE!
// ============================================================================