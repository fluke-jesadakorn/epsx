'use client';

import React, { createContext, useContext, useState, useCallback } from 'react';
import type { AdminAuthState, AdminAuthContextType } from '@epsx/auth';
import { handleSignIn, handleSignOut } from '@epsx/auth/actions';
import { auth } from '@/lib/firebase';
import { signInWithEmailAndPassword, signOut as firebaseSignOut } from 'firebase/auth';

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
      const idToken = await user.getIdToken();
      await handleSignIn(idToken);
      
      updateState({
        user,
        loading: false,
        isAdmin: true,
        error: null,
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
      await handleSignOut();
      updateState({
        user: null,
        loading: false,
        isAdmin: false,
        error: null,
      });
    } catch (error: any) {
      console.error('Sign out failed:', error);
      updateState({
        loading: false,
        error: error.message || 'Sign out failed',
      });
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
    ...state,
    signIn,
    signOut: signOutAdmin,
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
