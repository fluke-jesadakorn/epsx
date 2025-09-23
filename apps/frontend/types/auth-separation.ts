/**
 * User-Focused Authentication Types for Main Frontend
 * Minimal set of types actually used in the codebase
 */

import { JWTPayload } from 'jose';

// ============================================================================
// Core Authentication Types (Actually Used)
// ============================================================================

export interface UserJWTPayload extends JWTPayload {
  sub: string;
  email: string;
  name: string;
  iat: number;
  exp: number;
  iss: string;
  aud: string;
  token_type: 'user_access';
  permissions: {
    permissions: string[];
    package_tier: string;
    expires_at?: number;
  };
  user_context: {
    package_tier: string;
    firebase_uid?: string;
    platform_preferences: string[];
  };
  platforms: string[];
  primary_platform: string;
}

export interface UserProfile {
  id: string;
  email: string;
  name: string;
  role: 'user' | 'premium_user';
  permissions: string[];
  packageTier: string;
  firebaseUid?: string;
  lastActivityAt?: string;
  platforms: string[];
  primaryPlatform: string;
  platformContext?: string;
}

export interface UserSessionData {
  user: UserProfile;
  isLoggedIn: true;
  expiresAt: number;
  platformContext: {
    currentPlatform: string;
    availablePlatforms: string[];
  };
}

export interface UserPermissionCheck {
  permission: string;
  platform?: string;
  requiresSubscription?: boolean;
}

export interface PermissionValidation {
  hasPermission: boolean;
  reason?: 'valid' | 'no_permission' | 'tier_insufficient' | 'expired';
  requiredTier?: string;
  upgradeUrl?: string;
}

// Simplified access interfaces (used in auth utils)
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
// Type Guards and Helpers (Actually Used)
// ============================================================================

export function isUserJWT(payload: any): payload is UserJWTPayload {
  return payload?.token_type === 'user_access';
}

export function hasValidSubscription(user: UserProfile): boolean {
  return user.packageTier !== 'FREE';
}

export function canAccessFeature(user: UserProfile, feature: string): boolean {
  return user.permissions.some(p => 
    p.includes(feature) || 
    p.includes('*') ||
    p === 'epsx:*:*'
  );
}