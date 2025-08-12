'use client';

// Multi-Provider Authentication Context
// Unified authentication context supporting Firebase + OIDC providers

import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import { User as FirebaseUser } from 'firebase/auth';

// Import Firebase authentication
import { 
  getFirebaseAuth,
  getFirebaseTokenValidator,
  type FirebaseUserProfile,
  type FirebaseAuthResult,
  FirebaseAuthError
} from '@epsx/firebase-analytics';

/**
 * Unified user profile that works across all providers
 */
export interface UnifiedUserProfile {
  // Core identifiers
  id: string;
  email: string | null;
  
  // Display information
  displayName: string | null;
  photoURL: string | null;
  
  // Verification status
  emailVerified: boolean;
  
  // Provider information
  provider: 'firebase' | 'oidc' | 'google' | 'github';
  providerId: string;
  providerUserId: string; // Firebase UID, OIDC sub, etc.
  
  // Application-specific data
  role: string;
  permissions: string[];
  subscriptionTier?: string;
  
  // Metadata
  createdAt?: string;
  lastSignInAt?: string;
}

/**
 * Unified JWT token from backend
 */
export interface UnifiedJWT {
  access_token: string;
  token_type: string;
  expires_at: string;
  expires_in: number;
  session_id: string;
  jti: string;
  refresh_token?: string;
}

/**
 * Multi-provider authentication state
 */
export interface MultiProviderAuthState {
  // Authentication status
  isAuthenticated: boolean;
  isLoading: boolean;
  isInitialized: boolean;
  
  // User information
  user: UnifiedUserProfile | null;
  firebaseUser: FirebaseUser | null;
  
  // Tokens
  accessToken: string | null;
  unifiedJWT: UnifiedJWT | null;
  
  // Provider information
  currentProvider: 'firebase' | 'oidc' | null;
  
  // Error handling
  error: string | null;
  
  // Authentication methods
  loginWithGoogle: () => Promise<void>;
  loginWithGitHub: () => Promise<void>;
  loginWithCredentials: (email: string, password: string) => Promise<void>;
  loginWithOIDC: () => Promise<void>;
  
  // Account management
  createAccount: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  
  // Token management
  refreshToken: () => Promise<boolean>;
  getValidToken: () => Promise<string | null>;
  
  // Utilities
  clearError: () => void;
}

const MultiProviderAuthContext = createContext<MultiProviderAuthState | undefined>(undefined);

/**
 * Multi-provider authentication provider props
 */
interface MultiProviderAuthProviderProps {
  children: ReactNode;
  enableFirebase?: boolean;
  enableOIDC?: boolean;
  backendUrl?: string;
}

/**
 * Multi-provider authentication provider component
 */
export function MultiProviderAuthProvider({
  children,
  enableFirebase = true,
  enableOIDC = true,
  backendUrl
}: MultiProviderAuthProviderProps) {
  // State management
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isInitialized, setIsInitialized] = useState(false);
  const [user, setUser] = useState<UnifiedUserProfile | null>(null);
  const [firebaseUser, setFirebaseUser] = useState<FirebaseUser | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [unifiedJWT, setUnifiedJWT] = useState<UnifiedJWT | null>(null);
  const [currentProvider, setCurrentProvider] = useState<'firebase' | 'oidc' | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Firebase authentication manager
  const firebaseAuth = enableFirebase ? getFirebaseAuth() : null;
  const tokenValidator = enableFirebase ? getFirebaseTokenValidator() : null;

  /**
   * Clear any authentication errors
   */
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  /**
   * Map Firebase user to unified profile
   */
  const mapFirebaseUserToUnified = useCallback((
    firebaseUser: FirebaseUser,
    unifiedToken: UnifiedJWT
  ): UnifiedUserProfile => {
    // Parse JWT to extract backend claims
    let claims: any = {};
    try {
      const payload = JSON.parse(atob(unifiedToken.access_token.split('.')[1]));
      claims = payload;
    } catch (e) {
      console.warn('Failed to parse JWT claims:', e);
    }

    return {
      id: claims.sub || firebaseUser.uid,
      email: firebaseUser.email,
      displayName: firebaseUser.displayName,
      photoURL: firebaseUser.photoURL,
      emailVerified: firebaseUser.emailVerified,
      provider: 'firebase',
      providerId: firebaseUser.providerId,
      providerUserId: firebaseUser.uid,
      role: claims.role || 'user',
      permissions: claims.permissions || [],
      subscriptionTier: claims.subscription_tier,
      createdAt: firebaseUser.metadata.creationTime,
      lastSignInAt: firebaseUser.metadata.lastSignInTime,
    };
  }, []);

  /**
   * Handle Firebase authentication state changes
   */
  const handleFirebaseAuthStateChange = useCallback(async (firebaseUser: FirebaseUser | null) => {
    setIsLoading(true);
    setFirebaseUser(firebaseUser);

    if (!firebaseUser) {
      // User signed out
      setIsAuthenticated(false);
      setUser(null);
      setAccessToken(null);
      setUnifiedJWT(null);
      setCurrentProvider(null);
      setIsLoading(false);
      return;
    }

    try {
      // User signed in, exchange Firebase token for unified JWT
      if (tokenValidator) {
        const unifiedToken = await tokenValidator.getCurrentUserToken();
        
        if (unifiedToken) {
          const unifiedProfile = mapFirebaseUserToUnified(firebaseUser, unifiedToken);
          
          setUser(unifiedProfile);
          setAccessToken(unifiedToken.access_token);
          setUnifiedJWT(unifiedToken);
          setCurrentProvider('firebase');
          setIsAuthenticated(true);
          setError(null);
        } else {
          throw new Error('Failed to obtain unified token');
        }
      }
    } catch (error) {
      console.error('Firebase auth state change error:', error);
      setError(error instanceof Error ? error.message : 'Authentication failed');
      setIsAuthenticated(false);
      setUser(null);
      setAccessToken(null);
      setUnifiedJWT(null);
    } finally {
      setIsLoading(false);
    }
  }, [tokenValidator, mapFirebaseUserToUnified]);

  /**
   * Initialize authentication providers
   */
  useEffect(() => {
    let unsubscribeFirebase: (() => void) | null = null;

    const initializeAuth = async () => {
      try {
        if (enableFirebase && firebaseAuth) {
          // Set up Firebase auth state listener
          unsubscribeFirebase = firebaseAuth.onAuthStateChanged(handleFirebaseAuthStateChange);
        }

        // TODO: Initialize OIDC provider if enabled

        setIsInitialized(true);
      } catch (error) {
        console.error('Failed to initialize authentication:', error);
        setError('Authentication initialization failed');
        setIsLoading(false);
      }
    };

    initializeAuth();

    return () => {
      if (unsubscribeFirebase) {
        unsubscribeFirebase();
      }
    };
  }, [enableFirebase, enableOIDC, firebaseAuth, handleFirebaseAuthStateChange]);

  /**
   * Login with Google OAuth
   */
  const loginWithGoogle = useCallback(async () => {
    if (!firebaseAuth) {
      setError('Firebase authentication not enabled');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const result = await firebaseAuth.signInWithGoogle();
      console.log('Google sign-in successful:', result.user.email);
    } catch (error) {
      const errorMessage = error instanceof FirebaseAuthError 
        ? error.getUserFriendlyMessage()
        : 'Google sign-in failed';
      setError(errorMessage);
      console.error('Google sign-in error:', error);
    }
    // Note: setIsLoading(false) will be called by auth state change handler
  }, [firebaseAuth]);

  /**
   * Login with GitHub OAuth
   */
  const loginWithGitHub = useCallback(async () => {
    if (!firebaseAuth) {
      setError('Firebase authentication not enabled');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const result = await firebaseAuth.signInWithGitHub();
      console.log('GitHub sign-in successful:', result.user.email);
    } catch (error) {
      const errorMessage = error instanceof FirebaseAuthError 
        ? error.getUserFriendlyMessage()
        : 'GitHub sign-in failed';
      setError(errorMessage);
      console.error('GitHub sign-in error:', error);
    }
  }, [firebaseAuth]);

  /**
   * Login with email and password
   */
  const loginWithCredentials = useCallback(async (email: string, password: string) => {
    if (!firebaseAuth) {
      setError('Firebase authentication not enabled');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const result = await firebaseAuth.signInWithCredentials(email, password);
      console.log('Credential sign-in successful:', result.user.email);
    } catch (error) {
      const errorMessage = error instanceof FirebaseAuthError 
        ? error.getUserFriendlyMessage()
        : 'Email/password sign-in failed';
      setError(errorMessage);
      console.error('Credential sign-in error:', error);
      setIsLoading(false); // Set loading false on error since auth state won't change
    }
  }, [firebaseAuth]);

  /**
   * Login with OIDC (redirect to backend)
   */
  const loginWithOIDC = useCallback(async () => {
    if (!enableOIDC) {
      setError('OIDC authentication not enabled');
      return;
    }

    setError(null);

    try {
      // TODO: Implement OIDC redirect flow
      const oidcUrl = `${backendUrl || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'}/api/auth/oidc/authorize`;
      window.location.href = oidcUrl;
    } catch (error) {
      setError('OIDC sign-in failed');
      console.error('OIDC sign-in error:', error);
    }
  }, [enableOIDC, backendUrl]);

  /**
   * Create new account with email and password
   */
  const createAccount = useCallback(async (email: string, password: string) => {
    if (!firebaseAuth) {
      setError('Firebase authentication not enabled');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const result = await firebaseAuth.createAccount(email, password);
      console.log('Account creation successful:', result.user.email);
    } catch (error) {
      const errorMessage = error instanceof FirebaseAuthError 
        ? error.getUserFriendlyMessage()
        : 'Account creation failed';
      setError(errorMessage);
      console.error('Account creation error:', error);
      setIsLoading(false);
    }
  }, [firebaseAuth]);

  /**
   * Sign out current user
   */
  const logout = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      if (currentProvider === 'firebase' && firebaseAuth) {
        await firebaseAuth.signOut();
      }
      
      // TODO: Handle OIDC logout

      console.log('Logout successful');
    } catch (error) {
      console.error('Logout error:', error);
      setError('Logout failed');
      
      // Clear state anyway
      setIsAuthenticated(false);
      setUser(null);
      setFirebaseUser(null);
      setAccessToken(null);
      setUnifiedJWT(null);
      setCurrentProvider(null);
    } finally {
      setIsLoading(false);
    }
  }, [currentProvider, firebaseAuth]);

  /**
   * Refresh authentication token
   */
  const refreshToken = useCallback(async (): Promise<boolean> => {
    if (!tokenValidator || !currentProvider) {
      return false;
    }

    try {
      const newToken = await tokenValidator.refreshCurrentUserToken();
      
      if (newToken) {
        setUnifiedJWT(newToken);
        setAccessToken(newToken.access_token);
        return true;
      }
      
      return false;
    } catch (error) {
      console.error('Token refresh failed:', error);
      setError('Token refresh failed');
      return false;
    }
  }, [tokenValidator, currentProvider]);

  /**
   * Get valid access token (refresh if needed)
   */
  const getValidToken = useCallback(async (): Promise<string | null> => {
    if (!unifiedJWT || !tokenValidator) {
      return null;
    }

    try {
      const validToken = await tokenValidator.ensureValidToken(unifiedJWT);
      
      if (validToken) {
        if (validToken.access_token !== unifiedJWT.access_token) {
          // Token was refreshed
          setUnifiedJWT(validToken);
          setAccessToken(validToken.access_token);
        }
        return validToken.access_token;
      }
      
      return null;
    } catch (error) {
      console.error('Failed to get valid token:', error);
      return null;
    }
  }, [unifiedJWT, tokenValidator]);

  // Context value
  const value: MultiProviderAuthState = {
    // State
    isAuthenticated,
    isLoading,
    isInitialized,
    user,
    firebaseUser,
    accessToken,
    unifiedJWT,
    currentProvider,
    error,
    
    // Methods
    loginWithGoogle,
    loginWithGitHub,
    loginWithCredentials,
    loginWithOIDC,
    createAccount,
    logout,
    refreshToken,
    getValidToken,
    clearError,
  };

  return (
    <MultiProviderAuthContext.Provider value={value}>
      {children}
    </MultiProviderAuthContext.Provider>
  );
}

/**
 * Hook to use multi-provider authentication context
 */
export function useMultiProviderAuth(): MultiProviderAuthState {
  const context = useContext(MultiProviderAuthContext);
  
  if (context === undefined) {
    throw new Error('useMultiProviderAuth must be used within a MultiProviderAuthProvider');
  }
  
  return context;
}

/**
 * Hook to get current user with type safety
 */
export function useCurrentUser(): UnifiedUserProfile | null {
  const { user, isAuthenticated } = useMultiProviderAuth();
  return isAuthenticated ? user : null;
}

/**
 * Hook to get authentication status
 */
export function useAuthStatus(): {
  isAuthenticated: boolean;
  isLoading: boolean;
  isInitialized: boolean;
} {
  const { isAuthenticated, isLoading, isInitialized } = useMultiProviderAuth();
  return { isAuthenticated, isLoading, isInitialized };
}

/**
 * Hook to get valid access token
 */
export function useAccessToken(): {
  token: string | null;
  getValidToken: () => Promise<string | null>;
  refreshToken: () => Promise<boolean>;
} {
  const { accessToken, getValidToken, refreshToken } = useMultiProviderAuth();
  return { token: accessToken, getValidToken, refreshToken };
}

export default MultiProviderAuthProvider;