// Client-side JWT Parser for UI Control Only
// WARNING: This is for UI rendering only - never use for authorization decisions

import React from 'react';
import { secureTokenRefreshManager } from './secure-token-refresh';

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
  can_refresh: boolean;          // Can be refreshed
  requires_mfa: boolean;         // MFA required
}

export interface PermissionInfo {
  permission: string;
  expires_at?: Date;
  is_temporal: boolean;
  is_wildcard: boolean;
  platform: string;
  resource: string;
  action: string;
}

export interface UserPermissionSummary {
  user_id: string;
  total_permissions: number;
  active_permissions: number;
  expired_permissions: number;
  roles: string[];
  security_level: number;
  platform_contexts: string[];
  permissions_by_platform: Record<string, PermissionInfo[]>;
  earliest_expiry?: Date;
  has_admin_access: boolean;
  has_elevated_privileges: boolean;
}

/**
 * Client-side JWT Parser
 * 
 * SECURITY WARNING: This is for UI control only!
 * - Never use for authorization decisions
 * - All security checks must be done server-side
 * - This is only for showing/hiding UI elements
 * - Token validation happens only on the backend
 * 
 * Features:
 * - Parse JWT claims for UI rendering
 * - Extract granular permissions
 * - Handle temporal permissions with expiry
 * - Provide permission checking helpers
 * - Monitor token health and expiry
 */
export class JWTParser {
  private claims: JWTClaims | null = null;
  private rawToken: string | null = null;
  private lastParsed: number = 0;
  
  /**
   * Parse JWT token and extract claims
   * Note: This does NOT validate the signature - validation happens server-side only
   */
  parseToken(token: string): JWTClaims | null {
    try {
      // Cache parsed token to avoid re-parsing
      if (this.rawToken === token && this.claims && Date.now() - this.lastParsed < 1000) {
        return this.claims;
      }
      
      const parts = token.split('.');
      if (parts.length !== 3) {
        console.warn('Invalid JWT format - not 3 parts');
        return null;
      }
      
      // Decode payload (base64url)
      const payload = parts[1].replace(/-/g, '+').replace(/_/g, '/');
      const paddedPayload = payload + '='.repeat((4 - payload.length % 4) % 4);
      
      const decoded = JSON.parse(atob(paddedPayload));
      
      // Validate required fields
      if (!decoded.sub || !decoded.exp) {
        console.warn('JWT missing required fields (sub, exp)');
        return null;
      }
      
      // Ensure permissions and roles are arrays
      const claims: JWTClaims = {
        sub: decoded.sub,
        email: decoded.email,
        name: decoded.name,
        permissions: Array.isArray(decoded.permissions) ? decoded.permissions : [],
        roles: Array.isArray(decoded.roles) ? decoded.roles : [],
        platform_context: decoded.platform_context || 'epsx',
        security_level: decoded.security_level || 1,
        device_fingerprint: decoded.device_fingerprint || '',
        granted_by: decoded.granted_by || 'system',
        exp: decoded.exp,
        iat: decoded.iat || 0,
        jti: decoded.jti || '',
        session_id: decoded.session_id || '',
        can_refresh: decoded.can_refresh !== false,
        requires_mfa: decoded.requires_mfa === true,
      };
      
      // Cache parsed result
      this.claims = claims;
      this.rawToken = token;
      this.lastParsed = Date.now();
      
      // Schedule token refresh if not already scheduled
      secureTokenRefreshManager.scheduleRefresh(token);
      
      return claims;
      
    } catch (error) {
      console.warn('Failed to parse JWT token:', error);
      return null;
    }
  }
  
  /**
   * Get current claims (cached)
   */
  getCurrentClaims(): JWTClaims | null {
    return this.claims;
  }
  
  /**
   * Check if user has specific permission (UI only!)
   */
  hasPermission(permission: string): boolean {
    if (!this.claims) return false;
    
    const activePermissions = this.getActivePermissions();
    
    // Check exact match
    if (activePermissions.some(p => p.permission === permission)) {
      return true;
    }
    
    // Check wildcard permissions
    return this.checkWildcardPermission(activePermissions, permission);
  }
  
  /**
   * Check if user has any of the specified permissions
   */
  hasAnyPermission(permissions: string[]): boolean {
    return permissions.some(permission => this.hasPermission(permission));
  }
  
  /**
   * Check if user has all specified permissions
   */
  hasAllPermissions(permissions: string[]): boolean {
    return permissions.every(permission => this.hasPermission(permission));
  }
  
  /**
   * Check if user has admin access
   */
  isAdmin(): boolean {
    if (!this.claims) return false;
    
    // Check admin role
    if (this.claims.roles.includes('admin')) return true;
    
    // Check admin wildcard permissions
    return this.hasPermission('admin:*:*');
  }
  
  /**
   * Check if user has elevated privileges on any platform
   */
  hasElevatedPrivileges(): boolean {
    if (!this.claims) return false;
    
    if (this.claims.roles.includes('admin')) return true;
    
    const activePermissions = this.getActivePermissions();
    return activePermissions.some(p => p.is_wildcard);
  }
  
  /**
   * Get permissions for specific platform
   */
  getPlatformPermissions(platform: string): PermissionInfo[] {
    const activePermissions = this.getActivePermissions();
    return activePermissions.filter(p => p.platform === platform);
  }
  
  /**
   * Get comprehensive user permission summary
   */
  getPermissionSummary(): UserPermissionSummary | null {
    if (!this.claims) return null;
    
    const allPermissionInfos = this.parsePermissionInfos(this.claims.permissions);
    const activePermissions = allPermissionInfos.filter(p => !this.isPermissionExpired(p));
    const expiredPermissions = allPermissionInfos.filter(p => this.isPermissionExpired(p));
    
    // Group by platform
    const permissionsByPlatform: Record<string, PermissionInfo[]> = {};
    activePermissions.forEach(p => {
      if (!permissionsByPlatform[p.platform]) {
        permissionsByPlatform[p.platform] = [];
      }
      permissionsByPlatform[p.platform].push(p);
    });
    
    // Find earliest expiry
    const expiryDates = activePermissions
      .map(p => p.expires_at)
      .filter((date): date is Date => date !== undefined);
    const earliestExpiry = expiryDates.length > 0 ? new Date(Math.min(...expiryDates.map(d => d.getTime()))) : undefined;
    
    // Get unique platforms
    const platformContexts = Array.from(new Set(activePermissions.map(p => p.platform)));
    
    return {
      user_id: this.claims.sub,
      total_permissions: allPermissionInfos.length,
      active_permissions: activePermissions.length,
      expired_permissions: expiredPermissions.length,
      roles: this.claims.roles,
      security_level: this.claims.security_level,
      platform_contexts: platformContexts,
      permissions_by_platform: permissionsByPlatform,
      earliest_expiry: earliestExpiry,
      has_admin_access: this.isAdmin(),
      has_elevated_privileges: this.hasElevatedPrivileges(),
    };
  }
  
  /**
   * Get token health information
   */
  getTokenHealth(): {
    is_valid: boolean;
    is_expired: boolean;
    expires_in_minutes: number;
    needs_refresh: boolean;
    security_level: number;
    device_bound: boolean;
  } | null {
    if (!this.claims) return null;
    
    const now = Date.now();
    const expiresAt = this.claims.exp * 1000;
    const expiresInMinutes = Math.max(0, Math.floor((expiresAt - now) / (1000 * 60)));
    
    return {
      is_valid: true, // We can only validate format, not signature
      is_expired: now >= expiresAt,
      expires_in_minutes: expiresInMinutes,
      needs_refresh: expiresInMinutes <= 5,
      security_level: this.claims.security_level,
      device_bound: this.claims.device_fingerprint.length > 0,
    };
  }
  
  /**
   * Clear cached token data
   */
  clearCache(): void {
    this.claims = null;
    this.rawToken = null;
    this.lastParsed = 0;
  }
  
  // Private helper methods
  
  private getActivePermissions(): PermissionInfo[] {
    if (!this.claims) return [];
    
    const permissionInfos = this.parsePermissionInfos(this.claims.permissions);
    return permissionInfos.filter(p => !this.isPermissionExpired(p));
  }
  
  private parsePermissionInfos(permissions: string[]): PermissionInfo[] {
    return permissions.map(perm => this.parsePermissionInfo(perm));
  }
  
  private parsePermissionInfo(permission: string): PermissionInfo {
    const parts = permission.split(':');
    
    // Check if last part is timestamp (temporal permission)
    let expires_at: Date | undefined;
    let is_temporal = false;
    let effectiveParts = parts;
    
    if (parts.length > 3) {
      const lastPart = parts[parts.length - 1];
      const timestamp = parseInt(lastPart);
      if (!isNaN(timestamp) && timestamp > 1000000000) { // Valid timestamp
        expires_at = new Date(timestamp * 1000);
        is_temporal = true;
        effectiveParts = parts.slice(0, -1); // Remove timestamp part
      }
    }
    
    const platform = effectiveParts[0] || 'unknown';
    const resource = effectiveParts[1] || 'unknown';
    const action = effectiveParts[2] || 'unknown';
    
    const is_wildcard = effectiveParts.includes('*');
    
    return {
      permission,
      expires_at,
      is_temporal,
      is_wildcard,
      platform,
      resource,
      action,
    };
  }
  
  private isPermissionExpired(permissionInfo: PermissionInfo): boolean {
    if (!permissionInfo.is_temporal || !permissionInfo.expires_at) {
      return false;
    }
    
    return Date.now() >= permissionInfo.expires_at.getTime();
  }
  
  private checkWildcardPermission(activePermissions: PermissionInfo[], requiredPermission: string): boolean {
    const requiredParts = requiredPermission.split(':');
    
    return activePermissions.some(p => {
      if (!p.is_wildcard) return false;
      
      const permParts = [p.platform, p.resource, p.action];
      
      if (permParts.length !== requiredParts.length) return false;
      
      return permParts.every((part, index) => 
        part === '*' || part === requiredParts[index]
      );
    });
  }
}

// Global instance
export const jwtParser = new JWTParser();

// React hook for easy use in components
export function useJWTParser(token?: string) {
  const [claims, setClaims] = React.useState<JWTClaims | null>(null);
  const [summary, setSummary] = React.useState<UserPermissionSummary | null>(null);
  const [health, setHealth] = React.useState<ReturnType<JWTParser['getTokenHealth']>>(null);
  
  React.useEffect(() => {
    if (token) {
      const parsedClaims = jwtParser.parseToken(token);
      setClaims(parsedClaims);
      setSummary(jwtParser.getPermissionSummary());
      setHealth(jwtParser.getTokenHealth());
    } else {
      setClaims(null);
      setSummary(null);
      setHealth(null);
    }
  }, [token]);
  
  return {
    claims,
    summary,
    health,
    hasPermission: (permission: string) => jwtParser.hasPermission(permission),
    hasAnyPermission: (permissions: string[]) => jwtParser.hasAnyPermission(permissions),
    hasAllPermissions: (permissions: string[]) => jwtParser.hasAllPermissions(permissions),
    isAdmin: () => jwtParser.isAdmin(),
    hasElevatedPrivileges: () => jwtParser.hasElevatedPrivileges(),
    getPlatformPermissions: (platform: string) => jwtParser.getPlatformPermissions(platform),
  };
}

// Permission Gate Component for conditional rendering
interface PermissionGateProps {
  permission?: string;
  permissions?: string[];
  requireAll?: boolean;
  adminOnly?: boolean;
  elevatedOnly?: boolean;
  platform?: string;
  children: React.ReactNode;
  fallback?: React.ReactNode;
}

export function PermissionGate({ 
  permission, 
  permissions = [], 
  requireAll = false,
  adminOnly = false,
  elevatedOnly = false,
  platform,
  children,
  fallback = null 
}: PermissionGateProps) {
  const hasAccess = React.useMemo(() => {
    if (adminOnly && !jwtParser.isAdmin()) return false;
    if (elevatedOnly && !jwtParser.hasElevatedPrivileges()) return false;
    
    if (permission) {
      return jwtParser.hasPermission(permission);
    }
    
    if (permissions.length > 0) {
      return requireAll 
        ? jwtParser.hasAllPermissions(permissions)
        : jwtParser.hasAnyPermission(permissions);
    }
    
    if (platform) {
      const platformPerms = jwtParser.getPlatformPermissions(platform);
      return platformPerms.length > 0;
    }
    
    return true; // No restrictions specified
  }, [permission, permissions, requireAll, adminOnly, elevatedOnly, platform]);
  
  return hasAccess ? <>{children}</> : <>{fallback}</>;
}

// Export React for the hook (if not already imported)
declare global {
  const React: typeof import('react');
}