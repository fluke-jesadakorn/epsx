'use client';

import { useCallback, useEffect } from 'react';
import { useAccount, useDisconnect, useSignMessage } from 'wagmi';
import { useWeb3AuthStore } from './web3-store';
import { toast } from 'sonner';

export function useWeb3Auth() {
  const { address, isConnected, connector } = useAccount();
  const { disconnect: wagmiDisconnect } = useDisconnect();
  const { signMessageAsync } = useSignMessage();

  // Store selectors
  const {
    isAuthenticated,
    isAuthenticating,
    isLoading,
    hasInitialized,
    walletAddress,
    permissions,
    userTier,
    hasApiAccess,
    error,
  } = useWeb3AuthStore();

  // Store actions
  const {
    setConnected,
    setWalletAddress,
    setError,
    authenticate,
    disconnect,
    checkAuthStatus,
    refreshPermissions,
    linkEmail,
    generateApiKey,
    resetAuthState,
    initializeAuth,
  } = useWeb3AuthStore();

  // Sync Wagmi state with store
  useEffect(() => {
    console.log('🔍 Syncing Wagmi state with Web3 store:', {
      wagmiConnected: isConnected,
      wagmiAddress: address,
      storeConnected: useWeb3AuthStore.getState().isConnected,
      storeAddress: walletAddress,
    });

    if (address && isConnected) {
      setConnected(true);
      setWalletAddress(address);
      setError(undefined);
      
      // Initialize auth if not already done
      if (!hasInitialized) {
        initializeAuth();
      }
    } else {
      setConnected(false);
      setWalletAddress(undefined);
      resetAuthState();
    }
  }, [address, isConnected, hasInitialized, setConnected, setWalletAddress, setError, resetAuthState, initializeAuth, walletAddress]);

  // Cross-tab session invalidation
  useEffect(() => {
    if (typeof window === 'undefined' || !window.BroadcastChannel) return;

    let channel: BroadcastChannel | null = null;

    try {
      channel = new BroadcastChannel('auth_session');

      const handleSessionMessage = (event: MessageEvent) => {
        try {
          console.log('📡 Received cross-tab session message:', event.data);

          if (
            event.data?.type === 'SESSION_INVALIDATED' &&
            event.data?.source === 'web3_disconnect'
          ) {
            console.log('🔄 Processing session invalidation from another tab...');

            if (!event.data.walletAddress || event.data.walletAddress === address) {
              resetAuthState();

              try {
                window.localStorage.removeItem('oidc_session');
                window.sessionStorage.removeItem('oidc_session');
              } catch {}

              console.log('✅ Session invalidated in response to cross-tab disconnect');
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
            channel.close();
          }
        } catch (error) {
          console.warn('Error cleaning up cross-tab channel:', error);
        }
      };
    } catch (error) {
      console.warn('Failed to create cross-tab channel:', error);
      return () => {};
    }
  }, [address, resetAuthState]);

  // Enhanced authenticate function with Wagmi integration
  const authenticateWithWallet = useCallback(async () => {
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

    // Request wallet access first
    try {
      const provider = await connector.getProvider?.();
      if (provider && typeof provider === 'object' && 'request' in provider && typeof (provider as any).request === 'function') {
        try {
          console.log('🔑 Requesting wallet account access...');
          const accounts = await (provider as any).request({
            method: 'eth_requestAccounts',
          });
          console.log('✅ Wallet access granted, accounts:', accounts?.length || 0);

          if (accounts && accounts.length > 0) {
            const normalizedAddress = address.toLowerCase();
            const hasMatchingAccount = accounts.some(
              (acc: string) => acc.toLowerCase() === normalizedAddress
            );

            if (!hasMatchingAccount) {
              throw new Error('Connected wallet address not found in authorized accounts');
            }
          } else {
            throw new Error('No accounts returned from wallet');
          }
        } catch (authError: any) {
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
      setError(error.message);
      toast.error(error.message);
      return;
    }

    // Set up signing function for the store
    (window as any).__web3Auth_signMessage = async (message: string) => {
      try {
        return await signMessageAsync({ message });
      } catch (error: any) {
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
          throw new Error(`Wallet signing failed: ${error.message || 'Unknown wallet error'}`);
        }
      }
    };

    // Call store authenticate function
    await authenticate();

    // Clean up
    delete (window as any).__web3Auth_signMessage;
  }, [address, connector, signMessageAsync, authenticate, setError]);

  // Enhanced disconnect function with Wagmi integration
  const disconnectWallet = useCallback(async () => {
    console.log('🔄 Starting comprehensive wallet disconnect...');

    try {
      // Step 1: Disconnect individual connector first
      if (connector && typeof connector.disconnect === 'function') {
        try {
          console.log(`🔌 Disconnecting individual connector: ${connector.name}`);
          await connector.disconnect();
          console.log('✅ Individual connector disconnect completed');
          await new Promise(resolve => setTimeout(resolve, 150));
        } catch (connectorError) {
          console.error('❌ Individual connector disconnect failed:', connectorError);
        }
      }

      // Step 2: Disconnect from Wagmi globally
      console.log('🔌 Disconnecting from Wagmi globally...');
      if (wagmiDisconnect && typeof wagmiDisconnect === 'function') {
        try {
          wagmiDisconnect();
          console.log('📞 Wagmi disconnect called');
          
          // Wait for disconnect to complete
          let attempts = 0;
          const maxAttempts = 10;
          while (attempts < maxAttempts) {
            await new Promise(resolve => setTimeout(resolve, 100));
            attempts++;
            if (attempts >= 5) break;
          }
          console.log('✅ Wagmi global disconnect completed');
        } catch (wagmiError) {
          console.error('❌ Wagmi disconnect failed:', wagmiError);
        }
      }

      // Step 3: Call store disconnect function
      await disconnect();
    } catch (error) {
      console.error('❌ Error during wallet disconnect:', error);
      toast.error('Wallet disconnected with errors - refreshing page...');
      window.location.reload();
    }
  }, [connector, wagmiDisconnect, disconnect]);

  return {
    // State
    isConnected: isConnected && useWeb3AuthStore.getState().isConnected,
    isAuthenticated,
    isAuthenticating,
    isLoading,
    hasInitialized,
    walletAddress: address,
    permissions,
    userTier,
    hasApiAccess,
    error,

    // Actions
    authenticate: authenticateWithWallet,
    disconnect: disconnectWallet,
    checkAuthStatus,
    refreshPermissions,
    linkEmail,
    generateApiKey,
    resetAuthState,
  };
}

// Export the store for direct access when needed
export { useWeb3AuthStore } from './web3-store';

// Export permission hooks
export function useWeb3Permission(permission: string): boolean {
  const { permissions, isAuthenticated } = useWeb3AuthStore();
  
  if (!isAuthenticated) return false;
  
  return permissions.some(p => 
    p.permission === permission || 
    p.permission.includes('*') ||
    permission.startsWith(p.permission.replace('*', ''))
  );
}

export function useWeb3Tier(requiredTier: 'nft' | 'token' | 'dao' | 'enterprise'): boolean {
  const { userTier, isAuthenticated } = useWeb3AuthStore();
  
  if (!isAuthenticated) return false;
  
  const tierHierarchy = {
    nft: 1,
    token: 2,
    dao: 3,
    enterprise: 4,
  };

  const userLevel = tierHierarchy[userTier as keyof typeof tierHierarchy] || 0;
  const requiredLevel = tierHierarchy[requiredTier];

  return userLevel >= requiredLevel;
}