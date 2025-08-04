'use client';

import React, { createContext, useContext, useState, useEffect } from 'react';
import { loginAction, logoutAction } from '@/lib/actions/server-auth';
import { adminAuthAPI } from '@/lib/auth-api';

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
  checkAuth: () => Promise<void>;
}

// Admin auth context with signIn function
const AdminAuthContext = createContext<AuthContextType | null>(null);

export function AdminAuthProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<AuthState>({
    user: null,
    loading: true,
    error: null,
    initialized: false,
    navigating: false,
  });

  const signIn = async (email: string, password: string) => {
    setState(prev => ({ ...prev, loading: true, error: null, navigating: true }));
    
    try {
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
      const user = await adminAuthAPI.getCurrentUser();
      setState(prev => ({ ...prev, user, loading: false, navigating: false, initialized: true }));
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
      // Call server action (this will redirect automatically)
      await logoutAction();
      setState({ user: null, loading: false, error: null, initialized: true, navigating: false });
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
    setState(prev => ({ ...prev, loading: true }));
    try {
      const user = await adminAuthAPI.getCurrentUser();
      setState({ user, loading: false, error: null, initialized: true, navigating: false });
    } catch (error) {
      setState({ user: null, loading: false, error: null, initialized: true, navigating: false });
    }
  };

  useEffect(() => {
    checkAuth();
  }, []);

  const contextValue: AuthContextType = {
    ...state,
    signIn,
    logout,
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