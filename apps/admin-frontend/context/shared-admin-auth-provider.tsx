'use client';

import React, { createContext, useContext, useState, useCallback } from 'react';
import { auth } from '@/lib/firebase';
import { signInWithEmailAndPassword, signOut as firebaseSignOut } from 'firebase/auth';

// Local types for admin auth
export interface AdminAuthState {
  user: any | null;
  loading: boolean;
  error: string | null;
  isAuthenticated: boolean;
  claims: any;
  isInitialized: boolean;
  isAdmin: boolean;
}

export interface AdminAuthContextType {
  state: AdminAuthState;
  signIn: (email: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;
  refreshToken: () => Promise<void>;
  checkAdminStatus: () => Promise<boolean>;
  clearError: () => void;
}

// Create the admin auth context
const AdminAuthContext = createContext<AdminAuthContextType | undefined>(undefined);

interface AppAdminAuthProviderProps {
  children: React.ReactNode;
}

/**
 * Admin-specific Auth Provider that uses the shared auth package
 * with admin-specific authentication logic
 */
export function AppAdminAuthProvider({ children }: AppAdminAuthProviderProps) {
  const [state, setState] = useState<AdminAuthState>({
    user: null,
    loading: true,
    error: null,
    isAuthenticated: false,
    claims: null,
    isInitialized: false,
    isAdmin: false,
  });

  const updateState = useCallback((updates: Partial<AdminAuthState>) => {
    setState(prev => ({ ...prev, ...updates }));
  }, []);

  const signIn = useCallback(async (email: string, password: string) => {
    try {
      updateState({ loading: true, error: null });
      
      const userCredential = await signInWithEmailAndPassword(auth, email, password);
      const user = userCredential.user;
      
      // Check if user has admin role
      const idTokenResult = await user.getIdTokenResult();
      const isAdmin = idTokenResult.claims.role === 'ADMIN';
      
      if (!isAdmin) {
        await firebaseSignOut(auth);
        throw new Error('Access denied. Admin privileges required.');
      }
      
      // Create session using shared session management
      updateState({
        user,
        loading: false,
        isAdmin: true,
        error: null,
        isAuthenticated: true,
        claims: await user.getIdTokenResult(),
      });
    } catch (error: any) {
      console.error('Admin sign in failed:', error);
      updateState({
        loading: false,
        error: error.message || 'Sign in failed',
      });
      throw error;
    }
  }, [updateState]);

    const signOutAdmin = useCallback(async () => {
    try {
      updateState({ loading: true, error: null });
      await firebaseSignOut(auth);
      
      updateState({
        user: null,
        loading: false,
        isAdmin: false,
        error: null,
        isAuthenticated: false,
        claims: null,
      });
    } catch (error: any) {
      console.error('Sign out failed:', error);
      updateState({
        loading: false,
        error: error.message || 'Sign out failed',
      });
      throw error;
    }
  }, [updateState]);

  const checkAdminStatus = useCallback(async (): Promise<boolean> => {
    if (!state.user) return false;
    try {
      const idTokenResult = await state.user.getIdTokenResult();
      return idTokenResult.claims.role === 'ADMIN';
    } catch (error) {
      console.error('Failed to check admin status:', error);
      return false;
    }
  }, [state.user]);

  const clearError = useCallback(() => {
    updateState({ error: null });
  }, [updateState]);

  const contextValue: AdminAuthContextType = {
    state,
    signIn,
    signOut: signOutAdmin,
    refreshToken: async () => {
      // Implement refresh token logic if needed
      const user = auth.currentUser;
      if (user) {
        await user.getIdToken(true);
      }
    },
    checkAdminStatus,
    clearError,
  };

  return (
    <AdminAuthContext.Provider value={contextValue}>
      {children}
    </AdminAuthContext.Provider>
  );
}

/**
 * Hook to access the admin auth context
 */
export function useAdminAuth(): AdminAuthContextType {
  const context = useContext(AdminAuthContext);
  if (context === undefined) {
    throw new Error('useAdminAuth must be used within an AppAdminAuthProvider');
  }
  return context;
}
