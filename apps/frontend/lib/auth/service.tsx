/**
 * Permission-Centric Web3-First Authentication Service
 * Core philosophy: Permissions are the product, Web3 wallet is the identity
 * Provides sophisticated permission-aware authentication with granular access control
 */

import React from 'react';
import { authConfig, clientConfig } from '@/config/env';
import { logger as authLogger, safeError } from '@/lib/shared';
import { 
  isJWTExpired, 
  getJWTTimeToExpiry,
  derivePackageTierFromPermissions,
  deriveAccessiblePlatformsFromPermissions,
  // ⚠️ DEPRECATED: Local validation functions removed for security
  // hasPermissionGranular,
  // hasAnyPermissionGranular, 
  // hasAllPermissionsGranular,
  canViewAnalytics,
  canExportData,
  canAccessRealtime,
  canUseAdvancedFilters,
  isAdmin,
  getExpiringPermissions,
  getPermissionHealth,
  EnhancedUserClaims,
  GranularPermissionClaim,
  PermissionHealth
} from '@/lib/shared';

// 🔒 SECURITY CRITICAL: Import backend permission authority
import { BackendPermissionAuthorityClient } from '@/lib/permissions/backend-authority-client';

// Permission-centric user interface - Web3-first
export interface PermissionAwareUser {
  // Identity (Web3-first)
  id: string; // Wallet address or user ID
  walletAddress?: string; // Primary identity for Web3 users
  email?: string; // Optional for Web3-only users
  name?: string;
  photoURL?: string;
  emailVerified: boolean;
  
  // Core value proposition - Permissions are the product
  permissions: Record<string, GranularPermissionClaim>; // Granular permission system
  permissionHealth: PermissionHealth; // Health monitoring
  
  // Derived permission-based attributes
  packageTier: string; // Derived from permissions
  accessiblePlatforms: string[]; // Derived from permissions
  role: string; // Derived from permissions
  
  // Analytics and access tracking
  featureAccess: {
    analytics: boolean;
    export: boolean;
    realtime: boolean;
    advancedFilters: boolean;
    admin: boolean;
  };
  
  // Timestamps
  createdAt: string;
  lastLogin: string;
  expiresAt?: number;
  
  // Web3 specific
  authMethod: 'web3' | 'oidc';
}

// Backend user data interface (replaces 'any' type)
interface BackendUserData {
  id?: string;
  uid?: string;
  wallet_address?: string;
  email?: string;
  name?: string;
  displayName?: string;
  photoURL?: string;
  emailVerified?: boolean;
  role?: string;
  granular_permissions?: Record<string, GranularPermissionClaim>;
  createdAt?: string;
  lastLogin?: string;
  expiresAt?: number;
}

// Permission-aware authentication context
export interface PermissionAuthContextValue {
  user: PermissionAwareUser | null;
  loading: boolean;
  error: string | null;
  
  // Authentication methods
  connectWallet: () => Promise<void>; // Primary Web3 login
  logout: () => Promise<void>;
  
  // Permission management
  refreshUser: () => Promise<void>;
  refreshPermissions: () => Promise<void>;
  
  // 🔒 SECURITY CRITICAL: Permission checking now uses backend authority
  // These methods are now ASYNC and call the backend permission authority
  hasPermission: (permission: string) => Promise<boolean>;
  hasAnyPermission: (permissions: string[]) => Promise<boolean>;
  hasAllPermissions: (permissions: string[]) => Promise<boolean>;
  
  // Legacy synchronous methods (DEPRECATED - use async versions)
  hasPermissionSync: (permission: string) => boolean; // Local fallback only
  
  // Feature access helpers
  canAccessAnalytics: () => boolean;
  canExportData: () => boolean;
  canAccessRealtime: () => boolean;
  canUseAdvancedFilters: () => boolean;
  isAdmin: () => boolean;
  
  // Permission health
  getExpiringPermissions: () => Array<{ permission: string; expiresIn: number }>;
  getPermissionHealth: () => PermissionHealth;
}

// Default permission-aware context
const defaultAuthContext: PermissionAuthContextValue = {
  user: null,
  loading: true,
  error: null,
  connectWallet: async () => {
    throw new Error('PermissionAuth context not initialized');
  },
  logout: async () => {
    throw new Error('PermissionAuth context not initialized');
  },
  refreshUser: async () => {
    throw new Error('PermissionAuth context not initialized');
  },
  refreshPermissions: async () => {
    throw new Error('PermissionAuth context not initialized');
  },
  // 🔒 SECURITY CRITICAL: Async permission methods now call backend authority
  hasPermission: async () => false, // Fail closed for security
  hasAnyPermission: async () => false, // Fail closed for security
  hasAllPermissions: async () => false, // Fail closed for security
  
  // Legacy synchronous fallback (DEPRECATED)
  hasPermissionSync: () => false, // Fail closed for security
  
  canAccessAnalytics: () => false,
  canExportData: () => false,
  canAccessRealtime: () => false,
  canUseAdvancedFilters: () => false,
  isAdmin: () => false,
  getExpiringPermissions: () => [],
  getPermissionHealth: () => ({ score: 0, status: 'unknown', expiringPermissions: [], expiredPermissions: [] })
};

// Create Permission-aware React context
export const PermissionAuthContext = React.createContext<PermissionAuthContextValue>(defaultAuthContext);

// Backward compatibility
export const AuthContext = PermissionAuthContext;

// Permission-centric authentication service
export class PermissionAwareAuthService {
  private static instance: PermissionAwareAuthService;
  private currentUser: PermissionAwareUser | null = null;
  private listeners: Set<(user: PermissionAwareUser | null) => void> = new Set();
  private permissionRefreshInterval: NodeJS.Timeout | null = null;
  
  // 🔒 SECURITY CRITICAL: Backend permission authority client
  private backendPermissionClient: BackendPermissionAuthorityClient;

  private constructor() {
    // 🔒 SECURITY CRITICAL: Initialize backend permission authority client
    this.backendPermissionClient = new BackendPermissionAuthorityClient();
  }

  static getInstance(): PermissionAwareAuthService {
    if (!PermissionAwareAuthService.instance) {
      PermissionAwareAuthService.instance = new PermissionAwareAuthService();
    }
    return PermissionAwareAuthService.instance;
  }

  // Subscribe to auth state changes
  subscribe(callback: (user: PermissionAwareUser | null) => void): () => void {
    this.listeners.add(callback);
    return () => this.listeners.delete(callback);
  }

  // Notify all listeners of state changes
  private notifyListeners(): void {
    this.listeners.forEach(callback => callback(this.currentUser));
  }

  // Get current user
  getCurrentUser(): PermissionAwareUser | null {
    return this.currentUser;
  }

  // Start permission monitoring for expiry warnings
  private startPermissionMonitoring(): void {
    if (this.permissionRefreshInterval) {
      clearInterval(this.permissionRefreshInterval);
    }
    
    // Check for expiring permissions every 5 minutes
    this.permissionRefreshInterval = setInterval(() => {
      if (this.currentUser) {
        const expiringPermissions = this.getExpiringPermissions();
        if (expiringPermissions.length > 0) {
          authLogger.warn('Permissions expiring soon', { 
            expiringPermissions: expiringPermissions.map(p => p.permission)
          });
          // Could trigger UI notifications here
        }
      }
    }, 5 * 60 * 1000); // 5 minutes
  }

  // Stop permission monitoring
  private stopPermissionMonitoring(): void {
    if (this.permissionRefreshInterval) {
      clearInterval(this.permissionRefreshInterval);
      this.permissionRefreshInterval = null;
    }
  }

  // Load user with Web3-first approach and permission focus
  async loadUser(): Promise<PermissionAwareUser | null> {
    try {
      // Try Web3 authentication first
      let response = await fetch('/api/auth/current-user', {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json'
        }
      });

      // Fallback to traditional user endpoint if needed
      if (!response.ok && response.status === 404) {
        response = await fetch('/api/auth/user', {
          credentials: 'include',
          headers: {
            'Content-Type': 'application/json'
          }
        });
      }

      if (!response.ok) {
        if (response.status === 401) {
          this.currentUser = null;
          this.stopPermissionMonitoring();
          this.notifyListeners();
          return null;
        }
        throw new Error('Failed to load user data');
      }

      const userData = await response.json();
      const user = this.transformToPermissionAwareUser(userData);
      
      this.currentUser = user;
      this.startPermissionMonitoring(); // Start monitoring permission health
      this.notifyListeners();
      
      authLogger.info('Permission-aware user loaded successfully', { 
        userId: user.id,
        walletAddress: user.walletAddress,
        authMethod: user.authMethod,
        permissionCount: Object.keys(user.permissions).length,
        permissionHealth: user.permissionHealth.score
      });
      
      return user;
    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Failed to load user', { error: errorMsg });
      
      this.currentUser = null;
      this.stopPermissionMonitoring();
      this.notifyListeners();
      return null;
    }
  }

  // Transform backend data to permission-aware user format
  private transformToPermissionAwareUser(userData: BackendUserData): PermissionAwareUser {
    // Use granular permissions format (OIDC migration complete)
    const permissions: Record<string, GranularPermissionClaim> = userData.granular_permissions || {};
    
    // Calculate permission health
    const permissionHealth = this.calculatePermissionHealth(permissions);
    
    // Determine auth method
    const authMethod = userData.wallet_address ? 'web3' : 'oidc';
    
    return {
      // Identity (Web3-first)
      id: userData.id || userData.uid || userData.wallet_address,
      walletAddress: userData.wallet_address,
      email: userData.email,
      name: userData.name || userData.displayName,
      photoURL: userData.photoURL,
      emailVerified: userData.emailVerified || false,
      
      // Core permissions (the product)
      permissions,
      permissionHealth,
      
      // Derived attributes
      packageTier: derivePackageTierFromPermissions(Object.keys(permissions)),
      accessiblePlatforms: deriveAccessiblePlatformsFromPermissions(Object.keys(permissions)),
      role: userData.role || (isAdmin({ permissions } as EnhancedUserClaims) ? 'admin' : 'user'),
      
      // Feature access (calculated from permissions)
      featureAccess: {
        analytics: canViewAnalytics({ permissions } as EnhancedUserClaims),
        export: canExportData({ permissions } as EnhancedUserClaims),
        realtime: canAccessRealtime({ permissions } as EnhancedUserClaims),
        advancedFilters: canUseAdvancedFilters({ permissions } as EnhancedUserClaims),
        admin: isAdmin({ permissions } as EnhancedUserClaims)
      },
      
      // Timestamps
      createdAt: userData.createdAt || new Date().toISOString(),
      lastLogin: userData.lastLogin || new Date().toISOString(),
      expiresAt: userData.expiresAt,
      
      // Web3 specific
      authMethod
    };
  }
  
  // Calculate permission health score
  private calculatePermissionHealth(permissions: Record<string, GranularPermissionClaim>): PermissionHealth {
    const now = Date.now() / 1000;
    const expiringPermissions: Array<{ permission: string; expiresIn: number }> = [];
    const expiredPermissions: string[] = [];
    let totalPermissions = Object.keys(permissions).length;
    let healthyPermissions = 0;
    
    for (const [permission, claim] of Object.entries(permissions)) {
      if (!claim.expires_at) {
        // Permanent permission is healthy
        healthyPermissions++;
      } else {
        const expiresIn = (claim.expires_at * 1000) - Date.now();
        const hoursUntilExpiry = expiresIn / (1000 * 60 * 60);
        
        if (claim.expires_at <= now) {
          // Expired
          expiredPermissions.push(permission);
        } else if (hoursUntilExpiry <= 24) {
          // Expiring within 24 hours
          expiringPermissions.push({ permission, expiresIn });
          healthyPermissions += 0.5; // Partially healthy
        } else {
          // Healthy
          healthyPermissions++;
        }
      }
    }
    
    const score = totalPermissions > 0 ? Math.round((healthyPermissions / totalPermissions) * 100) : 100;
    let status: PermissionHealth['status'] = 'healthy';
    
    if (expiredPermissions.length > 0) {
      status = 'critical';
    } else if (expiringPermissions.length > 0) {
      status = 'warning';
    }
    
    return {
      score,
      status,
      expiringPermissions,
      expiredPermissions
    };
  }

  // Primary Web3 wallet connection
  async connectWallet(): Promise<void> {
    try {
      authLogger.info('Initiating Web3 wallet connection');
      
      // Trigger Web3 modal/connection flow
      // This will be handled by the Web3 components
      const walletConnectEvent = new CustomEvent('epsx:connect-wallet');
      window.dispatchEvent(walletConnectEvent);
      
    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Web3 wallet connection failed', { error: errorMsg });
      throw new Error(errorMsg);
    }
  }
  
  // Web3 authentication is the primary method - no fallback needed

  // Logout user (clears Web3 sessions and wallet connections)
  async logout(): Promise<void> {
    try {
      // Clear Web3 session
      await fetch('/api/auth/web3/logout', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json'
        }
      });

      this.currentUser = null;
      this.stopPermissionMonitoring();
      this.notifyListeners();
      
      authLogger.info('User logged out successfully (Web3 session and permissions cleared)');
      
      // Trigger wallet disconnect event
      const walletDisconnectEvent = new CustomEvent('epsx:disconnect-wallet');
      window.dispatchEvent(walletDisconnectEvent);
      
      // Redirect to home
      window.location.href = '/';

    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Web3 logout failed', { error: errorMsg });
      throw new Error(errorMsg);
    }
  }

  // Refresh Web3 session and permissions
  async refreshToken(): Promise<boolean> {
    try {
      const response = await fetch('/api/auth/web3/refresh', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json'
        }
      });

      if (!response.ok) {
        if (response.status === 401) {
          // Web3 session expired, logout user
          await this.logout();
          return false;
        }
        throw new Error('Web3 session refresh failed');
      }

      const userData = await response.json();
      const user = this.transformToPermissionAwareUser(userData);
      
      this.currentUser = user;
      this.notifyListeners();
      
      authLogger.info('Web3 session and permissions refreshed successfully', {
        walletAddress: user.walletAddress,
        permissionCount: Object.keys(user.permissions).length,
        permissionHealth: user.permissionHealth.score
      });
      return true;

    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Web3 session refresh failed', { error: errorMsg });
      return false;
    }
  }
  
  // Refresh only permissions (for permission health monitoring)
  async refreshPermissions(): Promise<void> {
    if (!this.currentUser) return;
    
    try {
      const response = await fetch('/api/auth/web3/permissions', {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json'
        }
      });
      
      if (!response.ok) {
        throw new Error('Failed to refresh permissions');
      }
      
      const permissionData = await response.json();
      
      // Update user permissions
      const updatedUser = {
        ...this.currentUser,
        permissions: permissionData.granular_permissions || {},
        permissionHealth: this.calculatePermissionHealth(permissionData.granular_permissions || {})
      };
      
      // Recalculate feature access
      updatedUser.featureAccess = {
        analytics: canViewAnalytics({ permissions: updatedUser.permissions } as EnhancedUserClaims),
        export: canExportData({ permissions: updatedUser.permissions } as EnhancedUserClaims),
        realtime: canAccessRealtime({ permissions: updatedUser.permissions } as EnhancedUserClaims),
        advancedFilters: canUseAdvancedFilters({ permissions: updatedUser.permissions } as EnhancedUserClaims),
        admin: isAdmin({ permissions: updatedUser.permissions } as EnhancedUserClaims)
      };
      
      this.currentUser = updatedUser;
      this.notifyListeners();
      
      authLogger.info('Permissions refreshed successfully', {
        permissionCount: Object.keys(updatedUser.permissions).length,
        permissionHealth: updatedUser.permissionHealth.score
      });
      
    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Permission refresh failed', { error: errorMsg });
      throw new Error(errorMsg);
    }
  }

  // Check if user is authenticated
  isAuthenticated(): boolean {
    return this.currentUser !== null;
  }

  // 🔒 SECURITY CRITICAL: Permission checking now uses backend permission authority
  // ⚡ THE SINGLE SOURCE OF TRUTH: All permission validation through backend API
  async hasPermission(permission: string): Promise<boolean> {
    if (!this.currentUser?.id) {
      authLogger.warn('hasPermission called without authenticated user');
      return false; // Fail closed for security
    }
    
    try {
      const result = await this.backendPermissionClient.validatePermission(
        this.currentUser.id,
        permission
      );
      
      authLogger.debug('Backend permission validation result', {
        userId: this.currentUser.id,
        permission,
        granted: result.granted,
        reason: result.reason
      });
      
      return result.granted;
    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Backend permission validation failed - failing closed', {
        userId: this.currentUser.id,
        permission,
        error: errorMsg
      });
      return false; // Fail closed for security
    }
  }

  // Check if user has any of the specified permissions (backend authority)
  async hasAnyPermission(permissions: string[]): Promise<boolean> {
    if (!this.currentUser?.id) {
      authLogger.warn('hasAnyPermission called without authenticated user');
      return false; // Fail closed for security
    }
    
    if (permissions.length === 0) return false;
    
    try {
      // Use bulk validation for efficiency
      const bulkRequest = {
        user_id: this.currentUser.id,
        permissions: permissions.map(permission => ({ permission }))
      };
      
      const result = await this.backendPermissionClient.validateBulkPermissions(bulkRequest);
      const hasAnyGranted = result.results.some(r => r.granted);
      
      authLogger.debug('Backend bulk permission validation result (any)', {
        userId: this.currentUser.id,
        permissions,
        hasAnyGranted,
        results: result.results
      });
      
      return hasAnyGranted;
    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Backend bulk permission validation failed - failing closed', {
        userId: this.currentUser.id,
        permissions,
        error: errorMsg
      });
      return false; // Fail closed for security
    }
  }

  // Check if user has all specified permissions (backend authority)
  async hasAllPermissions(permissions: string[]): Promise<boolean> {
    if (!this.currentUser?.id) {
      authLogger.warn('hasAllPermissions called without authenticated user');
      return false; // Fail closed for security
    }
    
    if (permissions.length === 0) return true; // Vacuously true
    
    try {
      // Use bulk validation for efficiency
      const bulkRequest = {
        user_id: this.currentUser.id,
        permissions: permissions.map(permission => ({ permission }))
      };
      
      const result = await this.backendPermissionClient.validateBulkPermissions(bulkRequest);
      const hasAllGranted = result.results.every(r => r.granted);
      
      authLogger.debug('Backend bulk permission validation result (all)', {
        userId: this.currentUser.id,
        permissions,
        hasAllGranted,
        results: result.results
      });
      
      return hasAllGranted;
    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Backend bulk permission validation failed - failing closed', {
        userId: this.currentUser.id,
        permissions,
        error: errorMsg
      });
      return false; // Fail closed for security
    }
  }

  // 🔒 DEPRECATED: Legacy synchronous permission checking (local fallback only)
  // ⚠️  WARNING: This method is INSECURE and should not be used for security decisions
  // Use async hasPermission() method instead for secure backend validation
  hasPermissionSync(permission: string): boolean {
    if (!this.currentUser) return false;
    
    console.warn(`
⚠️  SECURITY WARNING: Using DEPRECATED synchronous permission checking!

Permission: ${permission}
User: ${this.currentUser.id}

PROBLEM: 
- Local validation is HACKABLE
- Client-side permission checks are INSECURE
- This method bypasses backend permission authority

SOLUTION:
- Use async hasPermission() method instead
- All security decisions should use backend validation
- Update your code to handle async permission checking

This warning will be removed when local validation is fully deprecated.
`);
    
    // Return based on existing local permissions (cached from backend)
    // This is still insecure but provides temporary compatibility
    try {
      // Use existing feature access as fallback
      const featureMap: Record<string, boolean> = {
        'epsx:analytics:view': this.currentUser.featureAccess.analytics,
        'epsx:analytics:basic': this.currentUser.featureAccess.analytics,
        'epsx:analytics:premium': this.currentUser.featureAccess.analytics,
        'epsx:export:csv': this.currentUser.featureAccess.export,
        'epsx:export:excel': this.currentUser.featureAccess.export,
        'epsx:realtime:access': this.currentUser.featureAccess.realtime,
        'epsx:filters:advanced': this.currentUser.featureAccess.advancedFilters,
        'admin:users:manage': this.currentUser.featureAccess.admin,
        'admin:system:manage': this.currentUser.featureAccess.admin,
      };
      
      return featureMap[permission] || false;
    } catch (error) {
      authLogger.error('Legacy permission fallback failed', { permission, error: safeError(error).message });
      return false; // Fail closed
    }
  }

  // Feature access helpers (the core value proposition)
  canAccessAnalytics(): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.featureAccess.analytics;
  }
  
  canExportData(): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.featureAccess.export;
  }
  
  canAccessRealtime(): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.featureAccess.realtime;
  }
  
  canUseAdvancedFilters(): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.featureAccess.advancedFilters;
  }
  
  isAdmin(): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.featureAccess.admin;
  }

  // Permission health monitoring
  getExpiringPermissions(): Array<{ permission: string; expiresIn: number }> {
    if (!this.currentUser) return [];
    return this.currentUser.permissionHealth.expiringPermissions;
  }
  
  getPermissionHealth(): PermissionHealth {
    if (!this.currentUser) {
      return { score: 0, status: 'unknown', expiringPermissions: [], expiredPermissions: [] };
    }
    return this.currentUser.permissionHealth;
  }

  // Legacy compatibility
  getPackageTier(): string {
    return this.currentUser?.packageTier || 'free';
  }

  getAccessiblePlatforms(): string[] {
    return this.currentUser?.accessiblePlatforms || [];
  }
}

// Export permission-aware singleton instance
export const permissionAuthService = PermissionAwareAuthService.getInstance();

// Backward compatibility
export const authService = permissionAuthService;

// React hook for using permission-aware auth service
export function usePermissionAuth(): PermissionAuthContextValue {
  const context = React.useContext(PermissionAuthContext);
  if (!context) {
    throw new Error('usePermissionAuth must be used within PermissionAuthProvider');
  }
  return context;
}

// Backward compatibility aliases
export const useUnifiedAuth = usePermissionAuth;
export const useAuth = usePermissionAuth;

// Auth provider component
export function PermissionAuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = React.useState<PermissionAwareUser | null>(null);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);

  // Load user on mount
  React.useEffect(() => {
    const loadInitialUser = async () => {
      try {
        setLoading(true);
        setError(null);
        await permissionAuthService.loadUser();
      } catch (err) {
        setError(safeError(err).message);
      } finally {
        setLoading(false);
      }
    };

    loadInitialUser();

    // Subscribe to auth state changes
    const unsubscribe = permissionAuthService.subscribe((newUser) => {
      setUser(newUser);
      setLoading(false);
    });

    return unsubscribe;
  }, []);

  const contextValue: PermissionAuthContextValue = {
    user,
    loading,
    error,
    // Authentication methods
    connectWallet: async () => {
      try {
        setError(null);
        await permissionAuthService.connectWallet();
      } catch (err) {
        setError(safeError(err).message);
      }
    },
    logout: async () => {
      try {
        setError(null);
        await permissionAuthService.logout();
      } catch (err) {
        setError(safeError(err).message);
      }
    },
    
    // User and permission management
    refreshUser: async () => {
      try {
        setError(null);
        setLoading(true);
        await permissionAuthService.loadUser();
      } catch (err) {
        setError(safeError(err).message);
      } finally {
        setLoading(false);
      }
    },
    refreshPermissions: async () => {
      try {
        setError(null);
        await permissionAuthService.refreshPermissions();
      } catch (err) {
        setError(safeError(err).message);
      }
    },
    
    // 🔒 SECURITY CRITICAL: Permission checking methods now use backend authority (ASYNC)
    // ⚡ THE SINGLE SOURCE OF TRUTH: All permission validation through backend API
    hasPermission: (permission: string) => permissionAuthService.hasPermission(permission),
    hasAnyPermission: (permissions: string[]) => permissionAuthService.hasAnyPermission(permissions),
    hasAllPermissions: (permissions: string[]) => permissionAuthService.hasAllPermissions(permissions),
    
    // 🔒 DEPRECATED: Legacy synchronous permission checking (INSECURE - use only for compatibility)
    hasPermissionSync: (permission: string) => permissionAuthService.hasPermissionSync(permission),
    
    // Feature access methods (the core value proposition)
    canAccessAnalytics: () => permissionAuthService.canAccessAnalytics(),
    canExportData: () => permissionAuthService.canExportData(),
    canAccessRealtime: () => permissionAuthService.canAccessRealtime(),
    canUseAdvancedFilters: () => permissionAuthService.canUseAdvancedFilters(),
    isAdmin: () => permissionAuthService.isAdmin(),
    
    // Permission health monitoring
    getExpiringPermissions: () => permissionAuthService.getExpiringPermissions(),
    getPermissionHealth: () => permissionAuthService.getPermissionHealth()
  };

  return (
    <PermissionAuthContext.Provider value={contextValue}>
      {children}
    </PermissionAuthContext.Provider>
  );
}

// Backward compatibility alias
export const AuthProvider = PermissionAuthProvider;

// Export types for external use
export type { PermissionAwareUser, PermissionAuthContextValue };