'use client';

import React, { createContext, useContext, useState, useEffect } from 'react';
import { useSession, signIn as nextAuthSignIn, signOut as nextAuthSignOut } from 'next-auth/react';
import { useRouter } from 'next/navigation';

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  session_type: string;
  expires_at: string;
}

interface AuthState {
  user: AuthUser | null;
  loading: boolean;
  error: string | null;
  initialized: boolean;
  navigating: boolean;
}

interface AuthContextType extends AuthState {
  signIn: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  signOut: () => Promise<void>;
  checkAuth: () => Promise<void>;
}

// Admin auth context with signIn function
const AdminAuthContext = createContext<AuthContextType | null>(null);

export function AdminAuthProvider({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const { data: session, status } = useSession();
  const [state, setState] = useState<AuthState>({
    user: null,
    loading: true,
    error: null,
    initialized: false,
    navigating: false,
  });

  // Update state based on NextAuth session
  useEffect(() => {
    if (status === 'loading') {
      setState(prev => ({ ...prev, loading: true, initialized: false }));
      return;
    }

    if (status === 'authenticated' && session) {
      const user: AuthUser = {
        user_id: session.user.id,
        email: session.user.email,
        role: session.user.role,
        permissions: session.user.permissions,
        subscription_tier: session.user.subscription_tier,
        session_type: 'admin',
        expires_at: session.expires_at,
      };
      setState({
        user,
        loading: false,
        error: null,
        initialized: true,
        navigating: false,
      });
    } else {
      setState({
        user: null,
        loading: false,
        error: null,
        initialized: true,
        navigating: false,
      });
    }
  }, [session, status]);

  const signIn = async (email: string, password: string) => {
    setState(prev => ({ ...prev, loading: true, error: null, navigating: true }));
    
    try {
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
        router.push('/');
      }
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        error: error instanceof Error ? error.message : 'Login failed',
        loading: false,
        navigating: false
      }));
      throw error;
    }
  };

  const logout = async () => {
    setState(prev => ({ ...prev, loading: true, navigating: true }));
    try {
      await nextAuthSignOut({ redirect: false });
      router.push('/login');
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        error: error instanceof Error ? error.message : 'Logout failed',
        loading: false,
        navigating: false
      }));
    }
  };

  const checkAuth = async () => {
    // NextAuth handles this automatically through useSession
    return Promise.resolve();
  };

  const contextValue: AuthContextType = {
    ...state,
    signIn,
    logout,
    signOut: logout, // Alias for logout to match navigation component
    checkAuth,
  };

  return (
    <AdminAuthContext.Provider value={contextValue}>
      {children}
    </AdminAuthContext.Provider>
  );
}

export function useAdminAuth() {
  const context = useContext(AdminAuthContext);
  if (!context) {
    throw new Error('useAdminAuth must be used within AdminAuthProvider');
  }
  return context;
}