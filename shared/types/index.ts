/**
 * SHARED TYPES INDEX
 * Unified export of all type definitions shared across EPSX applications
 * Replaces duplicate type definitions in frontend and admin-frontend apps
 */

// ============================================================================
// CONSOLIDATED TYPES (New)
// ============================================================================

// Payment types (consolidated from both apps)
// export * from './payment';

// Authentication and authorization types (consolidated from both apps)
export * from './auth-separation';

// API types (consolidated from both apps)
export * from './api';

// Progressive authentication types (consolidated from both apps)
export * from './progressive-auth';

// Wallet types
export * from './analytics';
export * from './notifications';
export * from './wallet';
export * from './credits';

// Existing shared types
// export * from './auth';

// Domain types - Commented out to avoid duplicate exports with ./api and ./auth-separation
// export * from './domain';

// ============================================================================
// MIGRATION HELPERS
// ============================================================================

// ============================================================================
// CONVENIENCE EXPORTS
// ============================================================================

// Most commonly used types for quick access
export type {
  // Payment
  CreatePaymentRequest,
  PaymentResponse, PaymentStatusType, UserSubscription
} from './payment';

export type {
  AdminJWTPayload, AdminSessionData, AdminUserProfile, PermissionCheck,
  SecurityContext, SessionData,
  UserJWTPayload,
  // Auth
  UserProfile, UserSessionData
} from './auth-separation';

export type {
  ApiError,
  // API
  ApiResponse, AuthTokens, LoginRequest, PaginatedResponse
} from './api';

export type {
  AnalyticsRankingItem
} from './analytics';

export type {
  NotificationWSMessage
} from './notifications';

export type {
  AuthGateProps,
  // Progressive Auth
  AuthLevel,
  AuthState,
  ProgressiveAuthProps
} from './progressive-auth';

export type {
  AuthConfig, AuthenticatedUserProfile, BaseJWTPayload,
  PermissionValidation, SessionValidationResult, TokenValidationOptions
} from './auth-separation';

export type {
  CreditBalance,
  CreditTransaction,
  CreditTransactionType,
  CreditTransactionFilters,
  CreditStats,
  GrantCreditsRequest,
  RevokeCreditsRequest,
  CreditHistoryResponse,
  UpgradePreviewWithCredits
} from './credits';

