'use client';

import React, { createContext, useContext, useEffect, useState, useMemo, useCallback } from 'react';
import { PackageTier } from '@epsx/types';
import { ClientCookies } from '@/lib/cookies';
import { useAppState } from './app-state';
import { useOptimisticUpdates } from '@/lib/state/core';
import { useToasts } from './ui-context';
import { apiClient, isApiSuccess, isApiError } from '@/lib/api-client';

interface BackendUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  package_tier: string;
  expires_at: string;
  session_type: string;
  emailVerified: boolean;
  displayName?: string;
  photoURL?: string;
  phoneNumber?: string | null;
}

interface AuthContextType {
  user: BackendUser | null;
  loading: boolean;
  isInitialized: boolean;
  permissions: string[];
  packageTier: PackageTier;
  hasPermission: (permission: string) => boolean;
  refreshPermissions: () => Promise<void>;
  login: (email: string, password: string) => Promise<BackendUser>;
  register: (email: string, password: string, displayName?: string) => Promise<{ user_id: string; email: string; verification_sent: boolean; message: string }>;
  logout: () => Promise<void>;
  initializeFromServer: (serverAuthState?: any) => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

// Helper function to normalize backend user data
const normalizeUserData = (userData: any): BackendUser => ({
  ...userData,
  emailVerified: true, // Backend users are considered verified
  displayName: userData.display_name || userData.email?.split('@')[0] || null,
  photoURL: userData.photo_url || null,
  phoneNumber: userData.phone_number || null,
});

interface AuthProviderProps {
  children: React.ReactNode;
  initialAuthState?: {
    user?: BackendUser;
    permissions?: string[];
    packageTier?: PackageTier;
  };
}

export function AuthProvider({ children, initialAuthState }: AuthProviderProps) {
  // Use the new state management system
  const { state, actions } = useAppState();
  const { success, error: showError } = useToasts();
  
  // Legacy state for backward compatibility
  const [user, setUser] = useState<BackendUser | null>(initialAuthState?.user || null);
  const [loading, setLoading] = useState(!initialAuthState);
  const [isInitialized, setIsInitialized] = useState(!!initialAuthState);
  const [permissions, setPermissions] = useState<string[]>(initialAuthState?.permissions || []);
  const [packageTier, setPackageTier] = useState<PackageTier>(initialAuthState?.packageTier || PackageTier.FREE);
  
  const {
    startOptimisticUpdate,
    confirmOptimisticUpdate,
    rollbackOptimisticUpdate
  } = useOptimisticUpdates();

  const loadUserSession = useCallback(async () => {
    actions.ui.setLoading('auth', true);
    
    try {
      const response = await apiClient.getCurrentUser();

      if (isApiError(response)) {
        // Session expired or invalid
        setUser(null);
        setPermissions([]);
        setPackageTier(PackageTier.FREE);
        actions.user.setProfile(null);
        actions.user.updatePermissions([]);
        actions.user.setPackageTier('FREE');
        return;
      }

      if (isApiSuccess(response)) {
        const userData = response.data;
        const normalizedUserData = normalizeUserData(userData);
        
        // Update both legacy and new state
        setUser(normalizedUserData);
        setPermissions(userData.permissions || []);
        setPackageTier(userData.package_tier as PackageTier || PackageTier.FREE);
        
        // Update new state management
        actions.user.setProfile(normalizedUserData);
        actions.user.updatePermissions(userData.permissions || []);
        actions.user.setPackageTier(userData.package_tier || 'FREE');
      }
      
    } catch (error) {
      console.error('Error loading user session:', error);
      setUser(null);
      setPermissions([]);
      setPackageTier(PackageTier.FREE);
      
      actions.user.setProfile(null);
      actions.user.updatePermissions([]);
      actions.user.setPackageTier('FREE');
      
      showError('Session Error', 'Failed to load your session. Please try logging in again.');
    } finally {
      actions.ui.setLoading('auth', false);
    }
  }, [actions, showError]);

  const refreshPermissions = async () => {
    await loadUserSession();
  };

  const initializeFromServer = (serverAuthState?: any) => {
    if (serverAuthState) {
      if (serverAuthState.user) {
        const normalizedUserData = normalizeUserData(serverAuthState.user);
        setUser(normalizedUserData);
      }
      setPermissions(serverAuthState.permissions || []);
      setPackageTier(serverAuthState.packageTier || PackageTier.FREE);
    }
    setIsInitialized(true);
    setLoading(false);
  };

  const hasPermission = (permission: string): boolean => {
    // Check exact match
    if (permissions.includes(permission)) {
      return true;
    }

    // Check wildcard permissions (e.g., "admin.*" covers "admin.users.create")
    return permissions.some(userPermission => {
      if (userPermission.endsWith('.*')) {
        const prefix = userPermission.slice(0, -2);
        return permission.startsWith(prefix + '.');
      }
      return false;
    });
  };

  useEffect(() => {
    // Only initialize from client if we don't have initial server state
    if (!initialAuthState && !isInitialized) {
      const initAuth = async () => {
        try {
          await loadUserSession();
        } finally {
          setLoading(false);
          setIsInitialized(true);
        }
      };

      initAuth();
    }
  }, [initialAuthState, isInitialized]);

  const login = useCallback(async (email: string, password: string) => {
    const updateId = Math.random().toString(36);
    actions.ui.setLoading('login', true);
    
    try {
      const response = await apiClient.login({ 
        type: 'credentials', 
        email, 
        password 
      });

      if (isApiError(response)) {
        throw new Error(response.error || 'Login failed');
      }

      if (isApiSuccess(response)) {
        const userData = response.data;
        const normalizedUserData = normalizeUserData(userData);
        
        // Update both legacy and new state
        setUser(normalizedUserData);
        setPermissions(userData.permissions || []);
        setPackageTier(userData.package_tier as PackageTier || PackageTier.FREE);
        
        actions.user.setProfile(normalizedUserData);
        actions.user.updatePermissions(userData.permissions || []);
        actions.user.setPackageTier(userData.package_tier || 'FREE');
        
        success('Welcome back!', `Successfully logged in as ${normalizedUserData.email}`);
        
        return normalizedUserData;
      }

      throw new Error('Unexpected response format');
    } catch (error) {
      showError('Login Failed', error instanceof Error ? error.message : 'Unknown error occurred');
      throw error;
    } finally {
      actions.ui.setLoading('login', false);
    }
  }, [actions, success, showError]);


  const register = async (email: string, password: string, displayName?: string) => {
    try {
      const response = await apiClient.register({ 
        email, 
        password, 
        name: displayName,
      });

      if (isApiError(response)) {
        throw new Error(response.error || 'Registration failed');
      }

      if (isApiSuccess(response)) {
        return response.data;
      }

      throw new Error('Unexpected response format');
    } catch (error) {
      throw error;
    }
  };

  const logout = useCallback(async () => {
    const updateId = Math.random().toString(36);
    
    // Optimistic update - immediately clear user state
    startOptimisticUpdate(
      updateId,
      () => {
        setUser(null);
        setPermissions([]);
        setPackageTier(PackageTier.FREE);
        actions.user.setProfile(null);
        actions.user.updatePermissions([]);
        actions.user.setPackageTier('FREE');
      },
      () => {
        // Rollback if logout fails - restore previous state
        loadUserSession();
      }
    );
    
    try {
      const response = await apiClient.logout();
      
      if (isApiError(response)) {
        throw new Error(response.error || 'Logout failed');
      }
      
      confirmOptimisticUpdate(updateId);
      success('Logged out', 'You have been successfully logged out');
      
    } catch (error) {
      console.error('Logout error:', error);
      rollbackOptimisticUpdate(updateId);
      showError('Logout Failed', 'Failed to log out. Please try again.');
    }
  }, [actions, success, showError, startOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate, loadUserSession]);

  // Memoize context value to prevent unnecessary re-renders
  const contextValue = useMemo(() => ({
    user, 
    loading: loading || state.ui.loading.requests.auth || false, 
    isInitialized,
    permissions, 
    packageTier,
    hasPermission,
    refreshPermissions,
    login,
    register, 
    logout,
    initializeFromServer,
    // Additional state from new system
    isOptimisticUpdate: state.user.optimisticUpdates.length > 0,
    lastUpdated: state.user.lastUpdated
  }), [
    user, loading, isInitialized, permissions, packageTier, 
    hasPermission, refreshPermissions, login, 
    register, logout, initializeFromServer, state.ui.loading.requests.auth,
    state.user.optimisticUpdates.length, state.user.lastUpdated
  ]);

  return (
    <AuthContext.Provider value={contextValue}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
