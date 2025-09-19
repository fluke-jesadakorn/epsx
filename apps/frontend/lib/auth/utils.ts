/**
 * Authentication Utilities
 * JWT parsing, permission validation, token refresh, and user helpers
 */

import { JWTPayload, jwtVerify, SignJWT } from 'jose';
import {
  UserSessionData,
  UserProfile,
  UserJWTPayload,
  PackageTier,
  UserPermissionCheck,
  PermissionValidation,
  UserAnalyticsAccess,
  UserTradingAccess,
  hasValidSubscription,
  canAccessFeature,
  isPremiumTier,
  isTrialUser
} from '@/types/auth-separation';
import {
  derivePackageTierFromPermissions,
  deriveAccessiblePlatformsFromPermissions,
  derivePrimaryPlatformFromPermissions 
} from '../../../../shared/permissions/utils/platform';
import { authLogger, safeError } from '@/lib/utils/logging';

// ============================================================================
// JWT Interfaces
// ============================================================================

export interface JWTUser {
  uid: string;
  email: string;
  permissions: string[];
  role?: string;
  iat?: number;
  exp?: number;
}

export interface CreateJWTClaimsOptions {
  id: string;
  email: string;
  name?: string;
  permissions: string[];
  role?: string;
}

export interface EPSXJWTPayload extends JWTPayload {
  sub: string;
  email: string;
  name: string;
  permissions: string[];
  iat: number;
  exp: number;
}

export interface JWTClaims {
  sub: string;                    // User ID
  email?: string;                 // User email
  name?: string;                  // Display name
  permissions: string[];          // User permissions
  roles: string[];               // User roles
  platform_context: string;      // Platform context (epsx, admin)
  security_level: number;        // Security level (1-5)
  device_fingerprint: string;    // Device binding
  granted_by: string;            // Who granted permissions
  exp: number;                   // Expiry timestamp
  iat: number;                   // Issued at timestamp
  jti: string;                   // JWT ID
  session_id: string;            // Session ID
}

// ============================================================================
// Token Refresh Interfaces
// ============================================================================

interface RefreshConfig {
  refreshThresholdMinutes: number;  // Minutes before expiry to refresh
  maxRetries: number;               // Maximum refresh attempts
  backoffMultiplier: number;        // Exponential backoff base (ms)
  monitoringEnabled: boolean;       // Enable security monitoring
}

interface TokenPayload {
  sub: string;
  exp: number;
  iat: number;
  permissions?: string[];
  device_fingerprint?: string;
  jti?: string;
}

interface RefreshResponse {
  access_token: string;
  expires_in: number;
  success: boolean;
  error?: string;
}

interface SecurityMetrics {
  refreshAttempts: number;
  failedRefreshes: number;
  lastSuccessfulRefresh: number;
  suspiciousActivity: boolean;
  deviceFingerprint: string;
}

// ============================================================================
// JWT Utilities
// ============================================================================

/**
 * Derive accessible platforms from permissions
 */
export function getAccessiblePlatforms(permissions: string[]): string[] {
  return deriveAccessiblePlatformsFromPermissions(permissions);
}

/**
 * Derive package tier from permissions
 */
export function getPackageTier(permissions: string[]): string {
  return derivePackageTierFromPermissions(permissions);
}

/**
 * Derive primary platform from permissions
 */
export function getPrimaryPlatform(permissions: string[]): string {
  return derivePrimaryPlatformFromPermissions(permissions);
}

/**
 * Client-side JWT parsing for UI control only
 * WARNING: This is for UI rendering only - never use for authorization decisions
 */
export function parseJWTForUI(token: string): JWTClaims | null {
  try {
    if (!token || typeof token !== 'string') {
      return null;
    }

    // Basic JWT format validation
    const parts = token.split('.');
    if (parts.length !== 3) {
      return null;
    }

    // Decode payload (no verification - UI only)
    const payload = JSON.parse(atob(parts[1].replace(/-/g, '+').replace(/_/g, '/')));
    
    // Validate required fields
    if (!payload.sub || !payload.exp || !payload.iat) {
      return null;
    }

    return {
      sub: payload.sub,
      email: payload.email,
      name: payload.name,
      permissions: payload.permissions || [],
      roles: payload.roles || [],
      platform_context: payload.platform_context || 'epsx',
      security_level: payload.security_level || 1,
      device_fingerprint: payload.device_fingerprint || '',
      granted_by: payload.granted_by || 'system',
      exp: payload.exp,
      iat: payload.iat,
      jti: payload.jti || '',
      session_id: payload.session_id || ''
    };
  } catch (error) {
    authLogger.warn('Failed to parse JWT for UI', { error: safeError(error).message });
    return null;
  }
}

/**
 * Check if JWT token is expired
 */
export function isTokenExpired(token: string): boolean {
  const claims = parseJWTForUI(token);
  if (!claims) return true;
  
  return Date.now() >= claims.exp * 1000;
}

/**
 * Get time until JWT expires (in milliseconds)
 */
export function getTimeToExpiry(token: string): number {
  const claims = parseJWTForUI(token);
  if (!claims) return 0;
  
  return (claims.exp * 1000) - Date.now();
}

// ============================================================================
// Secure Token Refresh Manager
// ============================================================================

class SecureTokenRefreshManager {
  private config: RefreshConfig = {
    refreshThresholdMinutes: 5,
    maxRetries: 3,
    backoffMultiplier: 1000,
    monitoringEnabled: true
  };

  private metrics: SecurityMetrics = {
    refreshAttempts: 0,
    failedRefreshes: 0,
    lastSuccessfulRefresh: 0,
    suspiciousActivity: false,
    deviceFingerprint: this.generateDeviceFingerprint()
  };

  private refreshTimer: NodeJS.Timeout | null = null;
  private refreshPromise: Promise<RefreshResponse> | null = null;

  constructor(config?: Partial<RefreshConfig>) {
    if (config) {
      this.config = { ...this.config, ...config };
    }
  }

  /**
   * Generate device fingerprint for security
   */
  private generateDeviceFingerprint(): string {
    if (typeof window === 'undefined') return 'server';
    
    const components = [
      navigator.userAgent,
      navigator.language,
      screen.width + 'x' + screen.height,
      new Date().getTimezoneOffset(),
      navigator.hardwareConcurrency || 0
    ];
    
    return btoa(components.join('|')).substring(0, 16);
  }

  /**
   * Start automatic token refresh monitoring
   */
  public startAutoRefresh(): void {
    if (this.refreshTimer) return;

    this.refreshTimer = setInterval(async () => {
      await this.checkAndRefreshToken();
    }, 60000); // Check every minute
  }

  /**
   * Stop automatic token refresh
   */
  public stopAutoRefresh(): void {
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer);
      this.refreshTimer = null;
    }
  }

  /**
   * Check if token needs refresh and refresh if necessary
   */
  private async checkAndRefreshToken(): Promise<void> {
    try {
      const response = await fetch('/api/auth/token-status', {
        credentials: 'include'
      });

      if (!response.ok) return;

      const { needsRefresh, expiresIn } = await response.json();

      if (needsRefresh && expiresIn < this.config.refreshThresholdMinutes * 60) {
        await this.refreshToken();
      }
    } catch (error) {
      authLogger.error('Failed to check token status', { error: safeError(error).message });
    }
  }

  /**
   * Refresh the authentication token
   */
  public async refreshToken(): Promise<RefreshResponse> {
    // Prevent multiple simultaneous refresh attempts
    if (this.refreshPromise) {
      return this.refreshPromise;
    }

    this.refreshPromise = this.performRefresh();
    
    try {
      const result = await this.refreshPromise;
      return result;
    } finally {
      this.refreshPromise = null;
    }
  }

  /**
   * Perform the actual token refresh
   */
  private async performRefresh(): Promise<RefreshResponse> {
    let attempt = 0;
    
    while (attempt < this.config.maxRetries) {
      try {
        this.metrics.refreshAttempts++;
        
        const response = await fetch('/api/auth/refresh', {
          method: 'POST',
          credentials: 'include',
          headers: {
            'Content-Type': 'application/json',
            'X-Device-Fingerprint': this.metrics.deviceFingerprint
          }
        });

        if (!response.ok) {
          throw new Error(`Refresh failed with status: ${response.status}`);
        }

        const data = await response.json();
        
        this.metrics.lastSuccessfulRefresh = Date.now();
        this.metrics.suspiciousActivity = false;

        authLogger.info('Token refreshed successfully');

        return {
          access_token: data.access_token,
          expires_in: data.expires_in,
          success: true
        };

      } catch (error) {
        attempt++;
        this.metrics.failedRefreshes++;

        const errorMsg = safeError(error).message;
        authLogger.error(`Token refresh attempt ${attempt} failed`, { error: errorMsg });

        if (attempt >= this.config.maxRetries) {
          this.metrics.suspiciousActivity = true;
          return {
            access_token: '',
            expires_in: 0,
            success: false,
            error: errorMsg
          };
        }

        // Exponential backoff
        await new Promise(resolve => 
          setTimeout(resolve, this.config.backoffMultiplier * Math.pow(2, attempt - 1))
        );
      }
    }

    return {
      access_token: '',
      expires_in: 0,
      success: false,
      error: 'Max retry attempts exceeded'
    };
  }

  /**
   * Get current security metrics
   */
  public getSecurityMetrics(): SecurityMetrics {
    return { ...this.metrics };
  }

  /**
   * Reset security metrics
   */
  public resetMetrics(): void {
    this.metrics = {
      refreshAttempts: 0,
      failedRefreshes: 0,
      lastSuccessfulRefresh: 0,
      suspiciousActivity: false,
      deviceFingerprint: this.generateDeviceFingerprint()
    };
  }
}

// Export singleton instance
export const secureTokenRefreshManager = new SecureTokenRefreshManager();

// ============================================================================
// User Permission Helpers
// ============================================================================

/**
 * Check if user has specific permission with subscription validation
 */
export function hasUserPermission(
  user: UserSessionData | null,
  permission: string,
  options: UserPermissionCheck = {}
): PermissionValidation {
  if (!user) {
    return {
      hasPermission: false,
      reason: 'User not authenticated',
      requiresUpgrade: false
    };
  }

  // Check subscription status first
  if (options.requiresSubscription && !hasValidSubscription(user)) {
    return {
      hasPermission: false,
      reason: 'Valid subscription required',
      requiresUpgrade: true,
      upgradeUrl: '/upgrade'
    };
  }

  // Check permission
  const hasPermission = user.permissions.includes(permission);

  if (!hasPermission) {
    // Check if this is a premium feature
    const isPremiumFeature = permission.includes('premium') || permission.includes('pro');
    
    return {
      hasPermission: false,
      reason: isPremiumFeature ? 'Premium feature requires upgrade' : 'Permission denied',
      requiresUpgrade: isPremiumFeature,
      upgradeUrl: isPremiumFeature ? '/upgrade' : undefined
    };
  }

  return {
    hasPermission: true,
    reason: 'Permission granted'
  };
}

/**
 * Check if user can access analytics features
 */
export function canAccessAnalytics(user: UserSessionData | null): UserAnalyticsAccess {
  if (!user) {
    return {
      canAccess: false,
      level: 'none',
      features: [],
      reason: 'Authentication required'
    };
  }

  const permissions = user.permissions;
  const features: string[] = [];
  let level: 'none' | 'basic' | 'premium' | 'professional' = 'none';

  // Basic analytics
  if (permissions.includes('epsx:analytics:view')) {
    level = 'basic';
    features.push('view-rankings', 'basic-filters');
  }

  // Premium analytics
  if (permissions.includes('epsx:analytics:premium')) {
    level = 'premium';
    features.push('advanced-filters', 'export-data', 'historical-data');
  }

  // Professional analytics
  if (permissions.includes('epsx:analytics:professional')) {
    level = 'professional';
    features.push('real-time-data', 'api-access', 'custom-reports');
  }

  return {
    canAccess: level !== 'none',
    level,
    features,
    reason: level !== 'none' ? 'Access granted' : 'No analytics permissions'
  };
}

/**
 * Check if user can access trading features
 */
export function canAccessTrading(user: UserSessionData | null): UserTradingAccess {
  if (!user) {
    return {
      canAccess: false,
      level: 'none',
      features: [],
      reason: 'Authentication required'
    };
  }

  const permissions = user.permissions;
  const features: string[] = [];
  let level: 'none' | 'basic' | 'premium' | 'professional' = 'none';

  // Basic trading
  if (permissions.includes('epsx:trading:view')) {
    level = 'basic';
    features.push('view-prices', 'basic-charts');
  }

  // Premium trading
  if (permissions.includes('epsx:trading:premium')) {
    level = 'premium';
    features.push('advanced-charts', 'indicators', 'alerts');
  }

  // Professional trading
  if (permissions.includes('epsx:trading:professional')) {
    level = 'professional';
    features.push('real-time-trading', 'api-access', 'advanced-tools');
  }

  return {
    canAccess: level !== 'none',
    level,
    features,
    reason: level !== 'none' ? 'Access granted' : 'No trading permissions'
  };
}