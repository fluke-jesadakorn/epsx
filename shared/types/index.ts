/**
 * SHARED TYPES INDEX
 * Unified export of all type definitions shared across EPSX applications
 * Replaces duplicate type definitions in frontend and admin-frontend apps
 */

// ============================================================================
// CONSOLIDATED TYPES (New)
// ============================================================================

// Payment types (consolidated from both apps)
export * from './payment';

// Authentication and authorization types (consolidated from both apps)
export * from './auth-separation';

// API types (consolidated from both apps)
export * from './api';

// Progressive authentication types (consolidated from both apps)
export * from './progressive-auth';

// Tier group types (unified tier and permission group system)
export * from './tier-groups';

// ============================================================================
// EXISTING SHARED TYPES
// ============================================================================

// Core auth types
export * from './auth';

// Domain types
export * from './domain';

// ============================================================================
// MIGRATION HELPERS
// ============================================================================

/**
 * Migration map for old type names to new consolidated types
 * Use this to find replacement types during migration
 */
export const TYPE_MIGRATION_MAP = {
  // Payment types
  'CreatePaymentRequest': 'CreatePaymentRequest', // Same name, consolidated location
  'PaymentResponse': 'PaymentResponse', // Same name, consolidated location
  'AssetInfo': 'AssetInfo', // Same name, consolidated location
  'UserSubscription': 'UserSubscription', // Same name, consolidated location
  
  // Auth types
  'AdminJWTPayload': 'AdminJWTPayload', // Same name, consolidated location
  'UserJWTPayload': 'UserJWTPayload', // Same name, consolidated location
  'AdminUserProfile': 'AdminUserProfile', // Same name, consolidated location
  'UserProfile': 'UserProfile', // Same name, consolidated location
  'SessionData': 'SessionData', // Same name, consolidated location
  'PermissionCheck': 'PermissionCheck', // Same name, consolidated location
} as const;

/**
 * Get the consolidated type name for a legacy type
 */
export function getConsolidatedType(legacyTypeName: keyof typeof TYPE_MIGRATION_MAP): string {
  return TYPE_MIGRATION_MAP[legacyTypeName];
}

/**
 * Check if a type has been consolidated
 */
export function isTypeConsolidated(typeName: string): boolean {
  return Object.keys(TYPE_MIGRATION_MAP).includes(typeName);
}

// ============================================================================
// CONVENIENCE EXPORTS
// ============================================================================

// Most commonly used types for quick access
export type { 
  // Payment
  CreatePaymentRequest,
  PaymentResponse,
  UserSubscription,
  PaymentStatusType,
} from './payment';

export type {
  // Auth
  UserProfile,
  AdminUserProfile,
  UserSessionData,
  AdminSessionData,
  SessionData,
  UserJWTPayload,
  AdminJWTPayload,
  PermissionCheck,
  SecurityContext,
} from './auth-separation';

export type {
  // API
  ApiResponse,
  ApiError,
  PaginatedResponse,
  LoginRequest,
  AuthTokens,
  NotificationMessage,
  AnalyticsRankingItem,
} from './api';

export type {
  // Progressive Auth
  AuthLevel,
  AuthState,
  ProgressiveAuthProps,
  AuthGateProps
} from './progressive-auth';

export type {
  // Tier Groups
  TierGroup,
  TierGroupRequest,
  UpdateTierGroupRequest,
  UserTierAssignment,
  CreateTierAssignmentRequest,
  UnifiedUserPermissions,
  TierGroupListResponse,
  TierGroupAnalytics,
  LegacyTierMappings
} from './tier-groups';

export type {
  BaseJWTPayload,
  AuthenticatedUserProfile,
  SessionValidationResult,
  PermissionValidation,
  AuthConfig,
  TokenValidationOptions,
  LegacyJWTPayload,
  MigrationResult
} from './auth-separation';