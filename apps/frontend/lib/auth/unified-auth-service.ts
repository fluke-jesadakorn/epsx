/**
 * Unified Authentication Service
 * Standardizes Firebase and OIDC authentication patterns
 * Provides consistent interface for all authentication operations
 */

import { authConfig, clientConfig } from '@/config/env';
import { authLogger, safeError } from '@/lib/logger';
import { 
  isJWTExpired, 
  getJWTTimeToExpiry, 
  derivePackageTierFromPermissions,
  deriveAccessiblePlatformsFromPermissions 
} from '@/lib/auth-utils';

// Unified user interface
export interface UnifiedUser {
  id: string;
  email: string;
  name?: string;
  photoURL?: string;
  emailVerified: boolean;
  permissions: string[];
  packageTier: string;
  accessiblePlatforms: string[];
  role: string;
  createdAt: string;
  lastSignInAt: string;
}

// Authentication state interface
export interface AuthState {
  user: UnifiedUser | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  tokenInfo: {
    accessToken?: string;
    refreshToken?: string;
    expiresAt?: number;
    timeToExpiry?: number;
  };
}

// Authentication events
export type AuthEventType = 
  | 'auth_state_changed'
  | 'token_refreshed'
  | 'login_success'
  | 'login_failed'
  | 'logout_success'
  | 'session_expired';

export interface AuthEvent {
  type: AuthEventType;
  user?: UnifiedUser | null;
  error?: string;
  timestamp: number;
}

// Authentication service interface
interface AuthService {
  // Core authentication methods
  getCurrentUser(): Promise<UnifiedUser | null>;
  login(redirectTo?: string): Promise<void>;
  logout(): Promise<void>;
  
  // Token management
  getAccessToken(): Promise<string | null>;
  refreshToken(): Promise<boolean>;
  
  // Session management
  validateSession(): Promise<boolean>;
  extendSession(): Promise<boolean>;
  
  // Event handling
  onAuthStateChange(callback: (event: AuthEvent) => void): () => void;
  
  // Permission checking
  hasPermission(permission: string): boolean;
  hasAnyPermission(permissions: string[]): boolean;
  hasAllPermissions(permissions: string[]): boolean;
  canAccessPlatform(platform: string): boolean;
}

class UnifiedAuthService implements AuthService {
  private currentUser: UnifiedUser | null = null;
  private authState: AuthState = {
    user: null,
    isAuthenticated: false,
    isLoading: false,
    error: null,
    tokenInfo: {},
  };
  private eventListeners: Array<(event: AuthEvent) => void> = [];
  private refreshPromise: Promise<boolean> | null = null;

  constructor() {
    this.initializeAuth();
  }

  private async initializeAuth() {
    authLogger.info('Initializing unified auth service');
    
    try {
      // Try to restore session from storage
      await this.restoreSession();
      
      // Validate current session
      const isValid = await this.validateSession();
      if (!isValid && this.currentUser) {
        authLogger.warn('Session validation failed, clearing user');
        await this.clearSession();
      }
      
      authLogger.info('Auth service initialized successfully');
    } catch (error) {
      authLogger.error('Failed to initialize auth service', error);
      await this.clearSession();
    }
  }

  // Core authentication methods
  async getCurrentUser(): Promise<UnifiedUser | null> {
    if (this.currentUser) {
      // Validate token expiry
      const { accessToken } = this.authState.tokenInfo;
      if (accessToken && isJWTExpired(accessToken)) {
        authLogger.info('Access token expired, attempting refresh');
        const refreshed = await this.refreshToken();
        if (!refreshed) {
          authLogger.warn('Token refresh failed, clearing session');
          await this.clearSession();
          return null;
        }
      }
      return this.currentUser;
    }

    // Try to get user from API
    try {
      const response = await fetch('/api/auth/current-user', {
        credentials: 'include',
      });

      if (response.ok) {
        const userData = await response.json();
        this.currentUser = this.normalizeUser(userData);
        this.updateAuthState({ user: this.currentUser, isAuthenticated: true });
        return this.currentUser;
      } else {
        authLogger.info('No authenticated user found');
        return null;
      }
    } catch (error) {
      safeError('Failed to get current user', error);
      return null;
    }
  }

  async login(redirectTo?: string): Promise<void> {
    authLogger.info('Initiating unified login flow', { redirectTo });
    
    try {
      this.updateAuthState({ isLoading: true, error: null });
      
      const callbackUrl = redirectTo || window.location.href;
      const response = await fetch('/api/auth/initiate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ 
          callbackUrl,
          provider: 'unified' // Indicates unified auth flow
        }),
      });

      if (response.ok) {
        const { authUrl, state, codeVerifier } = await response.json();
        
        // Store PKCE parameters
        if (typeof window !== 'undefined') {
          sessionStorage.setItem('oauth_state', state);
          sessionStorage.setItem('code_verifier', codeVerifier);
        }
        
        // Emit login attempt event
        this.emitEvent({
          type: 'login_success',
          timestamp: Date.now(),
        });
        
        // Redirect to authorization URL
        window.location.href = authUrl;
      } else {
        throw new Error(`Login initiation failed: ${response.statusText}`);
      }
    } catch (error) {
      authLogger.error('Login failed', error);
      this.updateAuthState({ 
        isLoading: false, 
        error: 'Login failed. Please try again.' 
      });
      
      this.emitEvent({
        type: 'login_failed',
        error: error instanceof Error ? error.message : 'Login failed',
        timestamp: Date.now(),
      });
      
      throw error;
    }
  }

  async logout(): Promise<void> {
    authLogger.info('Initiating unified logout');
    
    try {
      // Call backend logout endpoint
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'include',
      });
      
      // Clear local session
      await this.clearSession();
      
      this.emitEvent({
        type: 'logout_success',
        user: null,
        timestamp: Date.now(),
      });
      
      authLogger.info('Logout completed successfully');
      
      // Redirect to login page
      window.location.href = '/';
    } catch (error) {
      safeError('Logout failed', error);
      
      // Force clear session even if logout call failed
      await this.clearSession();
      throw error;
    }
  }

  // Token management
  async getAccessToken(): Promise<string | null> {
    const { accessToken } = this.authState.tokenInfo;
    
    if (!accessToken) {
      return null;
    }
    
    // Check if token is expired
    if (isJWTExpired(accessToken)) {
      const refreshed = await this.refreshToken();
      return refreshed ? this.authState.tokenInfo.accessToken || null : null;
    }
    
    return accessToken;
  }

  async refreshToken(): Promise<boolean> {
    // Prevent multiple simultaneous refresh attempts
    if (this.refreshPromise) {
      return this.refreshPromise;
    }
    
    this.refreshPromise = this.performTokenRefresh();
    const result = await this.refreshPromise;
    this.refreshPromise = null;
    
    return result;
  }

  private async performTokenRefresh(): Promise<boolean> {
    authLogger.info('Attempting token refresh');
    
    try {
      const response = await fetch('/api/auth/refresh', {
        method: 'POST',
        credentials: 'include',
      });

      if (response.ok) {
        const tokenData = await response.json();
        
        // Update token info
        this.authState.tokenInfo = {
          accessToken: tokenData.access_token,
          refreshToken: tokenData.refresh_token,
          expiresAt: tokenData.expires_at,
          timeToExpiry: getJWTTimeToExpiry(tokenData.access_token),
        };
        
        // Get updated user info
        const user = await this.getCurrentUser();
        this.updateAuthState({ 
          user, 
          isAuthenticated: !!user,
          tokenInfo: this.authState.tokenInfo 
        });
        
        this.emitEvent({
          type: 'token_refreshed',
          user,
          timestamp: Date.now(),
        });
        
        authLogger.info('Token refresh successful');
        return true;
      } else {
        authLogger.warn('Token refresh failed', { status: response.status });
        return false;
      }
    } catch (error) {
      safeError('Token refresh error', error);
      return false;
    }
  }

  // Session management
  async validateSession(): Promise<boolean> {
    try {
      const response = await fetch('/api/auth/validate', {
        credentials: 'include',
      });
      
      return response.ok;
    } catch (error) {
      safeError('Session validation failed', error);
      return false;
    }
  }

  async extendSession(): Promise<boolean> {
    try {
      const response = await fetch('/api/auth/extend', {
        method: 'POST',
        credentials: 'include',
      });
      
      if (response.ok) {
        const { expiresAt } = await response.json();
        this.authState.tokenInfo.expiresAt = expiresAt;
        return true;
      }
      
      return false;
    } catch (error) {
      safeError('Session extension failed', error);
      return false;
    }
  }

  // Permission checking
  hasPermission(permission: string): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.permissions.includes(permission);
  }

  hasAnyPermission(permissions: string[]): boolean {
    if (!this.currentUser) return false;
    return permissions.some(permission => this.hasPermission(permission));
  }

  hasAllPermissions(permissions: string[]): boolean {
    if (!this.currentUser) return false;
    return permissions.every(permission => this.hasPermission(permission));
  }

  canAccessPlatform(platform: string): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.accessiblePlatforms.includes(platform);
  }

  // Event handling
  onAuthStateChange(callback: (event: AuthEvent) => void): () => void {
    this.eventListeners.push(callback);
    
    // Return unsubscribe function
    return () => {
      const index = this.eventListeners.indexOf(callback);
      if (index > -1) {
        this.eventListeners.splice(index, 1);
      }
    };
  }

  // Private helper methods
  private normalizeUser(userData: any): UnifiedUser {
    return {
      id: userData.id || userData.sub,
      email: userData.email,
      name: userData.name || userData.displayName,
      photoURL: userData.photoURL || userData.picture,
      emailVerified: userData.emailVerified || userData.email_verified || false,
      permissions: userData.permissions || [],
      packageTier: userData.packageTier || derivePackageTierFromPermissions(userData.permissions || []),
      accessiblePlatforms: userData.accessiblePlatforms || deriveAccessiblePlatformsFromPermissions(userData.permissions || []),
      role: userData.role || 'user',
      createdAt: userData.createdAt || userData.created_at || new Date().toISOString(),
      lastSignInAt: userData.lastSignInAt || userData.last_sign_in_at || new Date().toISOString(),
    };
  }

  private updateAuthState(updates: Partial<AuthState>) {
    this.authState = { ...this.authState, ...updates };
    
    // Emit state change event
    this.emitEvent({
      type: 'auth_state_changed',
      user: this.authState.user,
      timestamp: Date.now(),
    });
  }

  private emitEvent(event: AuthEvent) {
    this.eventListeners.forEach(listener => {
      try {
        listener(event);
      } catch (error) {
        authLogger.error('Error in auth event listener', error);
      }
    });
  }

  private async restoreSession() {
    // Try to get user from cookies/localStorage
    const user = await this.getCurrentUser();
    if (user) {
      this.currentUser = user;
      this.updateAuthState({ 
        user, 
        isAuthenticated: true 
      });
    }
  }

  private async clearSession() {
    this.currentUser = null;
    this.authState = {
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
      tokenInfo: {},
    };
    
    // Clear any stored auth data
    if (typeof window !== 'undefined') {
      sessionStorage.removeItem('oauth_state');
      sessionStorage.removeItem('code_verifier');
      localStorage.removeItem('epsx-user-state');
    }
  }

  // Getters for current state
  get user() { return this.currentUser; }
  get isAuthenticated() { return this.authState.isAuthenticated; }
  get isLoading() { return this.authState.isLoading; }
  get error() { return this.authState.error; }
}

// Singleton instance
export const unifiedAuthService = new UnifiedAuthService();

// React hook for using the unified auth service
export function useUnifiedAuth() {
  const [authState, setAuthState] = React.useState<AuthState>(() => ({
    user: unifiedAuthService.user,
    isAuthenticated: unifiedAuthService.isAuthenticated,
    isLoading: unifiedAuthService.isLoading,
    error: unifiedAuthService.error,
    tokenInfo: {},
  }));

  React.useEffect(() => {
    const unsubscribe = unifiedAuthService.onAuthStateChange((event) => {
      setAuthState({
        user: unifiedAuthService.user,
        isAuthenticated: unifiedAuthService.isAuthenticated,
        isLoading: unifiedAuthService.isLoading,
        error: unifiedAuthService.error,
        tokenInfo: {},
      });
    });

    return unsubscribe;
  }, []);

  return {
    ...authState,
    login: unifiedAuthService.login.bind(unifiedAuthService),
    logout: unifiedAuthService.logout.bind(unifiedAuthService),
    hasPermission: unifiedAuthService.hasPermission.bind(unifiedAuthService),
    hasAnyPermission: unifiedAuthService.hasAnyPermission.bind(unifiedAuthService),
    hasAllPermissions: unifiedAuthService.hasAllPermissions.bind(unifiedAuthService),
    canAccessPlatform: unifiedAuthService.canAccessPlatform.bind(unifiedAuthService),
  };
}