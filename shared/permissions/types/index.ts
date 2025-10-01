// ============================================================================
// SHARED PERMISSION TYPES - MAIN EXPORTS
// ============================================================================
// Using explicit exports to avoid naming conflicts between modules

// Core types
export type {
  Permission,
  ParsedPermission,
  PermissionSource,
  Platform,
  GranularPermissionClaim,
  GranularPermissionSet,
  TimestampedPermission,
  PermissionExpiryDetails,
  PermissionExpiryInfo,
  PermissionHealthInfo,
  UserPermissionSummary,
  PermissionCacheEntry,
  HashValidationResult,
  TokenValidationResult,
  LegacyPermissionMapping,
  MigrationStatus
} from './core'

// Claims types
export * from './claims'

// API types (excluding duplicates)
export type {
  GrantPermissionRequest,
  RevokePermissionRequest,
  BulkPermissionRequest,
  ExtendPermissionRequest,
  PermissionSearchFilters,
  PermissionValidationRequest,
  PermissionStatusResponse,
  UserPermissionOverview,
  BulkOperationResult,
  PermissionValidationResponse,
  SystemHealthResponse,
  PermissionApiClient,
  AdminPermissionApiClient,
  PaginatedResponse,
  PaginatedUsersResponse,
  PaginatedAuditResponse,
  NotificationEventType,
  PermissionNotificationData,
  BulkOperationNotificationData,
  PermissionNotificationEvent,
  AdminPermissionDashboard
} from './api'

// Audit types (primary sources for shared types)
export * from './audit'

// Error types
export * from './errors'