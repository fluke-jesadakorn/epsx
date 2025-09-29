// ============================================================================
// SHARED OPENID + WEB3 AUTHENTICATION PROVIDER
// Unified React provider for both frontend and admin-frontend
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Same authentication standard for both apps
 * - No permission logic in components
 * - Backend makes all authorization decisions
 * - Simple display-only state management
 * - Web3 wallet signing triggers OpenID tokens
 */

'use client';

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { 
  SharedOpenIDWeb3Client, 
  UserInfoResponse, 
  UnifiedApiResponse 
} from '../../auth/openid-web3-client';

// Shared authentication context value
export interface SharedAuthContextValue {
  // Authentication state
  user: UserInfoResponse | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  isSigningChallenge: boolean;
  error: string | null;

  // Authentication actions
  requestChallenge: (walletAddress: string) => Promise<{ nonce: string; message: string; wallet_address: string }>;
  authenticateWithWallet: (walletAddress: string, signature: string, message: string, nonce: string) => Promise<{ success: boolean; user?: UserInfoResponse; error?: string }>;
  logout: () => Promise<void>;
  refreshUser: () => Promise<void>;

  // Display helpers (NOT for authorization - backend decides everything)
  getWalletAddress: () => string | null;
  getUserTier: () => string;
  getUserPermissions: () => string[];
  hasPermissionForDisplay: (permission: string) => boolean; // Display only!

  // API helper with Bearer token authentication
  makeApiRequest: (endpoint: string, options?: RequestInit) => Promise<UnifiedApiResponse<any>>;
}

// Default context value
const defaultContextValue: SharedAuthContextValue = {
  user: null,
  isAuthenticated: false,
  isLoading: true,
  isSigningChallenge: false,
  error: null,
  requestChallenge: async () => { throw new Error('Not initialized'); },
  authenticateWithWallet: async () => ({ success: false, error: 'Not initialized' }),
  logout: async () => { throw new Error('Not initialized'); },
  refreshUser: async () => { throw new Error('Not initialized'); },
  getWalletAddress: () => null,
  getUserTier: () => 'free',
  getUserPermissions: () => [],
  hasPermissionForDisplay: () => false,
  makeApiRequest: async () => ({ success: false, error: { code: 500, message: 'Not initialized', reason: 'Provider not initialized' } }),
};

// Create context
const SharedAuthContext = createContext<SharedAuthContextValue>(defaultContextValue);

// Provider props
interface SharedOpenIDWeb3ProviderProps {
  children: React.ReactNode;
  clientId?: string;
  backendUrl?: string;
  onAuthError?: (error: string) => void;
}

// Provider component
export function SharedOpenIDWeb3Provider({ 
  children, 
  clientId = 'epsx-frontend',
  backendUrl,
  onAuthError 
}: SharedOpenIDWeb3ProviderProps) {
  const [user, setUser] = useState<UserInfoResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSigningChallenge, setIsSigningChallenge] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [client] = useState(() => {
    // Debug log the backend URL configuration
    const resolvedBackendUrl = backendUrl || 
      (typeof window !== 'undefined' ? 
        window.location.origin.replace(/:300[0-9]/, ':8080') : // Handle both :3000 and :3001
        'http://localhost:8080'
      );
    
    console.log('🔧 SharedOpenIDWeb3Provider: Backend URL configuration', {
      provided: backendUrl,
      resolved: resolvedBackendUrl,
      clientId
    });
    
    return new SharedOpenIDWeb3Client(clientId, resolvedBackendUrl);
  });

  // Initialize authentication state
  useEffect(() => {
    const initializeAuth = async () => {
      try {
        setIsLoading(true);
        setError(null);
        
        console.log('Initializing shared OpenID + Web3 authentication');
        
        // Try to load existing user from tokens
        await client.loadCurrentUser();
        
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to initialize authentication';
        console.error('Authentication initialization failed', { error: errorMessage });
        setError(errorMessage);
        onAuthError?.(errorMessage);
      } finally {
        setIsLoading(false);
      }
    };

    initializeAuth();

    // Subscribe to auth state changes
    const unsubscribe = client.subscribe((newUser) => {
      setUser(newUser);
      setIsLoading(false);
      
      if (newUser) {
        console.log('User state updated', {
          wallet_address: newUser.wallet_address,
          tier_level: newUser.tier_level
        });
      } else {
        console.log('User logged out');
      }
    });

    return unsubscribe;
  }, [client, onAuthError]);

  // Request Web3 challenge
  const requestChallenge = useCallback(async (walletAddress: string) => {
    try {
      setError(null);
      setIsSigningChallenge(true);
      console.log('Requesting Web3 challenge', { wallet_address: walletAddress });
      
      const challenge = await client.requestChallenge(walletAddress);
      
      console.log('Challenge received successfully');
      return challenge;
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to request challenge';
      console.error('Challenge request failed', { error: errorMessage });
      setError(errorMessage);
      onAuthError?.(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsSigningChallenge(false);
    }
  }, [client, onAuthError]);

  // Authenticate with Web3 wallet signature
  const authenticateWithWallet = useCallback(async (
    walletAddress: string,
    signature: string,
    message: string,
    nonce: string
  ) => {
    try {
      setError(null);
      setIsLoading(true);
      
      console.log('Authenticating with Web3 wallet', { wallet_address: walletAddress });
      
      const result = await client.authenticateWithSignature({
        wallet_address: walletAddress,
        signature,
        message,
        nonce
      });
      
      if (result.success) {
        console.log('Web3 authentication successful');
        
        // Store Web3 session data in cookies for server-side access
        try {
          const storeResponse = await fetch('/api/auth/web3/store-session', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({
              walletAddress,
              signature,
              message,
              nonce,
              chainId: 56
            }),
          });

          if (storeResponse.ok) {
            console.log('✅ Web3 session stored in cookies for server-side access');
          } else {
            console.warn('⚠️ Failed to store Web3 session in cookies');
          }
        } catch (sessionError) {
          console.warn('⚠️ Error storing Web3 session:', sessionError);
        }
      } else {
        console.error('Web3 authentication failed', { error: result.error });
        setError(result.error || 'Authentication failed');
        onAuthError?.(result.error || 'Authentication failed');
      }
      
      return result;
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Authentication failed';
      console.error('Web3 authentication error', { error: errorMessage });
      setError(errorMessage);
      onAuthError?.(errorMessage);
      return {
        success: false,
        error: errorMessage
      };
    } finally {
      setIsLoading(false);
    }
  }, [client, onAuthError]);

  // Logout user
  const logout = useCallback(async () => {
    try {
      setError(null);
      console.log('Logging out user');
      
      await client.logout();
      
      console.log('Logout successful');
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Logout failed';
      console.error('Logout error', { error: errorMessage });
      setError(errorMessage);
      onAuthError?.(errorMessage);
      throw new Error(errorMessage);
    }
  }, [client, onAuthError]);

  // Refresh user data
  const refreshUser = useCallback(async () => {
    try {
      setError(null);
      console.log('Refreshing user data');
      
      await client.loadCurrentUser();
      
      console.log('User data refreshed');
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to refresh user data';
      console.error('User refresh error', { error: errorMessage });
      setError(errorMessage);
      onAuthError?.(errorMessage);
      throw new Error(errorMessage);
    }
  }, [client, onAuthError]);

  // Display helper functions (NOT for authorization)
  const getWalletAddress = useCallback(() => {
    return client.getWalletAddress();
  }, [client]);

  const getUserTier = useCallback(() => {
    return client.getUserTier();
  }, [client]);

  const getUserPermissions = useCallback(() => {
    return client.getUserPermissions();
  }, [client]);

  const hasPermissionForDisplay = useCallback((permission: string) => {
    // THIS IS FOR DISPLAY ONLY - NOT AUTHORIZATION
    return client.hasPermissionForDisplay(permission);
  }, [client]);

  // API request helper
  const makeApiRequest = useCallback(async (endpoint: string, options?: RequestInit): Promise<UnifiedApiResponse<any>> => {
    try {
      return await client.makeAuthenticatedRequest<any>(endpoint, options);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'API request failed';
      console.error('API request error', { endpoint, error: errorMessage });
      return {
        success: false,
        error: {
          code: 500,
          message: 'API request failed',
          reason: errorMessage
        }
      } as UnifiedApiResponse<any>;
    }
  }, [client]);

  // Context value
  const contextValue: SharedAuthContextValue = {
    user,
    isAuthenticated: !!user && client.isAuthenticated(),
    isLoading,
    isSigningChallenge,
    error,
    requestChallenge,
    authenticateWithWallet,
    logout,
    refreshUser,
    getWalletAddress,
    getUserTier,
    getUserPermissions,
    hasPermissionForDisplay,
    makeApiRequest,
  };

  return (
    <SharedAuthContext.Provider value={contextValue}>
      {children}
    </SharedAuthContext.Provider>
  );
}

// Hook to use the shared authentication context
export function useSharedAuth(): SharedAuthContextValue {
  const context = useContext(SharedAuthContext);
  
  if (!context) {
    throw new Error('useSharedAuth must be used within SharedOpenIDWeb3Provider');
  }
  
  return context;
}

// Convenience exports for backward compatibility
export const useAuth = useSharedAuth;