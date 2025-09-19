// ============================================================================
// FRONTEND PERMISSION HOOKS
// ============================================================================
// Frontend-specific permission hooks that use shared permission logic

'use client'

import { useCallback, useState, useEffect } from 'react'
import { useAuth } from '@/lib/auth'
import {
  UsePermissionHookResult,
  EnhancedUserClaims,
  UserClaims,
  PermissionStatusResponse,
  PermissionExpiryDetails,
  PermissionHealthInfo,
  GranularPermissionError
} from '@/shared/permissions/types'
import {
  hasPermissionGranular,
  hasAnyPermissionGranular,
  hasAllPermissionsGranular,
  hasPermissionWithTime,
  hasAnyPermissionWithTime,
  hasAllPermissionsWithTime,
  getPermissionExpiryDetails,
  calculatePermissionHealth,
  canViewAnalytics,
  canExportData,
  canAccessRealtime,
  canManageProfile,
  canReceiveNotifications,
  canManageBilling,
  canUseAdvancedFilters
} from '@/shared/permissions/utils'
import {
  extractRankingLimitFromPermissions,
  deriveTierFromPermissions,
  getPackageFromPermissions,
  filterValidPermissions
} from '@/lib/permission-utils'
import { frontendPermissionApiClient } from './api-client'

// ============================================================================
// MAIN FRONTEND PERMISSION HOOK
// ============================================================================

export function useFrontendGranularPermissions(): UsePermissionHookResult {
  const { user, isAuthenticated } = useAuth.getState()
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<any>(null)

  // Support both enhanced and legacy user claims
  const enhancedUser = user as EnhancedUserClaims | null
  const legacyUser = user as UserClaims | null
  
  // Use granular permissions if available, otherwise fall back to legacy
  const permissions = enhancedUser?.permissions || {}
  const legacyPermissions = legacyUser?.permissions || []

  // Core permission checking functions
  const hasPermission = useCallback((permission: string): boolean => {
    if (!isAuthenticated) return false
    
    // Try granular permissions first
    if (enhancedUser && Object.keys(permissions).length > 0) {
      return hasPermissionGranular(permissions, permission)
    }
    
    // Fall back to legacy permissions with timestamp support
    if (legacyUser && legacyPermissions.length > 0) {
      return hasPermissionWithTime(legacyUser, permission)
    }
    
    return false
  }, [enhancedUser, legacyUser, permissions, legacyPermissions, isAuthenticated])

  const hasAnyPermission = useCallback((permissionList: string[]): boolean => {
    if (!isAuthenticated) return false
    
    // Try granular permissions first
    if (enhancedUser && Object.keys(permissions).length > 0) {
      return hasAnyPermissionGranular(permissions, permissionList)
    }
    
    // Fall back to legacy permissions
    if (legacyUser && legacyPermissions.length > 0) {
      return hasAnyPermissionWithTime(legacyUser, permissionList)
    }
    
    return false
  }, [enhancedUser, legacyUser, permissions, legacyPermissions, isAuthenticated])

  const hasAllPermissions = useCallback((permissionList: string[]): boolean => {
    if (!isAuthenticated) return false
    
    // Try granular permissions first
    if (enhancedUser && Object.keys(permissions).length > 0) {
      return hasAllPermissionsGranular(permissions, permissionList)
    }
    
    // Fall back to legacy permissions
    if (legacyUser && legacyPermissions.length > 0) {
      return hasAllPermissionsWithTime(legacyUser, permissionList)
    }
    
    return false
  }, [enhancedUser, legacyUser, permissions, legacyPermissions, isAuthenticated])

  const getPermissionExpiry = useCallback((permission: string): PermissionExpiryDetails | null => {
    if (!enhancedUser) return null
    return getPermissionExpiryDetails(permissions, permission)
  }, [enhancedUser, permissions])

  const getPermissionHealth = useCallback((): PermissionHealthInfo | null => {
    if (!enhancedUser) return null
    return calculatePermissionHealth(permissions)
  }, [enhancedUser, permissions])

  const isPermissionExpiring = useCallback((permission: string, withinHours: number = 24): boolean => {
    const expiry = getPermissionExpiry(permission)
    if (!expiry || expiry.is_permanent) return false
    
    const withinMs = withinHours * 60 * 60 * 1000
    return expiry.expires_in_ms !== undefined && expiry.expires_in_ms <= withinMs
  }, [getPermissionExpiry])

  const refreshPermissions = useCallback(async (): Promise<void> => {
    setLoading(true)
    setError(null)

    try {
      await frontendPermissionApiClient.refreshUserToken()
      // Refresh auth state would be handled by auth system
    } catch (err) {
      setError(err)
      console.error('Failed to refresh permissions:', err)
    } finally {
      setLoading(false)
    }
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
      code: 'VALIDATION_ERROR' as any,
      message: error.message,
      details: error.toString()
    } : null
  }
}

// ============================================================================
// SPECIALIZED FRONTEND HOOKS
// ============================================================================

export function useAnalyticsPermissions() {
  const { hasPermission, hasAnyPermission } = useFrontendGranularPermissions()
  const { user } = useAuth.getState()
  
  // Support both enhanced and legacy user claims
  const enhancedUser = user as EnhancedUserClaims | null
  const legacyUser = user as UserClaims | null

  return {
    canViewAnalytics: enhancedUser ? canViewAnalytics(enhancedUser) : canViewAnalytics(legacyUser as any),
    canExportData: enhancedUser ? canExportData(enhancedUser) : canExportData(legacyUser as any),
    canAccessRealtime: enhancedUser ? canAccessRealtime(enhancedUser) : canAccessRealtime(legacyUser as any),
    canUseAdvancedFilters: enhancedUser ? canUseAdvancedFilters(enhancedUser) : canUseAdvancedFilters(legacyUser as any),
    canManageAnalytics: hasPermission('epsx:analytics:manage'),
    hasPermission,
    hasAnyPermission
  }
}

export function useProfilePermissions() {
  const { hasPermission } = useFrontendGranularPermissions()
  const { user } = useAuth.getState()
  
  // Support both enhanced and legacy user claims
  const enhancedUser = user as EnhancedUserClaims | null
  const legacyUser = user as UserClaims | null

  return {
    canManageProfile: enhancedUser ? canManageProfile(enhancedUser) : canManageProfile(legacyUser as any),
    canViewProfile: hasPermission('epsx:profile:view'),
    canManageBilling: enhancedUser ? canManageBilling(enhancedUser) : canManageBilling(legacyUser as any),
    canReceiveNotifications: enhancedUser ? canReceiveNotifications(enhancedUser) : canReceiveNotifications(legacyUser as any),
    hasPermission
  }
}

export function useRankingPermissions() {
  const { user } = useAuth.getState()
  const legacyUser = user as UserClaims | null
  
  // For ranking permissions, we primarily use the legacy system for now
  const permissions = legacyUser?.permissions || []
  const validPermissions = filterValidPermissions(permissions)
  
  const rankingLimit = extractRankingLimitFromPermissions(validPermissions)
  const tier = deriveTierFromPermissions(validPermissions)
  const packageInfo = getPackageFromPermissions(validPermissions)

  return {
    rankingLimit,
    tier,
    packageInfo,
    canViewRanking: rankingLimit > 0,
    canViewUnlimited: rankingLimit === -1,
    permissions: validPermissions
  }
}

export function useLegacyPermissionMigration() {
  const [migrationStatus, setMigrationStatus] = useState<'pending' | 'in_progress' | 'completed' | 'error'>('pending')
  const [error, setError] = useState<string | null>(null)

  const migrateLegacyPermissions = useCallback(async () => {
    setMigrationStatus('in_progress')
    setError(null)

    try {
      const result = await frontendPermissionApiClient.convertLegacyPermissions()
      console.log('Legacy permission migration result:', result)
      setMigrationStatus('completed')
      return result
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Migration failed')
      setMigrationStatus('error')
      throw err
    }
  }, [])

  return {
    migrationStatus,
    error,
    migrateLegacyPermissions
  }
}

// ============================================================================
// PERMISSION REQUIREMENT HOOKS
// ============================================================================

export function useRequirePermission(permission: string) {
  const { hasPermission, loading, error } = useFrontendGranularPermissions()
  
  return {
    hasPermission: hasPermission(permission),
    loading,
    error
  }
}

export function useRequireAnyPermission(permissionList: string[]) {
  const { hasAnyPermission, loading, error } = useFrontendGranularPermissions()
  
  return {
    hasPermission: hasAnyPermission(permissionList),
    loading,
    error
  }
}

export function useRequireAnalyticsAccess() {
  const { canViewAnalytics } = useAnalyticsPermissions()
  const { loading, error } = useFrontendGranularPermissions()
  
  return {
    hasPermission: canViewAnalytics,
    loading,
    error
  }
}

// ============================================================================
// FEATURE ACCESS HOOKS
// ============================================================================

export function useFeatureAccess(feature: string) {
  const [hasAccess, setHasAccess] = useState<boolean | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const checkAccess = async () => {
      setLoading(true)
      setError(null)

      try {
        const result = await frontendPermissionApiClient.checkFeatureAccess(feature)
        setHasAccess(result.hasAccess)
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Feature check failed')
        setHasAccess(false)
      } finally {
        setLoading(false)
      }
    }

    checkAccess()
  }, [feature])

  return { hasAccess, loading, error }
}

// Export the main hook as default
export { useFrontendGranularPermissions as default }

// Backward compatibility
export { useFrontendGranularPermissions as useGranularPermissions }