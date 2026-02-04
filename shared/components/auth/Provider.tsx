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
  loginAction,
  logoutAction
} from '../../auth/actions';
import {
  SharedWeb3AuthClient,
  UnifiedApiResponse,
  UserInfoResponse,
} from '../../auth/client';
import {
  COOKIES,
  clearClientSideCookies,
  getClientCookie,
  getClientCookieJSON
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
  refreshSession: () => Promise<boolean>;

  // Display helpers (NOT for authorization - backend decides everything)
  getWalletAddress: () => string | null;
  getUserTier: () => string;
  getUserPermissions: () => string[];
  // API helper with Bearer token authentication

  makeApiRequest: (
    endpoint: string,
    options?: RequestInit
  ) => Promise<UnifiedApiResponse<any>>;

  // Modal state management
  showSignInModal: boolean;
  openSignInModal: () => void;
  closeSignInModal: () => void;
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
  refreshSession: async () => false,
  getWalletAddress: () => null,
  getUserTier: () => 'free',
  getUserPermissions: () => [],

  makeApiRequest: async () => ({
    success: false,
    error: {
      code: 500,
      message: 'Not initialized',
      reason: 'Provider not initialized',
    },
  }),
  showSignInModal: false,
  openSignInModal: () => { },
  closeSignInModal: () => { },
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
  initialUser = null, // Accept server-provided user state
}: SharedOpenIDWeb3ProviderProps & { initialUser?: UserInfoResponse | null }) {
  // Initialize user from server-provided state (SSR compatible)
  // CRITICAL: Do NOT read from cookies here - it causes hydration mismatch
  const [user, setUser] = useState<UserInfoResponse | null>(initialUser);
  const [isLoading, setIsLoading] = useState(true);
  const [isSigningChallenge, setIsSigningChallenge] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showSignInModal, setShowSignInModal] = useState(false);
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
      '[AUTH] Provider: Enhanced backend URL configuration',
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

        // First try to restore Web3 authentication from cookies/storage
        let hasStoredAuth = false;

        // If we have an initial user from server, we are already authenticated
        if (initialUser) {
          console.log('[AUTH] Provider: Hydrated from server state', {
            wallet: initialUser.wallet_address
          });
          // Update client with this user to Ensure sync
          client.setCurrentUser(initialUser);
          hasStoredAuth = true;
          // We don't return here because we might want to validate/refresh in background
        }

        if (typeof window !== 'undefined' && !hasStoredAuth) {
          try {
            // Priority 1: Check if client already has it (it runs loadTokensFromStorage in constructor)
            const clientUser = client.getCurrentUser();
            const clientHasAuth = client.isAuthenticated();

            if (clientUser && clientHasAuth) {
              console.log('[AUTH] Client successfully pre-loaded valid auth state');
              setUser(clientUser);
              hasStoredAuth = true;
              return;
            }

            // Priority 2: Manual check if client hasn't loaded yet or is being strict
            const storedUser = getClientCookieJSON<UserInfoResponse>(COOKIES.user);
            const authTime = getClientCookie(COOKIES.auth_time);
            const accessToken = getClientCookie(COOKIES.access_token);
            const tokenExpiry = getClientCookie(COOKIES.expires_at);

            console.log('[AUTH] Provider: Cookie restoration check', {
              clientId,
              hasStoredUser: !!storedUser,
              hasAccessToken: !!accessToken,
              hasTokenExpiry: !!tokenExpiry,
              tokenExpiryValue: tokenExpiry ? new Date(parseInt(tokenExpiry)).toISOString() : 'none',
            });

            // Trust stored user data - HttpOnly cookie validation happens server-side
            // Middleware validates the actual HttpOnly token on every request
            if (storedUser) {
              console.log('[AUTH] Restoring auth from cookies', {
                clientId,
                wallet: storedUser.wallet_address?.slice(0, 8),
              });
              setUser(storedUser);
              hasStoredAuth = true;
              return;
            }
          } catch (error) {
            console.warn('Failed to restore authentication from storage', error);
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

    // Safety timeout: unexpected hangs shouldn't block the UI forever
    const safetyTimeout = setTimeout(() => {
      setIsLoading(prev => {
        if (prev) {
          console.warn('[AUTH] Provider: Initialization took too long, forcing load completion');
          return false;
        }
        return prev;
      });
    }, 5000); // 5 second max load time

    initializeAuth();

    // Subscribe to auth state changes
    const unsubscribe = client.subscribe(newUser => {
      setUser(newUser);

      if (newUser) {
        console.log('User state updated', {
          wallet_address: newUser.wallet_address,
        });
      } else {
        console.log('User logged out');
      }
    });

    return () => {
      clearTimeout(safetyTimeout);
      unsubscribe();
    };
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

        if (result.success && result.user) {
          console.log(
            'Web3 authentication successful - initiating server session'
          );

          // Call server action to set cookies
          // client.accessToken should be populated by authenticateWithSignature
          const accessToken = result.user.access;

          if (accessToken) {
            console.log('[AUTH] Calling loginAction to persist session...');
            const loginResult = await loginAction(accessToken, result.user);

            if (!loginResult.success) {
              console.error('[AUTH] Error: loginAction failed:', loginResult.error);
              throw new Error('Failed to create server session');
            }
            console.log('[AUTH] loginAction successful');
          } else {
            console.warn('[AUTH] Warning: No access token returned from authentication, session might be incomplete');
          }

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

      console.log('[AUTH] Processing direct API authentication result', {
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

      // Cookies are set by loginAction server action
      // Only update in-memory React state here
      if (typeof window !== 'undefined') {
        console.log('[AUTH] Session established via server action', {
          clientId,
          wallet: user.wallet_address?.slice(0, 8),
        });
      }

      // Update user state directly
      setUser(user);

      console.log('[AUTH] Direct API authentication processed successfully');
    } catch (error) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : 'Failed to process authentication result';
      console.error(
        '[AUTH] Error: Direct API authentication processing failed:',
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

      // 1. Call server action to clear HttpOnly cookies
      try {
        await logoutAction();
        console.log('[AUTH] Server session cleared');
      } catch (e: any) {
        // Check if it's a redirect error (Standard Next.js Redirect)
        if (e?.message === 'NEXT_REDIRECT' || e?.digest?.startsWith('NEXT_REDIRECT')) {
          // Re-throw to allow Next.js client to handle the redirect
          throw e;
        } else {
          console.error('[AUTH] Error: Failed to clear server session:', e);
        }
      }

      // 2. Clear all client-side cookies (legacy/fallback)
      if (typeof window !== 'undefined') {
        try {
          clearClientSideCookies();
          console.log('[AUTH] Cleared authentication from cookies', { clientId });
        } catch (error) {
          console.warn('[AUTH] Warning: Failed to clear authentication cookies:', error);
        }
      }

      await client.logout();

      console.log('Logout successful');

    } catch (err: any) {
      if (err?.message === 'NEXT_REDIRECT' || err?.digest?.startsWith('NEXT_REDIRECT')) {
        // Re-throw if it bubbles up here (it shouldn't if we don't await the promise that throws)
        throw err;
      }

      const errorMessage = err instanceof Error ? err.message : 'Logout failed';
      console.error('Logout error', { error: errorMessage });
      setError(errorMessage);
      onAuthError?.(errorMessage);
      // Don't throw here to avoid crashing the UI for logout errors
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

  // Refresh session tokens
  const refreshSession = useCallback(async () => {
    try {
      console.log('[AUTH] Refreshing session tokens...');
      const success = await client.refreshTokens();
      if (success) {
        console.log('[AUTH] Session tokens refreshed successfully');
        // Force update user state from new tokens
        await client.loadCurrentUser();
      } else {
        console.warn('[AUTH] Warning: Session refresh failed');
      }
      return success;
    } catch (err) {
      console.error('[AUTH] Error: Session refresh error', err);
      return false;
    }
  }, [client]);

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

  // Modal state management
  const openSignInModal = useCallback(() => {
    setShowSignInModal(true);
  }, []);

  const closeSignInModal = useCallback(() => {
    setShowSignInModal(false);
  }, []);

  // Context value - Backend handles ALL validation (tokens, permissions, expiry)
  // Frontend only checks: "Do I have a user object?"
  const isAuthenticated = !!user;

  console.log('[AUTH] Provider: isAuthenticated calculation', {
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
    refreshSession,
    getWalletAddress,
    getUserTier,
    getUserPermissions,
    makeApiRequest,
    showSignInModal,
    openSignInModal,
    closeSignInModal,
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
    refreshSession,
    getWalletAddress,
    getUserTier,
    getUserPermissions,
    makeApiRequest,
    showSignInModal,
    openSignInModal,
    closeSignInModal,
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
