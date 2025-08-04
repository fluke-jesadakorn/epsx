'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { adminAuthAPI, authUtils } from '@/lib/auth-api';

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  isAdmin: boolean;
}

interface AuthState {
  user: AuthUser | null;
  loading: boolean;
  error: string | null;
}

export function useAuth() {
  const [state, setState] = useState<AuthState>({
    user: null,
    loading: true,
    error: null,
  });
  const router = useRouter();

  // Initialize auth state
  useEffect(() => {
    checkAuthState();
  }, []);

  const checkAuthState = async () => {
    try {
      setState(prev => ({ ...prev, loading: true, error: null }));
      const user = await adminAuthAPI.getCurrentUser();
      
      setState({
        user: {
          user_id: user.user_id,
          email: user.email,
          role: user.role,
          permissions: user.permissions,
          subscription_tier: user.subscription_tier,
          isAdmin: user.role === 'admin' || user.role === 'system_administrator',
        },
        loading: false,
        error: null,
      });
    } catch (error) {
      setState({
        user: null,
        loading: false,
        error: error instanceof Error ? error.message : 'Authentication failed',
      });
    }
  };

  const login = async (email: string, password: string) => {
    try {
      setState(prev => ({ ...prev, loading: true, error: null }));
      
      const response = await adminAuthAPI.login(email, password);
      
      // Set user state from login response
      setState({
        user: {
          user_id: response.user_id,
          email: response.email,
          role: response.role,
          permissions: response.permissions,
          subscription_tier: response.subscription_tier,
          isAdmin: response.role === 'admin' || response.role === 'system_administrator',
        },
        loading: false,
        error: null,
      });
      
      // Redirect to admin dashboard
      router.push('/admin');
    } catch (error) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : 'Login failed',
      }));
      throw error;
    }
  };

  const logout = async () => {
    try {
      await adminAuthAPI.logout();
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      // Always clear state and redirect
      setState({
        user: null,
        loading: false,
        error: null,
      });
      router.push('/login');
    }
  };

  const refresh = async () => {
    try {
      await adminAuthAPI.refreshSession();
      await checkAuthState(); // Refresh user data
    } catch (error) {
      console.error('Refresh failed:', error);
      setState(prev => ({ ...prev, error: 'Session refresh failed' }));
    }
  };

  return {
    // State
    user: state.user,
    loading: state.loading,
    error: state.error,
    isAdmin: state.user?.isAdmin || false,
    isAuthenticated: !!state.user,
    
    // Actions
    login,
    logout,
    refresh,
    checkAuthState,
  };
}

// Simple permission checking hook
export function usePermission(feature: string) {
  const [hasPermission, setHasPermission] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const checkPermission = async () => {
      try {
        setLoading(true);
        const result = await authUtils.hasPermission(feature);
        setHasPermission(result);
      } catch (error) {
        console.error('Permission check failed:', error);
        setHasPermission(false);
      } finally {
        setLoading(false);
      }
    };

    if (feature) {
      checkPermission();
    }
  }, [feature]);

  return { hasPermission, loading };
}