'use client';

import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import { useRouter } from 'next/navigation';

/**
 * Server-side authentication user profile
 */
export interface ServerSideUserProfile {
  id: string;
  email: string;
  name?: string;
  picture?: string;
  role: string;
  permissions: string[];
  emailVerified: boolean;
  sessionId: string;
}

/**
 * Server-side authentication state
 */
export interface ServerSideAuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  isInitialized: boolean;
  user: ServerSideUserProfile | null;
  error: string | null;
}

/**
 * Server-side authentication actions
 */
export interface ServerSideAuthActions {
  loginWithOIDC: (callbackUrl?: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshSession: () => Promise<void>;
  clearError: () => void;
}

/**
 * Combined context interface
 */
export interface ServerSideAuthContextType extends ServerSideAuthState, ServerSideAuthActions {
  isAdminUser: () => boolean;
  getAdminAccessLevel: () => string;
}

// Create the context
const ServerSideAuthContext = createContext<ServerSideAuthContextType | null>(null);

/**
 * Server-side authentication provider
 * Uses standard OpenID Connect flow with backend
 */
export function ServerSideAuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<ServerSideAuthState>({
    isAuthenticated: false,
    isLoading: true,
    isInitialized: false,
    user: null,
    error: null,
  });
  
  const router = useRouter();

  /**
   * Update authentication state
   */
  const updateAuthState = useCallback((updates: Partial<ServerSideAuthState>) => {
    setState(prev => ({ ...prev, ...updates }));
  }, []);

  /**
   * Initialize authentication by checking current session
   */
  const initializeAuth = useCallback(async () => {
    try {
      updateAuthState({ isLoading: true, error: null });
      
      const response = await fetch('/api/auth/session', {
        method: 'GET',
        credentials: 'same-origin', // Include cookies
        headers: {
          'Cache-Control': 'no-cache',
        },
      });
      
      if (response.ok) {
        const data = await response.json();
        
        if (data.authenticated && data.user) {
          updateAuthState({
            isAuthenticated: true,
            user: data.user,
            isLoading: false,
            isInitialized: true,
            error: null,
          });
          
          console.log('✅ Server-side session validated:', {
            userId: data.user.id,
            email: data.user.email,
            role: data.user.role,
            timestamp: new Date().toISOString(),
          });
        } else {
          updateAuthState({
            isAuthenticated: false,
            user: null,
            isLoading: false,
            isInitialized: true,
            error: data.error || null,
          });
        }
      } else {
        updateAuthState({
          isAuthenticated: false,
          user: null,
          isLoading: false,
          isInitialized: true,
          error: response.status === 401 ? null : 'Session validation failed',
        });
      }
    } catch (error) {
      console.error('🚨 Auth initialization error:', error);
      updateAuthState({
        isAuthenticated: false,
        user: null,
        isLoading: false,
        isInitialized: true,
        error: 'Authentication initialization failed',
      });
    }
  }, [updateAuthState]);

  /**
   * Start OpenID Connect authentication flow
   */
  const loginWithOIDC = useCallback(async (callbackUrl?: string) => {
    try {
      updateAuthState({ isLoading: true, error: null });
      
      // Build login URL with callback parameter
      const loginUrl = new URL('/api/auth/login', window.location.origin);
      if (callbackUrl) {
        loginUrl.searchParams.set('callbackUrl', callbackUrl);
      }
      
      console.log('🔐 Starting OIDC flow:', loginUrl.toString());
      
      // Redirect to server-side login endpoint
      window.location.href = loginUrl.toString();
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'OIDC login failed';
      updateAuthState({
        isLoading: false,
        error: errorMessage,
      });
      console.error('🚨 OIDC login error:', error);
    }
  }, [updateAuthState]);

  /**
   * Logout user
   */
  const logout = useCallback(async () => {
    try {
      updateAuthState({ isLoading: true });
      
      // Call server-side logout endpoint
      const response = await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'same-origin',
      });
      
      if (response.ok) {
        updateAuthState({
          isAuthenticated: false,
          user: null,
          isLoading: false,
          error: null,
        });
        
        console.log('✅ Server-side logout successful');
        
        // Redirect to login page
        router.push('/login');
      } else {
        throw new Error('Logout request failed');
      }
    } catch (error) {
      console.error('🚨 Logout error:', error);
      
      // Even if server call fails, clear local state and redirect
      updateAuthState({
        isAuthenticated: false,
        user: null,
        isLoading: false,
        error: 'Logout completed with warnings',
      });
      
      router.push('/login');
    }
  }, [updateAuthState, router]);

  /**
   * Refresh current session
   */
  const refreshSession = useCallback(async () => {
    await initializeAuth();
  }, [initializeAuth]);

  /**
   * Clear error state
   */
  const clearError = useCallback(() => {
    updateAuthState({ error: null });
  }, [updateAuthState]);

  /**
   * Check if current user has admin access
   */
  const isAdminUser = useCallback((): boolean => {
    const adminRoles = [
      'super_admin',
      'admin-full-004',
      'moderator-standard-003',
      'admin',
      'moderator'
    ];
    
    return state.user ? adminRoles.includes(state.user.role) : false;
  }, [state.user]);

  /**
   * Get admin access level
   */
  const getAdminAccessLevel = useCallback((): string => {
    if (!state.user) return 'none';
    
    switch (state.user.role) {
      case 'super_admin':
        return 'super';
      case 'admin-full-004':
      case 'admin':
        return 'full';
      case 'moderator-standard-003':
      case 'moderator':
        return 'standard';
      default:
        return 'none';
    }
  }, [state.user]);

  /**
   * Initialize authentication on mount
   */
  useEffect(() => {
    initializeAuth();
  }, [initializeAuth]);

  /**
   * Set up session refresh interval
   */
  useEffect(() => {
    if (!state.isAuthenticated) return;
    
    // Refresh session every 30 minutes
    const refreshInterval = setInterval(() => {
      refreshSession();
    }, 30 * 60 * 1000);
    
    return () => clearInterval(refreshInterval);
  }, [state.isAuthenticated, refreshSession]);

  // Context value
  const contextValue: ServerSideAuthContextType = {
    ...state,
    loginWithOIDC,
    logout,
    refreshSession,
    clearError,
    isAdminUser,
    getAdminAccessLevel,
  };

  return (
    <ServerSideAuthContext.Provider value={contextValue}>
      {children}
    </ServerSideAuthContext.Provider>
  );
}

/**
 * Hook to use server-side authentication context
 */
export function useServerSideAuth(): ServerSideAuthContextType {
  const context = useContext(ServerSideAuthContext);
  
  if (!context) {
    throw new Error('useServerSideAuth must be used within a ServerSideAuthProvider');
  }
  
  return context;
}

/**
 * Legacy compatibility hook
 */
export function useAdminAuth() {
  const {
    user,
    isLoading: loading,
    isInitialized: initialized,
    loginWithOIDC,
    logout,
    error,
    isAuthenticated
  } = useServerSideAuth();

  // Map to legacy format
  const legacyUser = user ? {
    user_id: user.id,
    email: user.email,
    role: user.role,
    permissions: user.permissions,
    subscription_tier: '',
    session_type: 'admin',
    expires_at: new Date(Date.now() + 8 * 60 * 60 * 1000).toISOString(), // 8 hours
  } : null;

  return {
    user: legacyUser,
    loading,
    initialized,
    error,
    isAuthenticated,
    signIn: async (email: string, password: string) => {
      // Server-side auth doesn't use email/password
      throw new Error('Use loginWithOIDC instead');
    },
    signOut: logout,
    logout,
    loginWithOIDC,
  };
}

/**
 * Legacy compatibility hook for auth status
 */
export function useAdminAuthStatus() {
  const { isAuthenticated, isLoading, isInitialized, isAdminUser, getAdminAccessLevel } = useServerSideAuth();
  
  return {
    isAuthenticated,
    isLoading,
    isInitialized,
    isAdmin: isAdminUser(),
    adminAccessLevel: getAdminAccessLevel(),
    hasError: false
  };
}