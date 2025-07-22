'use client';

import { auth } from '@/lib/firebase';
import { authService } from '@/services/auth/auth.service';
import type { AuthContextType, AuthState } from '@epsx/auth';
import { handleSignIn, handleSignOut } from '@epsx/auth/actions';
import { onAuthStateChanged } from 'firebase/auth';
import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
} from 'react';

// Create the auth context
const AppAuthContext = createContext<AuthContextType | undefined>(undefined);

interface AppAuthProviderProps {
  children: React.ReactNode;
}

/**
 * App-specific Auth Provider that uses the shared auth package
 * with your Firebase Auth implementation
 */
export function AppAuthProvider({ children }: AppAuthProviderProps) {
  const [state, setState] = useState<AuthState>({
    user: null,
    loading: true,
    error: null,
    isInitialized: false,
  });

  const updateState = useCallback((updates: Partial<AuthState>) => {
    setState((prev) => ({ ...prev, ...updates }));
  }, []);

  const handleAsyncOperation = useCallback(
    async (operation: () => Promise<any>) => {
      try {
        updateState({ loading: true, error: null });
        const result = await operation();
        updateState({ loading: false });
        return result;
      } catch (error) {
        const message =
          error instanceof Error ? error.message : 'An error occurred';
        updateState({ loading: false, error: message });
        throw error;
      }
    },
    [updateState],
  );

  // Implement auth actions using shared session management
  const signInWithEmailAndPassword = useCallback(
    async (credentials: { email: string; password: string }) => {
      await handleAsyncOperation(async () => {
        const user = await authService.signInWithEmailAndPassword(credentials);
        const idToken = await user.getIdToken();
        await handleSignIn(idToken);
      });
    },
    [handleAsyncOperation],
  );

  const signInWithGoogle = useCallback(async () => {
    await handleAsyncOperation(async () => {
      const user = await authService.signInWithGoogle();
      const idToken = await user.getIdToken();
      await handleSignIn(idToken);
    });
  }, [handleAsyncOperation]);

  const signUp = useCallback(
    async (data: { email: string; password: string; displayName?: string }) => {
      await handleAsyncOperation(async () => {
        const user = await authService.signUp(data);
        const idToken = await user.getIdToken();
        await handleSignIn(idToken);
      });
    },
    [handleAsyncOperation],
  );

  const signOut = useCallback(async () => {
    await handleAsyncOperation(async () => {
      await authService.signOut();
      await handleSignOut();
    });
  }, [handleAsyncOperation]);

  const sendPasswordResetEmail = useCallback(
    async (email: string) => {
      await handleAsyncOperation(() =>
        authService.sendPasswordResetEmail(email),
      );
    },
    [handleAsyncOperation],
  );

  const sendEmailVerification = useCallback(async () => {
    if (!state.user) {
      throw new Error('No user found');
    }
    await handleAsyncOperation(() =>
      authService.sendEmailVerification(state.user),
    );
  }, [state.user, handleAsyncOperation]);

  const refreshSession = useCallback(async () => {
    if (!state.user) return;
    await handleAsyncOperation(async () => {
      const token = await authService.getCurrentUserToken(true);
      if (token) {
        await handleSignIn(token);
      }
    });
  }, [state.user, handleAsyncOperation]);

  const clearError = useCallback(() => {
    updateState({ error: null });
  }, [updateState]);

  // Initialize auth state by listening to Firebase auth state changes
  useEffect(() => {
    const unsubscribe = onAuthStateChanged(auth, async (user) => {
      console.log('Auth state changed:', { hasUser: !!user, userEmail: user?.email });
      if (user) {
        try {
          // Set session when user is authenticated
          console.log('Setting session for user:', user.email);
          const idToken = await user.getIdToken();
          const result = await handleSignIn(idToken);
          console.log('Session creation result:', result);
          
          updateState({
            user,
            loading: false,
            isInitialized: true,
            error: null,
          });
        } catch (error) {
          console.error('Failed to initialize user state:', error);
          updateState({
            user,
            loading: false,
            isInitialized: true,
            error: 'Failed to initialize user session',
          });
        }
      } else {
        console.log('User signed out, clearing session');
        await handleSignOut();
        updateState({
          user: null,
          loading: false,
          isInitialized: true,
          error: null,
        });
      }
    });

    return unsubscribe;
  }, [updateState]);

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
    <AppAuthContext.Provider value={contextValue}>
      {children}
    </AppAuthContext.Provider>
  );
}

/**
 * Hook to access the auth context
 */
export function useAuth(): AuthContextType {
  const context = useContext(AppAuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AppAuthProvider');
  }
  return context;
}
