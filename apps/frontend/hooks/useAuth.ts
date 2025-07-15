'use client';

import { useAuth as useAuthContext } from '@/context/auth-context-improved';
import { authService } from '@/services/auth/auth.service';
import { useCallback } from 'react';

/**
 * Comprehensive auth hook that provides all authentication functionality
 */
export function useAuth() {
  const authContext = useAuthContext();
  
  // Additional auth utilities
  const updateProfile = useCallback(async (data: { displayName?: string; photoURL?: string }) => {
    try {
      await authService.updateUserProfile(data);
      // The auth context will automatically update when Firebase auth state changes
    } catch (error) {
      console.error('Profile update error:', error);
      throw error;
    }
  }, []);

  const linkGoogleAccount = useCallback(async () => {
    try {
      await authService.linkGoogleAccount();
    } catch (error) {
      console.error('Google account linking error:', error);
      throw error;
    }
  }, []);

  const unlinkProvider = useCallback(async (providerId: string) => {
    try {
      await authService.unlinkProvider(providerId);
    } catch (error) {
      console.error('Provider unlinking error:', error);
      throw error;
    }
  }, []);

  const changePassword = useCallback(async (currentPassword: string, newPassword: string) => {
    try {
      await authService.changePassword(currentPassword, newPassword);
    } catch (error) {
      console.error('Password change error:', error);
      throw error;
    }
  }, []);

  const getIdToken = useCallback(async (forceRefresh = false) => {
    try {
      return await authService.getCurrentUserToken(forceRefresh);
    } catch (error) {
      console.error('Token retrieval error:', error);
      throw error;
    }
  }, []);

  // Computed values
  const isAuthenticated = !!authContext.user;
  const isEmailVerified = authContext.user?.emailVerified ?? false;
  const isLoading = authContext.loading || !authContext.isInitialized;
  
  const providers = authContext.user?.providerData?.map(p => p.providerId) || [];
  const hasPassword = providers.includes('password');
  const hasGoogle = providers.includes('google.com');

  return {
    // Core auth state
    ...authContext,
    
    // Computed values
    isAuthenticated,
    isEmailVerified,
    isLoading,
    hasPassword,
    hasGoogle,
    providers,
    
    // Extended functionality
    updateProfile,
    linkGoogleAccount,
    unlinkProvider,
    changePassword,
    getIdToken,
  };
}

/**
 * Hook for checking specific auth requirements
 */
export function useAuthRequirements() {
  const { isAuthenticated, isEmailVerified, user, isLoading } = useAuth();
  
  const checkRequirements = useCallback((requirements: {
    authenticated?: boolean;
    emailVerified?: boolean;
    roles?: string[];
  }) => {
    if (isLoading) {
      return { met: false, loading: true };
    }
    
    if (requirements.authenticated && !isAuthenticated) {
      return { met: false, loading: false, reason: 'authentication_required' };
    }
    
    if (requirements.emailVerified && !isEmailVerified) {
      return { met: false, loading: false, reason: 'email_verification_required' };
    }
    
    if (requirements.roles && user) {
      // This would need to be implemented based on your role system
      // For now, just checking if user has a role property
      const userRole = (user as any).role;
      if (!requirements.roles.includes(userRole)) {
        return { met: false, loading: false, reason: 'insufficient_permissions' };
      }
    }
    
    return { met: true, loading: false };
  }, [isAuthenticated, isEmailVerified, user, isLoading]);
  
  return { checkRequirements };
}

/**
 * Hook for auth-related utilities
 */
export function useAuthUtils() {
  const { user } = useAuth();
  
  const getUserInitials = useCallback(() => {
    if (!user) return '';
    
    if (user.displayName) {
      return user.displayName
        .split(' ')
        .map(name => name.charAt(0))
        .join('')
        .toUpperCase()
        .slice(0, 2);
    }
    
    if (user.email) {
      return user.email.charAt(0).toUpperCase();
    }
    
    return 'U';
  }, [user]);
  
  const getUserDisplayName = useCallback(() => {
    if (!user) return '';
    return user.displayName || user.email || 'User';
  }, [user]);
  
  const getAvatarUrl = useCallback(() => {
    return user?.photoURL || null;
  }, [user]);
  
  return {
    getUserInitials,
    getUserDisplayName,
    getAvatarUrl,
  };
}
