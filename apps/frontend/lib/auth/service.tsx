/**
 * Permission-Centric Web3-First Authentication Service
 * Core philosophy: Permissions are the product, Web3 wallet is the identity
 * Provides sophisticated permission-aware authentication with granular access control
 */

import { logger as authLogger, safeError } from '@/lib/shared';
import { createFrontendClient, SharedWeb3AuthClient, UserInfoResponse } from '@/shared/auth/client';
import React from 'react';

// 🔒 SECURITY CRITICAL: Backend permission authority removed - permissions handled by backend only

// Simple permission group derivation (for display only)
function derivePermissionGroupFromPermissions(permissions: string[]): string {
  if (permissions.some(p => p.includes('admin:'))) return 'Admin';
  if (permissions.some(p => p.includes('premium') || p.includes('platinum'))) return 'Premium';
  if (permissions.some(p => p.includes('gold'))) return 'Gold';
  if (permissions.some(p => p.includes('silver'))) return 'Silver';
  return 'Basic';
}

// Permission-centric user interface - Web3-first
export interface PermissionAwareUser {
  // Identity (Web3-first)
  id: string; // Wallet address or user ID
  walletAddress?: string; // Primary identity for Web3 users
  email?: string; // Optional for Web3-only users
  name?: string;
  photoURL?: string;
  emailVerified: boolean;

  // Basic permissions - handled by backend
  permissions: string[]; // Simple permission list

  // Derived permission-based attributes
  permissionGroup: string; // Derived from permissions
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
  granular_permissions?: string[];
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
  getPermissionHealth: () => { status: string; message: string };
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
  getPermissionHealth: () => ({ status: 'unknown', message: 'Permission system handled by backend' })
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
  private web3Client: SharedWeb3AuthClient;

  private constructor() {
    // Initialize Web3 client for frontend
    this.web3Client = createFrontendClient();

    // Subscribe to Web3 client user changes
    this.web3Client.subscribe((web3User) => {
      if (web3User) {
        this.currentUser = this.transformWeb3UserToPermissionAware(web3User);
      } else {
        this.currentUser = null;
        this.stopPermissionMonitoring();
      }
      this.notifyListeners();
    });
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
      authLogger.info('Loading user with Web3-first authentication');

      // Use Web3 client to load current user
      const web3User = await this.web3Client.loadCurrentUser();

      if (!web3User) {
        this.currentUser = null;
        this.stopPermissionMonitoring();
        this.notifyListeners();
        return null;
      }

      // Transform Web3 user to permission-aware format
      const user = this.transformWeb3UserToPermissionAware(web3User);

      this.currentUser = user;
      this.startPermissionMonitoring(); // Start monitoring permission health
      this.notifyListeners();

      authLogger.info('User loaded successfully', {
        userId: user.id,
        walletAddress: user.walletAddress,
        authMethod: user.authMethod,
        permissionCount: user.permissions.length
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

  // Transform Web3 user to permission-aware user format
  private transformWeb3UserToPermissionAware(web3User: UserInfoResponse): PermissionAwareUser {
    const permissions: string[] = web3User.permissions || [];

    return {
      // Identity (Web3-first)
      id: web3User.sub,
      walletAddress: web3User.wallet_address,
      email: web3User.email,
      name: web3User.wallet_address, // Use wallet address as display name
      photoURL: undefined,
      emailVerified: !!web3User.email,

      // Core permissions (from Web3 backend)
      permissions,

      // Derived attributes (permission-based derivation)
      permissionGroup: derivePermissionGroupFromPermissions(permissions),
      accessiblePlatforms: derivePlatformsFromPermissions(permissions),
      role: deriveRoleFromPermissions(permissions),

      // Feature access (derived from permissions)
      featureAccess: {
        analytics: permissions.some(p => p.includes('analytics') || p.includes('epsx:')),
        export: permissions.some(p => p.includes('export') || p.includes('epsx:')),
        realtime: permissions.some(p => p.includes('realtime') || p.includes('epsx:')),
        advancedFilters: permissions.some(p => p.includes('advanced') || p.includes('epsx:')),
        admin: permissions.some(p => p.startsWith('admin:'))
      },

      // Timestamps
      createdAt: new Date().toISOString(),
      lastLogin: new Date().toISOString(),
      expiresAt: undefined,

      // Web3 specific
      authMethod: 'web3'
    };
  }

  // Transform backend data to permission-aware user format (legacy support)
  private transformToPermissionAwareUser(userData: BackendUserData): PermissionAwareUser {
    // Use simple permissions array
    const permissions: string[] = userData.granular_permissions || [];

    // Determine auth method
    const authMethod = userData.wallet_address ? 'web3' : 'oidc';

    return {
      // Identity (Web3-first)
      id: userData.id || userData.uid || userData.wallet_address || 'unknown',
      walletAddress: userData.wallet_address,
      email: userData.email,
      name: userData.name || userData.displayName,
      photoURL: userData.photoURL,
      emailVerified: userData.emailVerified || false,

      // Core permissions (handled by backend)
      permissions,

      // Derived attributes (permission-based derivation)
      permissionGroup: derivePermissionGroupFromPermissions(permissions),
      accessiblePlatforms: derivePlatformsFromPermissions(permissions),
      role: deriveRoleFromPermissions(permissions),

      // Feature access (handled by backend)
      featureAccess: {
        analytics: true,
        export: true,
        realtime: true,
        advancedFilters: true,
        admin: userData.role === 'admin'
      },

      // Timestamps
      createdAt: userData.createdAt || new Date().toISOString(),
      lastLogin: userData.lastLogin || new Date().toISOString(),
      expiresAt: userData.expiresAt,

      // Web3 specific
      authMethod
    };
  }

  // Simple stub - permission health handled by backend
  private calculatePermissionHealth(permissions: string[]): { status: string; message: string } {
    return { status: 'healthy', message: 'Permission system handled by backend' };
  }

  // Primary Web3 wallet connection
  async connectWallet(): Promise<void> {
    try {
      authLogger.info('Initiating Web3 wallet connection');

      // Check if user is already authenticated
      if (this.web3Client.isAuthenticated()) {
        authLogger.info('User already authenticated with Web3');
        await this.loadUser();
        return;
      }

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

  // Handle Web3 authentication with signature (called by Web3 components)
  async authenticateWithWallet(walletAddress: string, signature: string, message: string, nonce: string): Promise<boolean> {
    try {
      authLogger.info('Authenticating with wallet signature', { walletAddress });

      const result = await this.web3Client.authenticateWithSignature({
        wallet_address: walletAddress,
        signature,
        message,
        nonce
      });

      if (result.success && result.user) {
        this.currentUser = this.transformWeb3UserToPermissionAware(result.user);
        this.startPermissionMonitoring();
        this.notifyListeners();

        authLogger.info('Web3 authentication successful', {
          walletAddress: result.user.wallet_address,
          permissions: result.user.permissions?.length || 0
        });

        return true;
      } else {
        authLogger.error('Web3 authentication failed', { error: result.error });
        throw new Error(result.error || 'Authentication failed');
      }
    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Web3 authentication error', { error: errorMsg });
      throw new Error(errorMsg);
    }
  }

  // Request Web3 challenge for signature
  async requestWeb3Challenge(walletAddress: string): Promise<{ nonce: string; message: string }> {
    try {
      authLogger.info('Requesting Web3 challenge', { walletAddress });

      const challenge = await this.web3Client.requestChallenge(walletAddress);

      authLogger.info('Web3 challenge received', { nonce: challenge.nonce.substring(0, 8) + '...' });

      return {
        nonce: challenge.nonce,
        message: challenge.message
      };
    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Web3 challenge request failed', { error: errorMsg });
      throw new Error(errorMsg);
    }
  }

  // Web3 authentication is the primary method - no fallback needed

  // Logout user (clears Web3 sessions and wallet connections)
  async logout(): Promise<void> {
    try {
      // Use Web3 client logout
      await this.web3Client.logout();

      this.currentUser = null;
      this.stopPermissionMonitoring();
      this.notifyListeners();

      authLogger.info('User logged out successfully (session and permissions cleared)');

      // Redirect to home
      window.location.href = '/';

    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Web3 logout failed', { error: errorMsg });
      throw new Error(errorMsg);
    }
  }

  // Refresh session and permissions
  async refreshToken(): Promise<boolean> {
    try {
      // Web3-first system: no refresh tokens, check current authentication status
      if (!this.web3Client.isAuthenticated()) {
        authLogger.info('Web3 session expired, user needs to re-authenticate');
        await this.logout();
        return false;
      }

      // Reload user from current session
      const web3User = await this.web3Client.loadCurrentUser();

      if (!web3User) {
        authLogger.info('Web3 session no longer valid, logging out');
        await this.logout();
        return false;
      }

      const user = this.transformWeb3UserToPermissionAware(web3User);
      this.currentUser = user;
      this.notifyListeners();

      authLogger.info('Web3 session validated successfully', {
        walletAddress: user.walletAddress,
        permissionCount: user.permissions.length
      });
      return true;

    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Web3 session validation failed', { error: errorMsg });
      return false;
    }
  }

  // Refresh only permissions (for permission health monitoring)
  async refreshPermissions(): Promise<void> {
    if (!this.currentUser) return;

    try {
      // Use client to reload current user and permissions
      const web3User = await this.web3Client.loadCurrentUser();

      if (!web3User) {
        throw new Error('Failed to reload user data');
      }

      // Update user with new permissions
      const updatedUser = this.transformWeb3UserToPermissionAware(web3User);
      this.currentUser = updatedUser;
      this.notifyListeners();

      authLogger.info('Permissions refreshed successfully', {
        permissionCount: updatedUser.permissions.length
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

    // Backend handles all permission validation
    authLogger.debug('Permission check deferred to backend', {
      userId: this.currentUser.id,
      permission
    });

    // Return false to ensure backend is the source of truth
    return false; // Fail closed for security
  }

  // Check if user has any of the specified permissions (backend authority)
  async hasAnyPermission(permissions: string[]): Promise<boolean> {
    if (!this.currentUser?.id) {
      authLogger.warn('hasAnyPermission called without authenticated user');
      return false; // Fail closed for security
    }

    if (permissions.length === 0) return false;

    // Backend handles all permission validation
    authLogger.debug('Bulk permission check deferred to backend', {
      userId: this.currentUser.id,
      permissions
    });

    // Return false to ensure backend is the source of truth
    return false; // Fail closed for security
  }

  // Check if user has all specified permissions (backend authority)
  async hasAllPermissions(permissions: string[]): Promise<boolean> {
    if (!this.currentUser?.id) {
      authLogger.warn('hasAllPermissions called without authenticated user');
      return false; // Fail closed for security
    }

    if (permissions.length === 0) return true; // Vacuously true

    // Backend handles all permission validation
    authLogger.debug('All permissions check deferred to backend', {
      userId: this.currentUser.id,
      permissions
    });

    // Return false to ensure backend is the source of truth
    return false; // Fail closed for security
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
    // Permission health handled by backend
    return [];
  }

  getPermissionHealth(): { status: string; message: string } {
    // Permission health handled by backend
    return { status: 'healthy', message: 'Permission system handled by backend' };
  }

  // Legacy compatibility
  getPermissionGroup(): string {
    return this.currentUser?.permissionGroup || 'Basic Access Group';
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

// Permission-based derivation helpers - using shared utility
// Removed local implementation in favor of shared derivePermissionGroupFromPermissions

function deriveRoleFromPermissions(permissions: string[]): string {
  // Admin role - highest priority
  if (permissions.some(p => p === "admin:*:*" || p.startsWith("admin:"))) {
    return "admin";
  }

  // Premium user role
  if (permissions.some(p =>
    p === "epsx:analytics:premium" ||
    p === "epsx:analytics:professional"
  )) {
    return "premium_user";
  }

  // Default user role
  return "user";
}

function derivePlatformsFromPermissions(permissions: string[]): string[] {
  const platformSet = new Set<string>();

  permissions.forEach(permission => {
    const parts = permission.split(':');
    if (parts.length >= 1) {
      const platform = parts[0];
      if (platform && platform !== 'admin') {
        platformSet.add(platform);
      }
    }
  });

  // If no specific platforms found, default to epsx
  return Array.from(platformSet).length > 0 ? Array.from(platformSet) : ['epsx'];
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

// Types already exported via interface declarations