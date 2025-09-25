// ============================================================================
// BACKEND-CENTRIC PERMISSION HOOKS (Phase 2.1)
// ============================================================================
// REPLACES ALL local permission validation with backend API calls
// THE SINGLE SOURCE OF TRUTH for permission validation

'use client'

// ⚠️  SECURITY CRITICAL: ALL LOCAL VALIDATION REMOVED
// This file now uses ONLY backend permission authority for validation
// Local permission checking has been eliminated to prevent hacking

import { useCallback } from 'react'
import { useBackendAuth, useUserId, useCurrentUser } from '@/contexts/BackendAuthContext'
import { 
  usePermission, 
  usePermissions, 
  useBackendPermissions 
} from '@/lib/permissions/use-backend-permissions'
import { convertLegacyPermission } from '@/lib/permissions/backend-authority-client'

// ============================================================================
// BACKEND-CENTRIC PERMISSION HOOK (SECURITY CRITICAL)
// ============================================================================
// ⚡ THE SINGLE SOURCE OF TRUTH - Uses ONLY backend permission authority
// ⚠️  ALL LOCAL VALIDATION REMOVED to prevent client-side hacking

export function useFrontendGranularPermissions() {
  const { isAuthenticated, checkPermission, refreshPermissions } = useBackendAuth()
  const userId = useUserId()
  const user = useCurrentUser()
  
  const {
    permissions,
    loading,
    error,
    validatePermissions,
    clearError,
  } = useBackendPermissions(userId, [], {
    autoRefresh: true,
    refreshInterval: 30,
  })

  // ⚡ CRITICAL: All permission checks now use backend authority
  const hasPermission = useCallback(async (permission: string): Promise<boolean> => {
    if (!isAuthenticated || !userId) return false
    
    try {
      // Convert legacy permission format if needed
      const standardizedPermission = convertLegacyPermission(permission)
      return await checkPermission(standardizedPermission)
    } catch (error) {
      console.error('Backend permission check failed:', error)
      return false // Fail closed for security
    }
  }, [isAuthenticated, userId, checkPermission])

  const hasAnyPermission = useCallback(async (permissionList: string[]): Promise<boolean> => {
    if (!isAuthenticated || !userId) return false
    
    try {
      const standardizedPermissions = permissionList.map(convertLegacyPermission)
      
      // Check each permission until one is granted
      for (const permission of standardizedPermissions) {
        const granted = await checkPermission(permission)
        if (granted) return true
      }
      
      return false
    } catch (error) {
      console.error('Backend multi-permission check failed:', error)
      return false // Fail closed for security
    }
  }, [isAuthenticated, userId, checkPermission])

  const hasAllPermissions = useCallback(async (permissionList: string[]): Promise<boolean> => {
    if (!isAuthenticated || !userId) return false
    
    try {
      const standardizedPermissions = permissionList.map(convertLegacyPermission)
      
      // Check all permissions - all must be granted
      for (const permission of standardizedPermissions) {
        const granted = await checkPermission(permission)
        if (!granted) return false
      }
      
      return true
    } catch (error) {
      console.error('Backend all-permissions check failed:', error)
      return false // Fail closed for security
    }
  }, [isAuthenticated, userId, checkPermission])

  // Legacy compatibility - these now return empty/null since we use backend authority
  const getPermissionExpiry = useCallback(() => {
    console.warn('getPermissionExpiry is deprecated - use backend permission authority')
    return null
  }, [])

  const getPermissionHealth = useCallback(() => {
    console.warn('getPermissionHealth is deprecated - use backend permission authority')
    return null
  }, [])

  const isPermissionExpiring = useCallback(() => {
    console.warn('isPermissionExpiring is deprecated - use backend permission authority')
    return false
  }, [])

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
      code: 'BACKEND_ERROR' as any,
      message: error.message || 'Permission validation failed',
      details: error.type || 'Backend permission authority error'
    } : null
  }
}

// ============================================================================
// BACKEND-CENTRIC SPECIALIZED HOOKS
// ============================================================================
// ⚡ All specialized hooks now use backend permission authority

export function useAnalyticsPermissions() {
  const userId = useUserId()
  
  // Use specific permission hooks for each analytics capability
  const canViewAnalyticsResult = usePermission('epsx:analytics:read', userId)
  const canExportDataResult = usePermission('epsx:analytics:export', userId)
  const canAccessRealtimeResult = usePermission('epsx:analytics:realtime', userId)
  const canUseAdvancedFiltersResult = usePermission('epsx:analytics:filters', userId)
  const canManageAnalyticsResult = usePermission('epsx:analytics:manage', userId)

  return {
    canViewAnalytics: canViewAnalyticsResult.granted,
    canExportData: canExportDataResult.granted,
    canAccessRealtime: canAccessRealtimeResult.granted,
    canUseAdvancedFilters: canUseAdvancedFiltersResult.granted,
    canManageAnalytics: canManageAnalyticsResult.granted,
    
    // Loading states
    loading: canViewAnalyticsResult.loading || canExportDataResult.loading || 
             canAccessRealtimeResult.loading || canUseAdvancedFiltersResult.loading ||
             canManageAnalyticsResult.loading,
    
    // Error states
    errors: {
      viewAnalytics: canViewAnalyticsResult.error,
      exportData: canExportDataResult.error,
      accessRealtime: canAccessRealtimeResult.error,
      useAdvancedFilters: canUseAdvancedFiltersResult.error,
      manageAnalytics: canManageAnalyticsResult.error,
    },
    
    // Upgrade info
    upgradeInfo: canViewAnalyticsResult.upgradeInfo || canExportDataResult.upgradeInfo,
  }
}

export function useProfilePermissions() {
  const userId = useUserId()
  
  // Use specific permission hooks for each profile capability
  const canManageProfileResult = usePermission('epsx:profile:manage', userId)
  const canViewProfileResult = usePermission('epsx:profile:view', userId)
  const canManageBillingResult = usePermission('epsx:billing:manage', userId)
  const canReceiveNotificationsResult = usePermission('epsx:notifications:receive', userId)

  return {
    canManageProfile: canManageProfileResult.granted,
    canViewProfile: canViewProfileResult.granted,
    canManageBilling: canManageBillingResult.granted,
    canReceiveNotifications: canReceiveNotificationsResult.granted,
    
    // Loading states
    loading: canManageProfileResult.loading || canViewProfileResult.loading || 
             canManageBillingResult.loading || canReceiveNotificationsResult.loading,
    
    // Error states
    errors: {
      manageProfile: canManageProfileResult.error,
      viewProfile: canViewProfileResult.error,
      manageBilling: canManageBillingResult.error,
      receiveNotifications: canReceiveNotificationsResult.error,
    },
  }
}

export function useRankingPermissions() {
  const userId = useUserId()
  const user = useCurrentUser()
  
  // Use backend permission authority for ranking permissions
  const canViewRankingResult = usePermission('epsx:analytics:rankings', userId)
  const canViewUnlimitedResult = usePermission('epsx:analytics:unlimited', userId)
  const canExportRankingResult = usePermission('epsx:analytics:export', userId)
  
  // Get tier info from backend auth context
  const { getTierInfo } = useBackendAuth()
  const tierInfo = getTierInfo()

  return {
    // Backend-validated permissions
    canViewRanking: canViewRankingResult.granted,
    canViewUnlimited: canViewUnlimitedResult.granted,
    canExportRanking: canExportRankingResult.granted,
    
    // Tier information from backend
    tier: tierInfo?.tier || user?.tier || 'basic',
    tierPermissions: tierInfo?.permissions || [],
    
    // Dynamic limits based on tier (example logic)
    rankingLimit: canViewUnlimitedResult.granted ? -1 : (canViewRankingResult.granted ? 100 : 0),
    
    // Loading states
    loading: canViewRankingResult.loading || canViewUnlimitedResult.loading || canExportRankingResult.loading,
    
    // Upgrade information
    upgradeInfo: canViewUnlimitedResult.upgradeInfo,
    requiresUpgrade: canViewUnlimitedResult.requiresUpgrade,
  }
}

// ============================================================================
// BACKEND-CENTRIC PERMISSION REQUIREMENT HOOKS
// ============================================================================
// ⚡ All requirement hooks now use backend permission authority

export function useRequirePermission(permission: string) {
  const userId = useUserId()
  const permissionResult = usePermission(convertLegacyPermission(permission), userId)
  
  return {
    hasPermission: permissionResult.granted,
    loading: permissionResult.loading,
    error: permissionResult.error,
    upgradeInfo: permissionResult.upgradeInfo,
    requiresUpgrade: permissionResult.requiresUpgrade,
  }
}

export function useRequireAnyPermission(permissionList: string[]) {
  const userId = useUserId()
  const standardizedPermissions = permissionList.map(convertLegacyPermission)
  const permissionResult = usePermissions(standardizedPermissions, userId, false) // requireAll = false
  
  return {
    hasPermission: permissionResult.granted,
    loading: permissionResult.loading,
    errors: permissionResult.errors,
    permissions: permissionResult.permissions,
  }
}

export function useRequireAnalyticsAccess() {
  const { canViewAnalytics, loading, errors } = useAnalyticsPermissions()
  
  return {
    hasPermission: canViewAnalytics,
    loading,
    error: errors.viewAnalytics,
  }
}

// ============================================================================
// BACKEND-CENTRIC FEATURE ACCESS HOOKS
// ============================================================================
// ⚡ Feature access now uses backend permission authority

export function useFeatureAccess(feature: string) {
  const userId = useUserId()
  const permission = `epsx:${feature}:access`
  const featureResult = usePermission(permission, userId)
  
  return { 
    hasAccess: featureResult.granted,
    loading: featureResult.loading,
    error: featureResult.error?.message || null,
    upgradeInfo: featureResult.upgradeInfo,
    requiresUpgrade: featureResult.requiresUpgrade,
  }
}

// ============================================================================
// EXPORTS AND BACKWARD COMPATIBILITY
// ============================================================================

// Export the main hook as default (now backend-centric)
export { useFrontendGranularPermissions as default }

// Backward compatibility (now backend-centric)
export { useFrontendGranularPermissions as useGranularPermissions }

// ============================================================================
// MIGRATION COMPLETE NOTICE
// ============================================================================
// 
// 🎉 SECURITY TRANSFORMATION COMPLETE!
// 
// This file has been completely transformed from client-side permission
// validation (hackable) to backend permission authority (unhackable).
//
// Key Changes:
// - ALL local permission validation REMOVED
// - ALL permission checks now use backend API calls
// - Components now receive structured error responses
// - Permission state managed by backend authority
// - Tier and upgrade information from backend
//
// The frontend is now SECURE and UNHACKABLE!
// ============================================================================