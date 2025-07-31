'use client';

import { createContext, useContext, useEffect, useState, useCallback, ReactNode } from 'react';
import type { FrontendUser } from '@epsx/types';

// Extended user profile with auth-specific data
interface AuthUserProfile extends FrontendUser {
  permissions: string[];
  permission_profiles?: string[];
  emailVerified?: boolean;
}

// Unified auth state
export interface LocalAuthState {
  user: AuthUserProfile | null;
  isLoading: boolean;
  isInitialized: boolean;
  isAuthenticated: boolean;
  isAdmin: boolean;
}

export interface UnifiedAuthContextType {
  user: AuthUserProfile | null;
  isLoading: boolean;
  isInitialized: boolean;
  isAuthenticated: boolean;
  isAdmin: boolean;
  login: (email: string, password: string) => Promise<{ success: boolean; error?: string }>;
  logout: () => Promise<void>;
  hasPermission: (permission: string) => boolean;
  hasRole: (role: string) => boolean;
  hasProfile: (profile: string) => boolean;
  checkAccess: (permission: string, profile?: string, role?: string) => boolean;
  refreshAuth: () => Promise<void>;
}

const UnifiedAuthContext = createContext<UnifiedAuthContextType | null>(null);

export interface UnifiedAuthProviderProps {
  children: ReactNode;
  backendUrl?: string;
  isAdminContext?: boolean;
}

export function UnifiedAuthProvider({ 
  children, 
  backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080',
  isAdminContext = false
}: UnifiedAuthProviderProps) {
  const [authState, setAuthState] = useState<LocalAuthState>({
    user: null,
    isLoading: true,
    isInitialized: false,
    isAuthenticated: false,
    isAdmin: false,
  });

  // Check if user has specific permission
  const hasPermission = useCallback((permission: string): boolean => {
    if (!authState.user?.permissions) return false;
    
    return authState.user.permissions.some((userPerm: string) => {
      // Exact match
      if (userPerm === permission) return true;
      
      // SuperAdmin wildcards
      if (userPerm === '*' || userPerm === '*:*:*') return true;
      
      // Admin wildcards
      if (userPerm === 'admin:*:*' && permission.startsWith('admin.')) return true;
      
      // Dot notation wildcards
      if (userPerm.endsWith('.*')) {
        const prefix = userPerm.slice(0, -2);
        return permission.startsWith(prefix + '.');
      }
      
      // Colon notation wildcards
      if (userPerm.endsWith(':*:*')) {
        const domain = userPerm.slice(0, -4);
        return permission.startsWith(domain + '.') || permission.startsWith(domain + ':');
      }
      
      if (userPerm.endsWith(':*')) {
        const prefix = userPerm.slice(0, -2);
        return permission.startsWith(prefix + '.') || permission.startsWith(prefix + ':');
      }
      
      return false;
    });
  }, [authState.user?.permissions]);

  // Check if user has specific role
  const hasRole = useCallback((role: string): boolean => {
    if (!authState.user?.role) return false;
    
    const roleHierarchy: Record<string, number> = {
      'user': 1,
      'premium': 2,
      'moderator': 3,
      'admin': 4,
      'super_admin': 5,
      'superadmin': 5
    };
    
    const userLevel = roleHierarchy[authState.user.role.toLowerCase()] || 0;
    const requiredLevel = roleHierarchy[role.toLowerCase()] || 1;
    
    return userLevel >= requiredLevel;
  }, [authState.user?.role]);

  // Check if user has specific permission profile
  const hasProfile = useCallback((profile: string): boolean => {
    return authState.user?.permission_profiles?.includes(profile) || false;
  }, [authState.user?.permission_profiles]);

  // Comprehensive access check
  const checkAccess = useCallback((permission: string, profile?: string, role?: string): boolean => {
    // Check permission profile first
    if (profile && hasProfile(profile)) return true;
    
    // Check specific permission
    if (hasPermission(permission)) return true;
    
    // Check role fallback
    if (role && hasRole(role)) return true;
    
    return false;
  }, [hasPermission, hasProfile, hasRole]);

  // Get current user from backend
  const getCurrentUser = useCallback(async (): Promise<AuthUserProfile | null> => {
    try {
      const endpoint = isAdminContext ? '/api/admin/auth/profile' : '/api/v1/auth/profile';
      const response = fetch(`${backendUrl}${endpoint}`, {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      const result = await response;
      if (!result.ok) {
        if (result.status === 401) {
          return null; // Not authenticated
        }
        throw new Error(`Authentication check failed: ${result.status}`);
      }

      const userData = await result.json();
      return {
        id: userData.id,
        email: userData.email,
        name: userData.name || userData.displayName,
        displayName: userData.display_name || userData.displayName,
        avatar: userData.photo_url || userData.photoURL,
        role: userData.role,
        isAdmin: userData.role === 'admin' || userData.isAdmin || false,
        isActive: true,
        createdAt: new Date(userData.created_at || userData.createdAt || Date.now()),
        updatedAt: new Date(userData.updated_at || userData.updatedAt || Date.now()),
        permissions: userData.permissions || [],
        permission_profiles: userData.permission_profiles || [],
        emailVerified: userData.email_verified || userData.emailVerified,
      };
    } catch (error) {
      console.error('getCurrentUser error:', error);
      return null;
    }
  }, [backendUrl, isAdminContext]);

  // Login function
  const login = useCallback(async (email: string, password: string): Promise<{ success: boolean; error?: string }> => {
    try {
      setAuthState((prev: LocalAuthState) => ({ ...prev, isLoading: true }));
      
      const endpoint = isAdminContext ? '/api/admin/auth/login' : '/api/v1/auth/login';
      const response = await fetch(`${backendUrl}${endpoint}`, {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email, password }),
      });

      if (!response.ok) {
        const error = await response.text();
        return { success: false, error: error || 'Login failed' };
      }

      // Get user data after successful login
      const user = await getCurrentUser();
      const isAdminUser = user ? ['admin', 'super_admin', 'superadmin', 'moderator'].includes(user.role.toLowerCase()) : false;
      
      setAuthState({
        user,
        isLoading: false,
        isInitialized: true,
        isAuthenticated: !!user,
        isAdmin: isAdminUser,
      });

      return { success: true };
    } catch (error) {
      console.error('Login error:', error);
      setAuthState((prev: LocalAuthState) => ({ ...prev, isLoading: false }));
      return { success: false, error: 'Network error during login' };
    }
  }, [backendUrl, getCurrentUser, isAdminContext]);

  // Logout function
  const logout = useCallback(async (): Promise<void> => {
    try {
      const endpoint = isAdminContext ? '/api/admin/auth/logout' : '/api/v1/auth/logout';
      await fetch(`${backendUrl}${endpoint}`, {
        method: 'POST',
        credentials: 'include',
      });
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      // Clear auth state regardless of API call success
      setAuthState({
        user: null,
        isLoading: false,
        isInitialized: true,
        isAuthenticated: false,
        isAdmin: false,
      });
    }
  }, [backendUrl, isAdminContext]);

  // Refresh authentication state
  const refreshAuth = useCallback(async (): Promise<void> => {
    const user = await getCurrentUser();
    const isAdminUser = user ? ['admin', 'super_admin', 'superadmin', 'moderator'].includes(user.role.toLowerCase()) : false;
    
    setAuthState({
      user,
      isLoading: false,
      isInitialized: true,
      isAuthenticated: !!user,
      isAdmin: isAdminUser,
    });
  }, [getCurrentUser]);

  // Initialize auth state on mount
  useEffect(() => {
    refreshAuth();
  }, [refreshAuth]);

  const contextValue: UnifiedAuthContextType = {
    user: authState.user,
    isLoading: authState.isLoading,
    isInitialized: authState.isInitialized,
    isAuthenticated: authState.isAuthenticated,
    isAdmin: authState.isAdmin,
    login,
    logout,
    hasPermission,
    hasRole,
    hasProfile,
    checkAccess,
    refreshAuth,
  };

  return (
    <UnifiedAuthContext.Provider value={contextValue}>
      {children}
    </UnifiedAuthContext.Provider>
  );
}

// Hook to use unified auth context
export function useUnifiedAuth(): UnifiedAuthContextType {
  const context = useContext(UnifiedAuthContext);
  if (!context) {
    throw new Error('useUnifiedAuth must be used within a UnifiedAuthProvider');
  }
  return context;
}

// Convenience hooks
export function useAuth() {
  return useUnifiedAuth();
}

export function usePermissions() {
  const { hasPermission, hasRole, hasProfile, checkAccess } = useUnifiedAuth();
  return { hasPermission, hasRole, hasProfile, checkAccess };
}