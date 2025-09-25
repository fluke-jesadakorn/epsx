// ============================================================================
// BACKEND AUTH CONTEXT (Phase 2.1)
// Replaces local permission state management with backend-centric auth
// THE SINGLE SOURCE OF TRUTH for user authentication and authorization
// ============================================================================

'use client';

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { useBackendPermissions } from '@/lib/permissions/use-backend-permissions';
import { permissionAuthority } from '@/lib/permissions/backend-authority-client';

// ============================================================================
// AUTH CONTEXT TYPES
// ============================================================================

export interface User {
  id: string;
  wallet_address?: string;
  email?: string;
  tier?: string;
  permissions?: string[];
  created_at?: string;
  updated_at?: string;
}

export interface AuthState {
  // User data
  user: User | null;
  userId?: string;
  
  // Authentication state
  isAuthenticated: boolean;
  isLoading: boolean;
  
  // Permission state from backend
  permissions: Record<string, boolean>;
  permissionLoading: boolean;
  
  // Tier/subscription info
  currentTier?: string;
  tierPermissions?: string[];
  
  // Error state
  error: string | null;
  
  // Session info
  lastActivity?: string;
  sessionExpiry?: string;
}

export interface AuthContextValue extends AuthState {
  // Authentication methods
  login: (walletAddress: string, signature: string, message: string) => Promise<boolean>;
  logout: () => Promise<void>;
  refreshAuth: () => Promise<void>;
  
  // Permission methods
  checkPermission: (permission: string, resourcePath?: string) => Promise<boolean>;
  refreshPermissions: () => Promise<void>;
  
  // Utility methods
  isAdmin: () => boolean;
  hasAnyAdminPermission: () => boolean;
  getTierInfo: () => { tier: string; permissions: string[] } | null;
  
  // Error handling
  clearError: () => void;
}

// ============================================================================
// AUTH CONTEXT CREATION
// ============================================================================

const BackendAuthContext = createContext<AuthContextValue | null>(null);

export function useBackendAuth(): AuthContextValue {
  const context = useContext(BackendAuthContext);
  if (!context) {
    throw new Error('useBackendAuth must be used within a BackendAuthProvider');
  }
  return context;
}

// ============================================================================
// BACKEND AUTH PROVIDER COMPONENT
// ============================================================================

export function BackendAuthProvider({ children }: { children: React.ReactNode }) {
  // ============================================================================
  // STATE MANAGEMENT
  // ============================================================================
  
  const [authState, setAuthState] = useState<AuthState>({
    user: null,
    isAuthenticated: false,
    isLoading: true,
    permissions: {},
    permissionLoading: false,
    error: null,
  });

  // ============================================================================
  // BACKEND PERMISSIONS INTEGRATION
  // ============================================================================
  
  const {
    permissions,
    loading: permissionLoading,
    currentTier,
    tierInfo,
    checkPermission,
    refreshPermissions: refreshBackendPermissions,
    clearError: clearPermissionError,
  } = useBackendPermissions(
    authState.userId,
    [], // We'll load permissions dynamically
    {
      autoRefresh: true,
      refreshInterval: 30, // 30 minutes
      cacheTimeout: 60,   // 60 minutes
    }
  );

  // ============================================================================
  // AUTHENTICATION METHODS
  // ============================================================================
  
  const login = useCallback(async (
    walletAddress: string, 
    signature: string, 
    message: string
  ): Promise<boolean> => {
    setAuthState(prev => ({ ...prev, isLoading: true, error: null }));

    try {
      // Store authentication data
      localStorage.setItem('wallet_address', walletAddress);
      localStorage.setItem('wallet_signature', signature);
      localStorage.setItem('auth_message', message);
      localStorage.setItem('auth_timestamp', Date.now().toString());
      localStorage.setItem('chain_id', '56'); // BSC Mainnet

      // Create user object
      const user: User = {
        id: walletAddress,
        wallet_address: walletAddress,
        tier: currentTier || 'basic',
        permissions: Object.keys(permissions).filter(p => permissions[p]),
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };

      // Update auth state
      setAuthState(prev => ({
        ...prev,
        user,
        userId: walletAddress,
        isAuthenticated: true,
        isLoading: false,
        currentTier: currentTier || 'basic',
        tierPermissions: tierInfo?.tier_permissions || [],
        lastActivity: new Date().toISOString(),
        sessionExpiry: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(), // 24 hours
      }));

      // Refresh permissions after login
      await refreshBackendPermissions();

      return true;
    } catch (error) {
      console.error('Login failed:', error);
      setAuthState(prev => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Login failed',
      }));
      return false;
    }
  }, [currentTier, permissions, tierInfo, refreshBackendPermissions]);

  const logout = useCallback(async (): Promise<void> => {
    try {
      // Clear local storage
      localStorage.removeItem('wallet_address');
      localStorage.removeItem('wallet_signature');
      localStorage.removeItem('auth_message');
      localStorage.removeItem('auth_timestamp');
      localStorage.removeItem('chain_id');
      localStorage.removeItem('auth_token');

      // Reset auth state
      setAuthState({
        user: null,
        userId: undefined,
        isAuthenticated: false,
        isLoading: false,
        permissions: {},
        permissionLoading: false,
        currentTier: undefined,
        tierPermissions: undefined,
        error: null,
      });

      // Clear permission cache
      clearPermissionError();

    } catch (error) {
      console.error('Logout failed:', error);
    }
  }, [clearPermissionError]);

  const refreshAuth = useCallback(async (): Promise<void> => {
    setAuthState(prev => ({ ...prev, isLoading: true }));

    try {
      const walletAddress = localStorage.getItem('wallet_address');
      const signature = localStorage.getItem('wallet_signature');

      if (!walletAddress || !signature) {
        await logout();
        return;
      }

      // Validate session with backend by checking a basic permission
      const isValid = await checkPermission('epsx:general:access');
      
      if (isValid) {
        // Session is valid, update user info
        const user: User = {
          id: walletAddress,
          wallet_address: walletAddress,
          tier: currentTier || 'basic',
          permissions: Object.keys(permissions).filter(p => permissions[p]),
          updated_at: new Date().toISOString(),
        };

        setAuthState(prev => ({
          ...prev,
          user,
          userId: walletAddress,
          isAuthenticated: true,
          isLoading: false,
          currentTier: currentTier || 'basic',
          tierPermissions: tierInfo?.tier_permissions || [],
          lastActivity: new Date().toISOString(),
        }));
      } else {
        // Session invalid, logout
        await logout();
      }
    } catch (error) {
      console.error('Auth refresh failed:', error);
      await logout();
    }
  }, [checkPermission, currentTier, permissions, tierInfo, logout]);

  // ============================================================================
  // UTILITY METHODS
  // ============================================================================
  
  const isAdmin = useCallback((): boolean => {
    return permissions['admin:general:access'] === true ||
           Object.keys(permissions).some(p => p.startsWith('admin:') && permissions[p]);
  }, [permissions]);

  const hasAnyAdminPermission = useCallback((): boolean => {
    return Object.keys(permissions).some(p => p.startsWith('admin:') && permissions[p]);
  }, [permissions]);

  const getTierInfo = useCallback((): { tier: string; permissions: string[] } | null => {
    if (!currentTier || !tierInfo) return null;
    
    return {
      tier: currentTier,
      permissions: tierInfo.tier_permissions || [],
    };
  }, [currentTier, tierInfo]);

  const clearError = useCallback(() => {
    setAuthState(prev => ({ ...prev, error: null }));
  }, []);

  // ============================================================================
  // EFFECTS
  // ============================================================================
  
  // Initialize auth state from localStorage
  useEffect(() => {
    const initializeAuth = async () => {
      const walletAddress = localStorage.getItem('wallet_address');
      const signature = localStorage.getItem('wallet_signature');

      if (walletAddress && signature) {
        // Try to restore session
        await refreshAuth();
      } else {
        // No saved auth, set loading to false
        setAuthState(prev => ({ ...prev, isLoading: false }));
      }
    };

    initializeAuth();
  }, []);

  // Update auth state when permissions change
  useEffect(() => {
    if (authState.isAuthenticated) {
      setAuthState(prev => ({
        ...prev,
        permissions,
        permissionLoading,
        currentTier,
        tierPermissions: tierInfo?.tier_permissions,
      }));
    }
  }, [permissions, permissionLoading, currentTier, tierInfo, authState.isAuthenticated]);

  // Session expiry check
  useEffect(() => {
    if (!authState.sessionExpiry || !authState.isAuthenticated) return;

    const checkSessionExpiry = () => {
      const expiry = new Date(authState.sessionExpiry!).getTime();
      const now = Date.now();
      
      if (now >= expiry) {
        logout();
      }
    };

    const intervalId = setInterval(checkSessionExpiry, 60000); // Check every minute
    return () => clearInterval(intervalId);
  }, [authState.sessionExpiry, authState.isAuthenticated, logout]);

  // ============================================================================
  // CONTEXT VALUE
  // ============================================================================
  
  const contextValue: AuthContextValue = {
    // State
    ...authState,
    
    // Methods
    login,
    logout,
    refreshAuth,
    checkPermission,
    refreshPermissions: refreshBackendPermissions,
    
    // Utilities
    isAdmin,
    hasAnyAdminPermission,
    getTierInfo,
    clearError,
  };

  return (
    <BackendAuthContext.Provider value={contextValue}>
      {children}
    </BackendAuthContext.Provider>
  );
}

// ============================================================================
// CONVENIENCE HOOKS
// ============================================================================

// Hook to get current user
export function useCurrentUser(): User | null {
  const { user } = useBackendAuth();
  return user;
}

// Hook to get user ID
export function useUserId(): string | undefined {
  const { userId } = useBackendAuth();
  return userId;
}

// Hook to check if user is authenticated
export function useIsAuthenticated(): boolean {
  const { isAuthenticated } = useBackendAuth();
  return isAuthenticated;
}

// Hook to check if user is admin
export function useIsAdmin(): boolean {
  const { isAdmin } = useBackendAuth();
  return isAdmin();
}

// Hook to get user tier info
export function useUserTier(): { tier: string; permissions: string[] } | null {
  const { getTierInfo } = useBackendAuth();
  return getTierInfo();
}

// Hook for authentication loading state
export function useAuthLoading(): boolean {
  const { isLoading } = useBackendAuth();
  return isLoading;
}

// ============================================================================
// PERMISSION HOOKS THAT USE BACKEND AUTH
// ============================================================================

// Hook to check permission with automatic user ID
export function usePermissionCheck(permission: string, resourcePath?: string) {
  const { userId, checkPermission } = useBackendAuth();
  
  const checkPerm = useCallback(async () => {
    if (!userId) return false;
    return await checkPermission(permission, resourcePath);
  }, [userId, permission, resourcePath, checkPermission]);

  return {
    checkPermission: checkPerm,
    userId,
  };
}

// Hook for admin permission checks
export function useAdminPermissionCheck(action: string, resource: string = 'general') {
  return usePermissionCheck(`admin:${resource}:${action}`);
}