'use client'

// ============================================================================
// ADMIN FRONTEND GRANULAR PERMISSIONS - MIGRATION TO SHARED SYSTEM
// ============================================================================
// This file now uses the shared permission system for consistency

// Re-export the new shared permission system
export * from '@/lib/permissions'

// For backward compatibility, export the main admin hook as default
export { 
  useAdminGranularPermissions as default,
  useAdminGranularPermissions,
  useAdminPermissionDashboard,
  useUserPermissionManagement,
  useAdminPermissions
} from '@/lib/permissions'