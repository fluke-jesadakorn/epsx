'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useSession, signIn as nextAuthSignIn, signOut as nextAuthSignOut } from 'next-auth/react';

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
  const { data: session, status } = useSession();
  const [state, setState] = useState<AuthState>({
    user: null,
    loading: true,
    error: null,
  });
  const router = useRouter();

  // Update state based on NextAuth session
  useEffect(() => {
    if (status === 'loading') {
      setState(prev => ({ ...prev, loading: true }));
      return;
    }

    if (status === 'authenticated' && session) {
      const user: AuthUser = {
        user_id: session.user.id,
        email: session.user.email,
        role: session.user.role,
        permissions: session.user.permissions,
        subscription_tier: session.user.subscription_tier,
        isAdmin: session.user.role === 'admin' || session.user.role === 'system_administrator',
        isPremium: ['premium', 'enterprise', 'platinum', 'gold'].includes(session.user.subscription_tier?.toLowerCase()),
      };
      setState({
        user,
        loading: false,
        error: null,
      });
    } else {
      setState({
        user: null,
        loading: false,
        error: null,
      });
    }
  }, [session, status]);

  const checkAuthState = async () => {
    // NextAuth handles this automatically through useSession
    return Promise.resolve();
  };

  const login = async (email: string, password: string) => {
    try {
      setState(prev => ({ ...prev, loading: true, error: null }));
      
      const result = await nextAuthSignIn('credentials', {
        email,
        password,
        redirect: false,
      });

      if (result?.error) {
        throw new Error(result.error);
      }

      if (result?.ok) {
        // NextAuth will handle the session update
        router.push('/dashboard');
      }
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
      await nextAuthSignOut({ redirect: false });
      router.push('/');
    } catch (error) {
      console.error('Logout error:', error);
      setState({
        user: null,
        loading: false,
        error: null,
      });
      router.push('/');
    }
  };

  const refresh = async () => {
    // NextAuth handles session refresh automatically
    return Promise.resolve();
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

// Simple permission checking hook using NextAuth session
export function usePermission(feature: string) {
  const { data: session, status } = useSession();
  const [hasPermission, setHasPermission] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (status === 'loading') {
      setLoading(true);
      return;
    }

    if (status === 'authenticated' && session?.user) {
      const userPermissions = session.user.permissions || [];
      const hasAccess = userPermissions.includes(feature);
      setHasPermission(hasAccess);
    } else {
      setHasPermission(false);
    }
    
    setLoading(false);
  }, [feature, session, status]);

  return { hasPermission, loading };
}

// Multi-permission checking hook using NextAuth session
export function usePermissions(features: string[], requireAll: boolean = false) {
  const { data: session, status } = useSession();
  const [hasAccess, setHasAccess] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);
  const [permissionResults, setPermissionResults] = useState<{[key: string]: boolean}>({});

  useEffect(() => {
    if (status === 'loading') {
      setLoading(true);
      return;
    }

    if (status === 'authenticated' && session?.user) {
      const userPermissions = session.user.permissions || [];
      
      const resultMap = features.reduce((acc, feature) => {
        acc[feature] = userPermissions.includes(feature);
        return acc;
      }, {} as {[key: string]: boolean});
      
      setPermissionResults(resultMap);
      
      const access = requireAll 
        ? features.every(feature => userPermissions.includes(feature))
        : features.some(feature => userPermissions.includes(feature));
      
      setHasAccess(access);
    } else {
      setHasAccess(false);
      setPermissionResults({});
    }
    
    setLoading(false);
  }, [features, requireAll, session, status]);

  return { 
    hasAccess, 
    loading, 
    permissions: permissionResults,
    hasPermission: (feature: string) => permissionResults[feature] || false,
  };
}