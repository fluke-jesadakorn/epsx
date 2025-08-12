'use client';

import { useState, useEffect, useCallback } from 'react';
import { sharedCookieManager } from '../config/shared-cookie';
import { getCrossAppSync } from '../sync/cross-app-sync';

export interface SharedAuthUser {
  id: string;
  email: string;
  name?: string;
  role?: string;
  permissions?: string[];
}

export interface SharedAuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  user: SharedAuthUser | null;
  token: string | null;
  error: string | null;
}

export interface SharedAuthActions {
  login: (credentials: { email: string; password: string }) => Promise<void>;
  loginWithOIDC: () => Promise<void>;
  logout: () => Promise<void>;
  refreshToken: () => Promise<void>;
  clearError: () => void;
}

export type UseSharedAuthReturn = SharedAuthState & SharedAuthActions;

/**
 * Shared authentication hook for both frontend and admin-frontend
 * Handles cross-app session synchronization automatically
 */
export function useSharedAuth(): UseSharedAuthReturn {
  const [state, setState] = useState<SharedAuthState>({
    isAuthenticated: false,
    isLoading: true,
    user: null,
    token: null,
    error: null,
  });

  const crossAppSync = getCrossAppSync();

  /**
   * Update authentication state
   */
  const updateAuthState = useCallback((
    isAuthenticated: boolean,
    user: SharedAuthUser | null = null,
    token: string | null = null,
    error: string | null = null
  ) => {
    setState({
      isAuthenticated,
      isLoading: false,
      user,
      token,
      error,
    });
  }, []);

  /**
   * Initialize authentication state from stored token
   */
  const initializeAuth = useCallback(async () => {
    try {
      const token = await sharedCookieManager.getAuthToken();
      
      if (token) {
        // Validate token with backend
        const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/validate`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`,
          },
        });

        if (response.ok) {
          const userData = await response.json();
          updateAuthState(true, userData.user, token);
        } else {
          // Token is invalid, clear it
          await sharedCookieManager.clearAuthToken();
          updateAuthState(false);
        }
      } else {
        updateAuthState(false);
      }
    } catch (error) {
      console.error('Auth initialization failed:', error);
      updateAuthState(false, null, null, 'Failed to initialize authentication');
    }
  }, [updateAuthState]);

  /**
   * Login with email/password credentials
   */
  const login = useCallback(async (credentials: { email: string; password: string }) => {
    setState(prev => ({ ...prev, isLoading: true, error: null }));

    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(credentials),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Login failed');
      }

      const { token, user, expiresAt } = await response.json();
      
      // Store token using shared cookie manager
      await sharedCookieManager.setAuthToken(token, user.id);
      
      // Notify other app
      crossAppSync.notifyLogin(token, user.id, expiresAt);
      
      updateAuthState(true, user, token);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Login failed';
      updateAuthState(false, null, null, message);
    }
  }, [updateAuthState, crossAppSync]);

  /**
   * Login with OIDC
   */
  const loginWithOIDC = useCallback(async () => {
    setState(prev => ({ ...prev, isLoading: true, error: null }));

    try {
      // Redirect to OIDC provider
      const oidcUrl = `${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/oidc/login`;
      window.location.href = oidcUrl;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'OIDC login failed';
      updateAuthState(false, null, null, message);
    }
  }, [updateAuthState]);

  /**
   * Logout user
   */
  const logout = useCallback(async () => {
    setState(prev => ({ ...prev, isLoading: true }));

    try {
      // Call backend logout endpoint
      if (state.token) {
        await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/logout`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${state.token}`,
          },
        });
      }

      // Clear stored token
      await sharedCookieManager.clearAuthToken();
      
      // Notify other app
      crossAppSync.notifyLogout();
      
      updateAuthState(false);
    } catch (error) {
      // Even if backend call fails, clear local state
      await sharedCookieManager.clearAuthToken();
      crossAppSync.notifyLogout();
      updateAuthState(false, null, null, 'Logout completed with warnings');
    }
  }, [state.token, updateAuthState, crossAppSync]);

  /**
   * Refresh authentication token
   */
  const refreshToken = useCallback(async () => {
    if (!state.token) return;

    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/refresh`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${state.token}`,
        },
      });

      if (!response.ok) {
        throw new Error('Token refresh failed');
      }

      const { token, user, expiresAt } = await response.json();
      
      // Store new token
      await sharedCookieManager.setAuthToken(token, user.id);
      
      // Notify other app
      crossAppSync.notifyTokenRefresh(token, expiresAt);
      
      updateAuthState(true, user, token);
    } catch (error) {
      // Token refresh failed, logout user
      await logout();
    }
  }, [state.token, updateAuthState, crossAppSync, logout]);

  /**
   * Clear error state
   */
  const clearError = useCallback(() => {
    setState(prev => ({ ...prev, error: null }));
  }, []);

  /**
   * Handle cross-app authentication events
   */
  useEffect(() => {
    // Setup cross-app sync handlers
    crossAppSync.onMessage('auth-login', async (data) => {
      if (data?.token && data?.userId) {
        // Another app logged in, sync the state
        try {
          const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/validate`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${data.token}`,
            },
          });

          if (response.ok) {
            const userData = await response.json();
            updateAuthState(true, userData.user, data.token);
          }
        } catch (error) {
          console.error('Cross-app login sync failed:', error);
        }
      }
    });

    crossAppSync.onMessage('auth-logout', async () => {
      // Another app logged out, sync the state
      await sharedCookieManager.clearAuthToken();
      updateAuthState(false);
    });

    crossAppSync.onMessage('auth-refresh', async (data) => {
      if (data?.token) {
        // Another app refreshed token, sync the state
        try {
          const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/validate`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${data.token}`,
            },
          });

          if (response.ok) {
            const userData = await response.json();
            updateAuthState(true, userData.user, data.token);
          }
        } catch (error) {
          console.error('Cross-app token refresh sync failed:', error);
        }
      }
    });

    // Initialize cross-app sync
    crossAppSync.initialize();

    return () => {
      crossAppSync.destroy();
    };
  }, [crossAppSync, updateAuthState]);

  /**
   * Initialize authentication on mount
   */
  useEffect(() => {
    initializeAuth();
  }, [initializeAuth]);

  /**
   * Setup shared cookie sync listeners
   */
  useEffect(() => {
    sharedCookieManager.setupSyncListeners((isLoggedIn) => {
      if (!isLoggedIn && state.isAuthenticated) {
        // User was logged out in another tab/app
        updateAuthState(false);
      } else if (isLoggedIn && !state.isAuthenticated) {
        // User was logged in in another tab/app
        initializeAuth();
      }
    });
  }, [state.isAuthenticated, updateAuthState, initializeAuth]);

  /**
   * Auto-refresh token before expiration
   */
  useEffect(() => {
    if (!state.isAuthenticated || !state.token) return;

    // Set up token refresh 5 minutes before expiration
    const refreshInterval = setInterval(() => {
      refreshToken();
    }, 25 * 60 * 1000); // 25 minutes

    return () => clearInterval(refreshInterval);
  }, [state.isAuthenticated, state.token, refreshToken]);

  return {
    ...state,
    login,
    loginWithOIDC,
    logout,
    refreshToken,
    clearError,
  };
}

// Types are already exported above via interface declarations