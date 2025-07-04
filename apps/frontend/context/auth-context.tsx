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

import { handleSignIn, handleSignOut, refreshSession as _refreshSession } from '@/app/actions/auth';
import { auth } from '@/lib/firebase';

interface AuthContextType {
  user: User | null;
  loading: boolean;
  error: string | null;
  signInWithGoogle: () => Promise<void>;
  signInWithEmailPassword: (email: string, password: string) => Promise<void>;
  signUp: (email: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;
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

  // Listen for token changes and implement token refresh
  useEffect(() => {
    const unsubscribe = onIdTokenChanged(auth, async (user) => {
      if (user) {
        try {
          // Force token refresh to get a fresh token
          const idToken = await user.getIdToken(true);
          // Use server action to create session
          await handleSignIn(idToken);
          setUser(user);
        } catch (error) {
          console.error('Session sync error:', error);
          setError('Failed to sync session');
          setUser(null);
        }
      } else {
        // User signed out - clear session
        try {
          await handleSignOut();
        } catch (error) {
          console.error('Session cleanup error:', error);
        }
        setUser(null);
      }
      setLoading(false);
    });

    // Set up token refresh interval (refresh every 50 minutes, tokens expire after 1 hour)
    const tokenRefreshInterval = setInterval(async () => {
      const currentUser = auth.currentUser;
      if (currentUser) {
        try {
          console.log('Refreshing token...');
          const freshToken = await currentUser.getIdToken(true);
          await handleSignIn(freshToken);
          console.log('Token refreshed successfully');
        } catch (error) {
          console.error('Token refresh failed:', error);
        }
      }
    }, 50 * 60 * 1000); // 50 minutes

    return () => {
      unsubscribe();
      clearInterval(tokenRefreshInterval);
    };
  }, []);

  const signInWithGoogle = async (): Promise<void> => {
    try {
      setError(null);
      const provider = new GoogleAuthProvider();
      await signInWithPopup(auth, provider);
    } catch (error) {
      console.error('Google sign-in error:', error);
      setError('Failed to sign in with Google');
      throw error;
    }
  };

  const signInWithEmailPassword = async (
    email: string,
    password: string,
  ): Promise<void> => {
    try {
      setError(null);
      await signInWithEmailPwd(auth, email, password);
    } catch (error) {
      console.error('Email/Password sign-in error:', error);
      setError('Failed to sign in with email/password');
      throw error;
    }
  };

  const signUp = async (email: string, password: string): Promise<void> => {
    try {
      setError(null);
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
    }
  };

  const signOut = async (): Promise<void> => {
    try {
      setError(null);
      await firebaseSignOut(auth);
      // Use server action to clear session
      await handleSignOut();
    } catch (error) {
      console.error('Sign-out error:', error);
      setError('Failed to sign out');
      throw error;
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
