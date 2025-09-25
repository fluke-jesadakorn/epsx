'use client'

// ============================================================================
// ADMIN FRONTEND GRANULAR PERMISSION GUARD - MIGRATION TO SHARED SYSTEM
// ============================================================================
// This file now uses the shared permission guard system for consistency

// Re-export the new shared permission guards
export {
  AdminPermissionGuard as default,
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
} from '@/lib/permissions'

// Backward compatibility exports
export { AdminPermissionGuard as AdminGranularPermissionGuard } from '@/lib/permissions'

// Legacy convenience exports for existing code
export {
  RequireUserManagementAccess as RequireUserManagementGranular,
  RequireSystemManagementAccess as RequireSystemManagementGranular,
  RequirePermissionManagementAccess as RequirePermissionManagementGranular,
  RequireAnalyticsAccess as RequireAnalyticsAccessGranular
} from '@/lib/permissions'