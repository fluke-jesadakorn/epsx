// ============================================================================
// BACKEND-CENTRIC PERMISSIONS HOOK (Phase 2.1)
// Replaces ALL local permission validation with backend API calls
// THE SINGLE SOURCE OF TRUTH for permission state management
// ============================================================================

'use client';

import { useState, useEffect, useCallback, useMemo } from 'react';
import { 
  permissionAuthority, 
  hasPermission, 
  hasAnyPermission, 
  hasAllPermissions,
  PermissionValidationError,
  AuthenticationRequiredError,
  PermissionDeniedError,
  InsufficientTierError,
  UsageLimitExceededError,
  type PermissionErrorResponse,
  type UserPermissionsResponse 
} from './backend-authority-client';

// ============================================================================
// PERMISSION HOOK STATE TYPES
// ============================================================================

export interface PermissionState {
  // Permission validation results
  permissions: Record<string, boolean>;
  
  // Loading states
  loading: boolean;
  validating: Record<string, boolean>;
  
  // Error states  
  error: PermissionError | null;
  errors: Record<string, PermissionError>;
  
  // User context
  userId?: string;
  currentTier?: string;
  tierInfo?: {
    current_tier: string;
    tier_permissions: string[];
  };
  
  // Permission metadata
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

export interface PermissionError {
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
}

export interface PermissionHookResult extends PermissionState {
  // Permission checking functions
  checkPermission: (permission: string, resourcePath?: string) => Promise<boolean>;
  checkAnyPermission: (permissions: string[]) => Promise<boolean>;
  checkAllPermissions: (permissions: string[]) => Promise<boolean>;
  
  // Bulk operations
  validatePermissions: (permissions: string[]) => Promise<void>;
  
  // Cache management
  refreshPermissions: () => Promise<void>;
  clearCache: () => void;
  
  // Error handling
  clearError: (permission?: string) => void;
  handlePermissionError: (error: any, permission?: string) => void;
  
  // Utility functions
  hasPermission: (permission: string) => boolean;
  isLoading: (permission?: string) => boolean;
  getError: (permission?: string) => PermissionError | null;
  
  // Upgrade/tier helpers
  requiresUpgrade: (permission: string) => boolean;
  getUpgradeInfo: (permission: string) => any;
  getTierInfo: () => any;
}

// ============================================================================
// MAIN BACKEND PERMISSIONS HOOK
// THE SINGLE SOURCE OF TRUTH for all permission state
// ============================================================================

export function useBackendPermissions(
  userId?: string,
  initialPermissions?: string[],
  options: {
    autoRefresh?: boolean;
    refreshInterval?: number; // minutes
    cacheTimeout?: number; // minutes
  } = {}
): PermissionHookResult {
  
  const {
    autoRefresh = false,
    refreshInterval = 30, // 30 minutes
    cacheTimeout = 60, // 60 minutes
  } = options;

  // ============================================================================
  // HOOK STATE MANAGEMENT
  // ============================================================================
  
  const [state, setState] = useState<PermissionState>({
    permissions: {},
    loading: false,
    validating: {},
    error: null,
    errors: {},
    userId,
    usageInfo: {},
  });

  // ============================================================================
  // PERMISSION VALIDATION FUNCTIONS
  // ============================================================================
  
  const checkPermission = useCallback(async (
    permission: string,
    resourcePath?: string
  ): Promise<boolean> => {
    if (!userId) {
      console.warn('No user ID provided for permission check');
      return false;
    }

    // Set loading state for this specific permission
    setState(prev => ({
      ...prev,
      validating: { ...prev.validating, [permission]: true }
    }));

    try {
      const result = await permissionAuthority.validatePermission(
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
            reset_at: result.next_refresh,
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

  const checkAnyPermission = useCallback(async (permissions: string[]): Promise<boolean> => {
    if (!userId) return false;

    try {
      return await hasAnyPermission(userId, permissions);
    } catch (error) {
      handlePermissionError(error);
      return false; // Fail closed for security
    }
  }, [userId]);

  const checkAllPermissions = useCallback(async (permissions: string[]): Promise<boolean> => {
    if (!userId) return false;

    try {
      return await hasAllPermissions(userId, permissions);
    } catch (error) {
      handlePermissionError(error);
      return false; // Fail closed for security
    }
  }, [userId]);

  // ============================================================================
  // BULK PERMISSION VALIDATION
  // ============================================================================
  
  const validatePermissions = useCallback(async (permissions: string[]) => {
    if (!userId || permissions.length === 0) return;

    setState(prev => ({ ...prev, loading: true }));

    try {
      const permissionRequests = permissions.map(permission => ({ permission }));
      const result = await permissionAuthority.validateBulkPermissions(userId, permissionRequests);

      // Update state with bulk results
      const newPermissions: Record<string, boolean> = {};
      const newErrors: Record<string, PermissionError> = {};
      
      result.results.forEach(({ permission, granted, reason }) => {
        newPermissions[permission] = granted;
        if (!granted && reason) {
          newErrors[permission] = {
            type: 'permission_denied',
            message: reason,
            userMessage: `Access denied: ${reason}`,
            suggestedActions: ['Check your subscription plan', 'Contact support'],
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
  // USER PERMISSIONS REFRESH
  // ============================================================================
  
  const refreshPermissions = useCallback(async () => {
    if (!userId) return;

    setState(prev => ({ ...prev, loading: true }));

    try {
      const userPermissions = await permissionAuthority.getUserPermissions(userId);
      
      // Update state with comprehensive user permissions
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
        tierInfo: userPermissions.tier_info,
        currentTier: userPermissions.tier_info?.current_tier,
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
    console.error('Permission validation error:', error);
    
    let permissionError: PermissionError;

    if (error instanceof AuthenticationRequiredError) {
      permissionError = {
        type: 'authentication_required',
        message: error.message,
        userMessage: error.errorData.user_message,
        suggestedActions: error.errorData.suggested_actions,
      };
    } else if (error instanceof PermissionDeniedError) {
      permissionError = {
        type: 'permission_denied',
        message: error.message,
        userMessage: error.errorData.user_message,
        suggestedActions: error.errorData.suggested_actions,
        upgradeInfo: error.upgradeInfo,
      };
    } else if (error instanceof InsufficientTierError) {
      permissionError = {
        type: 'insufficient_tier',
        message: error.message,
        userMessage: error.errorData.user_message,
        suggestedActions: error.errorData.suggested_actions,
        upgradeInfo: error.upgradeInfo,
      };
    } else if (error instanceof UsageLimitExceededError) {
      permissionError = {
        type: 'usage_limit_exceeded',
        message: error.message,
        userMessage: error.errorData.user_message,
        suggestedActions: error.errorData.suggested_actions,
        context: { usageInfo: error.usageInfo },
      };
    } else {
      permissionError = {
        type: 'unknown_error',
        message: error.message || 'Permission validation failed',
        userMessage: 'Unable to validate permissions. Please try again.',
        suggestedActions: ['Try again', 'Check your connection', 'Contact support'],
      };
    }

    setState(prev => {
      const newState = { ...prev };
      
      if (permission) {
        newState.errors = { ...prev.errors, [permission]: permissionError };
      } else {
        newState.error = permissionError;
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

  const getError = useCallback((permission?: string): PermissionError | null => {
    if (permission) {
      return state.errors[permission] || null;
    }
    return state.error;
  }, [state.errors, state.error]);

  const requiresUpgrade = useCallback((permission: string): boolean => {
    const error = state.errors[permission];
    return error?.type === 'insufficient_tier' || error?.type === 'permission_denied';
  }, [state.errors]);

  const getUpgradeInfo = useCallback((permission: string) => {
    const error = state.errors[permission];
    return error?.upgradeInfo || null;
  }, [state.errors]);

  const getTierInfo = useCallback(() => {
    return state.tierInfo || null;
  }, [state.tierInfo]);

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

  // ============================================================================
  // EFFECTS FOR AUTO-REFRESH AND INITIAL LOADING
  // ============================================================================
  
  // Initial permission validation
  useEffect(() => {
    if (userId && initialPermissions && initialPermissions.length > 0) {
      validatePermissions(initialPermissions);
    }
  }, [userId, initialPermissions?.join(','), validatePermissions]);

  // Auto-refresh permissions
  useEffect(() => {
    if (!autoRefresh || !userId) return;

    const intervalId = setInterval(() => {
      refreshPermissions();
    }, refreshInterval * 60 * 1000); // Convert minutes to milliseconds

    return () => clearInterval(intervalId);
  }, [autoRefresh, userId, refreshInterval, refreshPermissions]);

  // Cache expiry check
  useEffect(() => {
    if (!state.cacheExpiry) return;

    const expiryTime = new Date(state.cacheExpiry).getTime();
    const now = Date.now();
    
    if (now >= expiryTime) {
      refreshPermissions();
    }
  }, [state.cacheExpiry, refreshPermissions]);

  // ============================================================================
  // RETURN HOOK RESULT
  // ============================================================================
  
  return {
    // State
    ...state,
    
    // Functions
    checkPermission,
    checkAnyPermission,
    checkAllPermissions,
    validatePermissions,
    refreshPermissions,
    clearCache,
    clearError,
    handlePermissionError,
    
    // Utilities
    hasPermission: hasPermissionLocal,
    isLoading,
    getError,
    requiresUpgrade,
    getUpgradeInfo,
    getTierInfo,
  };
}

// ============================================================================
// CONVENIENCE HOOKS FOR SPECIFIC USE CASES
// ============================================================================

// Single permission check hook
export function usePermission(
  permission: string, 
  userId?: string,
  resourcePath?: string
) {
  const { 
    checkPermission, 
    hasPermission, 
    isLoading, 
    getError,
    requiresUpgrade,
    getUpgradeInfo
  } = useBackendPermissions(userId, [permission]);

  useEffect(() => {
    if (userId && permission) {
      checkPermission(permission, resourcePath);
    }
  }, [userId, permission, resourcePath, checkPermission]);

  return {
    granted: hasPermission(permission),
    loading: isLoading(permission),
    error: getError(permission),
    requiresUpgrade: requiresUpgrade(permission),
    upgradeInfo: getUpgradeInfo(permission),
    refresh: () => checkPermission(permission, resourcePath),
  };
}

// Multiple permissions check hook
export function usePermissions(
  permissions: string[], 
  userId?: string,
  requireAll: boolean = false
) {
  const {
    validatePermissions,
    checkAllPermissions,
    checkAnyPermission,
    hasPermission,
    isLoading,
    getError,
  } = useBackendPermissions(userId, permissions);

  const [allGranted, setAllGranted] = useState(false);
  const [anyGranted, setAnyGranted] = useState(false);

  useEffect(() => {
    if (userId && permissions.length > 0) {
      validatePermissions(permissions);
      
      // Check if all/any permissions are granted
      if (requireAll) {
        checkAllPermissions(permissions).then(setAllGranted);
      } else {
        checkAnyPermissions(permissions).then(setAnyGranted);
      }
    }
  }, [userId, permissions.join(','), validatePermissions, checkAllPermissions, checkAnyPermission, requireAll]);

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
    }), {} as Record<string, PermissionError | null>),
  };
}

// Admin permissions hook
export function useAdminPermissions(userId?: string) {
  const adminPermissions = [
    'admin:users:read',
    'admin:users:create', 
    'admin:users:update',
    'admin:users:delete',
    'admin:permissions:manage',
    'admin:security:read',
    'admin:performance:read',
  ];

  return usePermissions(adminPermissions, userId);
}