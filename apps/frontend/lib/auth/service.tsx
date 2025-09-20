/**
 * Unified Authentication Service
 * Standardizes Firebase and OIDC authentication patterns
 * Provides consistent interface for all authentication operations
 */

import React from 'react';
import { authConfig, clientConfig } from '@/config/env';
import { authLogger, safeError } from '@/lib/utils/logging';
import { 
  isJWTExpired, 
  getJWTTimeToExpiry
} from '../../../../shared/auth/jwt';
import {
  derivePackageTierFromPermissions,
  deriveAccessiblePlatformsFromPermissions 
} from '../../../../shared/permissions/utils/platform';

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
  lastLogin: string;
  expiresAt?: number;
}

// Authentication context interface
export interface AuthContextValue {
  user: UnifiedUser | null;
  loading: boolean;
  error: string | null;
  login: () => Promise<void>;
  logout: () => Promise<void>;
  refreshUser: () => Promise<void>;
}

// Default context value
const defaultAuthContext: AuthContextValue = {
  user: null,
  loading: true,
  error: null,
  login: async () => {
    throw new Error('AuthContext not initialized');
  },
  logout: async () => {
    throw new Error('AuthContext not initialized');
  },
  refreshUser: async () => {
    throw new Error('AuthContext not initialized');
  }
};

// Create React context
export const AuthContext = React.createContext<AuthContextValue>(defaultAuthContext);

// Authentication service class
export class UnifiedAuthService {
  private static instance: UnifiedAuthService;
  private currentUser: UnifiedUser | null = null;
  private listeners: Set<(user: UnifiedUser | null) => void> = new Set();

  private constructor() {}

  static getInstance(): UnifiedAuthService {
    if (!UnifiedAuthService.instance) {
      UnifiedAuthService.instance = new UnifiedAuthService();
    }
    return UnifiedAuthService.instance;
  }

  // Subscribe to auth state changes
  subscribe(callback: (user: UnifiedUser | null) => void): () => void {
    this.listeners.add(callback);
    return () => this.listeners.delete(callback);
  }

  // Notify all listeners of state changes
  private notifyListeners(): void {
    this.listeners.forEach(callback => callback(this.currentUser));
  }

  // Get current user
  getCurrentUser(): UnifiedUser | null {
    return this.currentUser;
  }

  // Load user from token
  async loadUser(): Promise<UnifiedUser | null> {
    try {
      const response = await fetch('/api/auth/user', {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json'
        }
      });

      if (!response.ok) {
        if (response.status === 401) {
          this.currentUser = null;
          this.notifyListeners();
          return null;
        }
        throw new Error('Failed to load user data');
      }

      const userData = await response.json();
      const user = this.transformToUnifiedUser(userData);
      
      this.currentUser = user;
      this.notifyListeners();
      
      authLogger.info('User loaded successfully', { 
        userId: user.id,
        email: user.email 
      });
      
      return user;
    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Failed to load user', { error: errorMsg });
      
      this.currentUser = null;
      this.notifyListeners();
      return null;
    }
  }

  // Transform backend user data to unified format
  private transformToUnifiedUser(userData: any): UnifiedUser {
    const permissions = userData.permissions || [];
    
    return {
      id: userData.id || userData.uid,
      email: userData.email,
      name: userData.name || userData.displayName,
      photoURL: userData.photoURL,
      emailVerified: userData.emailVerified || false,
      permissions,
      packageTier: derivePackageTierFromPermissions(permissions),
      accessiblePlatforms: deriveAccessiblePlatformsFromPermissions(permissions),
      role: userData.role || 'user',
      createdAt: userData.createdAt || new Date().toISOString(),
      lastLogin: userData.lastLogin || new Date().toISOString(),
      expiresAt: userData.expiresAt
    };
  }

  // Initiate login flow
  async login(): Promise<void> {
    try {
      const currentUrl = window.location.href;
      
      const response = await fetch('/api/auth/initiate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          redirectTo: currentUrl
        }),
        credentials: 'include'
      });

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.error || 'Failed to initiate authentication');
      }

      authLogger.info('OAuth PKCE flow initiated');
      
      // Redirect to authorization server
      window.location.href = data.authUrl;

    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Login failed', { error: errorMsg });
      throw new Error(errorMsg);
    }
  }

  // Logout user
  async logout(): Promise<void> {
    try {
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json'
        }
      });

      this.currentUser = null;
      this.notifyListeners();
      
      authLogger.info('User logged out successfully');
      
      // Redirect to home
      window.location.href = '/';

    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Logout failed', { error: errorMsg });
      throw new Error(errorMsg);
    }
  }

  // Refresh user token
  async refreshToken(): Promise<boolean> {
    try {
      const response = await fetch('/api/auth/refresh', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json'
        }
      });

      if (!response.ok) {
        if (response.status === 401) {
          // Refresh token expired, logout user
          await this.logout();
          return false;
        }
        throw new Error('Token refresh failed');
      }

      const userData = await response.json();
      const user = this.transformToUnifiedUser(userData);
      
      this.currentUser = user;
      this.notifyListeners();
      
      authLogger.info('Token refreshed successfully');
      return true;

    } catch (error) {
      const errorMsg = safeError(error).message;
      authLogger.error('Token refresh failed', { error: errorMsg });
      return false;
    }
  }

  // Check if user is authenticated
  isAuthenticated(): boolean {
    return this.currentUser !== null;
  }

  // Check if user has specific permission
  hasPermission(permission: string): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.permissions.includes(permission);
  }

  // Check if user has any of the specified permissions
  hasAnyPermission(permissions: string[]): boolean {
    if (!this.currentUser) return false;
    return permissions.some(permission => 
      this.currentUser!.permissions.includes(permission)
    );
  }

  // Check if user has all specified permissions
  hasAllPermissions(permissions: string[]): boolean {
    if (!this.currentUser) return false;
    return permissions.every(permission => 
      this.currentUser!.permissions.includes(permission)
    );
  }

  // Get user's package tier
  getPackageTier(): string {
    return this.currentUser?.packageTier || 'free';
  }

  // Get user's accessible platforms
  getAccessiblePlatforms(): string[] {
    return this.currentUser?.accessiblePlatforms || [];
  }
}

// Export singleton instance
export const authService = UnifiedAuthService.getInstance();

// React hook for using auth service
export function useUnifiedAuth(): AuthContextValue {
  const context = React.useContext(AuthContext);
  if (!context) {
    throw new Error('useUnifiedAuth must be used within AuthProvider');
  }
  return context;
}

// Backward compatibility alias
export const useAuth = useUnifiedAuth;

// Auth provider component
export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = React.useState<UnifiedUser | null>(null);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);

  // Load user on mount
  React.useEffect(() => {
    const loadInitialUser = async () => {
      try {
        setLoading(true);
        setError(null);
        await authService.loadUser();
      } catch (err) {
        setError(safeError(err).message);
      } finally {
        setLoading(false);
      }
    };

    loadInitialUser();

    // Subscribe to auth state changes
    const unsubscribe = authService.subscribe((newUser) => {
      setUser(newUser);
      setLoading(false);
    });

    return unsubscribe;
  }, []);

  const contextValue: AuthContextValue = {
    user,
    loading,
    error,
    login: async () => {
      try {
        setError(null);
        await authService.login();
      } catch (err) {
        setError(safeError(err).message);
      }
    },
    logout: async () => {
      try {
        setError(null);
        await authService.logout();
      } catch (err) {
        setError(safeError(err).message);
      }
    },
    refreshUser: async () => {
      try {
        setError(null);
        setLoading(true);
        await authService.loadUser();
      } catch (err) {
        setError(safeError(err).message);
      } finally {
        setLoading(false);
      }
    }
  };

  return (
    <AuthContext.Provider value={contextValue}>
      {children}
    </AuthContext.Provider>
  );
}