'use client';

import React, { createContext, useContext, useEffect, useState } from 'react';
import type { ReactNode } from 'react';

import type { User } from 'firebase/auth';
import {
  createUserWithEmailAndPassword,
  GoogleAuthProvider,
  onIdTokenChanged,
  signInWithEmailAndPassword as signInWithEmailPwd,
  signInWithPopup,
  signOut as firebaseSignOut,
} from 'firebase/auth';

import { handleSignOut, refreshSession as _refreshSession } from '@/app/actions/auth';
import { auth } from '@/lib/firebase';

interface AuthContextType {
  user: User | null;
  loading: boolean;
  error: string | null;
  signInWithGoogle: () => Promise<void>;
  signInWithEmailPassword: (email: string, password: string) => Promise<void>;
  signUp: (email: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;
  clearSession: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({
  children,
}: {
  children: ReactNode;
}): React.ReactElement {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    console.log('Auth context: Setting up onIdTokenChanged listener');
    let hasHandledInitialAuth = false;
    let lastTokenTime = 0; // Track last token processing time to avoid rapid updates
    let authChangeCount = 0; // Track rapid auth changes to prevent loops
    let sessionCreatedSuccessfully = false; // Track if session was successfully created

    const unsubscribe = onIdTokenChanged(auth, async (user) => {
      authChangeCount++;
      console.log('Auth context: onIdTokenChanged fired', { 
        user: user ? user.email : 'null',
        hasHandledInitialAuth,
        isInitialized,
        authChangeCount,
        sessionCreatedSuccessfully
      });

      // Prevent excessive auth state changes that might indicate a loop
      if (authChangeCount > 10) {
        console.warn('Auth context: Too many auth state changes detected, potential loop. Stopping processing.');
        setLoading(false);
        return;
      }
      
      try {
        setUser(user);
        
        if (user) {
          // Only handle sign-in if this is the initial auth or if enough time has passed since last update
          const now = Date.now();
          const shouldProcessToken = !hasHandledInitialAuth || (now - lastTokenTime > 5000); // 5 second minimum between updates
          
          if (shouldProcessToken && !sessionCreatedSuccessfully) {
            console.log('Auth context: User found, storing auth state client-side');
            const idToken = await user.getIdToken();
            
            // Store authentication state client-side only
            if (typeof window !== 'undefined') {
              localStorage.setItem('authToken', idToken);
              localStorage.setItem('userEmail', user.email || '');
              localStorage.setItem('userId', user.uid);
              localStorage.setItem('sessionJustCreated', 'true');
            }
            
            console.log('Auth context: Client-side auth state set successfully');
            lastTokenTime = now;
            sessionCreatedSuccessfully = true;
          } else {
            console.log('Auth context: Skipping token update (too recent or session already created)');
          }
        } else {
          // Handle sign-out - clear client-side auth state
          console.log('Auth context: No user, clearing client-side auth state');
          if (typeof window !== 'undefined') {
            localStorage.removeItem('authToken');
            localStorage.removeItem('userEmail');
            localStorage.removeItem('userId');
            localStorage.removeItem('sessionJustCreated');
          }
          await handleSignOut();
          console.log('Auth context: Client-side auth state cleared');
        }
      } catch (error) {
        console.error('Auth context: Error in auth flow', error);
        if (user) {
          setError('Failed to create session');
        }
      }

      // Mark as initialized after first auth state change
      if (!hasHandledInitialAuth) {
        hasHandledInitialAuth = true;
        setIsInitialized(true);
        console.log('Auth context: Marking as initialized');
      }
      
      // Always set loading to false after processing
      setLoading(false);
    });

    // Reset auth change counter after a period to allow for legitimate auth changes
    const resetCounterInterval = setInterval(() => {
      if (authChangeCount > 0) {
        console.log('Auth context: Resetting auth change counter');
        authChangeCount = 0;
      }
    }, 30000); // Reset every 30 seconds

    return () => {
      console.log('Auth context: Cleaning up onIdTokenChanged listener');
      unsubscribe();
      clearInterval(resetCounterInterval);
    };
  }, []);

  const signInWithGoogle = async (): Promise<void> => {
    try {
      setError(null);
      setLoading(true);
      const provider = new GoogleAuthProvider();
      await signInWithPopup(auth, provider);
    } catch (error) {
      console.error('Google sign-in error:', error);
      setError('Failed to sign in with Google');
      throw error;
    } finally {
      setLoading(false);
    }
  };

  const signInWithEmailPassword = async (
    email: string,
    password: string,
  ): Promise<void> => {
    try {
      setError(null);
      setLoading(true);
      await signInWithEmailPwd(auth, email, password);
    } catch (error) {
      console.error('Email/Password sign-in error:', error);
      setError('Failed to sign in with email/password');
      throw error;
    } finally {
      setLoading(false);
    }
  };

  const signUp = async (email: string, password: string): Promise<void> => {
    try {
      setError(null);
      setLoading(true);
      await createUserWithEmailAndPassword(auth, email, password);
    } catch (error: any) {
      console.error('Sign-up error:', error);
      let errorMessage = 'Failed to create account';
      
      if (error.code) {
        switch (error.code) {
          case 'auth/email-already-in-use':
            errorMessage = 'This email is already in use';
            break;
          case 'auth/invalid-email':
            errorMessage = 'Please enter a valid email address';
            break;
          case 'auth/weak-password':
            errorMessage = 'Password should be at least 6 characters';
            break;
          default:
            errorMessage = error.message || 'Failed to create account';
        }
      }
      
      setError(errorMessage);
      throw error;
    } finally {
      setLoading(false);
    }
  };

  const signOut = async (): Promise<void> => {
    try {
      setError(null);
      setLoading(true);
      
      // Clear client-side auth state first
      if (typeof window !== 'undefined') {
        localStorage.removeItem('authToken');
        localStorage.removeItem('userEmail');
        localStorage.removeItem('userId');
        localStorage.removeItem('sessionJustCreated');
      }
      
      await firebaseSignOut(auth);
      // Use server action to clear any remaining session
      await handleSignOut();
    } catch (error) {
      console.error('Sign-out error:', error);
      setError('Failed to sign out');
      throw error;
    } finally {
      setLoading(false);
    }
  };

  const clearSession = async (): Promise<void> => {
    try {
      setError(null);
      setLoading(true);
      console.log('Auth context: Clearing session - clearing localStorage');
      
      // Clear client-side auth state
      if (typeof window !== 'undefined') {
        localStorage.removeItem('authToken');
        localStorage.removeItem('userEmail');
        localStorage.removeItem('userId');
        localStorage.removeItem('sessionJustCreated');
      }
      
      console.log('Auth context: Clearing session - signing out from Firebase');
      await firebaseSignOut(auth);
      console.log('Auth context: Clearing session - calling handleSignOut');
      await handleSignOut();
      console.log('Auth context: Session cleared successfully');
      setUser(null);
    } catch (error) {
      console.error('Clear session error:', error);
      setError('Failed to clear session');
      throw error;
    } finally {
      setLoading(false);
    }
  };

  const value = {
    user,
    loading,
    error,
    signInWithGoogle,
    signInWithEmailPassword,
    signUp,
    signOut,
    clearSession,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextType {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
