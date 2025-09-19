// ============================================================================
// FRONTEND GRANULAR PERMISSION TYPES - MIGRATION TO SHARED SYSTEM
// ============================================================================
// This file now uses the shared permission types for consistency

// Re-export all shared permission types
export * from '@/shared/permissions/types'

// Frontend-specific extensions
export * from '@/lib/permissions/types'

// Legacy compatibility exports
export {
  PermissionSource,
  GranularPermissionClaim,
  GranularPermissionSet,
  EnhancedUserClaims,
  TokenValidationResult,
  HashValidationResult,
  PermissionStatusResponse,
  PermissionHealthInfo,
  GrantPermissionRequest,
  RevokePermissionRequest,
  BulkPermissionRequest,
  ExtendPermissionRequest,
  PermissionAuditEntry,
  ParsedPermission,
  PermissionExpiryDetails,
  UserPermissionSummary,
  GranularPermissionError,
  UsePermissionHookResult,
  PermissionApiClient
} from '@/shared/permissions/types'

// Legacy imports for backward compatibility
import { GranularPermissionError as SharedGranularPermissionError } from '@/shared/permissions/types'

// Re-export the shared error class for backward compatibility
export { SharedGranularPermissionError as PermissionError }