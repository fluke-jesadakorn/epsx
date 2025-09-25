'use client'

// ============================================================================
// FRONTEND GRANULAR PERMISSIONS - MIGRATION TO SHARED SYSTEM
// ============================================================================
// This file now uses the shared permission system for consistency

// Re-export the new shared permission system
export * from '@/lib/permissions'

// For backward compatibility, export the main frontend hook as default
export { 
  useFrontendGranularPermissions as default,
  useFrontendGranularPermissions as useGranularPermissions,
  useAnalyticsPermissions,
  useProfilePermissions,
  useRankingPermissions,
  useRequirePermission,
  useRequireAnyPermission,
  useRequireAnalyticsAccess,
  useFeatureAccess
} from '@/lib/permissions'

// Legacy exports for backward compatibility
export { 
  useAnalyticsPermissions as useAdminPermissions, // Map old admin hook to analytics
  useRequireAnalyticsAccess as useRequireAdmin
} from '@/lib/permissions'