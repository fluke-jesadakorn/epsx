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

import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
} from 'react';
import {
  SharedWeb3AuthClient,
  UnifiedApiResponse,
  UserInfoResponse,
} from '../../auth/client';
import {
  COOKIES,
  COOKIE_OPTIONS,
  clearClientSideCookies,
  getClientCookie,
  getClientCookieJSON,
  setClientCookie,
  setClientCookieJSON,
} from '../../auth/cookies';

// Shared authentication context value
export interface SharedAuthContextValue {
  // Authentication state
  user: UserInfoResponse | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  isSigningChallenge: boolean;
  error: string | null;

  // Authentication actions
  requestChallenge: (
    walletAddress: string
  ) => Promise<{ nonce: string; message: string; wallet_address: string }>;
  authenticateWithWallet: (
    walletAddress: string,
    signature: string,
    message: string,
    nonce: string
  ) => Promise<{ success: boolean; user?: UserInfoResponse; error?: string }>;
  authenticateWithDirectApi: (result: {
    wallet_address: string;
    permissions: string[];
    tier_level?: string;
    is_new_user: boolean;
    access_token?: string;
  }) => Promise<void>;
  logout: () => Promise<void>;
  refreshUser: () => Promise<void>;

  // Display helpers (NOT for authorization - backend decides everything)
  getWalletAddress: () => string | null;
  getUserTier: () => string;
  getUserPermissions: () => string[];
  hasPermissionForDisplay: (permission: string) => boolean; // Display only!

  // API helper with Bearer token authentication
  makeApiRequest: (
    endpoint: string,
    options?: RequestInit
  ) => Promise<UnifiedApiResponse<any>>;
}

// Default context value
const defaultContextValue: SharedAuthContextValue = {
  user: null,
  isAuthenticated: false,
  isLoading: true,
  isSigningChallenge: false,
  error: null,
  requestChallenge: async () => {
    throw new Error('Not initialized');
  },
  authenticateWithWallet: async () => ({
    success: false,
    error: 'Not initialized',
  }),
  authenticateWithDirectApi: async () => {
    throw new Error('Not initialized');
  },
  logout: async () => {
    throw new Error('Not initialized');
  },
  refreshUser: async () => {
    throw new Error('Not initialized');
  },
  getWalletAddress: () => null,
  getUserTier: () => 'free',
  getUserPermissions: () => [],
  hasPermissionForDisplay: () => false,
  makeApiRequest: async () => ({
    success: false,
    error: {
      code: 500,
      message: 'Not initialized',
      reason: 'Provider not initialized',
    },
  }),
};

// Create context
const SharedAuthContext =
  createContext<SharedAuthContextValue>(defaultContextValue);

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
  onAuthError,
}: SharedOpenIDWeb3ProviderProps) {
  const [user, setUser] = useState<UserInfoResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSigningChallenge, setIsSigningChallenge] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [client] = useState(() => {
    // Enhanced backend URL resolution with environment variable support
    const resolvedBackendUrl =
      backendUrl ||
      (typeof window !== 'undefined'
        ? // Try environment variables first, then dynamic port replacement
        process.env.NEXT_PUBLIC_BACKEND_URL ||
        window.location.origin.replace(/:300[0-9]/, ':8080')
        : process.env.BACKEND_URL || 'http://localhost:8080');

    console.log(
      '🔧 SharedOpenIDWeb3Provider: Enhanced backend URL configuration',
      {
        provided: backendUrl,
        resolved: resolvedBackendUrl,
        envPublic: process.env.NEXT_PUBLIC_BACKEND_URL,
        envServer: process.env.BACKEND_URL,
        clientId,
      }
    );

    return new SharedWeb3AuthClient(clientId, resolvedBackendUrl);
  });

  // Initialize authentication state
  useEffect(() => {
    const initializeAuth = async () => {
      try {
        setIsLoading(true);
        setError(null);

        // First try to restore Web3 authentication from cookies
        let hasStoredAuth = false;
        if (typeof window !== 'undefined') {
          try {
            // Restore from cookies or localStorage
            let storedUser = getClientCookieJSON<UserInfoResponse>(COOKIES.user);
            if (!storedUser) {
              // Fallback to localStorage
              const userStr = localStorage.getItem('oidc.user');
              if (userStr) {
                try {
                  storedUser = JSON.parse(userStr);
                } catch (e) { }
              }
            }

            const authTime = getClientCookie(COOKIES.auth_time) || localStorage.getItem('oidc.auth_time');
            const accessToken = getClientCookie(COOKIES.access) || getClientCookie(COOKIES.client_session) || localStorage.getItem('oidc.access_token');
            const tokenExpiry = getClientCookie(COOKIES.expires_at) || localStorage.getItem('oidc.expires_at');

            console.log('🔍 Cookie/Storage restoration check', {
              clientId,
              hasStoredUser: !!storedUser,
              hasAuthTime: !!authTime,
              hasTokenExpiry: !!tokenExpiry,
              tokenExpiryValue: tokenExpiry,
              storedUserPermissions: storedUser?.permissions,
              isPermissionsArray: Array.isArray(storedUser?.permissions)
            });

            if (storedUser && authTime) {
              const authAge = Date.now() - parseInt(authTime);
              const maxAge = 24 * 60 * 60 * 1000; // 24 hours
              const isTokenValid = tokenExpiry
                ? parseInt(tokenExpiry) > Date.now()
                : false;

              console.log('🔍 Auth validation check', {
                authAge: Math.round(authAge / 1000 / 60) + 'min',
                maxAge: '24h',
                isTokenValid,
                tokenExpiry: tokenExpiry
                  ? new Date(parseInt(tokenExpiry)).toISOString()
                  : 'none',
                now: new Date().toISOString(),
              });

              console.log('🔍 Auth validation check', {
                authAge: Math.round(authAge / 1000 / 60) + 'min',
                maxAge: '24h',
                isTokenValid,
                tokenExpiry: tokenExpiry ? new Date(parseInt(tokenExpiry)).toISOString() : 'none',
                now: new Date().toISOString()
              });

              if (authAge < maxAge && isTokenValid) {
                console.log('✅ Restoring auth from cookies', {
                  clientId,
                  wallet: storedUser.wallet_address?.slice(0, 8) + '...',
                  permissions: storedUser.permissions?.length || 0,
                  hasPermissions: Array.isArray(storedUser.permissions),
                });
                setUser(storedUser);
                hasStoredAuth = true;
                return; // Skip OpenID client loading
              } else {
                console.log('🗑️ Clearing expired auth from cookies', {
                  clientId,
                  authAge: Math.round(authAge / 1000 / 60) + 'min',
                  tokenValid: isTokenValid,
                  reason: authAge >= maxAge ? 'auth too old' : 'token expired'
                });
                // Clear expired or invalid authentication
                clearClientSideCookies();
              }
            } else {
              console.log('⚠️ Missing required cookies', {
                hasStoredUser: !!storedUser,
                hasAuthTime: !!authTime
              });
            }
          } catch (error) {
            console.warn('Failed to restore authentication from cookies', error);
          }
        }

        // If no stored auth, user needs to authenticate
        if (!hasStoredAuth) {
          return;
        }

        // Fallback: Try to load existing user from OpenID tokens (with timeout)
        const timeoutPromise = new Promise((_, reject) =>
          setTimeout(() => reject(new Error('OpenID client timeout')), 3000)
        );

        await Promise.race([client.loadCurrentUser(), timeoutPromise]);
      } catch (err) {
        const errorMessage =
          err instanceof Error
            ? err.message
            : 'Failed to initialize authentication';
        console.error('Authentication initialization failed', {
          error: errorMessage,
        });
        setError(errorMessage);
        onAuthError?.(errorMessage);
      } finally {
        setIsLoading(false);
      }
    };

    initializeAuth();

    // Subscribe to auth state changes
    const unsubscribe = client.subscribe(newUser => {
      setUser(newUser);
      setIsLoading(false);

      if (newUser) {
        console.log('User state updated', {
          wallet_address: newUser.wallet_address,
        });
      } else {
        console.log('User logged out');
      }
    });

    return unsubscribe;
  }, [client, onAuthError]);

  // Request Web3 challenge
  const requestChallenge = useCallback(
    async (walletAddress: string) => {
      try {
        setError(null);
        setIsSigningChallenge(true);
        console.log('Requesting Web3 challenge', {
          wallet_address: walletAddress,
        });

        const challenge = await client.requestChallenge(walletAddress);

        console.log('Challenge received successfully');
        return challenge;
      } catch (err) {
        // Enhanced error handling and logging
        let errorMessage = 'Challenge request failed';

        if (err instanceof Error) {
          errorMessage = err.message || errorMessage;
          console.error('Challenge request failed:', {
            type: 'Error',
            name: err.name,
            message: err.message,
            backendUrl: client.getBackendUrl(),
            clientId: client.getClientId(),
            isNetworkError: err instanceof TypeError,
            isFetchError: err.name === 'FetchError',
          });

          // Log stack trace for debugging
          if (err.stack && process.env.NODE_ENV === 'development') {
            console.error('Stack trace:', err.stack);
          }
        } else {
          errorMessage = String(err) || errorMessage;
          console.error('Challenge request failed with non-Error object:', {
            type: typeof err,
            value: err,
            backendUrl: client.getBackendUrl(),
            clientId: client.getClientId(),
          });
        }

        // Ensure error message is meaningful
        if (!errorMessage || errorMessage.trim() === '') {
          errorMessage = 'Challenge request failed: Unknown error';
        }

        setError(errorMessage);
        onAuthError?.(errorMessage);
        throw new Error(errorMessage);
      } finally {
        setIsSigningChallenge(false);
      }
    },
    [client, onAuthError]
  );

  // Authenticate with Web3 wallet signature
  const authenticateWithWallet = useCallback(
    async (
      walletAddress: string,
      signature: string,
      message: string,
      nonce: string
    ) => {
      try {
        setError(null);
        setIsLoading(true);

        console.log('Authenticating with Web3 wallet', {
          wallet_address: walletAddress,
        });

        const result = await client.authenticateWithSignature({
          wallet_address: walletAddress,
          signature,
          message,
          nonce,
        });

        if (result.success) {
          console.log(
            'Web3 authentication successful - backend handles session storage'
          );
          // Backend already stored the session when it authenticated the signature
          // No need to store session on frontend
        } else {
          const errorMsg = result.error || 'Authentication failed';
          console.error('Web3 authentication failed', {
            error: errorMsg,
            wallet_address: walletAddress,
            signature: signature.substring(0, 20) + '...',
            message: message.substring(0, 50) + '...',
            nonce: nonce
          });
          setError(errorMsg);
          onAuthError?.(errorMsg);
        }

        return result;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : 'Authentication failed';
        console.error('Web3 authentication error', { error: errorMessage });
        setError(errorMessage);
        onAuthError?.(errorMessage);
        return {
          success: false,
          error: errorMessage,
        };
      } finally {
        setIsLoading(false);
      }
    },
    [client, onAuthError]
  );

  // Authenticate with direct API result (bypass OpenID flow)
  const authenticateWithDirectApi = useCallback(async (result: {
    wallet_address: string;
    permissions: string[];
    tier_level?: string;
    is_new_user: boolean;
    access_token?: string;
  }) => {
    try {
      setError(null);
      setIsLoading(true);

      console.log('🔄 Processing direct API authentication result', {
        wallet: result.wallet_address,
        tier: result.tier_level,
        permissions: result.permissions.length,
        isNew: result.is_new_user
      });

      // Create user info compatible with UserInfoResponse
      const user: UserInfoResponse = {
        sub: result.wallet_address,
        wallet_address: result.wallet_address,
        tier_level: result.tier_level ?? 'free',
        auth_method: 'web3_siwe',
        permissions: result.permissions,
        packageTier: result.tier_level ?? 'free', // For compatibility
        access: result.access_token, // JWT for SSE authentication
      };

      // Persist user data to cookies for page refresh survival
      if (typeof window !== 'undefined') {
        try {
          // 1. Save to Cookies (Primary)
          setClientCookieJSON(COOKIES.user, user);
          setClientCookie(COOKIES.auth_time, Date.now().toString(), COOKIE_OPTIONS.maxAge.auth_time);

          // Set token expiry (same as access token)
          const expiryTime = Date.now() + (COOKIE_OPTIONS.maxAge.access * 1000);
          setClientCookie(COOKIES.expires_at, expiryTime.toString(), COOKIE_OPTIONS.maxAge.expires_at);

          // CRITICAL: Set client_session cookie with access_token for server-side auth
          if (result.access_token) {
            setClientCookie(COOKIES.client_session, result.access_token, COOKIE_OPTIONS.maxAge.access);
            console.log('🔑 Set client_session cookie for server-side auth');
          }

          // 2. Save to localStorage (Fallback/Redundancy)
          // This ensures session survives even if cookies are blocked/size-limited
          localStorage.setItem('oidc.user', JSON.stringify(user));
          localStorage.setItem('oidc.auth_time', Date.now().toString());
          localStorage.setItem('oidc.expires_at', expiryTime.toString());

          if (result.access_token) {
            localStorage.setItem('oidc.access_token', result.access_token);
          }

          console.log('💾 Persisted Web3 authentication to storage (cookies + localStorage)', {
            clientId,
            keys: {
              user: COOKIES.user,
              authTime: COOKIES.auth_time,
              expiresAt: COOKIES.expires_at,
              clientSession: COOKIES.client_session
            }
          });
        } catch (error) {
          console.warn('⚠️ Failed to persist authentication data to storage:', error);
        }
      }

      // Update user state directly
      setUser(user);

      console.log('✅ Direct API authentication processed successfully');
    } catch (error) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : 'Failed to process authentication result';
      console.error(
        '❌ Direct API authentication processing failed:',
        errorMessage
      );
      setError(errorMessage);
      onAuthError?.(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  },
    [onAuthError]
  );

  // Logout user
  const logout = useCallback(async () => {
    try {
      setError(null);
      console.log('Logging out user');

      // Clear all client-side cookies
      if (typeof window !== 'undefined') {
        try {
          clearClientSideCookies();
          console.log('🗑️ Cleared authentication from cookies', { clientId });
        } catch (error) {
          console.warn('⚠️ Failed to clear authentication cookies:', error);
        }
      }

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
      const errorMessage =
        err instanceof Error ? err.message : 'Failed to refresh user data';
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

  const hasPermissionForDisplay = useCallback(
    (permission: string) => {
      // THIS IS FOR DISPLAY ONLY - NOT AUTHORIZATION
      return client.hasPermissionForDisplay(permission);
    },
    [client]
  );

  // API request helper
  const makeApiRequest = useCallback(
    async (
      endpoint: string,
      options?: RequestInit
    ): Promise<UnifiedApiResponse<any>> => {
      try {
        return await client.makeAuthenticatedRequest<any>(endpoint, options);
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : 'API request failed';
        console.error('API request error', { endpoint, error: errorMessage });
        return {
          success: false,
          error: {
            code: 500,
            message: 'API request failed',
            reason: errorMessage,
          },
        } as UnifiedApiResponse<any>;
      }
    },
    [client]
  );

  // Context value - Backend handles ALL validation (tokens, permissions, expiry)
  // Frontend only checks: "Do I have a user object?"
  const isAuthenticated = !!user;

  console.log('🔍 Provider: isAuthenticated calculation', {
    clientId,
    isAuthenticated,
    hasUser: !!user,
    wallet: user?.wallet_address?.slice(0, 8),
    permissionsLength: user?.permissions?.length || 0,
    isLoading,
  });

  const contextValue: SharedAuthContextValue = React.useMemo(() => ({
    user,
    // Backend-authoritative auth: Frontend only checks if user exists
    // Backend validates tokens, permissions, and expiry on every API request
    isAuthenticated,
    isLoading,
    isSigningChallenge,
    error,
    requestChallenge,
    authenticateWithWallet,
    authenticateWithDirectApi,
    logout,
    refreshUser,
    getWalletAddress,
    getUserTier,
    getUserPermissions,
    hasPermissionForDisplay,
    makeApiRequest,
  }), [
    user,
    isAuthenticated,
    isLoading,
    isSigningChallenge,
    error,
    requestChallenge,
    authenticateWithWallet,
    authenticateWithDirectApi,
    logout,
    refreshUser,
    getWalletAddress,
    getUserTier,
    getUserPermissions,
    hasPermissionForDisplay,
    makeApiRequest,
  ]);

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
    throw new Error(
      'useSharedAuth must be used within SharedOpenIDWeb3Provider'
    );
  }

  return context;
}

// Convenience exports for backward compatibility
export const useAuth = useSharedAuth;
