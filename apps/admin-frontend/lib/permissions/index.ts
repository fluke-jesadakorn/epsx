// ============================================================================
// ADMIN FRONTEND PERMISSIONS - MAIN EXPORTS
// ============================================================================
// Main export file for admin-specific permission system

// Re-export all shared permission utilities and types
export * from '@/shared/permissions'

// Export admin-specific implementations
export * from './types'
export * from './api-client'
export * from './hooks'
export * from './guards'

// Export commonly used admin permission utilities
export {
  isAdmin,
  canManageUsers,
  canManagePermissions,
  canViewAuditLogs,
  canManageSystem
} from '@/shared/permissions/utils'

// Export admin hooks
export {
  useAdminGranularPermissions,
  useAdminPermissionDashboard,
  useUserPermissionManagement,
  useAdminPermissions
} from './hooks'

// Export admin guards
export {
  AdminPermissionGuard,
  RequireAdminPermission,
  RequireAnyAdminPermission,
  RequireAllAdminPermissions,
  RequireUserManagementAccess,
  RequireSystemManagementAccess,
  RequirePermissionManagementAccess,
  RequireAnalyticsAccess,
  RequireFullAdminAccess,
  RequireAuditLogAccess,
  RequireStrictAdminAccess,
  RequireValidAdminPermissionFor
} from './guards'

// Export API client
export { adminPermissionApiClient } from './api-client'