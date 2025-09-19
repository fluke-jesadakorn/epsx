'use client'

// ============================================================================
// FRONTEND GRANULAR PERMISSION GUARD - MIGRATION TO SHARED SYSTEM
// ============================================================================
// This file now uses the shared permission guard system for consistency

// Re-export the new shared permission guards
export {
  FrontendPermissionGuard as default,
  FrontendPermissionGuard,
  RequireFrontendPermission,
  RequireAnyFrontendPermission,
  RequireAllFrontendPermissions,
  RequireAnalyticsAccess,
  RequireExportAccess,
  RequireRealtimeAccess,
  RequireAdvancedFilters,
  RequireProfileManagement,
  RequireBillingAccess,
  RequireTierAccess,
  RequireRankingAccess,
  // Backward compatibility
  GranularPermissionGuard,
  RequirePermission,
  RequireAnyPermission,
  RequireAllPermissions
} from '@/lib/permissions'

// Legacy convenience exports for existing code
export {
  RequireFrontendPermission as RequireGranularPermission,
  RequireAnyFrontendPermission as RequireAnyGranularPermission,
  RequireAllFrontendPermissions as RequireAllGranularPermissions,
  RequireFrontendPermission as RequireGranularAccess,
  RequireFrontendPermission as RequireValidForDuration
} from '@/lib/permissions'