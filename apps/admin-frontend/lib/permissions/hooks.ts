// ============================================================================
// ADMIN FRONTEND PERMISSION HOOKS
// ============================================================================
// Admin-specific permission hooks that use shared permission logic

'use client'

import { useCallback, useState, useEffect } from 'react'
import { useAuth } from '@/lib/auth'
import {
  AdminPermissionHookResult,
  PermissionStatusResponse,
  UserPermissionOverview,
  GrantPermissionRequest,
  RevokePermissionRequest,
  BulkPermissionRequest,
  ExtendPermissionRequest,
  BulkOperationResult,
  PermissionAuditEntry,
  AdminPermissionDashboard,
  PermissionSearchFilters,
  PermissionTemplate,
  EnhancedUserClaims,
  GranularPermissionError
} from '@/shared/permissions/types'
import {
  hasPermissionGranular,
  hasAnyPermissionGranular,
  hasAllPermissionsGranular,
  getPermissionExpiryDetails,
  calculatePermissionHealth,
  isAdmin,
  canManageUsers,
  canManagePermissions,
  canViewAuditLogs,
  canManageSystem
} from '@/shared/permissions/utils'
import { adminPermissionApiClient } from './api-client'

// ============================================================================
// MAIN ADMIN PERMISSION HOOK
// ============================================================================

export function useAdminGranularPermissions(): AdminPermissionHookResult {
  const { user, isAuthenticated } = useAuth.getState()
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<any>(null)

  // Cast user to EnhancedUserClaims if available
  const enhancedUser = user as EnhancedUserClaims | null
  const permissions = enhancedUser?.permissions || {}

  // Check admin permissions using shared utilities
  const isAdminUser = isAdmin(enhancedUser)
  const canManagePermissionsAdmin = canManagePermissions(enhancedUser)
  const canManageUsersAdmin = canManageUsers(enhancedUser)
  const canViewAuditLogsAdmin = canViewAuditLogs(enhancedUser)
  const canManageSystemAdmin = canManageSystem(enhancedUser)

  // Core permission checking functions using shared utilities
  const hasPermission = useCallback((permission: string): boolean => {
    if (!enhancedUser || !isAuthenticated) return false
    return hasPermissionGranular(permissions, permission)
  }, [enhancedUser, permissions, isAuthenticated])

  const hasAnyPermission = useCallback((permissionList: string[]): boolean => {
    if (!enhancedUser || !isAuthenticated) return false
    return hasAnyPermissionGranular(permissions, permissionList)
  }, [enhancedUser, permissions, isAuthenticated])

  const hasAllPermissions = useCallback((permissionList: string[]): boolean => {
    if (!enhancedUser || !isAuthenticated) return false
    return hasAllPermissionsGranular(permissions, permissionList)
  }, [enhancedUser, permissions, isAuthenticated])

  const getPermissionExpiry = useCallback((permission: string) => {
    if (!enhancedUser) return null
    return getPermissionExpiryDetails(permissions, permission)
  }, [enhancedUser, permissions])

  const getPermissionHealth = useCallback(() => {
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
    if (!enhancedUser) return

    setLoading(true)
    setError(null)

    try {
      await adminPermissionApiClient.refreshUserToken()
      // Refresh auth state would be handled by auth system
    } catch (err) {
      setError(err)
      console.error('Failed to refresh permissions:', err)
    } finally {
      setLoading(false)
    }
  }, [enhancedUser])

  // Admin API functions
  const getUserPermissions = useCallback(async (userId: string): Promise<PermissionStatusResponse> => {
    if (!canManageUsersAdmin && !canViewAuditLogsAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      const response = await adminPermissionApiClient.getUserPermissions(userId)
      return response
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [canManageUsersAdmin, canViewAuditLogsAdmin])

  const getAllUsersWithPermissions = useCallback(async (filters?: PermissionSearchFilters): Promise<UserPermissionOverview[]> => {
    if (!canManageUsersAdmin && !canViewAuditLogsAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      const response = await adminPermissionApiClient.getAllUsersWithPermissions(filters)
      return response
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [canManageUsersAdmin, canViewAuditLogsAdmin])

  const grantPermission = useCallback(async (request: GrantPermissionRequest): Promise<void> => {
    if (!canManagePermissionsAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      await adminPermissionApiClient.grantPermission(request)
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [canManagePermissionsAdmin])

  const revokePermission = useCallback(async (request: RevokePermissionRequest): Promise<void> => {
    if (!canManagePermissionsAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      await adminPermissionApiClient.revokePermission(request)
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [canManagePermissionsAdmin])

  const bulkGrantPermissions = useCallback(async (request: BulkPermissionRequest): Promise<BulkOperationResult> => {
    if (!canManagePermissionsAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      const response = await adminPermissionApiClient.bulkGrantPermissions(request)
      return response
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [canManagePermissionsAdmin])

  const bulkRevokePermissions = useCallback(async (
    request: Omit<BulkPermissionRequest, 'expires_at' | 'source'>
  ): Promise<BulkOperationResult> => {
    if (!canManagePermissionsAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      const response = await adminPermissionApiClient.bulkRevokePermissions(request)
      return response
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [canManagePermissionsAdmin])

  const extendPermission = useCallback(async (request: ExtendPermissionRequest): Promise<void> => {
    if (!canManagePermissionsAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      await adminPermissionApiClient.extendPermission(request)
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [canManagePermissionsAdmin])

  const createPermissionTemplate = useCallback(async (
    template: Omit<PermissionTemplate, 'id' | 'created_at' | 'created_by'>
  ): Promise<PermissionTemplate> => {
    if (!canManagePermissionsAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      const response = await adminPermissionApiClient.createPermissionTemplate(template)
      return response
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [canManagePermissionsAdmin])

  const applyPermissionTemplate = useCallback(async (
    templateId: string, 
    userIds: string[]
  ): Promise<BulkOperationResult> => {
    if (!canManagePermissionsAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      const response = await adminPermissionApiClient.applyPermissionTemplate(templateId, userIds)
      return response
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [canManagePermissionsAdmin])

  const getDashboard = useCallback(async (): Promise<AdminPermissionDashboard> => {
    if (!isAdminUser) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      const response = await adminPermissionApiClient.getDashboard()
      return response
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [isAdminUser])

  const getPermissionAudit = useCallback(async (
    userId?: string, 
    limit: number = 100
  ): Promise<PermissionAuditEntry[]> => {
    if (!canViewAuditLogsAdmin) {
      throw new GranularPermissionError('Insufficient permissions', 'ADMIN_REQUIRED')
    }

    setLoading(true)
    try {
      const response = await adminPermissionApiClient.getPermissionAudit(userId, limit)
      return response
    } catch (err) {
      setError(err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [canViewAuditLogsAdmin])

  return {
    // Core permission functions
    hasPermission,
    hasAnyPermission,
    hasAllPermissions,
    getPermissionExpiry,
    getPermissionHealth,
    isPermissionExpiring,
    refreshPermissions,

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
      code: 'ADMIN_REQUIRED' as any,
      message: error.message,
      details: error.toString()
    } : null
  }
}

// ============================================================================
// SPECIALIZED ADMIN HOOKS
// ============================================================================

export function useAdminPermissionDashboard() {
  const adminPermissions = useAdminGranularPermissions()
  const [dashboard, setDashboard] = useState<AdminPermissionDashboard | null>(null)

  useEffect(() => {
    const loadDashboard = async () => {
      try {
        const data = await adminPermissions.getDashboard()
        setDashboard(data)
      } catch (err) {
        console.error('Failed to load admin dashboard:', err)
      }
    }

    loadDashboard()
  }, [adminPermissions])

  return {
    dashboard,
    refreshDashboard: () => adminPermissions.getDashboard().then(setDashboard),
    ...adminPermissions
  }
}

export function useUserPermissionManagement(userId: string) {
  const adminPermissions = useAdminGranularPermissions()
  const [userPermissions, setUserPermissions] = useState<PermissionStatusResponse | null>(null)

  useEffect(() => {
    const loadUserPermissions = async () => {
      try {
        const data = await adminPermissions.getUserPermissions(userId)
        setUserPermissions(data)
      } catch (err) {
        console.error('Failed to load user permissions:', err)
      }
    }

    if (userId) {
      loadUserPermissions()
    }
  }, [userId, adminPermissions])

  const refreshUserPermissions = useCallback(async () => {
    const data = await adminPermissions.getUserPermissions(userId)
    setUserPermissions(data)
    return data
  }, [userId, adminPermissions])

  return {
    userPermissions,
    refreshUserPermissions,
    ...adminPermissions
  }
}

// ============================================================================
// ADMIN PERMISSION HELPERS
// ============================================================================

export function useAdminPermissions() {
  const { hasPermission, hasAnyPermission } = useAdminGranularPermissions()

  return {
    hasPermission,
    hasAnyPermission,
    isAdmin: hasPermission('admin:*:*'),
    canManageUsers: hasAnyPermission(['admin:users:manage', 'epsx:users:manage']),
    canViewUsers: hasAnyPermission(['admin:users:read', 'admin:users:manage']),
    canManagePermissions: hasPermission('admin:permissions:manage'),
    canViewAuditLogs: hasAnyPermission(['admin:audit:read', 'admin:*:*']),
    canManageSystem: hasPermission('admin:system:manage')
  }
}

// Export the main hook as default
export { useAdminGranularPermissions as default }