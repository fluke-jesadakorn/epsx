'use client';

import { useState, useCallback, useEffect } from 'react';
import { useAccount, useSignMessage, useDisconnect } from 'wagmi';
import { SiweMessage } from 'siwe';
import { toast } from 'sonner';

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
  checkAuthStatus: () => Promise<void>;
  refreshPermissions: () => Promise<void>;
  linkEmail: (email: string, password: string) => Promise<void>;
  generateApiKey: (name: string) => Promise<string>;
  resetAuthState: () => void;
}

export function useWeb3Auth(): Web3AuthState & Web3AuthActions {
  const { address, isConnected, connector, chain } = useAccount();
  const { disconnect: wagmiDisconnect } = useDisconnect();
  const { signMessageAsync, error: signError, isLoading: isSignLoading } = useSignMessage();

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

    const channel = new BroadcastChannel('auth_session');
    
    const handleSessionMessage = (event: MessageEvent) => {
      console.log('📡 Received cross-tab session message:', event.data);
      
      // Only process explicit disconnect messages, ignore auto-disconnects
      if (event.data.type === 'SESSION_INVALIDATED' && event.data.source === 'web3_disconnect') {
        console.log('🔄 Processing explicit session invalidation from another tab...');
        
        // Only invalidate if the wallet address matches
        if (!event.data.walletAddress || event.data.walletAddress === address) {
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
          
          console.log('✅ Session invalidated in response to cross-tab disconnect');
          toast.info('Session was ended in another tab');
        }
      }
    };

    channel.addEventListener('message', handleSessionMessage);
    
    return () => {
      channel.removeEventListener('message', handleSessionMessage);
      channel.close();
    };
  }, [address]);

  // Auto-check auth status when wallet connects (with proper hydration handling)
  useEffect(() => {
    if (!isHydrated) return; // Wait for hydration

    console.log('🔍 Web3Auth State Check (post-hydration):');
    console.log('  address:', address);
    console.log('  isConnected:', isConnected);
    console.log('  isHydrated:', isHydrated);

    if (address) {
      // Always update the wallet address in state when available
      setState(prev => ({
        ...prev,
        isConnected: !!isConnected,
        walletAddress: address,
      }));

      // Check auth status when we have an address
      checkAuthStatus();
    } else {
      // Clear state when no address
      setState(prev => ({
        ...prev,
        isConnected: false,
        isAuthenticated: false,
        walletAddress: undefined,
        permissions: [],
        userTier: 'free',
        hasApiAccess: false,
      }));
    }
  }, [address, isConnected, isHydrated]);

  const checkAuthStatus = useCallback(async () => {
    if (!address) return;

    try {
      // Check for session if wallet is connected
      const response = await fetch('/api/auth/session', {
        credentials: 'include',
        cache: 'no-cache',
      });

      if (response.ok) {
        const session = await response.json();
        if (session.isAuthenticated && session.user?.wallet_address === address) {
          setState(prev => ({
            ...prev,
            isConnected: true,
            isAuthenticated: true,
            walletAddress: address,
          }));
          await refreshPermissions();
          return;
        }
      } else if (response.status === 401) {
        // 401 is expected in progressive auth when no session exists yet
        // This is normal behavior - user is connected but not authenticated
        console.log('🔗 Wallet connected but no authenticated session (expected in progressive auth)');
      } else {
        console.warn('Session check returned unexpected status:', response.status);
      }

      // Not authenticated but connected (normal state in progressive auth)
      setState(prev => ({
        ...prev,
        isConnected: true,
        isAuthenticated: false,
        walletAddress: address,
        error: null, // Clear any previous errors
      }));
    } catch (error) {
      console.error('Failed to check auth status:', error);
      // In progressive auth, treat network errors as non-critical
      setState(prev => ({
        ...prev,
        isConnected: true,
        isAuthenticated: false,
        walletAddress: address,
        // Only set error for actual failures, not auth state
      }));
    }
  }, [address]);

  const refreshPermissions = useCallback(async () => {
    if (!address) return;

    try {
      const response = await fetch(`/api/auth/web3/permissions?wallet_address=${encodeURIComponent(address)}`, {
        method: 'GET',
        credentials: 'include',
      });

      if (response.ok) {
        const { permissions, user_tier, has_api_access } = await response.json();
        setState(prev => ({
          ...prev,
          permissions: permissions || [],
          userTier: user_tier || 'free',
          hasApiAccess: has_api_access || false,
        }));
      } else if (response.status === 405) {
        console.log('Permissions endpoint not available (405) - using default values');
        // Set default values when permissions endpoint is not available
        setState(prev => ({
          ...prev,
          permissions: [],
          userTier: 'free',
          hasApiAccess: false,
        }));
        return; // Return early to avoid error throwing
      }
    } catch (error) {
      console.error('Failed to fetch permissions:', error);
      // Set default values on error
      setState(prev => ({
        ...prev,
        permissions: [],
        userTier: 'free',
        hasApiAccess: false,
      }));
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
      if (provider && typeof provider.request === 'function') {
        // First, request account access to ensure proper authorization
        try {
          console.log('🔑 Requesting wallet account access...');
          const accounts = await provider.request({ 
            method: 'eth_requestAccounts' 
          });
          console.log('✅ Wallet access granted, accounts:', accounts?.length || 0);
          
          // Verify the current address is in the authorized accounts
          if (accounts && accounts.length > 0) {
            const normalizedAddress = address.toLowerCase();
            const hasMatchingAccount = accounts.some((acc: string) => 
              acc.toLowerCase() === normalizedAddress
            );
            
            if (!hasMatchingAccount) {
              throw new Error('Connected wallet address not found in authorized accounts');
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
            throw new Error('Wallet not authorized - please connect your wallet first');
          } else if (authError.message?.includes('User rejected')) {
            throw new Error('Wallet access was rejected');
          } else {
            throw new Error(`Wallet authorization failed: ${authError.message || 'Unknown error'}`);
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
      const challengeResponse = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/web3/challenge`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wallet_address: address,
        }),
      });

      if (!challengeResponse.ok) {
        throw new Error(`Failed to get authentication challenge: ${challengeResponse.status}`);
      }

      const challenge = await challengeResponse.json();
      const messageString = challenge.message;

      // Sign message with wallet
      if (!signMessageAsync) {
        throw new Error('Wallet signing function not available. Please reconnect your wallet.');
      }
      
      let signature: string;
      try {
        signature = await signMessageAsync({ message: messageString });
      } catch (error: any) {
        // Handle user rejection gracefully
        if (error.code === 4001 || error.message?.includes('User rejected') || error.message?.includes('User denied')) {
          throw new Error('Signature was cancelled by user');
        } else if (error.message?.includes('Method not found')) {
          throw new Error('Wallet does not support message signing');
        } else if (error.message?.includes('Connection lost')) {
          throw new Error('Wallet connection lost - please reconnect');
        } else {
          throw new Error(`Wallet signing failed: ${error.message || 'Unknown wallet error'}`);
        }
      }

      // Verify signature with backend
      const authResponse = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/web3/verify`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wallet_address: address,
          signature,
          message: messageString,
          nonce: challenge.nonce,
        }),
      });

      if (!authResponse.ok) {
        const errorText = await authResponse.text();
        let errorData;
        try {
          errorData = JSON.parse(errorText);
        } catch {
          errorData = { error: `Authentication failed: ${authResponse.status}` };
        }
        throw new Error(errorData.error || 'Authentication failed');
      }

      const authResult = await authResponse.json();

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

      await refreshPermissions();
      toast.success('Successfully authenticated with Web3 wallet');

    } catch (error: any) {
      // Handle common error types
      let errorMessage = 'Authentication failed';
      if (error.message?.includes('User rejected') || error.message?.includes('cancelled')) {
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
    try {
      // Server-side session invalidation (if authenticated)
      if (state.isAuthenticated) {
        try {
          const web3LogoutResponse = await fetch('/api/auth/web3/logout', {
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
          
          if (!web3LogoutResponse.ok) {
            // Fallback to general session invalidation
            await fetch('/api/auth/session', {
              method: 'DELETE',
              credentials: 'include',
            });
          }
        } catch (error) {
          console.error('Session invalidation error:', error);
        }
      }

      // Clear client-side session storage
      if (typeof window !== 'undefined') {
        try {
          // Clear localStorage markers
          window.localStorage.removeItem('oidc_session');
          window.localStorage.removeItem('web3_auth_state');
          window.localStorage.removeItem('wagmi.cache');
          window.localStorage.removeItem('wagmi.store');
          
          // Clear sessionStorage
          window.sessionStorage.removeItem('oidc_session');
          window.sessionStorage.removeItem('web3_auth_state');
          
          // Clear authentication cookies
          const cookiesToClear = [
            'oidc_session',
            'access_token', 
            'id_token', 
            'refresh_token',
            'epsx_frontend_jwt',
            'next-auth.session-token',
            'next-auth.csrf-token'
          ];
          
          cookiesToClear.forEach(cookieName => {
            document.cookie = `${cookieName}=; Max-Age=0; path=/; SameSite=Lax`;
            document.cookie = `${cookieName}=; Max-Age=0; path=/; domain=${window.location.hostname}; SameSite=Lax`;
            document.cookie = `${cookieName}=; Max-Age=0; path=/; SameSite=Lax; Secure`;
          });
        } catch (storageError) {
          console.error('Storage clearing error:', storageError);
        }
      }

      // Disconnect wallet
      try {
        wagmiDisconnect();
      } catch (walletError) {
        console.error('Wallet disconnect error:', walletError);
      }

      // Reset application state
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

      // Broadcast session invalidation to other tabs
      if (typeof window !== 'undefined' && window.BroadcastChannel) {
        try {
          const channel = new BroadcastChannel('auth_session');
          channel.postMessage({ 
            type: 'SESSION_INVALIDATED', 
            timestamp: Date.now(),
            source: 'web3_disconnect',
            walletAddress: address
          });
          channel.close();
        } catch (broadcastError) {
          console.warn('Failed to broadcast session invalidation:', broadcastError);
        }
      }

      toast.success('Successfully disconnected');

    } catch (error) {
      console.error('Disconnect error:', error);
      
      // Force state reset even if there are errors
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
      
      toast.error('Disconnected with errors - you may need to refresh the page');
    }
  }, [wagmiDisconnect, state.isAuthenticated]);

  const linkEmail = useCallback(async (email: string, password: string) => {
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
  }, [address, refreshPermissions]);

  const generateApiKey = useCallback(async (name: string): Promise<string> => {
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
  }, [address, state.hasApiAccess]);

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
    case 'nft': return '🎨';
    case 'token': return '🪙';
    case 'dao': return '🗳️';
    case 'manual': return '👤';
    default: return '🔑';
  }
}

export function getPermissionBadgeColor(source: Web3Permission['source']): string {
  switch (source) {
    case 'nft': return 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300';
    case 'token': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300';
    case 'dao': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
    case 'manual': return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
    default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-300';
  }
}

export function getTierDescription(tier: Web3AuthState['userTier']): string {
  switch (tier) {
    case 'free': return 'Basic access to platform features';
    case 'nft': return 'Enhanced access via NFT ownership';
    case 'token': return 'Token-gated premium features';
    case 'dao': return 'DAO governance access and voting';
    case 'enterprise': return 'Full API access and team management';
    default: return 'Standard user access';
  }
}

export function isPermissionExpired(permission: Web3Permission): boolean {
  if (!permission.expires_at) return false;
  return new Date(permission.expires_at) < new Date();
}

export function formatAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}