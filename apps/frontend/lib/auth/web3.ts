'use client';

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
    [key: string]: any;
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

export function useWeb3Auth(): Web3AuthState & Web3AuthActions {
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
    if (typeof window === 'undefined' || !window.BroadcastChannel) return;

    let channel: BroadcastChannel | null = null;

    try {
      channel = new BroadcastChannel('auth_session');

      const handleSessionMessage = (event: MessageEvent) => {
        try {
          console.log('📡 Received cross-tab session message:', event.data);

          // Only process explicit disconnect messages, ignore auto-disconnects
          if (
            event.data?.type === 'SESSION_INVALIDATED' &&
            event.data?.source === 'web3_disconnect'
          ) {
            console.log(
              '🔄 Processing explicit session invalidation from another tab...'
            );

            // Only invalidate if the wallet address matches
            if (
              !event.data.walletAddress ||
              event.data.walletAddress === address
            ) {
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
              } catch {}

              console.log(
                '✅ Session invalidated in response to cross-tab disconnect'
              );
              toast.info('Session was ended in another tab');
            }
          }
        } catch (error) {
          console.warn('Error handling cross-tab message:', error);
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
        } catch (error) {
          console.warn('Error cleaning up cross-tab channel:', error);
        }
      };
    } catch (error) {
      console.warn('Failed to create cross-tab channel:', error);
      return () => {}; // No-op cleanup
    }
  }, [address]);

  // Auto-check auth status when wallet connects (with proper hydration handling)
  useEffect(() => {
    if (!isHydrated) return; // Wait for hydration

    console.log('🔍 Web3Auth State Check (post-hydration):');
    console.log('  Wagmi address:', address);
    console.log('  Wagmi isConnected:', isConnected);
    console.log('  Internal state.isConnected:', state.isConnected);
    console.log('  Internal state.walletAddress:', state.walletAddress);
    console.log('  isHydrated:', isHydrated);

    if (address && isConnected) {
      // IMPORTANT: Always sync internal state with Wagmi state first
      console.log('✅ Wagmi shows wallet connected - syncing internal state');
      setState(prev => ({
        ...prev,
        isConnected: true, // Force sync with Wagmi
        walletAddress: address,
        error: undefined, // Clear errors when successfully connected
      }));

      // Check auth status when we have an address (inline to avoid dependency issues)
      (async () => {
        try {
          // Check for session if wallet is connected
          const response = await fetch('/api/auth/session', {
            credentials: 'include',
            cache: 'no-cache',
          });

          if (response.ok) {
            const session = await response.json();
            if (
              session.isAuthenticated &&
              session.user?.wallet_address === address
            ) {
              console.log('✅ Found existing authenticated session');
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

                if (permResponse.ok) {
                  const { permissions, user_tier, has_api_access } =
                    await permResponse.json();
                  setState(prev => ({
                    ...prev,
                    permissions: permissions || [],
                    userTier: user_tier || 'free',
                    hasApiAccess: has_api_access || false,
                  }));
                }
              } catch (permError) {
                console.warn('Failed to fetch permissions:', permError);
              }
              return;
            }
          } else if (response.status === 401) {
            // 401 is expected in progressive auth when no session exists yet
            console.log(
              '🔗 Wallet connected but no authenticated session (expected in progressive auth)'
            );
          } else if (response.status === 500) {
            // 500 indicates backend/API issues - don't treat as auth failure
            console.warn(
              'Session API returned 500 - backend may be unavailable (non-critical in Web3-first mode)'
            );
          } else {
            console.warn(
              'Session check returned unexpected status:',
              response.status
            );
          }

          // Not authenticated but connected (normal state in progressive auth)
          console.log(
            '🔗 Wallet connected but not authenticated - ready for sign-in'
          );
          setState(prev => ({
            ...prev,
            isConnected: true, // CRITICAL: Keep this true when Wagmi shows connected
            isAuthenticated: false,
            walletAddress: address,
            error: undefined, // Clear any previous errors
          }));
        } catch (error) {
          console.warn(
            'Failed to check auth status (non-critical in Web3-first mode):',
            error
          );
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
      console.log(
        '🔄 Wagmi shows wallet disconnected - clearing internal state'
      );
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
    if (!address) return;

    try {
      // Simplified auth status check - just check session
      const response = await fetch('/api/auth/session', {
        credentials: 'include',
        cache: 'no-cache',
      });

      if (response.ok) {
        const session = await response.json();
        if (
          session.isAuthenticated &&
          session.user?.wallet_address === address
        ) {
          setState(prev => ({
            ...prev,
            isConnected: true,
            isAuthenticated: true,
            walletAddress: address,
          }));
          return true;
        }
      }

      // Not authenticated but may be connected
      setState(prev => ({
        ...prev,
        isAuthenticated: false,
        error: undefined,
      }));
      return false;
    } catch (error) {
      console.warn('Auth status check failed:', error);
      return false;
    }
  }, [address]);

  const refreshPermissions = useCallback(async () => {
    if (!address) return false;

    try {
      const response = await fetch(
        `/api/auth/web3/permissions?wallet_address=${encodeURIComponent(address)}`,
        {
          method: 'GET',
          credentials: 'include',
        }
      );

      if (response.ok) {
        const { permissions, user_tier, has_api_access } =
          await response.json();
        setState(prev => ({
          ...prev,
          permissions: permissions || [],
          userTier: user_tier || 'free',
          hasApiAccess: has_api_access || false,
        }));
        return true;
      } else if (response.status === 405) {
        console.log(
          'Permissions endpoint not available (405) - using default values'
        );
        // Set default values when permissions endpoint is not available
        setState(prev => ({
          ...prev,
          permissions: [],
          userTier: 'free',
          hasApiAccess: false,
        }));
        return true; // Successfully set defaults
      }
      return false;
    } catch (error) {
      console.warn('Failed to fetch permissions:', error);
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

  const authenticate = useCallback(async () => {
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
      if (provider && typeof provider === 'object' && 'request' in provider && typeof (provider as any).request === 'function') {
        // First, request account access to ensure proper authorization
        try {
          console.log('🔑 Requesting wallet account access...');
          const accounts = await (provider as any).request({
            method: 'eth_requestAccounts',
          });
          console.log(
            '✅ Wallet access granted, accounts:',
            accounts?.length || 0
          );

          // Verify the current address is in the authorized accounts
          if (accounts && accounts.length > 0) {
            const normalizedAddress = address.toLowerCase();
            const hasMatchingAccount = accounts.some(
              (acc: string) => acc.toLowerCase() === normalizedAddress
            );

            if (!hasMatchingAccount) {
              throw new Error(
                'Connected wallet address not found in authorized accounts'
              );
            }

            console.log('✅ Wallet authorization verified');
          } else {
            throw new Error('No accounts returned from wallet');
          }
        } catch (authError: any) {
          // Handle specific authorization errors
          if (authError.code === 4001) {
            throw new Error('Wallet access denied by user');
          } else if (authError.code === 4100) {
            throw new Error(
              'Wallet not authorized - please connect your wallet first'
            );
          } else if (authError.message?.includes('User rejected')) {
            throw new Error('Wallet access was rejected');
          } else {
            throw new Error(
              `Wallet authorization failed: ${authError.message || 'Unknown error'}`
            );
          }
        }
      }
    } catch (error: any) {
      console.error('❌ Wallet authorization failed:', error.message);
      setState(prev => ({
        ...prev,
        isAuthenticating: false,
        error: error.message,
      }));
      toast.error(error.message);
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

      const challenge = await challengeResponse.json();
      const messageString = challenge.message;

      // Sign message with wallet
      if (!signMessageAsync) {
        throw new Error(
          'Wallet signing function not available. Please reconnect your wallet.'
        );
      }

      let signature: string;
      try {
        signature = await signMessageAsync({ message: messageString });
      } catch (error: any) {
        // Handle user rejection gracefully
        if (
          error.code === 4001 ||
          error.message?.includes('User rejected') ||
          error.message?.includes('User denied')
        ) {
          throw new Error('Signature was cancelled by user');
        } else if (error.message?.includes('Method not found')) {
          throw new Error('Wallet does not support message signing');
        } else if (error.message?.includes('Connection lost')) {
          throw new Error('Wallet connection lost - please reconnect');
        } else {
          throw new Error(
            `Wallet signing failed: ${error.message || 'Unknown wallet error'}`
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
          errorData = JSON.parse(errorText);
        } catch {
          errorData = {
            error: `Authentication failed: ${authResponse.status}`,
          };
        }
        throw new Error(errorData.error || 'Authentication failed');
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
        } catch {}
      }

      // Refresh permissions after successful authentication
      await refreshPermissions();
      toast.success('Successfully authenticated with Web3 wallet');
    } catch (error: any) {
      // Handle common error types
      let errorMessage = 'Authentication failed';
      if (
        error.message?.includes('User rejected') ||
        error.message?.includes('cancelled')
      ) {
        errorMessage = 'Wallet signature was cancelled';
      } else if (error.message?.includes('timeout')) {
        errorMessage = 'Request timeout. Please try again.';
      } else if (error.message?.includes('expired')) {
        errorMessage = 'Authentication expired - please try again';
      } else if (error.message) {
        errorMessage = error.message;
      }

      setState(prev => ({
        ...prev,
        isAuthenticating: false,
        isAuthenticated: false,
        error: errorMessage,
      }));

      toast.error(errorMessage);
    }
  }, [address, signMessageAsync, refreshPermissions, chain]);

  const disconnect = useCallback(async () => {
    console.log('🔄 Starting comprehensive wallet disconnect...');
    console.log('🔍 Disconnect function called with:', {
      wagmiDisconnectExists: !!wagmiDisconnect,
      currentAddress: address,
      wagmiIsConnected: isConnected,
      internalIsConnected: state.isConnected,
      isAuthenticated: state.isAuthenticated,
      currentConnector: connector?.name || 'none',
    });

    // Debug: Log current localStorage state
    if (typeof window !== 'undefined') {
      console.log('📋 Current localStorage before disconnect:');
      const allKeys = [];
      for (let i = 0; i < window.localStorage.length; i++) {
        const key = window.localStorage.key(i);
        if (key) allKeys.push(key);
      }
      console.log('📋 Total localStorage keys:', allKeys.length);

      const relevantKeys = allKeys.filter(
        key =>
          key.includes('wagmi') ||
          key.includes('wallet') ||
          key.includes('auth')
      );
      console.log('📋 Wallet/auth related keys:', relevantKeys.length);

      relevantKeys.forEach(key => {
        const value = window.localStorage.getItem(key);
        console.log(`  - ${key}: ${value?.substring(0, 100)}...`);
      });
    }

    try {
      // Step 1: Disconnect individual connector FIRST (if available)
      if (connector && typeof connector.disconnect === 'function') {
        try {
          console.log(
            `🔌 Disconnecting individual connector: ${connector.name}`
          );
          console.log('🔍 Connector state before disconnect:', {
            connected: connector.connected,
            ready: connector.ready,
            id: connector.id,
          });

          // Call individual connector disconnect
          await connector.disconnect();
          console.log('✅ Individual connector disconnect completed');

          // Verify connector state was reset
          console.log('🔍 Connector state after disconnect:', {
            connected: connector.connected,
            ready: connector.ready,
            id: connector.id,
          });

          // Wait a moment for connector cleanup
          await new Promise(resolve => setTimeout(resolve, 150));
        } catch (connectorError) {
          console.error(
            '❌ Individual connector disconnect failed:',
            connectorError
          );
          // Continue with wagmi disconnect even if connector disconnect fails
        }
      }

      // Step 2: THEN disconnect from Wagmi globally
      console.log('🔌 Disconnecting from Wagmi globally...');
      if (wagmiDisconnect && typeof wagmiDisconnect === 'function') {
        try {
          // Call wagmi disconnect
          wagmiDisconnect();
          console.log('📞 Wagmi disconnect called');

          // Wait for Wagmi to actually disconnect by polling its state
          let attempts = 0;
          const maxAttempts = 10;
          while (attempts < maxAttempts) {
            await new Promise(resolve => setTimeout(resolve, 100));
            attempts++;

            // Note: We can't access latest Wagmi state here due to closure
            // The useEffect will handle state sync once Wagmi updates
            if (attempts >= 5) break; // Give reasonable time for disconnect
          }
          console.log(
            '✅ Wagmi global disconnect completed (or timed out after reasonable wait)'
          );
        } catch (wagmiError) {
          console.error('❌ Wagmi disconnect failed:', wagmiError);
          // Continue with cleanup even if wagmi disconnect fails
        }
      }

      // Step 3: Validate all disconnect operations completed
      console.log('🔍 Post-disconnect validation:');
      if (connector) {
        console.log(
          `  - Connector ${connector.name} connected state:`,
          connector.connected
        );
      }
      console.log('  - Wagmi state will be validated by useEffect');

      // Step 2: Clear storage to ensure clean state - ENHANCED VERSION
      try {
        if (typeof window !== 'undefined') {
          console.log('🧹 Starting localStorage cleanup...');

          // Get all localStorage keys that match our patterns
          const allKeys = [];
          for (let i = 0; i < window.localStorage.length; i++) {
            const key = window.localStorage.key(i);
            if (key) allKeys.push(key);
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

          console.log(
            `🗑️ Found ${keysToRemove.length} keys to remove:`,
            keysToRemove
          );

          // Remove each key and log result
          keysToRemove.forEach(key => {
            try {
              const hadValue = window.localStorage.getItem(key) !== null;
              window.localStorage.removeItem(key);
              const nowRemoved = window.localStorage.getItem(key) === null;
              console.log(
                `  - ${key}: ${hadValue ? 'had value' : 'empty'} → ${nowRemoved ? 'REMOVED ✅' : 'FAILED ❌'}`
              );
            } catch (e) {
              console.error(`❌ Failed to remove ${key}:`, e);
            }
          });

          // Clear basic cookies
          const cookiesToClear = [
            'oidc_session',
            'access_token',
            'id_token',
            'refresh_token',
          ];
          console.log('🍪 Clearing cookies:', cookiesToClear);
          cookiesToClear.forEach(cookieName => {
            try {
              document.cookie = `${cookieName}=; Max-Age=0; path=/; SameSite=Lax`;
              document.cookie = `${cookieName}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/`;
              console.log(`  - Cookie ${cookieName} cleared`);
            } catch (e) {
              console.warn(`Could not clear cookie ${cookieName}:`, e);
            }
          });
        }
        console.log('✅ Storage cleanup completed');
      } catch (storageError) {
        console.error('❌ Storage cleanup error:', storageError);
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
              session_token: null,
              logout_reason: 'user_initiated_disconnect',
            }),
          });
          console.log('✅ Server-side session invalidated');
        } catch (error) {
          console.warn('⚠️ Server-side session invalidation failed:', error);
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
      console.log(
        '✅ Application state reset (useEffect will handle final sync)'
      );

      // Debug: Log localStorage state after cleanup
      if (typeof window !== 'undefined') {
        console.log('📋 Final localStorage state after disconnect:');
        const remainingKeys = [];
        for (let i = 0; i < window.localStorage.length; i++) {
          const key = window.localStorage.key(i);
          if (
            key &&
            (key.includes('wagmi') ||
              key.includes('wallet') ||
              key.includes('auth'))
          ) {
            remainingKeys.push(key);
            console.log(
              `  - REMAINING: ${key}: ${window.localStorage.getItem(key)?.substring(0, 50)}...`
            );
          }
        }
        if (remainingKeys.length === 0) {
          console.log(
            '  ✅ No wallet/auth related keys remaining in localStorage'
          );
        } else {
          console.warn(
            `  ⚠️ ${remainingKeys.length} keys still present:`,
            remainingKeys
          );
        }
      }

      toast.success('Wallet disconnected successfully');
      console.log('✅ Complete wallet disconnect finished');

      // Refresh the page to ensure complete state reset
      window.location.reload();
    } catch (error) {
      console.error('❌ Error during wallet disconnect:', error);

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
        } catch {}
      }

      toast.error('Wallet disconnected with some errors - refreshing page...');

      // Refresh the page even on errors to ensure complete state reset
      window.location.reload();
    }
  }, [wagmiDisconnect, state.isAuthenticated, address, isConnected, connector]);

  const linkEmail = useCallback(
    async (email: string, password: string) => {
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
        const error = await response.json();
        throw new Error(error.message || 'Failed to link email');
      }

      toast.success('Email linked successfully');
      await refreshPermissions();
    },
    [address, refreshPermissions]
  );

  const generateApiKey = useCallback(
    async (name: string): Promise<string> => {
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
        const error = await response.json();
        throw new Error(error.message || 'Failed to generate API key');
      }

      const { api_key } = await response.json();
      toast.success('API key generated successfully');
      return api_key;
    },
    [address, state.hasApiAccess]
  );

  const resetAuthState = useCallback(() => {
    console.log('🔄 Manually resetting authentication state...');

    // Clear any pending timers or promises by forcing a state reset
    setState({
      isConnected: !!address,
      isAuthenticated: false,
      isAuthenticating: false,
      permissions: [],
      userTier: 'free',
      hasApiAccess: false,
      walletAddress: address,
      error: undefined,
    });

    console.log('✅ Authentication state reset complete');
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
  if (!permission.expires_at) return false;
  return new Date(permission.expires_at) < new Date();
}

export function formatAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}
