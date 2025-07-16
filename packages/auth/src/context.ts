'use client';

import { createContext, useContext, useCallback, useState } from 'react';
import type { ReactNode } from 'react';
import type { 
  AuthState, 
  AuthContextType, 
  AdminAuthState, 
  AdminAuthContextType,
  SignInCredentials,
  SignUpData 
} from './types';

// Base Auth Context
const AuthContext = createContext<AuthContextType | undefined>(undefined);

interface AuthProviderProps {
  children: ReactNode;
  authService?: any; // Firebase auth service instance
  onSignIn?: (idToken: string) => Promise<void>;
  onSignOut?: () => Promise<void>;
}

/**
 * Base Auth Provider
 * This is a foundation that apps can extend with their specific Firebase Auth implementation
 */
export function AuthProvider({ 
  children, 
  authService,
  onSignIn,
  onSignOut 
}: AuthProviderProps) {
  const [state, setState] = useState<AuthState>({
    user: null,
    loading: true,
    error: null,
    isInitialized: false,
  });

  const updateState = useCallback((updates: Partial<AuthState>) => {
    setState(prev => ({ ...prev, ...updates }));
  }, []);

  const handleAsyncOperation = useCallback(async (operation: () => Promise<any>) => {
    try {
      updateState({ loading: true, error: null });
      const result = await operation();
      updateState({ loading: false });
      return result;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'An error occurred';
      updateState({ loading: false, error: message });
      throw error;
    }
  }, [updateState]);

  // Auth actions (to be implemented by consuming apps)
  const signInWithEmailAndPassword = useCallback(async (credentials: SignInCredentials) => {
    if (!authService) {
      throw new Error('Auth service not provided');
    }
    await handleAsyncOperation(async () => {
      const user = await authService.signInWithEmailAndPassword(credentials);
      if (onSignIn) {
        const idToken = await user.getIdToken();
        await onSignIn(idToken);
      }
    });
  }, [authService, onSignIn, handleAsyncOperation]);

  const signInWithGoogle = useCallback(async () => {
    if (!authService) {
      throw new Error('Auth service not provided');
    }
    await handleAsyncOperation(async () => {
      const user = await authService.signInWithGoogle();
      if (onSignIn) {
        const idToken = await user.getIdToken();
        await onSignIn(idToken);
      }
    });
  }, [authService, onSignIn, handleAsyncOperation]);

  const signUp = useCallback(async (data: SignUpData) => {
    if (!authService) {
      throw new Error('Auth service not provided');
    }
    await handleAsyncOperation(async () => {
      const user = await authService.signUp(data);
      if (onSignIn) {
        const idToken = await user.getIdToken();
        await onSignIn(idToken);
      }
    });
  }, [authService, onSignIn, handleAsyncOperation]);

  const signOut = useCallback(async () => {
    if (!authService) {
      throw new Error('Auth service not provided');
    }
    await handleAsyncOperation(async () => {
      await authService.signOut();
      if (onSignOut) {
        await onSignOut();
      }
    });
  }, [authService, onSignOut, handleAsyncOperation]);

  const sendPasswordResetEmail = useCallback(async (email: string) => {
    if (!authService) {
      throw new Error('Auth service not provided');
    }
    await handleAsyncOperation(() => authService.sendPasswordResetEmail(email));
  }, [authService, handleAsyncOperation]);

  const sendEmailVerification = useCallback(async () => {
    if (!authService || !state.user) {
      throw new Error('Auth service not provided or no user found');
    }
    await handleAsyncOperation(() => authService.sendEmailVerification(state.user));
  }, [authService, state.user, handleAsyncOperation]);

  const refreshSession = useCallback(async () => {
    if (!state.user) return;
    await handleAsyncOperation(async () => {
      const token = await authService?.getCurrentUserToken(true);
      if (token && onSignIn) {
        await onSignIn(token);
      }
    });
  }, [authService, state.user, onSignIn, handleAsyncOperation]);

  const clearError = useCallback(() => {
    updateState({ error: null });
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

  // Note: JSX will be handled by the consuming apps
  // This package provides the logic, apps provide the JSX
  return { contextValue, AuthContext, children };
}

export function useAuth(): AuthContextType {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}

// Admin Auth Context
const AdminAuthContext = createContext<AdminAuthContextType | undefined>(undefined);

interface AdminAuthProviderProps {
  children: ReactNode;
  authService?: any; // Firebase auth service instance
  onSignIn?: (idToken: string) => Promise<void>;
  onSignOut?: () => Promise<void>;
}

/**
 * Admin Auth Provider
 * Extended version of AuthProvider with admin-specific functionality
 */
export function AdminAuthProvider({ 
  children, 
  authService,
  onSignOut 
}: AdminAuthProviderProps) {
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

  const signIn = useCallback(async (email: string, _password: string) => {
    if (!authService) {
      throw new Error('Auth service not provided');
    }
    
    try {
      updateState({ loading: true, error: null });
      
      // Implementation would use Firebase Auth here
      console.log('Admin sign in:', email);
      // const userCredential = await signInWithEmailAndPassword(auth, email, password);
      // const user = userCredential.user;
      
      // Check if user has admin role
      // const idTokenResult = await user.getIdTokenResult();
      // const isAdmin = idTokenResult.claims.role === 'ADMIN';
      
      // For now, just placeholder
      updateState({
        loading: false,
        error: 'Implementation needed: Connect to Firebase Auth',
      });
    } catch (error: any) {
      console.error('Admin sign in failed:', error);
      updateState({
        loading: false,
        error: error.message || 'Sign in failed',
      });
      throw error;
    }
  }, [authService, updateState]);

  const signOutAdmin = useCallback(async () => {
    try {
      updateState({ loading: true, error: null });
      if (onSignOut) {
        await onSignOut();
      }
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
  }, [onSignOut, updateState]);

  const checkAdminStatus = useCallback(async (): Promise<boolean> => {
    if (!state.user) return false;
    // Implementation would check Firebase custom claims
    console.log('Checking admin status for user:', state.user);
    return false; // Placeholder
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

  // Note: JSX will be handled by the consuming apps
  // This package provides the logic, apps provide the JSX
  return { contextValue, AdminAuthContext, children };
}

export function useAdminAuth(): AdminAuthContextType {
  const context = useContext(AdminAuthContext);
  if (context === undefined) {
    throw new Error('useAdminAuth must be used within an AdminAuthProvider');
  }
  return context;
}

// Export contexts for apps to use
export { AuthContext, AdminAuthContext };
