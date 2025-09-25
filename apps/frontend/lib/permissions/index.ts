// ============================================================================
// FRONTEND PERMISSIONS - MAIN EXPORTS
// ============================================================================
// Central export file for all frontend permission functionality

// Re-export shared permission system
export * from '@/shared/permissions'

// Frontend-specific types
export * from './types'

// Frontend-specific API client
export { 
  frontendPermissionApiClient,
  FrontendPermissionApiClientImpl 
} from './api-client'

// Frontend-specific hooks
export {
  useFrontendGranularPermissions,
  useAnalyticsPermissions,
  useProfilePermissions,
  useRankingPermissions,
  useRequirePermission,
  useRequireAnyPermission,
  useRequireAnalyticsAccess,
  useFeatureAccess,
  // Backward compatibility
  useGranularPermissions
} from './hooks'

// Frontend-specific guards
export {
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
} from './guards'

// Export main hook as default for convenience
export { useFrontendGranularPermissions as default } from './hooks'