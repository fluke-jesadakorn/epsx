'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { mainAuthAPI, authUtils } from '@/lib/auth-api';
import { loginAction, logoutAction } from '@/lib/actions/server-auth';

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  isAdmin: boolean;
  isPremium: boolean;
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
      const user = await mainAuthAPI.getCurrentUser();
      
      setState({
        user: {
          user_id: user.user_id,
          email: user.email,
          role: user.role,
          permissions: user.permissions,
          subscription_tier: user.subscription_tier,
          isAdmin: user.role === 'admin' || user.role === 'system_administrator',
          isPremium: ['premium', 'enterprise', 'platinum', 'gold'].includes(user.subscription_tier?.toLowerCase()),
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
      
      // Create FormData for server action
      const formData = new FormData();
      formData.append('email', email);
      formData.append('password', password);
      
      // Call server action
      const result = await loginAction(formData);
      
      if (!result.success) {
        throw new Error(result.error || 'Login failed');
      }
      
      // After successful login, get user data
      const user = await mainAuthAPI.getCurrentUser();
      
      // Set user state from response
      setState({
        user: {
          user_id: user.user_id,
          email: user.email,
          role: user.role,
          permissions: user.permissions,
          subscription_tier: user.subscription_tier,
          isAdmin: user.role === 'admin' || user.role === 'system_administrator',
          isPremium: ['premium', 'enterprise', 'platinum', 'gold'].includes(user.subscription_tier?.toLowerCase()),
        },
        loading: false,
        error: null,
      });
      
      // Redirect to dashboard
      router.push('/dashboard');
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
      setState(prev => ({ ...prev, loading: true }));
      // Call server action (this will redirect automatically)
      await logoutAction();
      setState({
        user: null,
        loading: false,
        error: null,
      });
    } catch (error) {
      console.error('Logout error:', error);
      // Always clear state and redirect even if server action fails
      setState({
        user: null,
        loading: false,
        error: null,
      });
      router.push('/');
    }
  };

  const refresh = async () => {
    try {
      await mainAuthAPI.refreshSession();
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
    isPremium: state.user?.isPremium || false,
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

// Multi-permission checking hook
export function usePermissions(features: string[], requireAll: boolean = false) {
  const [hasAccess, setHasAccess] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);
  const [permissionResults, setPermissionResults] = useState<{[key: string]: boolean}>({});

  useEffect(() => {
    const checkPermissions = async () => {
      try {
        setLoading(true);
        
        const results = await Promise.all(
          features.map(async (feature) => ({
            feature,
            hasAccess: await authUtils.hasPermission(feature)
          }))
        );
        
        const resultMap = results.reduce((acc, { feature, hasAccess }) => {
          acc[feature] = hasAccess;
          return acc;
        }, {} as {[key: string]: boolean});
        
        setPermissionResults(resultMap);
        
        const access = requireAll 
          ? results.every(r => r.hasAccess)
          : results.some(r => r.hasAccess);
        
        setHasAccess(access);
      } catch (error) {
        console.error('Permission check failed:', error);
        setHasAccess(false);
        setPermissionResults({});
      } finally {
        setLoading(false);
      }
    };

    if (features && features.length > 0) {
      checkPermissions();
    }
  }, [features, requireAll]);

  return { 
    hasAccess, 
    loading, 
    permissions: permissionResults,
    hasPermission: (feature: string) => permissionResults[feature] || false,
  };
}

// Feature access hook for complex feature checking
export function useFeatureAccess() {
  const [features, setFeatures] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchFeatures = async () => {
      try {
        setLoading(true);
        const response = await mainAuthAPI.getFeatures();
        setFeatures(response.features);
      } catch (error) {
        console.error('Failed to fetch features:', error);
        setFeatures([]);
      } finally {
        setLoading(false);
      }
    };

    fetchFeatures();
  }, []);

  const hasFeature = (featureName: string) => {
    return features.some(f => f.feature === featureName && f.enabled);
  };

  const getFeature = (featureName: string) => {
    return features.find(f => f.feature === featureName);
  };

  return {
    features,
    loading,
    hasFeature,
    getFeature,
  };
}