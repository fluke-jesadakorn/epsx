'use client'

import { useAuth } from '@/lib/auth'
import { useCallback } from 'react'
import { 
  UserClaims, 
  hasPermission, 
  hasAnyPermission, 
  hasPlatformPermission, 
  canAccessPlatform,
  getPlatformPermissions,
  isAdmin,
  canManageUsers,
  canViewUsers,
  canManageSystem,
  canViewAuditLogs,
  canViewAnalytics,
  canExportData,
  canAccessRealtime,
  canManageProfile,
  canReceiveNotifications,
  canManageBilling,
  canUseAdvancedFilters,
  PERMISSION_SETS,
  // New timestamp-aware functions
  hasPermissionWithTime,
  hasPermissionForDuration,
  getPermissionExpiryInfo,
  getTimeUntilNextExpiry,
  filterValidPermissions,
  isAdminWithTime,
  canManageUsersWithTime,
  checkFeatureAccessWithTime,
  type TimestampedPermission,
  type PermissionExpiryInfo
} from '@/types/permissions'

export interface PermissionCheck {
  hasPermission: boolean
  loading: boolean
  error?: string
}

export interface PermissionHookReturn {
  // Direct permission checks
  can: (permission: string) => boolean
  hasAnyPermission: (permissions: string[]) => boolean
  hasAllPermissions: (permissions: string[]) => boolean
  
  // Structured permission checks
  hasPermission: (platform: string, resource: string, action: string) => boolean
  canRead: (resource: string, platform?: string) => boolean
  canWrite: (resource: string, platform?: string) => boolean
  canDelete: (resource: string, platform?: string) => boolean
  canManage: (resource: string, platform?: string) => boolean
  canCreate: (resource: string, platform?: string) => boolean
  
  // Platform-specific checks
  canAccessPlatform: (platform: string) => boolean
  isCurrentPlatform: (platform: string) => boolean
  getPlatformPermissions: (platform: string) => string[]
  
  // Current context
  currentPlatform: string
  availablePlatforms: string[]
  user: UserClaims | null
  isAuthenticated: boolean
  
  // Admin checks
  isAdmin: boolean
  canManageUsers: boolean
  canViewUsers: boolean
  canManageSystem: boolean
  canViewAuditLogs: boolean
  
  // Feature checks
  canViewAnalytics: boolean
  canExportData: boolean
  canAccessRealtime: boolean
  canManageProfile: boolean
  canReceiveNotifications: boolean
  canManageBilling: boolean
  canUseAdvancedFilters: boolean
  
  // Utility functions
  getAccessLevel: (resource: string, platform?: string) => 'none' | 'read' | 'write' | 'manage'
  hasPermissionSet: (permissionSet: readonly string[]) => boolean
  
  // Embedded Timestamp Support
  canWithTime: (permission: string) => boolean
  canForDuration: (permission: string, durationMinutes?: number) => boolean
  hasValidPermissions: () => string[]
  getPermissionExpiry: () => PermissionExpiryInfo
  getTimeUntilExpiry: () => number | null
  
  // Admin checks with timestamp validation
  isAdminWithTime: boolean
  canManageUsersWithTime: boolean
  
  // Feature checks with timestamp validation
  canViewAnalyticsWithTime: boolean
  canExportDataWithTime: boolean
  canAccessRealtimeWithTime: boolean
  canManageProfileWithTime: boolean
  canReceiveNotificationsWithTime: boolean
  canManageBillingWithTime: boolean
  canUseAdvancedFiltersWithTime: boolean
  
  // Permission health and expiry info
  permissionHealth: PermissionExpiryInfo
  hasExpiredPermissions: boolean
  hasExpiringSoonPermissions: boolean
  nextExpiryTime: number | null
}

export function usePermissions(): PermissionHookReturn {
  const { 
    user,
    isAuthenticated,
    getCurrentPlatform,
    getAvailablePlatforms,
  } = useAuth.getState()
  
  const currentPlatform = getCurrentPlatform() || 'epsx'
  const availablePlatforms = getAvailablePlatforms() || ['epsx']
  
  // Direct permission check
  const can = useCallback((permission: string) => {
    return hasPermission(user, permission)
  }, [user])
  
  // Multiple permission checks
  const hasAnyPermissionCheck = useCallback((permissions: string[]) => {
    return hasAnyPermission(user, permissions)
  }, [user])
  
  const hasAllPermissions = useCallback((permissions: string[]) => {
    if (!user) return false
    return permissions.every(permission => hasPermission(user, permission))
  }, [user])
  
  // Structured permission check with platform context
  const hasPermissionStructured = useCallback((platform: string, resource: string, action: string) => {
    return hasPlatformPermission(user, platform, resource, action)
  }, [user])
  
  // Common permission shortcuts
  const canRead = useCallback((resource: string, platform?: string) => {
    const targetPlatform = platform || currentPlatform
    return hasPermissionStructured(targetPlatform, resource, 'read')
  }, [hasPermissionStructured, currentPlatform])
  
  const canWrite = useCallback((resource: string, platform?: string) => {
    const targetPlatform = platform || currentPlatform
    return hasPermissionStructured(targetPlatform, resource, 'write')
  }, [hasPermissionStructured, currentPlatform])
  
  const canDelete = useCallback((resource: string, platform?: string) => {
    const targetPlatform = platform || currentPlatform
    return hasPermissionStructured(targetPlatform, resource, 'delete')
  }, [hasPermissionStructured, currentPlatform])
  
  const canManageResource = useCallback((resource: string, platform?: string) => {
    const targetPlatform = platform || currentPlatform
    return hasPermissionStructured(targetPlatform, resource, 'manage')
  }, [hasPermissionStructured, currentPlatform])
  
  const canCreate = useCallback((resource: string, platform?: string) => {
    const targetPlatform = platform || currentPlatform
    return hasPermissionStructured(targetPlatform, resource, 'create')
  }, [hasPermissionStructured, currentPlatform])
  
  // Platform checks
  const canAccessPlatformCheck = useCallback((platform: string) => {
    return canAccessPlatform(user, platform)
  }, [user])
  
  const isCurrentPlatform = useCallback((platform: string) => {
    return currentPlatform === platform
  }, [currentPlatform])
  
  const getPlatformPermissionsForUser = useCallback((platform: string) => {
    return getPlatformPermissions(user, platform)
  }, [user])
  
  // Get access level for a resource
  const getAccessLevel = useCallback((resource: string, platform?: string): 'none' | 'read' | 'write' | 'manage' => {
    if (canManageResource(resource, platform)) return 'manage'
    if (canWrite(resource, platform)) return 'write' 
    if (canRead(resource, platform)) return 'read'
    return 'none'
  }, [canManageResource, canWrite, canRead])
  
  // Check if user has all permissions from a permission set
  const hasPermissionSet = useCallback((permissionSet: readonly string[]) => {
    return permissionSet.every(permission => can(permission))
  }, [can])
  
  // Embedded Timestamp Support
  const canWithTime = useCallback((permission: string) => {
    return hasPermissionWithTime(user, permission)
  }, [user])
  
  const canForDuration = useCallback((permission: string, durationMinutes: number = 0) => {
    return hasPermissionForDuration(user, permission, durationMinutes)
  }, [user])
  
  const hasValidPermissions = useCallback(() => {
    if (!user) return []
    return filterValidPermissions(user.permissions)
  }, [user])
  
  const getPermissionExpiry = useCallback(() => {
    return getPermissionExpiryInfo(user)
  }, [user])
  
  const getTimeUntilExpiry = useCallback(() => {
    return getTimeUntilNextExpiry(user)
  }, [user])
  
  // Pre-computed permission checks for performance (original)
  const adminCheck = isAdmin(user)
  const canManageUsersCheck = canManageUsers(user)
  const canViewUsersCheck = canViewUsers(user)
  const canManageSystemCheck = canManageSystem(user)
  const canViewAuditLogsCheck = canViewAuditLogs(user)
  const canViewAnalyticsCheck = canViewAnalytics(user)
  const canExportDataCheck = canExportData(user)
  const canAccessRealtimeCheck = canAccessRealtime(user)
  const canManageProfileCheck = canManageProfile(user)
  const canReceiveNotificationsCheck = canReceiveNotifications(user)
  const canManageBillingCheck = canManageBilling(user)
  const canUseAdvancedFiltersCheck = canUseAdvancedFilters(user)
  
  // Pre-computed timestamp-aware permission checks
  const adminWithTimeCheck = isAdminWithTime(user)
  const canManageUsersWithTimeCheck = canManageUsersWithTime(user)
  const canViewAnalyticsWithTimeCheck = checkFeatureAccessWithTime(user, 'view_eps')
  const canExportDataWithTimeCheck = checkFeatureAccessWithTime(user, 'export_data')
  const canAccessRealtimeWithTimeCheck = checkFeatureAccessWithTime(user, 'realtime')
  const canManageProfileWithTimeCheck = checkFeatureAccessWithTime(user, 'profile')
  const canReceiveNotificationsWithTimeCheck = checkFeatureAccessWithTime(user, 'notifications')
  const canManageBillingWithTimeCheck = checkFeatureAccessWithTime(user, 'billing')
  const canUseAdvancedFiltersWithTimeCheck = checkFeatureAccessWithTime(user, 'advanced_filters')
  
  // Permission health information
  const permissionHealth = getPermissionExpiryInfo(user)
  const hasExpiredPermissions = permissionHealth.expired.length > 0
  const hasExpiringSoonPermissions = permissionHealth.hasExpiringPermissions
  const nextExpiryTime = getTimeUntilNextExpiry(user)
  
  return {
    // Direct checks
    can,
    hasAnyPermission: hasAnyPermissionCheck,
    hasAllPermissions,
    
    // Structured checks
    hasPermission: hasPermissionStructured,
    canRead,
    canWrite,
    canDelete,
    canManage: canManageResource,
    canCreate,
    
    // Platform checks
    canAccessPlatform: canAccessPlatformCheck,
    isCurrentPlatform,
    getPlatformPermissions: getPlatformPermissionsForUser,
    
    // Context
    currentPlatform,
    availablePlatforms,
    user,
    isAuthenticated,
    
    // Admin checks
    isAdmin: adminCheck,
    canManageUsers: canManageUsersCheck,
    canViewUsers: canViewUsersCheck,
    canManageSystem: canManageSystemCheck,
    canViewAuditLogs: canViewAuditLogsCheck,
    
    // Feature checks
    canViewAnalytics: canViewAnalyticsCheck,
    canExportData: canExportDataCheck,
    canAccessRealtime: canAccessRealtimeCheck,
    canManageProfile: canManageProfileCheck,
    canReceiveNotifications: canReceiveNotificationsCheck,
    canManageBilling: canManageBillingCheck,
    canUseAdvancedFilters: canUseAdvancedFiltersCheck,
    
    // Utilities
    getAccessLevel,
    hasPermissionSet,
    
    // Embedded Timestamp Support
    canWithTime,
    canForDuration,
    hasValidPermissions,
    getPermissionExpiry,
    getTimeUntilExpiry,
    
    // Admin checks with timestamp validation
    isAdminWithTime: adminWithTimeCheck,
    canManageUsersWithTime: canManageUsersWithTimeCheck,
    
    // Feature checks with timestamp validation
    canViewAnalyticsWithTime: canViewAnalyticsWithTimeCheck,
    canExportDataWithTime: canExportDataWithTimeCheck,
    canAccessRealtimeWithTime: canAccessRealtimeWithTimeCheck,
    canManageProfileWithTime: canManageProfileWithTimeCheck,
    canReceiveNotificationsWithTime: canReceiveNotificationsWithTimeCheck,
    canManageBillingWithTime: canManageBillingWithTimeCheck,
    canUseAdvancedFiltersWithTime: canUseAdvancedFiltersWithTimeCheck,
    
    // Permission health and expiry info
    permissionHealth,
    hasExpiredPermissions,
    hasExpiringSoonPermissions,
    nextExpiryTime,
  }
}

// Specialized hooks for common use cases
export function useAnalyticsPermissions() {
  const permissions = usePermissions()
  
  return {
    canViewAnalytics: permissions.canViewAnalytics,
    canExportData: permissions.canExportData,
    canAccessRealTimeData: permissions.canAccessRealtime,
    canManageFilters: permissions.canWrite('filters'),
    canAccessAdvancedAnalytics: permissions.can('epsx:analytics:advanced'),
    canAccessPremiumFeatures: permissions.hasPermissionSet(PERMISSION_SETS.PREMIUM_USER),
    ...permissions
  }
}

export function useUserManagementPermissions() {
  const permissions = usePermissions()
  
  return {
    canViewUsers: permissions.canViewUsers,
    canCreateUsers: permissions.canCreate('users'),
    canEditUsers: permissions.canWrite('users'),
    canDeleteUsers: permissions.canDelete('users'),
    canManageUsers: permissions.canManageUsers,
    canManagePermissions: permissions.hasPermission('admin', 'permissions', 'manage'),
    canViewAuditLogs: permissions.canViewAuditLogs,
    isAdmin: permissions.isAdmin,
    ...permissions
  }
}

export function usePaymentPermissions() {
  const permissions = usePermissions()
  const canAccessPayPlatform = permissions.canAccessPlatform('epsx-pay')
  
  return {
    canAccessPayments: canAccessPayPlatform && permissions.canRead('payments', 'epsx-pay'),
    canProcessPayments: canAccessPayPlatform && permissions.canWrite('payments', 'epsx-pay'),
    canRefundPayments: canAccessPayPlatform && permissions.hasPermission('epsx-pay', 'payments', 'refund'),
    canViewTransactions: canAccessPayPlatform && permissions.canRead('transactions', 'epsx-pay'),
    canManagePayments: canAccessPayPlatform && permissions.canManage('payments', 'epsx-pay'),
    canAccessPayPlatform,
    ...permissions
  }
}

export function useTokenPermissions() {
  const permissions = usePermissions()
  const canAccessTokenPlatform = permissions.canAccessPlatform('epsx-token')
  
  return {
    canAccessTokens: canAccessTokenPlatform && permissions.canRead('tokens', 'epsx-token'),
    canVote: canAccessTokenPlatform && permissions.hasPermission('epsx-token', 'governance', 'vote'),
    canCreateProposals: canAccessTokenPlatform && permissions.hasPermission('epsx-token', 'governance', 'propose'),
    canManageGovernance: canAccessTokenPlatform && permissions.canManage('governance', 'epsx-token'),
    canStakeTokens: canAccessTokenPlatform && permissions.hasPermission('epsx-token', 'tokens', 'stake'),
    canAccessTokenPlatform,
    ...permissions
  }
}

// Permission requirement helpers for conditional rendering
export function useRequirePermission(permission: string, platform?: string): PermissionCheck {
  const { user, isAuthenticated } = useAuth.getState()
  const fullPermission = platform ? `${platform}:${permission}` : permission
  
  return {
    hasPermission: isAuthenticated && hasPermission(user, fullPermission),
    loading: false,
  }
}

export function useRequireAnyPermission(permissions: string[]): PermissionCheck {
  const { user, isAuthenticated } = useAuth.getState()
  
  return {
    hasPermission: isAuthenticated && hasAnyPermission(user, permissions),
    loading: false,
  }
}

export function useRequireAdmin(): PermissionCheck {
  const { user, isAuthenticated } = useAuth.getState()
  
  return {
    hasPermission: isAuthenticated && isAdmin(user),
    loading: false,
  }
}

export function useRequirePlatformAccess(platform: string): PermissionCheck {
  const { user, isAuthenticated } = useAuth.getState()
  
  return {
    hasPermission: isAuthenticated && canAccessPlatform(user, platform),
    loading: false,
  }
}