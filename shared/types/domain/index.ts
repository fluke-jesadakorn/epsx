/**
 * CANONICAL DOMAIN TYPES INDEX
 * Consolidated exports for all EPSX domain types
 * 
 * This replaces 200+ duplicate interface definitions across frontend and admin apps
 * with a single source of truth for all business domain concepts.
 */

// ============================================================================
// USER DOMAIN TYPES
// ============================================================================
export {
  // Core user types
  BaseUser,
  UserRole,
  UserStatus,
  PackageTier,
  UserProfile,
  UserSession,
  AdminUserProfile,
  
  // Business access types
  UserAnalyticsAccess,
  UserTradingAccess,
  
  // Billing & subscriptions
  BillingStatus,
  StockRankingPackage,
  
  // Module access
  ModuleAccess,
  ModuleQuota,
  
  // Developer access
  ApiKey,
  
  // Activity tracking
  ActivityRecord,
  LoginRecord,
  UsageMetrics,
  
  // API request/response types
  UserListFilters,
  UserListResponse,
  UserProfileUpdateData,
  UserStatusUpdateData,
  UserRoleUpdateData,
  BillingUpdateData,
  UserOperationError,
  UserOperationResult,
  
  // Type guards & helpers
  isAdminUser,
  isPremiumUser,
  hasValidSubscription,
  canAccessFeature,
  getUserTierLevel,
  hasMinimumTier,
  
  // Legacy compatibility
  User,
  UnifiedUserData,
  SessionData as UserSessionData_Legacy,
  SubscriptionTier
} from './User'

// ============================================================================
// PERMISSION DOMAIN TYPES
// ============================================================================
export {
  // Core permission types (re-exported from shared system)
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
  MigrationStatus,
  
  // Claims types
  EnhancedUserClaims,
  PermissionClaims,
  AdminPermissionClaims,
  CrossPlatformClaims,
  EmbeddedPermissionClaims,
  
  // API types
  PermissionAPIRequest,
  PermissionAPIResponse,
  PermissionValidationRequest,
  PermissionValidationResponse,
  PermissionUpdateRequest,
  PermissionUpdateResponse,
  PermissionListRequest,
  PermissionListResponse,
  PermissionSyncRequest,
  PermissionSyncResponse,
  PermissionRevokeRequest,
  PermissionRevokeResponse,
  BulkPermissionRequest,
  BulkPermissionResponse,
  PermissionImportRequest,
  PermissionImportResponse,
  PermissionExportRequest,
  PermissionExportResponse,
  
  // Error types
  PermissionError,
  ValidationError,
  ExpiryError,
  CacheError,
  SyncError,
  ImportError,
  ExportError,
  PermissionErrorContext,
  
  // Audit types
  PermissionAuditLog,
  PermissionChangeEvent,
  PermissionRevocationEvent,
  PermissionExpiryEvent,
  PermissionBulkEvent,
  PermissionImportEvent,
  PermissionSystemEvent,
  
  // Business domain types
  EPSXPermission,
  PermissionTemplate,
  PermissionCategory,
  PermissionScope,
  PermissionCheck,
  PermissionValidation,
  PermissionRole,
  EffectivePermissions,
  PermissionAssignment,
  BulkPermissionOperation,
  PermissionAnalytics,
  
  // Feature-specific permissions
  AnalyticsPermissions,
  TradingPermissions,
  AdminPermissions,
  APIPermissions,
  
  // Permission context & inheritance
  PermissionInheritance,
  PlatformPermissionContext,
  PermissionConstraint,
  
  // Management types
  PermissionRequest,
  PermissionPolicy,
  
  // Helper functions
  parsePermission,
  isWildcardPermission,
  matchesPermissionPattern,
  resolveUserPermissions,
  hasEffectivePermission,
  
  // Legacy compatibility
  PermissionProfile,
  PermissionResult,
  PermissionString
} from './Permission'

// ============================================================================
// SESSION DOMAIN TYPES
// ============================================================================
export {
  // Core session types
  SessionAppType,
  SessionValidationRequest,
  BaseSessionData,
  UserSessionData,
  AdminSessionData,
  WalletSessionData,
  SessionData,
  
  // Validation types
  SessionValidationResponse,
  SessionCacheEntry,
  SessionRefreshRequest,
  SessionRefreshResponse,
  
  // Management types
  SessionCreateRequest,
  SessionCreateResponse,
  SessionTerminateRequest,
  SessionTerminateResponse,
  UserActiveSessions,
  
  // Security types
  SessionSecurityEventType,
  SessionSecurityEvent,
  SessionSecurityPolicy,
  
  // Wallet session types
  WalletAuthChallenge,
  WalletAuthVerification,
  WalletSessionStatus,
  
  // Analytics types
  SessionAnalytics,
  UserSessionPattern,
  
  // Configuration types
  SessionValidatorConfig,
  SessionValidatorStats,
  
  // Type guards & helpers
  isUserSession,
  isAdminSession,
  isSessionExpired,
  isSessionExpiringSoon,
  getSessionTimeRemaining,
  getSessionDuration,
  validateSessionRequest,
  
  // Legacy compatibility
  UserSession as UserSession_Legacy,
  AdminSession,
  SessionResult,
  SessionCache
} from './Session'

// ============================================================================
// AUTHENTICATION DOMAIN TYPES
// ============================================================================
export {
  // Core auth types (re-exported from shared system)
  User as AuthUser,
  AuthState as SharedAuthState,
  AuthConfig,
  AuthError,
  AuthResponse,
  OIDCConfig,
  FirebaseConfig,
  
  // JWT token types
  BaseJWTPayload,
  UserJWTPayload,
  AdminJWTPayload,
  RefreshTokenPayload,
  APIKeyPayload,
  EPSXJWTPayload,
  
  // Authentication request/response types
  LoginRequest,
  LoginResponse,
  OAuthAuthorizationRequest,
  OAuthTokenRequest,
  OAuthTokenResponse,
  OIDCUserInfo,
  
  // Web3 authentication types
  Web3ConnectRequest,
  Web3AuthChallenge,
  SIWEMessage,
  Web3AuthVerification,
  Web3AuthResponse,
  
  // Firebase authentication types
  FirebaseCustomClaims,
  FirebaseTokenExchange,
  FirebaseTokenExchangeResponse,
  
  // Multi-factor authentication types
  TwoFactorMethod,
  TwoFactorSetupRequest,
  TwoFactorSetupResponse,
  TwoFactorVerificationRequest,
  TwoFactorVerificationResponse,
  
  // Password & account management types
  PasswordResetRequest,
  PasswordResetConfirmation,
  PasswordChangeRequest,
  EmailVerificationRequest,
  EmailVerificationConfirmation,
  RegistrationRequest,
  RegistrationResponse,
  
  // Authentication state types
  AuthenticationState,
  AuthenticationContext,
  
  // Configuration types
  OIDCClientConfig,
  AuthProviderConfig,
  
  // Type guards & helpers
  isUserJWT,
  isAdminJWT,
  isRefreshToken,
  isAPIKeyToken,
  extractUserId,
  isTokenExpired,
  isTokenExpiringSoon,
  getTokenTimeRemaining,
  
  // Legacy compatibility
  UserClaims,
  AdminClaims,
  AuthState as AuthState_Legacy,
  AuthResult
} from './Auth'

// ============================================================================
// CONVENIENCE TYPE UNIONS
// ============================================================================

/**
 * All user-related types
 */
export type AnyUserType = UserProfile | AdminUserProfile | BaseUser

/**
 * All session-related types  
 */
export type AnySessionType = UserSessionData | AdminSessionData | WalletSessionData

/**
 * All JWT payload types
 */
export type AnyJWTPayload = UserJWTPayload | AdminJWTPayload | RefreshTokenPayload | APIKeyPayload

/**
 * All permission-related types
 */
export type AnyPermissionType = Permission | EPSXPermission | PermissionTemplate | PermissionRole

/**
 * All authentication response types
 */
export type AnyAuthResponse = LoginResponse | Web3AuthResponse | RegistrationResponse

// ============================================================================
// DOMAIN TYPE CATEGORIES
// ============================================================================

/**
 * All core domain types organized by category
 */
export const DomainTypes = {
  User: {
    Core: 'UserProfile' as const,
    Admin: 'AdminUserProfile' as const,
    Session: 'UserSession' as const,
    Analytics: 'UserAnalyticsAccess' as const,
    Trading: 'UserTradingAccess' as const
  },
  Permission: {
    Core: 'EPSXPermission' as const,
    Template: 'PermissionTemplate' as const,
    Role: 'PermissionRole' as const,
    Validation: 'PermissionValidation' as const,
    Effective: 'EffectivePermissions' as const
  },
  Session: {
    User: 'UserSessionData' as const,
    Admin: 'AdminSessionData' as const,
    Wallet: 'WalletSessionData' as const,
    Validation: 'SessionValidationResponse' as const
  },
  Auth: {
    JWT: 'EPSXJWTPayload' as const,
    Login: 'LoginResponse' as const,
    Web3: 'Web3AuthResponse' as const,
    State: 'AuthenticationState' as const
  }
} as const

/**
 * Type for accessing domain type names
 */
export type DomainTypeName = typeof DomainTypes[keyof typeof DomainTypes][keyof typeof DomainTypes[keyof typeof DomainTypes]]