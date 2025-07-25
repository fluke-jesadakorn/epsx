'use client';

import { apiClient, isApiError, isApiSuccess } from '@epsx/api-client';
import { PackageTier } from '@epsx/types';
import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react';

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

interface OptimizedAuthContextType {
  user: BackendUser | null;
  loading: boolean;
  isInitialized: boolean;
  permissions: string[];
  packageTier: PackageTier;
  hasPermission: (permission: string) => boolean;
  login: (email: string, password: string) => Promise<BackendUser>;
  logout: () => Promise<void>;
}

const OptimizedAuthContext = createContext<
  OptimizedAuthContextType | undefined
>(undefined);

// Simplified user data normalization
const normalizeUserData = (userData: any): BackendUser => ({
  ...userData,
  emailVerified: true,
  displayName: userData.display_name || userData.email?.split('@')[0] || null,
  photoURL: userData.photo_url || null,
  phoneNumber: userData.phone_number || null,
});

interface OptimizedAuthProviderProps {
  children: React.ReactNode;
  initialAuthState?: {
    user?: BackendUser;
    permissions?: string[];
    packageTier?: PackageTier;
  };
}

export function OptimizedAuthProvider({
  children,
  initialAuthState,
}: OptimizedAuthProviderProps) {
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

  const loadUserSession = useCallback(async () => {
    if (loading) return; // Prevent multiple simultaneous requests

    setLoading(true);

    try {
      const response = await apiClient.getCurrentUser();

      if (isApiError(response)) {
        setUser(null);
        setPermissions([]);
        setPackageTier(PackageTier.FREE);
        return;
      }

      if (isApiSuccess(response)) {
        const userData = response.data;
        const normalizedUserData = normalizeUserData(userData);

        setUser(normalizedUserData);
        setPermissions(userData.permissions || []);
        setPackageTier(
          (userData.package_tier as PackageTier) || PackageTier.FREE
        );
      }
    } catch (error) {
      console.error('Error loading user session:', error);
      setUser(null);
      setPermissions([]);
      setPackageTier(PackageTier.FREE);
    } finally {
      setLoading(false);
    }
  }, [loading]);

  // Initialize auth state only once if no initial state provided
  useEffect(() => {
    if (!initialAuthState && !isInitialized) {
      loadUserSession().finally(() => setIsInitialized(true));
    }
  }, [initialAuthState, isInitialized, loadUserSession]);

  const hasPermission = useCallback(
    (permission: string): boolean => {
      if (permissions.includes(permission)) return true;

      return permissions.some(userPermission => {
        if (userPermission.endsWith('.*')) {
          const prefix = userPermission.slice(0, -2);
          return permission.startsWith(prefix + '.');
        }
        return false;
      });
    },
    [permissions]
  );

  const login = useCallback(async (email: string, password: string) => {
    setLoading(true);

    try {
      const response = await apiClient.login({
        type: 'credentials',
        email,
        password,
      });

      if (isApiError(response)) {
        throw new Error(response.error || 'Login failed');
      }

      if (isApiSuccess(response)) {
        const userData = response.data;
        const normalizedUserData = normalizeUserData(userData);

        setUser(normalizedUserData);
        setPermissions(userData.permissions || []);
        setPackageTier(
          (userData.package_tier as PackageTier) || PackageTier.FREE
        );

        return normalizedUserData;
      }

      throw new Error('Unexpected response format');
    } finally {
      setLoading(false);
    }
  }, []);

  const logout = useCallback(async () => {
    try {
      const response = await apiClient.logout();

      if (isApiError(response)) {
        console.error('Logout failed:', response.error);
      }

      // Always clear local state regardless of API response
      setUser(null);
      setPermissions([]);
      setPackageTier(PackageTier.FREE);
    } catch (error) {
      console.error('Logout error:', error);
      // Still clear local state on error
      setUser(null);
      setPermissions([]);
      setPackageTier(PackageTier.FREE);
    }
  }, []);

  // Memoize context value with minimal dependencies
  const contextValue = useMemo(
    () => ({
      user,
      loading,
      isInitialized,
      permissions,
      packageTier,
      hasPermission,
      login,
      logout,
    }),
    [
      user,
      loading,
      isInitialized,
      permissions,
      packageTier,
      hasPermission,
      login,
      logout,
    ]
  );

  return (
    <OptimizedAuthContext.Provider value={contextValue}>
      {children}
    </OptimizedAuthContext.Provider>
  );
}

export function useOptimizedAuth() {
  const context = useContext(OptimizedAuthContext);
  if (context === undefined) {
    throw new Error(
      'useOptimizedAuth must be used within an OptimizedAuthProvider'
    );
  }
  return context;
}
