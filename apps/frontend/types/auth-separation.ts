/**
 * FRONTEND AUTH SEPARATION TYPES - MIGRATED TO SHARED
 * All authentication separation types moved to shared/types/auth-separation with compatibility layer
 * This file now re-exports shared types for backward compatibility (user-focused)
 */

// Re-export everything from shared auth-separation types
export * from '../../../shared/types/auth-separation';

// Import for local re-export with legacy names (maintaining compatibility)
import type {
  UserJWTPayload as SharedUserJWTPayload,
  UserProfile as SharedUserProfile,
  UserSessionData as SharedUserSessionData,
  UserPermissionCheck as SharedUserPermissionCheck,
  PermissionValidation as SharedPermissionValidation,
  UserAnalyticsAccess as SharedUserAnalyticsAccess,
  UserTradingAccess as SharedUserTradingAccess
} from '../../../shared/types/auth-separation';

// Re-export with exact same names for backward compatibility (user-focused subset)
export type UserJWTPayload = SharedUserJWTPayload;
export type UserProfile = SharedUserProfile;
export type UserSessionData = SharedUserSessionData;
export type UserPermissionCheck = SharedUserPermissionCheck;
export type PermissionValidation = SharedPermissionValidation;
export type UserAnalyticsAccess = SharedUserAnalyticsAccess;
export type UserTradingAccess = SharedUserTradingAccess;