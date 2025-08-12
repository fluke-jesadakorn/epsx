'use client';

// Simplified admin authentication context for direct backend authentication
import { useAdminOIDCAuth, type AdminUserProfile } from './admin-oidc-auth';

/**
 * Legacy compatibility interface
 */
interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  session_type: string;
  expires_at: string;
}

/**
 * Simplified useAdminAuth hook that maps to multi-provider auth
 */
export function useAdminAuth() {
  const {
    user: adminUser,
    isLoading: loading,
    isInitialized: initialized,
    loginWithCredentials,
    logout,
    error,
    isAuthenticated
  } = useAdminOIDCAuth();

  // Map admin user to legacy format
  const user: AuthUser | null = adminUser ? {
    user_id: adminUser.id,
    email: adminUser.email || '',
    role: adminUser.role,
    permissions: adminUser.permissions,
    subscription_tier: adminUser.subscriptionTier || '',
    session_type: 'admin',
    expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString() // 24 hours from now
  } : null;

  return {
    user,
    loading,
    initialized,
    error,
    isAuthenticated,
    signIn: async (email: string, password: string) => {
      try {
        await loginWithCredentials(email, password);
      } catch (error) {
        // Re-throw to allow login form to handle it
        throw error;
      }
    },
    signOut: logout,
    logout
  };
}

/**
 * Simplified useAdminAuthStatus hook
 */
export function useAdminAuthStatus() {
  const { isAuthenticated, isLoading, isInitialized, isAdminUser, getAdminAccessLevel } = useAdminOIDCAuth();
  
  return {
    isAuthenticated,
    isLoading,
    isInitialized,
    isAdmin: isAdminUser(),
    adminAccessLevel: getAdminAccessLevel(),
    hasError: false
  };
}