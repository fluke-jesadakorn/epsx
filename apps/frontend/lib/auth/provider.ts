'use client';

import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';
import { useAccount, useDisconnect, useSignMessage } from 'wagmi';

export interface Web3Permission {
  permission: string;
  source: 'manual' | 'nft' | 'token' | 'dao';
  expires_at?: string;
  metadata?: {
    nft_collection?: string;
    token_contract?: string;
    dao_name?: string;
    required_amount?: string;
    [key: string]: unknown;
  };
}

export interface Web3AuthState {
  isConnected: boolean;
  isAuthenticated: boolean;
  isAuthenticating: boolean;
  walletAddress?: string;
  permissions: Web3Permission[];
  userTier: 'free' | 'nft' | 'token' | 'dao' | 'enterprise';
  hasApiAccess: boolean;
  error?: string;
}

export interface Web3AuthActions {
  authenticate: () => Promise<void>;
  disconnect: () => Promise<void>;
  checkAuthStatus: () => Promise<boolean | undefined>;
  refreshPermissions: () => Promise<boolean>;
  linkEmail: (email: string, password: string) => Promise<void>;
  generateApiKey: (name: string) => Promise<string>;
  resetAuthState: () => void;
}

interface SessionMessage {
  type?: string;
  source?: string;
  walletAddress?: string;
  isAuthenticated?: boolean;
  user?: unknown;
}

interface SessionData {
  isAuthenticated?: boolean;
  user?: {
    wallet_address?: string;
    [key: string]: unknown;
  };
  permissions?: unknown;
  user_tier?: string;
  has_api_access?: boolean;
}

function isSessionMessage(data: unknown): data is SessionMessage {
  return typeof data === 'object' && data !== null && !Array.isArray(data);
}

function isSessionData(data: unknown): data is SessionData {
  return typeof data === 'object' && data !== null && !Array.isArray(data);
}

// eslint-disable-next-line max-lines-per-function
export function useWeb3Auth(): Web3AuthState & Web3AuthActions {
  const router = useRouter();
  const { address, isConnected, connector, chain } = useAccount();
  const { disconnect: wagmiDisconnect } = useDisconnect();
  const {
    signMessageAsync,
  } = useSignMessage();

  const [state, setState] = useState<Web3AuthState>({
    isConnected: false,
    isAuthenticated: false,
    isAuthenticating: false,
    permissions: [],
    userTier: 'free',
    hasApiAccess: false,
  });

  const [isHydrated, setIsHydrated] = useState(false);

  // Handle hydration
  useEffect(() => {
    setIsHydrated(true);
  }, []);

  // Cross-tab session invalidation listener (only for explicit disconnect)
  useEffect(() => {
    if (typeof window === 'undefined' || !window.BroadcastChannel) {
      return;
    }

    let channel: BroadcastChannel | null = null;

    try {
      channel = new BroadcastChannel('auth_session');

      const handleSessionMessage = (event: MessageEvent) => {
        try {
          if (!isSessionMessage(event.data)) {
            return;
          }

          // Only process explicit disconnect messages, ignore auto-disconnects
          if (event.data.type === 'SESSION_INVALIDATED' && event.data.source === 'web3_disconnect') {
            // Only invalidate if the wallet address matches
            const shouldInvalidate = event.data.walletAddress === null || event.data.walletAddress === undefined || event.data.walletAddress === address;
            if (shouldInvalidate) {
              // Reset authentication state immediately
               
              setState(prev => ({
                ...prev,
                isAuthenticated: false,
                isAuthenticating: false,
                permissions: [],
                userTier: 'free',
                hasApiAccess: false,
                walletAddress: undefined,
                error: undefined,
              }));

              // Clear local session markers
              try {
                window.localStorage.removeItem('oidc_session');
                window.sessionStorage.removeItem('oidc_session');
              } catch (_error) {
                // Intentionally empty - storage cleanup is best effort
              }

              toast.info('Session was ended in another tab');
            }
          }
        } catch (_error) {
          // Intentionally empty - message handling is best effort
        }
      };

      channel.addEventListener('message', handleSessionMessage);

      return () => {
        try {
          if (channel) {
            channel.removeEventListener('message', handleSessionMessage);
            if (typeof channel.close === 'function') {
              channel.close();
            }
          }
        } catch (_error) {
          // Intentionally empty - cleanup is best effort
        }
      };
    } catch (_error) {
      return () => { }; // No-op cleanup
    }
  }, [address]);

  // Auto-check auth status when wallet connects (with proper hydration handling)
   
  useEffect(() => {
    if (!isHydrated) {return;} // Wait for hydration

    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    if (address && isConnected) {
      // IMPORTANT: Always sync internal state with Wagmi state first
      setState(prev => ({
        ...prev,
        isConnected: true, // Force sync with Wagmi
        walletAddress: address,
        error: undefined, // Clear errors when successfully connected
      }));

      // Check auth status when we have an address (inline to avoid dependency issues)
      void (async () => {
        try {
          // Check for session if wallet is connected
          const response = await fetch('/api/auth/session', {
            credentials: 'include',
            cache: 'no-cache',
          });

          if (!response.ok) {
            if (response.status !== 401 && response.status !== 500) {
              // Handle other error statuses if needed
            }
            return;
          }

          const session = await response.json() as unknown;
          if (!isSessionData(session)) {
            return;
          }

          if (!session.isAuthenticated || session.user?.wallet_address !== address) {
            return;
          }

          setState(prev => ({
            ...prev,
            isConnected: true,
            isAuthenticated: true,
            walletAddress: address,
          }));

          // Refresh permissions inline to avoid dependency issues
          try {
            const permResponse = await fetch(
              `/api/auth/web3/permissions?wallet_address=${encodeURIComponent(address)}`,
              {
                method: 'GET',
                credentials: 'include',
              }
            );

            if (!permResponse.ok) {
              return;
            }

            const perms = await permResponse.json() as unknown;
            if (!isSessionData(perms)) {
              return;
            }

            setState(prev => {
              const permsArray = Array.isArray(perms.permissions) ? (perms.permissions as Web3Permission[]) : [];
              return {
                ...prev,
                permissions: permsArray,
                userTier: typeof perms.user_tier === 'string' ? (perms.user_tier as 'free' | 'nft' | 'token' | 'dao' | 'enterprise') : 'free',
                hasApiAccess: Boolean(perms.has_api_access),
              };
            });
          } catch (_permError) {
            // Intentionally empty - permission refresh is best effort
          }
        } catch (_error) {
          // In progressive auth, treat network errors as non-critical but keep connected state
          setState(prev => ({
            ...prev,
            isConnected: true, // CRITICAL: Keep this true when Wagmi shows connected
            isAuthenticated: false,
            walletAddress: address,
            // Only set error for actual failures, not auth state
          }));
        }
      })();
    } else if (!address || !isConnected) {
      // Only clear state when Wagmi actually shows disconnected
      setState(prev => ({
        ...prev,
        isConnected: false,
        isAuthenticated: false,
        walletAddress: undefined,
        permissions: [],
        userTier: 'free',
        hasApiAccess: false,
        error: undefined,
      }));
    }
  }, [address, isConnected, isHydrated]);
   
  const checkAuthStatus = useCallback(async () => {
    if (address === null || address === undefined) {
      return;
    }

    try {
      // Simplified auth status check - just check session
      const response = await fetch('/api/auth/session', {
        credentials: 'include',
        cache: 'no-cache',
      });

      if (!response.ok) {
        setState(prev => ({
          ...prev,
          isAuthenticated: false,
          error: undefined,
        }));
        return false;
      }

      const session = await response.json() as unknown;
      if (!isSessionData(session)) {
        setState(prev => ({
          ...prev,
          isAuthenticated: false,
          error: undefined,
        }));
        return false;
      }

      if (session.isAuthenticated && session.user?.wallet_address === address) {
        setState(prev => ({
          ...prev,
          isConnected: true,
          isAuthenticated: true,
          walletAddress: address,
        }));
        return true;
      }

      // Not authenticated but may be connected
      setState(prev => ({
        ...prev,
        isAuthenticated: false,
        error: undefined,
      }));
      return false;
    } catch (_error) {
      return false;
    }
  }, [address]);
   
  const refreshPermissions = useCallback(async () => {
    if (address === null || address === undefined) {
      return false;
    }

    try {
      const response = await fetch(
        `/api/auth/web3/permissions?wallet_address=${encodeURIComponent(address)}`,
        {
          method: 'GET',
          credentials: 'include',
        }
      );

      if (response.ok) {
        const perms = await response.json() as unknown;
        if (!isSessionData(perms)) {
          return false;
        }

        setState(prev => ({
          ...prev,
          permissions: Array.isArray(perms.permissions) ? (perms.permissions as Web3Permission[]) : [],
          userTier: typeof perms.user_tier === 'string' ? (perms.user_tier as 'free' | 'nft' | 'token' | 'dao' | 'enterprise') : 'free',
          hasApiAccess: Boolean(perms.has_api_access),
        }));
        return true;
      }

      if (response.status === 405) {
        // Set default values when permissions endpoint is not available
        setState(prev => ({
          ...prev,
          permissions: [],
          userTier: 'free',
          hasApiAccess: false,
        }));
        return true;
      }

      return false;
    } catch (_error) {
      // Set default values on error
      setState(prev => ({
        ...prev,
        permissions: [],
        userTier: 'free',
        hasApiAccess: false,
      }));
      return false;
    }
  }, [address]);

  // eslint-disable-next-line max-lines-per-function, sonarjs/cognitive-complexity, complexity
  const authenticate = useCallback(async () => {
    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    if (!address) {
      toast.error('Please connect your wallet first');
      return;
    }

    // Enhanced wallet validation
    if (!connector) {
      toast.error('Wallet connector not found. Please reconnect your wallet.');
      return;
    }

    if (connector.ready === false) {
      toast.error('Wallet is not ready. Please check your wallet connection.');
      return;
    }

    // Request wallet access first to prevent authorization errors
     
    try {
      const provider = await connector.getProvider?.();
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/strict-boolean-expressions, @typescript-eslint/no-explicit-any
      if (provider && typeof provider === 'object' && 'request' in provider && typeof (provider as any).request === 'function') {
        // First, request account access to ensure proper authorization
        try {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-explicit-any
          const accounts = await (provider as any).request({
            method: 'eth_requestAccounts',
          });

          // Verify the current address is in the authorized accounts
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/strict-boolean-expressions
          if (accounts && accounts.length > 0) {
            const normalizedAddress = address.toLowerCase();
            // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
            const hasMatchingAccount = accounts.some(
              (acc: string) => acc.toLowerCase() === normalizedAddress
            );

            // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
            if (!hasMatchingAccount) {
              throw new Error(
                'Connected wallet address not found in authorized accounts'
              );
            }
          } else {
            throw new Error('No accounts returned from wallet');
          }
        } catch (authError: unknown) {
          // Handle specific authorization errors
          const err = authError as { code?: number; message?: string };
          if (err.code === 4001) {
            throw new Error('Wallet access denied by user');
          } else if (err.code === 4100) {
            throw new Error(
              'Wallet not authorized - please connect your wallet first'
            );
          } else if (err.message?.includes('User rejected')) {
            throw new Error('Wallet access was rejected');
          } else {
            throw new Error(
              `Wallet authorization failed: ${err.message ?? 'Unknown error'}`
            );
          }
        }
      }
    } catch (error: unknown) {
      const err = error as { message?: string };
      setState(prev => ({
        ...prev,
        isAuthenticating: false,
        error: err.message,
      }));
      toast.error(err.message ?? 'Wallet authorization failed');
      return;
    }

    // Prevent multiple simultaneous authentication attempts
    if (state.isAuthenticating) {
      toast.error('Authentication already in progress. Please wait.');
      return;
    }

    setState(prev => ({ ...prev, isAuthenticating: true, error: undefined }));

    try {
      // Get challenge from backend
      const challengeResponse = await fetch(
        `${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/web3/challenge`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            wallet_address: address,
          }),
        }
      );

      if (!challengeResponse.ok) {
        throw new Error(
          `Failed to get authentication challenge: ${challengeResponse.status}`
        );
      }

      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const challenge = await challengeResponse.json();
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      const messageString = challenge.message;

      // Sign message with wallet
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition, @typescript-eslint/strict-boolean-expressions
      if (!signMessageAsync) {
        throw new Error(
          'Wallet signing function not available. Please reconnect your wallet.'
        );
      }

      let signature: string;
      try {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        signature = await signMessageAsync({ message: messageString });
      } catch (error: unknown) {
        // Handle user rejection gracefully
        const err = error as { code?: number; message?: string };
         
        if (
          err.code === 4001 ||
          err.message?.includes('User rejected') ||
          err.message?.includes('User denied')
        ) {
          throw new Error('Signature was cancelled by user');
           
        } else if (err.message?.includes('Method not found') ?? false) {
          throw new Error('Wallet does not support message signing');
           
        } else if (err.message?.includes('Connection lost') ?? false) {
          throw new Error('Wallet connection lost - please reconnect');
        } else {
          throw new Error(
            `Wallet signing failed: ${err.message ?? 'Unknown wallet error'}`
          );
        }
      }

      // Verify signature with backend
      const authResponse = await fetch(
        `${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/web3/verify`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            wallet_address: address,
            signature,
            message: messageString,
            nonce: challenge.nonce,
          }),
        }
      );

      if (!authResponse.ok) {
        const errorText = await authResponse.text();
         
        let errorData;
        try {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
          errorData = JSON.parse(errorText);
        } catch {
          errorData = {
            error: `Authentication failed: ${authResponse.status}`,
          };
        }
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        throw new Error(errorData.error ?? 'Authentication failed');
      }
       
      await authResponse.json();

      // Success
      setState(prev => ({
        ...prev,
        isAuthenticated: true,
        isAuthenticating: false,
        walletAddress: address,
      }));

      // Mark session for future auto-probing
      if (typeof window !== 'undefined') {
        try {
          window.localStorage.setItem('oidc_session', '1');
          document.cookie = 'oidc_session=1; path=/; SameSite=Lax';
        } catch (_error) {
          // Intentionally empty - session markers are optional
        }
      }

      // Refresh permissions after successful authentication
      await refreshPermissions();
      toast.success('Successfully authenticated with Web3 wallet');
    } catch (error: unknown) {
      // Handle common error types
      const err = error as { message?: string };
      let errorMessage = 'Authentication failed';
       
      if (
        err.message?.includes('User rejected') ||
        err.message?.includes('cancelled')
      ) {
        errorMessage = 'Wallet signature was cancelled';
         
      } else if (err.message?.includes('timeout') ?? false) {
        errorMessage = 'Request timeout. Please try again.';
         
      } else if (err.message?.includes('expired') ?? false) {
        errorMessage = 'Authentication expired - please try again';
      } else if (err.message) {
        errorMessage = err.message;
      }

      setState(prev => ({
        ...prev,
        isAuthenticating: false,
        isAuthenticated: false,
        error: errorMessage,
      }));

      toast.error(errorMessage);
    }
  }, [address, signMessageAsync, refreshPermissions, chain, connector, state.isAuthenticating]); // eslint-disable-line react-hooks/exhaustive-deps

  // eslint-disable-next-line sonarjs/cognitive-complexity, complexity
  const disconnect = useCallback(async () => {
    try {
      // Step 1: Disconnect individual connector FIRST (if available)
      if (connector && typeof connector.disconnect === 'function') {
        try {
          // Call individual connector disconnect
          await connector.disconnect();

          // Wait a moment for connector cleanup
          await new Promise(resolve => setTimeout(resolve, 150));
        } catch (_connectorError) {
          // Continue with wagmi disconnect even if connector disconnect fails
        }
      }

      // Step 2: THEN disconnect from Wagmi globally
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      if (wagmiDisconnect && typeof wagmiDisconnect === 'function') {
        try {
          // Call wagmi disconnect
          wagmiDisconnect();

          // Wait for Wagmi to actually disconnect by polling its state
          let attempts = 0;
          const maxAttempts = 10;
          while (attempts < maxAttempts) {
            await new Promise(resolve => setTimeout(resolve, 100));
            attempts++;

            // Note: We can't access latest Wagmi state here due to closure
            // The useEffect will handle state sync once Wagmi updates
            if (attempts >= 5) {break;} // Give reasonable time for disconnect
          }
        } catch (_wagmiError) {
          // Continue with cleanup even if wagmi disconnect fails
        }
      }

      // Step 2: Clear storage to ensure clean state - ENHANCED VERSION
       
      try {
        if (typeof window !== 'undefined') {
          // Get all localStorage keys that match our patterns
          const allKeys = [];
          for (let i = 0; i < window.localStorage.length; i++) {
            const key = window.localStorage.key(i);
            if (key !== null) {allKeys.push(key);}
          }

          // Find keys to remove
           
          const keysToRemove = allKeys.filter(
            key =>
              key.startsWith('wagmi.') ||
              key.startsWith('rk-') ||
              key.startsWith('rainbow') ||
              key.includes('wallet') ||
              key.includes('auth') ||
              key.includes('oidc') ||
              key === 'web3_auth_state'
          );

          // Remove each key
          keysToRemove.forEach(key => {
            try {
              window.localStorage.removeItem(key);
            } catch (_e) {
              // Intentionally empty - key removal is best effort
            }
          });

          // Clear basic cookies
          const cookiesToClear = [
            'oidc_session',
            'access_token',
            'id_token',
            'refresh_token',
          ];
          cookiesToClear.forEach(cookieName => {
            try {
              document.cookie = `${cookieName}=; Max-Age=0; path=/; SameSite=Lax`;
              document.cookie = `${cookieName}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/`;
            } catch (_e) {
              // Intentionally empty - cookie clearing is best effort
            }
          });
        }
      } catch (_storageError) {
        // Intentionally empty - storage cleanup is best effort
      }

      // Step 3: Server-side session invalidation (if needed)
      if (state.isAuthenticated && address) {
        try {
          await fetch('/api/auth/web3/logout', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            credentials: 'include',
            body: JSON.stringify({
              wallet_address: address,
              logout_reason: 'user_initiated_disconnect',
            }),
          });
        } catch (_error) {
          // Continue with cleanup
        }
      }

      // Step 4: Reset application state immediately (don't wait for useEffect)
      // The useEffect will handle final state sync when Wagmi state updates
      setState({
        isConnected: false,
        isAuthenticated: false,
        isAuthenticating: false,
        permissions: [],
        userTier: 'free',
        hasApiAccess: false,
        walletAddress: undefined,
        error: undefined,
      });

      toast.success('Wallet disconnected successfully');

      // Refresh page data without full reload
      router.refresh();
    } catch (_error) {
      // Force state reset and storage cleanup even if there are errors
      setState({
        isConnected: false,
        isAuthenticated: false,
        isAuthenticating: false,
        permissions: [],
        userTier: 'free',
        hasApiAccess: false,
        walletAddress: undefined,
        error: undefined,
      });

      // Force clear storage on error
      if (typeof window !== 'undefined') {
        try {
          window.localStorage.removeItem('wagmi.cache');
          window.localStorage.removeItem('wagmi.store');
          window.localStorage.removeItem('wagmi.recentConnector');
        } catch (_storageErr) {
          // Intentionally empty - storage cleanup is best effort
        }
      }

      toast.error('Wallet disconnected with some errors - refreshing page...');

      // Refresh page data without full reload
      router.refresh();
    }
  }, [wagmiDisconnect, state.isAuthenticated, address, isConnected, connector, router]); // eslint-disable-line react-hooks/exhaustive-deps
   
  const linkEmail = useCallback(
    async (email: string, password: string) => {
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      if (!address) {
        throw new Error('Wallet not connected');
      }

      const response = await fetch('/api/auth/web3/link-email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wallet_address: address,
          email,
          password,
        }),
        credentials: 'include',
      });

      if (!response.ok) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const error = await response.json();
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        throw new Error(error.message ?? 'Failed to link email');
      }

      toast.success('Email linked successfully');
      await refreshPermissions();
    },
    [address, refreshPermissions]
  );
   
  const generateApiKey = useCallback(
    async (name: string): Promise<string> => {
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      if (!address || !state.hasApiAccess) {
        throw new Error('API access not available');
      }

      const response = await fetch('/api/auth/web3/api-keys', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wallet_address: address,
          name,
        }),
        credentials: 'include',
      });

      if (!response.ok) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const error = await response.json();
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        throw new Error(error.message ?? 'Failed to generate API key');
      }

      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const { api_key } = await response.json();
      toast.success('API key generated successfully');
      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return api_key;
    },
    [address, state.hasApiAccess]
  );

  const resetAuthState = useCallback(() => {
    // Clear any pending timers or promises by forcing a state reset
    setState({
      isConnected: Boolean(address),
      isAuthenticated: false,
      isAuthenticating: false,
      permissions: [],
      userTier: 'free',
      hasApiAccess: false,
      walletAddress: address,
      error: undefined,
    });
  }, [address]);

  return {
    ...state,
    authenticate,
    disconnect,
    checkAuthStatus,
    refreshPermissions,
    linkEmail,
    generateApiKey,
    resetAuthState,
  };
}

// Utility functions for permission management
export function getPermissionIcon(source: Web3Permission['source']): string {
  switch (source) {
    case 'nft':
      return '🎨';
    case 'token':
      return '🪙';
    case 'dao':
      return '🗳️';
    case 'manual':
      return '👤';
    default:
      return '🔑';
  }
}

export function getPermissionBadgeColor(
  source: Web3Permission['source']
): string {
  switch (source) {
    case 'nft':
      return 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300';
    case 'token':
      return 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300';
    case 'dao':
      return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
    case 'manual':
      return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
    default:
      return 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-300';
  }
}

export function getTierDescription(tier: Web3AuthState['userTier']): string {
  switch (tier) {
    case 'free':
      return 'Basic access to platform features';
    case 'nft':
      return 'Enhanced access via NFT ownership';
    case 'token':
      return 'Token-gated premium features';
    case 'dao':
      return 'DAO governance access and voting';
    case 'enterprise':
      return 'Full API access and team management';
    default:
      return 'Standard user access';
  }
}

export function isPermissionExpired(permission: Web3Permission): boolean {
  if (permission.expires_at === undefined) {return false;}
  return new Date(permission.expires_at) < new Date();
}

export function formatAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}
