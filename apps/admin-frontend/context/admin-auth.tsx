'use client';

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import type { ReactNode } from 'react';
import { onAuthStateChanged, signInWithEmailAndPassword, signOut } from 'firebase/auth';
import type { User as FirebaseUser } from 'firebase/auth';
import { auth } from '@/lib/firebase';

export interface AdminAuthState {
  user: FirebaseUser | null;
  loading: boolean;
  error: string | null;
  isInitialized: boolean;
  isAdmin: boolean;
}

export interface AdminAuthActions {
  signIn: (email: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;
  checkAdminStatus: () => Promise<boolean>;
  clearError: () => void;
}

export type AdminAuthContextType = AdminAuthState & AdminAuthActions;

const AdminAuthContext = createContext<AdminAuthContextType | undefined>(undefined);

interface AdminAuthProviderProps {
  children: ReactNode;
}

export function AdminAuthProvider({ children }: AdminAuthProviderProps): React.ReactElement {
  const [state, setState] = useState<AdminAuthState>({
    user: null,
    loading: true,
    error: null,
    isInitialized: false,
    isAdmin: false,
  });

  // Helper function to update state
  const updateState = useCallback((updates: Partial<AdminAuthState>) => {
    setState(prev => ({ ...prev, ...updates }));
  }, []);

  // Check if user has admin role
  const checkAdminStatus = useCallback(async (): Promise<boolean> => {
    try {
      if (!state.user) return false;
      
      const idTokenResult = await state.user.getIdTokenResult();
      const isAdmin = idTokenResult.claims.role === 'ADMIN';
      
      updateState({ isAdmin });
      return isAdmin;
    } catch (error) {
      console.error('Failed to check admin status:', error);
      updateState({ isAdmin: false });
      return false;
    }
  }, [state.user, updateState]);

  // Initialize auth state
  useEffect(() => {
    const unsubscribe = onAuthStateChanged(auth, async (user) => {
      if (user) {
        // Check admin status
        try {
          const idTokenResult = await user.getIdTokenResult();
          const isAdmin = idTokenResult.claims.role === 'ADMIN';
          
          updateState({
            user,
            loading: false,
            isInitialized: true,
            isAdmin,
            error: null,
          });
        } catch (error) {
          console.error('Failed to get user claims:', error);
          updateState({
            user,
            loading: false,
            isInitialized: true,
            isAdmin: false,
            error: 'Failed to verify admin permissions',
          });
        }
      } else {
        updateState({
          user: null,
          loading: false,
          isInitialized: true,
          isAdmin: false,
          error: null,
        });
      }
    });

    return unsubscribe;
  }, [updateState]);

  // Sign in function
  const signIn = useCallback(async (email: string, password: string) => {
    try {
      updateState({ loading: true, error: null });
      
      const userCredential = await signInWithEmailAndPassword(auth, email, password);
      const user = userCredential.user;
      
      // Check if user has admin role
      const idTokenResult = await user.getIdTokenResult();
      const isAdmin = idTokenResult.claims.role === 'ADMIN';
      
      if (!isAdmin) {
        await signOut(auth);
        throw new Error('Access denied. Admin privileges required.');
      }
      
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

  // Sign out function
  const signOutAdmin = useCallback(async () => {
    try {
      updateState({ loading: true, error: null });
      await signOut(auth);
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

  // Clear error function
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

export function useAdminAuth(): AdminAuthContextType {
  const context = useContext(AdminAuthContext);
  if (context === undefined) {
    throw new Error('useAdminAuth must be used within an AdminAuthProvider');
  }
  return context;
}
