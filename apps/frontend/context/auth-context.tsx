'use client';

import { logger } from '@/lib/logger';
import { useOptimisticUpdates } from '@/lib/state/core';
import { ApiClientFactory } from '@epsx/api-client';
import { PackageTier } from '@epsx/types';
import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react';
import { useAppState } from './app-state';
import { useToasts } from './ui-context';

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
  register: (
    email: string,
    password: string,
    displayName?: string
  ) => Promise<any>;
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

export function AuthProvider({
  children,
  initialAuthState,
}: AuthProviderProps) {
  // Use the new state management system
  const { state, actions } = useAppState();
  const { success, error: showError } = useToasts();

  // Initialize API client for auth operations
  const apiClient = ApiClientFactory.getClientInstance();

  // Legacy state for backward compatibility
  const [user, setUser] = useState<BackendUser | null>(
    initialAuthState?.user || null
  );
  const [loading, setLoading] = useState(!initialAuthState);
  const [isInitialized, setIsInitialized] = useState(!!initialAuthState);
  const [permissions, setPermissions] = useState<string[]>(
    initialAuthState?.permissions || []
  );
  const [packageTier, setPackageTier] = useState<PackageTier>(
    initialAuthState?.packageTier || PackageTier.FREE
  );

  const {
    startOptimisticUpdate,
    confirmOptimisticUpdate,
    rollbackOptimisticUpdate,
  } = useOptimisticUpdates();

  // Removed loadUserSession - auth state now provided server-side via ServerAuthProvider

  const refreshPermissions = useCallback(async () => {
    // Note: In the new architecture, permissions are managed server-side
    // This function is kept for compatibility but does not fetch from server
    logger.info(
      'refreshPermissions: Permissions are now managed server-side through ServerAuthProvider'
    );

    // For now, this is a no-op. Future enhancement could trigger a page refresh
    // or emit an event to reload server state if needed
  }, []);

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
      if (userPermission.endsWith(':*')) {
        const prefix = userPermission.slice(0, -2);
        return permission.startsWith(prefix + ':');
      }
      return false;
    });
  };

  useEffect(() => {
    // In the new server-side auth architecture, we rely entirely on server-provided state
    // If no initial auth state is provided, set as not authenticated
    if (!initialAuthState && !isInitialized) {
      setUser(null);
      setPermissions([]);
      setPackageTier(PackageTier.FREE);
      setLoading(false);
      setIsInitialized(true);
    }
  }, [initialAuthState, isInitialized]);

  const login = useCallback(
    async (email: string, password: string) => {
      actions.ui.setLoading('login', true);

      try {
        // Use API client instead of server action
        const response = await apiClient.auth.login({ 
          type: 'credentials',
          email, 
          password 
        });

        if (response.error || !response.data) {
          throw new Error(response.error || 'Login failed');
        }

        const userData = response.data as any; // Cast to any since the backend returns extended user data
        const normalizedUserData = normalizeUserData(userData);

        // Update both legacy and new state
        setUser(normalizedUserData);
        setPermissions(userData.permissions || []);
        setPackageTier(
          (userData.package_tier as PackageTier) || PackageTier.FREE
        );

        actions.user.setProfile(normalizedUserData);
        actions.user.updatePermissions(userData.permissions || []);
        actions.user.setPackageTier(userData.package_tier || 'FREE');

        success(
          'Welcome back!',
          `Successfully logged in as ${normalizedUserData.email}`
        );

        return normalizedUserData;
      } catch (error) {
        showError(
          'Login Failed',
          error instanceof Error ? error.message : 'Unknown error occurred'
        );
        throw error;
      } finally {
        actions.ui.setLoading('login', false);
      }
    },
    [actions, success, showError, apiClient.auth]
  );

  const register = async (
    email: string,
    password: string,
    displayName?: string
  ) => {
    try {
      // Use API client instead of server action
      const response = await apiClient.auth.register({
        email,
        password,
        name: displayName,
      });

      if (response.error) {
        throw new Error(response.error);
      }

      return response.data;
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
        // Rollback if logout fails - in new architecture, would need to refresh page
        // or re-fetch server state, but for now we'll just log the error
        logger.error(
          'Logout rollback: Cannot restore state in server-side auth architecture'
        );
      }
    );

    try {
      // Use API client instead of server action
      const response = await apiClient.auth.logout();

      if (response.error) {
        throw new Error(response.error);
      }

      confirmOptimisticUpdate(updateId);
      success('Logged out', 'You have been successfully logged out');
    } catch (error) {
      logger.error('Logout error', {
        error: error instanceof Error ? error.message : String(error),
      });
      rollbackOptimisticUpdate(updateId);
      showError('Logout Failed', 'Failed to log out. Please try again.');
    }
  }, [
    actions,
    success,
    showError,
    startOptimisticUpdate,
    confirmOptimisticUpdate,
    rollbackOptimisticUpdate,
    apiClient.auth,
  ]);

  // Memoize context value to prevent unnecessary re-renders
  const contextValue = useMemo(
    () => ({
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
      lastUpdated: state.user.lastUpdated,
    }),
    [
      user,
      loading,
      isInitialized,
      permissions,
      packageTier,
      hasPermission,
      refreshPermissions,
      login,
      register,
      logout,
      initializeFromServer,
      state.ui.loading.requests.auth,
      state.user.optimisticUpdates.length,
      state.user.lastUpdated,
    ]
  );

  return (
    <AuthContext.Provider value={contextValue}>{children}</AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
