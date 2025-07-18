'use client';

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import type { ReactNode } from 'react';
import { onAuthStateChanged } from 'firebase/auth';
import type { User as FirebaseUser } from 'firebase/auth';

import { auth } from '@/lib/firebase';
import { authService } from '@/services/auth/auth.service';
import { handleSignIn, handleSignOut } from '@/app/actions/auth';
import { initializeUserClaims } from '@/lib/custom-claims';
import { AuthError, ErrorCode } from '@/types/auth/errors';
import type { SignInCredentials, SignUpData } from '@/types/auth/service';

export interface AuthState {
  user: FirebaseUser | null;
  loading: boolean;
  error: string | null;
  isInitialized: boolean;
}

export interface AuthActions {
  signInWithEmailAndPassword: (credentials: SignInCredentials) => Promise<void>;
  signInWithGoogle: () => Promise<void>;
  signUp: (data: SignUpData) => Promise<void>;
  signOut: () => Promise<void>;
  sendPasswordResetEmail: (email: string) => Promise<void>;
  sendEmailVerification: () => Promise<void>;
  clearError: () => void;
  refreshSession: () => Promise<void>;
}

export type AuthContextType = AuthState & AuthActions;

const AuthContext = createContext<AuthContextType | undefined>(undefined);

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps): React.ReactElement {
  const [state, setState] = useState<AuthState>({
    user: null,
    loading: true,
    error: null,
    isInitialized: false,
  });

  // Helper function to update state
  const updateState = useCallback((updates: Partial<AuthState>) => {
    setState(prev => ({ ...prev, ...updates }));
  }, []);

  // Helper function to handle async operations
  const handleAsyncOperation = useCallback(async (
    operation: () => Promise<any>,
    onSuccess?: (result: any) => void
  ): Promise<any> => {
    try {
      updateState({ loading: true, error: null });
      const result = await operation();
      onSuccess?.(result);
      return result;
    } catch (error) {
      const errorMessage = error instanceof AuthError 
        ? error.message 
        : 'An unexpected error occurred';
      updateState({ error: errorMessage });
      throw error;
    } finally {
      updateState({ loading: false });
    }
  }, [updateState]);

  // Initialize auth listener
  useEffect(() => {
    let mounted = true;

    const unsubscribe = onAuthStateChanged(auth, async (user) => {
      if (!mounted) return;

      try {
        if (user) {
          // Check if this is a new user (just created) by checking metadata
          const isNewUser = user.metadata.creationTime === user.metadata.lastSignInTime;
          
          // User signed in - create session
          const idToken = await user.getIdToken();
          await handleSignIn(idToken);
          
          // Initialize custom claims for new users
          if (isNewUser && user.email) {
            try {
              await initializeUserClaims(user.uid, user.email);
              console.log('Custom claims initialized for new user:', user.uid);
            } catch (error) {
              console.error('Failed to initialize custom claims:', error);
              // Don't block sign up for this error
            }
          }
        } else {
          // User signed out - clear session
          await handleSignOut();
        }

        if (mounted) {
          updateState({
            user,
            loading: false,
            isInitialized: true,
            error: null,
          });
        }
      } catch (error) {
        console.error('Auth state change error:', error);
        if (mounted) {
          updateState({
            user,
            loading: false,
            isInitialized: true,
            error: 'Failed to sync authentication state',
          });
        }
      }
    });

    return () => {
      mounted = false;
      unsubscribe();
    };
  }, [updateState]);

  // Auth actions
  const signInWithEmailAndPassword = useCallback(async (credentials: SignInCredentials) => {
    await handleAsyncOperation(() => authService.signInWithEmailAndPassword(credentials));
  }, [handleAsyncOperation]);

  const signInWithGoogle = useCallback(async () => {
    await handleAsyncOperation(() => authService.signInWithGoogle());
  }, [handleAsyncOperation]);

  const signUp = useCallback(async (data: SignUpData) => {
    await handleAsyncOperation(() => authService.signUp(data));
  }, [handleAsyncOperation]);

  const signOut = useCallback(async () => {
    await handleAsyncOperation(() => authService.signOut());
  }, [handleAsyncOperation]);

  const sendPasswordResetEmail = useCallback(async (email: string) => {
    await handleAsyncOperation(() => authService.sendPasswordResetEmail(email));
  }, [handleAsyncOperation]);

  const sendEmailVerification = useCallback(async () => {
    if (!state.user) {
      throw new AuthError(ErrorCode.USER_NOT_FOUND, 'No user found');
    }
    await handleAsyncOperation(() => authService.sendEmailVerification(state.user!));
  }, [handleAsyncOperation, state.user]);

  const clearError = useCallback(() => {
    updateState({ error: null });
  }, [updateState]);

  const refreshSession = useCallback(async () => {
    if (!state.user) return;
    
    await handleAsyncOperation(async () => {
      const token = await authService.getCurrentUserToken(true);
      if (token) {
        await handleSignIn(token);
      }
    });
  }, [handleAsyncOperation, state.user]);

  const contextValue: AuthContextType = {
    ...state,
    signInWithEmailAndPassword,
    signInWithGoogle,
    signUp,
    signOut,
    sendPasswordResetEmail,
    sendEmailVerification,
    clearError,
    refreshSession,
  };

  return (
    <AuthContext.Provider value={contextValue}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextType {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
