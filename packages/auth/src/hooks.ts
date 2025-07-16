'use client';

import { useCallback } from 'react';
import { useAuth, useAdminAuth } from './context';

/**
 * Comprehensive auth hook that provides all authentication functionality
 */
export function useAuthWithActions() {
  const authContext = useAuth();
  
  // Computed values
  const isAuthenticated = !!authContext.user;
  const isEmailVerified = authContext.user?.emailVerified ?? false;
  const isLoading = authContext.loading || !authContext.isInitialized;
  
  const providers = authContext.user?.providerData?.map((p: any) => p.providerId) || [];
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
  };
}

/**
 * Hook for checking specific auth requirements
 */
export function useAuthRequirements() {
  const { user } = useAuth();
  const isAuthenticated = !!user;

  const requireAuth = useCallback(() => {
    if (!isAuthenticated) {
      throw new Error('Authentication required');
    }
    return user;
  }, [isAuthenticated, user]);

  const requireEmailVerified = useCallback(() => {
    const authUser = requireAuth();
    if (!authUser?.emailVerified) {
      throw new Error('Email verification required');
    }
    return authUser;
  }, [requireAuth]);

  const requireGuest = useCallback(() => {
    if (isAuthenticated) {
      throw new Error('Already authenticated');
    }
  }, [isAuthenticated]);

  return {
    requireAuth,
    requireEmailVerified,
    requireGuest,
  };
}

/**
 * Hook for auth-related utilities
 */
export function useAuthUtils() {
  const { user, refreshSession, clearError } = useAuth();

  const getIdToken = useCallback(async (forceRefresh = false) => {
    if (!user || typeof user.getIdToken !== 'function') {
      throw new Error('No authenticated user found');
    }
    try {
      return await user.getIdToken(forceRefresh);
    } catch (error) {
      console.error('Token retrieval error:', error);
      throw error;
    }
  }, [user]);

  const ensureValidSession = useCallback(async () => {
    try {
      if (!user) {
        throw new Error('No user found');
      }
      
      // Try to get a fresh token
      await getIdToken(true);
      
      // Refresh the session
      await refreshSession();
    } catch (error) {
      console.error('Session validation error:', error);
      throw error;
    }
  }, [user, getIdToken, refreshSession]);

  const withAuthErrorHandling = useCallback(<T extends (...args: any[]) => Promise<any>>(
    operation: T
  ) => {
    return async (...args: Parameters<T>) => {
      try {
        clearError();
        return await operation(...args);
      } catch (error) {
        console.error('Auth operation error:', error);
        throw error;
      }
    };
  }, [clearError]);

  return {
    getIdToken,
    ensureValidSession,
    withAuthErrorHandling,
  };
}

/**
 * Admin-specific auth hooks
 */
export function useAdminAuthWithActions() {
  const adminContext = useAdminAuth();
  
  const isAdminAuthenticated = adminContext.isAdmin && !!adminContext.user;
  const isAdminLoading = adminContext.loading || !adminContext.isInitialized;

  const requireAdmin = useCallback(() => {
    if (!isAdminAuthenticated) {
      throw new Error('Admin authentication required');
    }
    return adminContext.user;
  }, [isAdminAuthenticated, adminContext.user]);

  return {
    // Core admin auth state
    ...adminContext,
    
    // Computed values
    isAdminAuthenticated,
    isAdminLoading,
    
    // Utilities
    requireAdmin,
  };
}

/**
 * Hook for session management utilities
 */
export function useSessionUtils() {
  const { refreshSession } = useAuth();

  const handleTokenRefresh = useCallback(async () => {
    try {
      await refreshSession();
      return { success: true };
    } catch (error) {
      console.error('Token refresh failed:', error);
      return { success: false, error };
    }
  }, [refreshSession]);

  const scheduleTokenRefresh = useCallback((intervalMs: number = 30 * 60 * 1000) => {
    // Set up automatic token refresh every 30 minutes by default
    const interval = setInterval(async () => {
      try {
        await handleTokenRefresh();
      } catch (error) {
        console.error('Scheduled token refresh failed:', error);
      }
    }, intervalMs);

    return () => clearInterval(interval);
  }, [handleTokenRefresh]);

  return {
    handleTokenRefresh,
    scheduleTokenRefresh,
  };
}

// Re-export the base hooks for convenience
export { useAuth, useAdminAuth } from './context';
