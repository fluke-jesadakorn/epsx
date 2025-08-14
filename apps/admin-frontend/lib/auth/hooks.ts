/**
 * Custom Authentication Hooks
 * Replaces useSession and other Auth.js hooks
 */
'use client';

import { useContext, useEffect, useState } from 'react';
import { AuthContext, AuthContextType } from './provider';

/**
 * Custom useAuth hook - replaces useSession
 * Provides authentication state and methods
 */
export function useAuth(): AuthContextType {
  const context = useContext(AuthContext);
  
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  
  return context;
}

/**
 * Hook for authentication status only
 */
export function useAuthStatus() {
  const { session, isLoading } = useAuth();
  
  return {
    isAuthenticated: session?.isLoggedIn || false,
    isLoading,
    user: session?.user || null,
  };
}

/**
 * Hook for admin-specific functionality
 */
export function useAdminAuth() {
  const { session, isLoading } = useAuth();
  const user = session?.user;
  
  return {
    isAdmin: user?.role === 'admin' || (user?.admin_modules?.length || 0) > 0,
    isSystemAdmin: user?.isSystemAdmin?.() || false,
    hasAdminModule: (module: string) => user?.hasAdminModule?.(module) || false,
    adminModules: user?.admin_modules || [],
    permissions: user?.permissions || [],
    isLoading,
  };
}

/**
 * Hook for sign in functionality
 */
export function useSignIn() {
  const [isLoading, setIsLoading] = useState(false);
  
  const signIn = async () => {
    try {
      setIsLoading(true);
      window.location.href = '/api/auth/login';
    } catch (error) {
      console.error('Sign in error:', error);
      setIsLoading(false);
    }
  };
  
  return { signIn, isLoading };
}

/**
 * Hook for sign out functionality
 */
export function useSignOut() {
  const { refreshSession } = useAuth();
  const [isLoading, setIsLoading] = useState(false);
  
  const signOut = async () => {
    try {
      setIsLoading(true);
      
      const response = await fetch('/api/auth/logout', {
        method: 'POST',
      });
      
      if (response.ok) {
        // Refresh session to reflect logged out state
        await refreshSession();
        // Redirect to login
        window.location.href = '/login';
      } else {
        throw new Error('Logout failed');
      }
    } catch (error) {
      console.error('Sign out error:', error);
      // Still redirect to login on error
      window.location.href = '/login';
    } finally {
      setIsLoading(false);
    }
  };
  
  return { signOut, isLoading };
}