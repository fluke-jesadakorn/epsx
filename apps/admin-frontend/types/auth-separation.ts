/**
 * ADMIN FRONTEND AUTH SEPARATION TYPES - MIGRATED TO SHARED
 * All authentication separation types moved to shared/types/auth-separation with compatibility layer
 * This file now re-exports shared types for backward compatibility
 */

// Re-export everything from shared auth-separation types
export * from '../../../shared/types/auth-separation';

// Import for local re-export with legacy names (maintaining compatibility)
import type {
  BaseJWTPayload as SharedBaseJWTPayload,
  AdminJWTPayload as SharedAdminJWTPayload,
  UserJWTPayload as SharedUserJWTPayload,
  AdminUserProfile as SharedAdminUserProfile,
  UserProfile as SharedUserProfile,
  AdminSessionData as SharedAdminSessionData,
  UserSessionData as SharedUserSessionData,
  SessionData as SharedSessionData,
  AuthenticatedUserProfile as SharedAuthenticatedUserProfile,
  SessionValidationResult as SharedSessionValidationResult,
  PermissionCheck as SharedPermissionCheck,
  SecurityContext as SharedSecurityContext,
  AuthConfig as SharedAuthConfig,
  TokenValidationOptions as SharedTokenValidationOptions,
  LegacyJWTPayload as SharedLegacyJWTPayload,
  MigrationResult as SharedMigrationResult
} from '../../../shared/types/auth-separation';

// Re-export with exact same names for backward compatibility
export type BaseJWTPayload = SharedBaseJWTPayload;
export type AdminJWTPayload = SharedAdminJWTPayload;
export type UserJWTPayload = SharedUserJWTPayload;
export type AdminUserProfile = SharedAdminUserProfile;
export type UserProfile = SharedUserProfile;
export type AdminSessionData = SharedAdminSessionData;
export type UserSessionData = SharedUserSessionData;
export type SessionData = SharedSessionData;
export type AuthenticatedUserProfile = SharedAuthenticatedUserProfile;
export type SessionValidationResult = SharedSessionValidationResult;
export type PermissionCheck = SharedPermissionCheck;
export type SecurityContext = SharedSecurityContext;
export type AuthConfig = SharedAuthConfig;
export type TokenValidationOptions = SharedTokenValidationOptions;
export type LegacyJWTPayload = SharedLegacyJWTPayload;
export type MigrationResult = SharedMigrationResult;