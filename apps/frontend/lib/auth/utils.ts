/**
 * Authentication Utilities
 * JWT parsing, permission validation, token refresh, and user helpers
 * Re-exports shared JWT utilities and adds frontend-specific extensions
 */

// Re-export shared JWT types and utilities
import { getDisplayTierFromPermissions, getRankingLimitFromPermissions } from '@/app/constants/packages';
import { authLogger, safeError } from '@/lib/utils/logging';

export {
  decodeJWT, getJWTPermissions, getJWTTimeToExpiry, hasJWTPermission,
  isJWTAdmin, isJWTExpired, type CreateJWTClaimsOptions,
  type EPSXJWTPayload, type JWTUser
} from '@/shared/auth/jwt';

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
  sid: string;            // Session ID
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
  const platforms = new Set<string>();

  for (const permission of permissions) {
    const platform = permission.split(':')[0];
    if (platform) {
      platforms.add(platform);
    }
  }

  return Array.from(platforms);
}

/**
 * Get display tier from permissions (replaces package tier)
 */
export function getDisplayTier(permissions: string[]): string {
  return getDisplayTierFromPermissions(permissions);
}

/**
 * Get primary platform from permissions
 */
export function getPrimaryPlatform(permissions: string[]): string {
  const platforms = getAccessiblePlatforms(permissions);

  // Priority order: admin > epsx > others
  if (platforms.includes('admin')) {return 'admin';}
  if (platforms.includes('epsx')) {return 'epsx';}

  return platforms[0] || 'epsx';
}

/**
 * Get ranking limit from permissions
 */
export function getRankingLimit(permissions: string[]): number {
  return getRankingLimitFromPermissions(permissions);
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
      permissions: payload.permissions ?? [],
      roles: payload.roles ?? [],
      platform_context: payload.platform_context ?? 'epsx',
      security_level: payload.security_level ?? 1,
      device_fingerprint: payload.device_fingerprint ?? '',
      granted_by: payload.granted_by ?? 'system',
      exp: payload.exp,
      iat: payload.iat,
      jti: payload.jti ?? '',
      sid: payload.sid ?? ''
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
  if (!claims) {return true;}

  return Date.now() >= claims.exp * 1000;
}

/**
 * Get time until JWT expires (in milliseconds)
 */
export function getTimeToExpiry(token: string): number {
  const claims = parseJWTForUI(token);
  if (!claims) {return 0;}

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
    if (typeof window === 'undefined') {return 'server';}

    const components = [
      navigator.userAgent,
      navigator.language,
      `${screen.width  }x${  screen.height}`,
      new Date().getTimezoneOffset(),
      navigator.hardwareConcurrency || 0
    ];

    return btoa(components.join('|')).slice(0, 16);
  }

  /**
   * Start automatic token refresh monitoring
   */
  public startAutoRefresh(): void {
    if (this.refreshTimer) {return;}

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

      if (!response.ok) {return;}

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
      return await this.refreshPromise;
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

// Export functions that are used in other modules
export { getRankingLimitFromPermissions } from '@/app/constants/packages';
