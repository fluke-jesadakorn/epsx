/**
 * CONSOLIDATED AUTHENTICATION TYPES
 * Unified authentication and authorization types shared across applications
 * Supports both admin and user contexts with proper separation and security
 * Uses permission groups instead of legacy tier system
 */

import type { JWTPayload } from 'jose';
import type { PermissionGroup } from './domain/user';

// ============================================================================
// BASE AUTHENTICATION TYPES
// ============================================================================

export interface BaseJWTPayload extends JWTPayload {
  sub: string;
  email: string;
  name: string;
  iat: number;
  exp: number;
  iss: string;
  aud: string;
}

// ============================================================================
// ADMIN AUTHENTICATION TYPES (Enhanced Security)
// ============================================================================

export interface AdminJWTPayload extends BaseJWTPayload {
  // Token identification
  token_type: 'admin_access';
  jti: string; // JWT ID for tracking

  // Enhanced security context
  permissions: {
    system_access: {
      capabilities: string[]; // Structured permissions: "admin:resource:action"
      security_level: 'standard' | 'elevated' | 'critical';
      context_restrictions?: string[];
    };
    audit_trail: {
      login_method: 'oauth' | 'mfa' | 'sso';
      device_verified: boolean;
      ip_address: string;
      user_agent: string;
      sid: string;
    };
  };

  // Administrative context
  admin_context: {
    role: string;
    department?: string;
    supervisor?: string;
    clearance_level: number;
  };

  // Security features
  mfa_verified: boolean;
  device_binding: {
    device_id: string;
    device_fingerprint: string;
    trust_level: 'trusted' | 'verified' | 'unverified';
  };

  // Platform information
  platforms: string[];
  primary_platform: 'admin';
}

export interface AdminUserProfile {
  id: string;
  email: string;
  name: string;
  role: string;

  // Admin-specific permissions (structured)
  permissions: string[];
  securityLevel: 'standard' | 'elevated' | 'critical';

  // Admin metadata
  department?: string;
  clearanceLevel: number;
  lastLoginAt?: string;
  mfaEnabled: boolean;

  // Platform context
  platforms: string[];
  primaryPlatform: 'admin';

  // Permission group (derived from permissions)
  permission_group: PermissionGroup;
}

export interface AdminSessionData {
  user: AdminUserProfile;
  isLoggedIn: true;
  sessionType: 'admin';
  expiresAt: number;
  securityContext: {
    mfaVerified: boolean;
    deviceTrusted: boolean;
    securityLevel: string;
    sessionId: string;
  };
}

// ============================================================================
// USER AUTHENTICATION TYPES (Performance Optimized)
// ============================================================================

export interface UserJWTPayload extends BaseJWTPayload {
  // Token identification
  token_type: 'user_access';

  // Lightweight permissions structure
  permissions: {
    permissions: string[]; // Structured permissions: "epsx:resource:action"
    permission_group: PermissionGroup;
    expires_at?: number; // For time-limited permissions
  };

  // User context (minimal for performance)
  user_context: {
    permission_group: PermissionGroup;
    wallet_address?: string; // Web3 integration
    firebase_uid?: string;
    platform_preferences: string[];
  };

  // Platform information
  platforms: string[];
  primary_platform: string;
}

export interface UserProfile {
  id: string;
  email: string;
  name: string;
  role: 'user' | 'premium_user';

  // User-specific permissions (structured)
  permissions: string[];
  permission_group: PermissionGroup;

  // User metadata (lightweight)
  walletAddress?: string; // Web3 integration
  firebaseUid?: string;
  lastActivityAt?: string;

  // Platform context
  platforms: string[];
  primaryPlatform: string;
  platformContext?: string;
}

export interface UserSessionData {
  user: UserProfile;
  isLoggedIn: true;
  sessionType: 'user';
  expiresAt: number;
  platformContext: {
    currentPlatform: string;
    availablePlatforms: string[];
  };
}

// ============================================================================
// UNIFIED TYPES FOR COMPATIBILITY
// ============================================================================

export type SessionData = AdminSessionData | UserSessionData;
export type AuthenticatedUserProfile = AdminUserProfile | UserProfile;
export type AuthJWTPayload = AdminJWTPayload | UserJWTPayload;

export interface SessionValidationResult {
  valid: boolean;
  sessionType?: 'admin' | 'user';
  user?: AuthenticatedUserProfile;
  securityContext?: unknown;
  error?: string;
}

// ============================================================================
// PERMISSION SYSTEM TYPES
// ============================================================================

export interface PermissionCheck {
  permission: string;
  platform?: string;
  context?: 'admin' | 'user';
}

export interface UserPermissionCheck {
  permission: string;
  platform?: string;
  requiresSubscription?: boolean;
}

export interface PermissionValidation {
  hasPermission: boolean;
  reason?: 'valid' | 'no_permission' | 'permission_group_insufficient' | 'expired';
  requiredPermissionGroup?: PermissionGroup;
  upgradeUrl?: string;
}

export interface SecurityContext {
  sessionType: 'admin' | 'user';
  securityLevel?: string;
  mfaVerified?: boolean;
  deviceTrusted?: boolean;
  ipAddress?: string;
  userAgent?: string;
}

// ============================================================================
// ACCESS CONTROL TYPES
// ============================================================================

export interface UserAnalyticsAccess {
  canViewRankings: boolean;
  canExportData: boolean;
  maxStocksTracked: number;
  realTimeAccess: boolean;
}

export interface UserTradingAccess {
  paperTrading: boolean;
  liveTrading: boolean;
  advancedOrders: boolean;
}

// ============================================================================
// AUTHENTICATION CONFIGURATION TYPES
// ============================================================================

export interface AuthConfig {
  sessionType: 'admin' | 'user';
  requireMFA?: boolean;
  securityLevel?: string;
  platformRestrictions?: string[];
}

export interface TokenValidationOptions {
  requireFresh?: boolean;
  maxAge?: number;
  requiredSecurityLevel?: string;
  platformContext?: string;
}

// ============================================================================
// MIGRATION SUPPORT TYPES
// ============================================================================

export interface LegacyJWTPayload {
  uid: string;
  email: string;
  permissions: string[];
  admin_modules?: string[];
  permission_group?: PermissionGroup;
  firebase_uid?: string;
}

export interface MigrationResult {
  success: boolean;
  newTokenType: 'admin_access' | 'user_access';
  user: AuthenticatedUserProfile;
  securityUpgraded: boolean;
  warnings?: string[];
}

// ============================================================================
// TYPE GUARDS AND UTILITIES
// ============================================================================

export function isAdminSession(session: SessionData): session is AdminSessionData {
  return session.sessionType === 'admin';
}

export function isUserSession(session: SessionData): session is UserSessionData {
  return session.sessionType === 'user';
}

export function isAdminJWT(payload: AdminJWTPayload | UserJWTPayload): payload is AdminJWTPayload {
  return payload.token_type === 'admin_access';
}

export function isUserJWT(payload: AdminJWTPayload | UserJWTPayload | unknown): payload is UserJWTPayload {
  if (typeof payload !== 'object' || payload === null) { return false; }
  return (payload as Record<string, unknown>).token_type === 'user_access';
}

export function isAdminUser(user: AuthenticatedUserProfile): user is AdminUserProfile {
  return 'securityLevel' in user && 'clearanceLevel' in user;
}

export function isRegularUser(user: AuthenticatedUserProfile): user is UserProfile {
  return !('securityLevel' in user) && 'permission_group' in user;
}

export function hasValidSubscription(user: UserProfile): boolean {
  return user.permission_group !== 'Basic Access Group';
}

export function canAccessFeature(user: UserProfile, feature: string): boolean {
  return user.permissions.some(p =>
    p.includes(feature) ||
    p.includes('*') ||
    p === 'epsx:*:*'
  );
}

// ============================================================================
// SESSION MANAGEMENT UTILITIES
// ============================================================================

export function getSessionExpiryTime(session: SessionData): number {
  return session.expiresAt;
}

export function isSessionExpired(session: SessionData): boolean {
  return Date.now() > session.expiresAt;
}

export function getTimeUntilExpiry(session: SessionData): number {
  return Math.max(0, session.expiresAt - Date.now());
}

// ============================================================================
// SECURITY UTILITIES
// ============================================================================

export function getRequiredSecurityLevel(permission: string): 'standard' | 'elevated' | 'critical' {
  if (permission.includes('admin:system:') || permission.includes('admin:security:')) {
    return 'critical';
  }
  if (permission.includes('admin:users:') || permission.includes('admin:audit:')) {
    return 'elevated';
  }
  return 'standard';
}

export function validateSecurityContext(
  user: AdminUserProfile,
  requiredLevel: 'standard' | 'elevated' | 'critical'
): boolean {
  const levelOrder = { 'standard': 0, 'elevated': 1, 'critical': 2 };
  return levelOrder[user.securityLevel] >= levelOrder[requiredLevel];
}