/**
 * CANONICAL DOMAIN TYPES INDEX
 * Consolidated exports for all EPSX domain types
 * 
 * This replaces 200+ duplicate interface definitions across frontend and admin apps
 * with a single source of truth for all business domain concepts.
 */

// Import types for local use in union types
import type {
  AdminJWTPayload,
  APIKeyPayload,
  LoginResponse,
  RefreshTokenPayload,
  RegistrationResponse,
  UserJWTPayload,
  Web3AuthResponse
} from './auth'
import type {
  EPSXPermission,
  Permission,
  PermissionRole,
  PermissionTemplate
} from './permission'
import type {
  AdminSessionData,
  UserSessionData,
  WalletSessionData
} from './session'
import type {
  AdminUserProfile,
  BaseUser,
  UserProfile
} from './user'

// ============================================================================
// USER DOMAIN TYPES
// ============================================================================
export type {

  // Activity tracking
  ActivityRecord, AdminUserProfile,
  // Developer access
  ApiKey,
  // Core user types
  BaseUser,
  // Billing & subscriptions
  BillingStatus, BillingUpdateData,
  getUserTierLevel,
  LoginRecord,
  // Module access
  ModuleAccess,
  ModuleQuota, PackageTier, StockRankingPackage, SubscriptionTier, UnifiedUserData, UsageMetrics,
  // Legacy compatibility
  User,
  // API request/response types
  UserListFilters,
  UserListResponse, UserOperationError,
  UserOperationResult, UserProfile, UserProfileUpdateData, UserRole, UserRoleUpdateData, UserSession, SessionData as UserSessionData_Legacy, UserStatus, UserStatusUpdateData
} from './user'

// ============================================================================
// PERMISSION DOMAIN TYPES
// ============================================================================
export type {
  BulkPermissionOperation, EffectivePermissions,
  // Business domain types
  EPSXPermission, ParsedPermission,
  // Core permission types
  Permission, PermissionAnalytics, PermissionAssignment, PermissionCategory, PermissionCheck, PermissionExpiryDetails,
  PermissionExpiryInfo,
  // Permission context & inheritance
  PermissionInheritance, PermissionPolicy,
  // Management types
  PermissionRequest, PermissionRole, PermissionScope, PermissionSource, PermissionTemplate, PermissionValidation, Platform, PlatformPermissionContext, TimestampedPermission
} from './permission'

// ============================================================================
// SESSION DOMAIN TYPES
// ============================================================================
export type {
  AdminSession, AdminSessionData, BaseSessionData, getSessionDuration, getSessionTimeRemaining, isAdminSession,
  isSessionExpired,
  isSessionExpiringSoon,
  // Type guards & helpers
  isUserSession,
  // Analytics types
  SessionAnalytics,
  // Core session types
  SessionAppType, SessionCache, SessionCacheEntry,
  // Management types
  SessionCreateRequest,
  SessionCreateResponse, SessionData, SessionRefreshRequest,
  SessionRefreshResponse, SessionResult, SessionSecurityEvent,
  // Security types
  SessionSecurityEventType, SessionSecurityPolicy, SessionTerminateRequest,
  SessionTerminateResponse, SessionValidationRequest,
  // Validation types
  SessionValidationResponse,
  // Configuration types
  SessionValidatorConfig,
  SessionValidatorStats, UserActiveSessions,
  // Legacy compatibility
  UserSession as UserSession_Legacy, UserSessionData, UserSessionPattern, validateSessionRequest,
  // Wallet session types
  WalletAuthChallenge,
  WalletAuthVerification, WalletSessionData, WalletSessionStatus
} from './session'

// ============================================================================
// AUTHENTICATION DOMAIN TYPES
// ============================================================================
export type {
  AdminClaims, AdminJWTPayload, APIKeyPayload, AuthenticationContext,
  // Authentication state types
  AuthenticationState, AuthProviderConfig, AuthResponse, AuthResult, AuthState as AuthState_Legacy,
  // Core auth types (re-exported from shared system)
  User as AuthUser,
  // JWT token types
  BaseJWTPayload, EmailVerificationConfirmation, EmailVerificationRequest, EPSXJWTPayload, extractUserId, getTokenTimeRemaining, isAdminJWT, isAPIKeyToken, isRefreshToken, isTokenExpired,
  isTokenExpiringSoon,
  // Type guards & helpers
  isUserJWT,
  // Authentication request/response types
  LoginRequest,
  LoginResponse,
  OAuthAuthorizationRequest,
  OAuthTokenRequest,
  OAuthTokenResponse,
  // Configuration types
  OIDCClientConfig, OIDCUserInfo, PasswordChangeRequest, PasswordResetConfirmation,
  // Password & account management types
  PasswordResetRequest, RefreshTokenPayload, RegistrationRequest,
  RegistrationResponse, AuthState as SharedAuthState, SIWEMessage,
  // Multi-factor authentication types
  TwoFactorMethod,
  TwoFactorSetupRequest,
  TwoFactorSetupResponse,
  TwoFactorVerificationRequest,
  TwoFactorVerificationResponse,
  // Legacy compatibility
  UserClaims, UserJWTPayload, Web3AuthChallenge, Web3AuthResponse, Web3AuthVerification,
  // Web3 authentication types
  Web3ConnectRequest
} from './auth'

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
    Session: 'Usersession' as const,
  },
  Permission: {
    Core: 'EPSXpermission' as const,
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