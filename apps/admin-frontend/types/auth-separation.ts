/**
 * Separated Authentication Types for Admin/User Context
 * Supports the new separated JWT architecture with optimized interfaces
 */

import { JWTPayload } from 'jose';

// Base authentication interfaces
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
// Admin Authentication Types (Enhanced Security)
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
      session_id: string;
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
  
  // Package tier (derived from permissions)
  packageTier: 'ENTERPRISE';
}

// ============================================================================
// User Authentication Types (Performance Optimized)
// ============================================================================

export interface UserJWTPayload extends BaseJWTPayload {
  // Token identification
  token_type: 'user_access';
  
  // Lightweight permissions structure
  permissions: {
    permissions: string[]; // Structured permissions: "epsx:resource:action"
    package_tier: string;
    expires_at?: number; // For time-limited permissions
  };
  
  // User context (minimal for performance)
  user_context: {
    package_tier: string;
    firebase_uid?: string;
    platform_preferences: string[];
  };
  
  // Platform information
  platforms: string[];
  primary_platform: string;
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

export interface UserProfile {
  id: string;
  email: string;
  name: string;
  role: 'user' | 'premium_user';
  
  // User-specific permissions (structured)
  permissions: string[];
  packageTier: string;
  
  // User metadata (lightweight)
  firebaseUid?: string;
  lastActivityAt?: string;
  
  // Platform context
  platforms: string[];
  primaryPlatform: string;
  platformContext?: string;
}

// ============================================================================
// Unified Types for Compatibility
// ============================================================================

export type SessionData = AdminSessionData | UserSessionData;

export type AuthenticatedUserProfile = AdminUserProfile | UserProfile;

export interface SessionValidationResult {
  valid: boolean;
  sessionType?: 'admin' | 'user';
  user?: AuthenticatedUserProfile;
  securityContext?: any;
  error?: string;
}

// ============================================================================
// Permission System Types
// ============================================================================

export interface PermissionCheck {
  permission: string;
  platform?: string;
  context?: 'admin' | 'user';
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
// Authentication Utilities Types
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
// Migration Support Types
// ============================================================================

export interface LegacyJWTPayload {
  uid: string;
  email: string;
  permissions: string[];
  admin_modules?: string[];
  package_tier?: string;
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
// Type Guards
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

export function isUserJWT(payload: AdminJWTPayload | UserJWTPayload): payload is UserJWTPayload {
  return payload.token_type === 'user_access';
}

export function isAdminUser(user: AuthenticatedUserProfile): user is AdminUserProfile {
  return 'securityLevel' in user && 'clearanceLevel' in user;
}

export function isRegularUser(user: AuthenticatedUserProfile): user is UserProfile {
  return !('securityLevel' in user) && 'packageTier' in user;
}