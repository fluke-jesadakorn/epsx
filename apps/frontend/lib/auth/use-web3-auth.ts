'use client';

import { useCallback, useEffect, useState } from 'react';
import { useAccount, useDisconnect, useSignMessage } from 'wagmi';
import { useWeb3AuthStore } from './web3-store';
import { toast } from 'sonner';

export function useWeb3Auth() {
  const [isHydrated, setIsHydrated] = useState(false);
  const { address, isConnected, connector } = useAccount();
  const { disconnect: wagmiDisconnect } = useDisconnect();
  const { signMessageAsync } = useSignMessage();

  // Prevent hydration issues
  useEffect(() => {
    setIsHydrated(true);
  }, []);

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

  // Sync Wagmi state with store (only after hydration)
  useEffect(() => {
    // Skip state updates during hydration to prevent React errors
    if (!isHydrated) return;

    const storeState = useWeb3AuthStore.getState();
    console.log('🔍 [DEBUG] Syncing Wagmi state with Web3 store:', {
      wagmiConnected: isConnected,
      wagmiAddress: address,
      storeConnected: storeState.isConnected,
      storeAddress: walletAddress,
      storeInitialized: storeState.hasInitialized,
      storeAuthenticated: storeState.isAuthenticated,
      connectorName: connector?.name,
      timestamp: new Date().toISOString()
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
      // Wallet disconnected - always clear state to prevent conflicts
      setConnected(false);
      setWalletAddress(undefined);
      resetAuthState();
      
      // Additional safety: ensure hasInitialized is reset for fresh connections
      if (hasInitialized && !isConnected) {
        console.log('🔄 Resetting initialization flag for fresh connection');
        // The resetAuthState should handle this, but ensure it's reset
      }
    }
  }, [isHydrated, address, isConnected, hasInitialized, setConnected, setWalletAddress, setError, resetAuthState, initializeAuth, walletAddress]);

  // Cross-tab session invalidation (only after hydration)
  useEffect(() => {
    if (!isHydrated || typeof window === 'undefined' || !window.BroadcastChannel) return;

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
  }, [isHydrated, address, resetAuthState]);

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
    const initialState = useWeb3AuthStore.getState();
    console.log('🔄 [DEBUG] Starting comprehensive wallet disconnect...', {
      initialState: {
        isConnected: initialState.isConnected,
        isAuthenticated: initialState.isAuthenticated,
        walletAddress: initialState.walletAddress,
        hasInitialized: initialState.hasInitialized
      },
      wagmiState: {
        isConnected,
        address,
        connectorName: connector?.name
      },
      timestamp: new Date().toISOString()
    });

    try {
      // Step 1: Clear MetaMask-specific state if applicable with null checks
      if (connector?.name === 'MetaMask' && typeof window !== 'undefined' && window.ethereum) {
        try {
          console.log('🦊 Clearing MetaMask-specific state...');
          // Clear MetaMask permissions cache with null checks
          if (window.ethereum && typeof window.ethereum.request === 'function') {
            await window.ethereum.request({
              method: 'wallet_revokePermissions',
              params: [{ eth_accounts: {} }]
            });
            console.log('✅ MetaMask permissions cleared');
          }
        } catch (metaMaskError: any) {
          // Handle null reference errors gracefully
          if (metaMaskError?.message?.includes('Cannot set properties of null') || 
              metaMaskError?.message?.includes('Cannot read properties of null')) {
            console.log('🦊 MetaMask cleanup error (safe to ignore):', metaMaskError.message);
          } else {
            console.log('ℹ️ MetaMask permission revoke not supported:', metaMaskError);
          }
        }
      }

      // Step 1.5: Sequential wagmi storage clearing to avoid conflicts
      if (typeof window !== 'undefined') {
        console.log('🧹 Starting sequential wagmi storage clearing...');
        
        // Clear wagmi core storage
        const wagmiKeys = [
          'wagmi.cache',
          'wagmi.store', 
          'wagmi.recentConnector',
          'wagmi.recent-connectors',
          'wagmi.wallet',
          'wagmi.connected',
          'wagmi.accountIndex'
        ];

        for (const key of wagmiKeys) {
          try {
            window.localStorage.removeItem(key);
            console.log(`✅ Cleared ${key}`);
            // Small delay to prevent race conditions
            await new Promise(resolve => setTimeout(resolve, 10));
          } catch (error) {
            console.warn(`Failed to clear ${key}:`, error);
          }
        }

        // Clear RainbowKit storage
        const rainbowKeys = [
          'rk-recent',
          'rk-wallet-data', 
          'rainbow-wallet-data',
          'rainbowkit-recent-wallet'
        ];

        for (const key of rainbowKeys) {
          try {
            window.localStorage.removeItem(key);
            console.log(`✅ Cleared ${key}`);
            await new Promise(resolve => setTimeout(resolve, 10));
          } catch (error) {
            console.warn(`Failed to clear ${key}:`, error);
          }
        }

        console.log('✅ Sequential wagmi storage clearing completed');
      }

      // Step 2: Call store disconnect function first to handle backend cleanup
      await disconnect();

      // Step 3: Disconnect individual connector with enhanced clearing and null checks
      if (connector && typeof connector.disconnect === 'function') {
        try {
          console.log(`🔌 Disconnecting individual connector: ${connector.name}`);
          await connector.disconnect();
          console.log('✅ Individual connector disconnect completed');
          
          // Extra wait for MetaMask to properly clear state
          const waitTime = connector.name === 'MetaMask' ? 300 : 100;
          await new Promise(resolve => setTimeout(resolve, waitTime));
        } catch (connectorError: any) {
          // Handle null reference errors gracefully
          if (connectorError?.message?.includes('Cannot set properties of null') || 
              connectorError?.message?.includes('Cannot read properties of null')) {
            console.log('🔌 Connector cleanup error (safe to ignore):', connectorError.message);
          } else {
            console.error('❌ Individual connector disconnect failed:', connectorError);
          }
        }
      }

      // Step 4: Disconnect from Wagmi globally
      console.log('🔌 Disconnecting from Wagmi globally...');
      if (wagmiDisconnect && typeof wagmiDisconnect === 'function') {
        try {
          wagmiDisconnect();
          console.log('📞 Wagmi disconnect called');
          
          // Brief wait for state to sync
          await new Promise(resolve => setTimeout(resolve, 200));
          console.log('✅ Wagmi global disconnect completed');
        } catch (wagmiError) {
          console.error('❌ Wagmi disconnect failed:', wagmiError);
        }
      }

      // Final state check
      const finalState = useWeb3AuthStore.getState();
      console.log('✅ [DEBUG] Wallet disconnect completed successfully', {
        finalState: {
          isConnected: finalState.isConnected,
          isAuthenticated: finalState.isAuthenticated,
          walletAddress: finalState.walletAddress,
          hasInitialized: finalState.hasInitialized
        },
        wagmiState: {
          isConnected,
          address
        },
        timestamp: new Date().toISOString()
      });
      toast.success('Wallet disconnected successfully');
    } catch (error) {
      console.error('❌ Error during wallet disconnect:', error);
      // Reset state manually if disconnect fails
      resetAuthState();
      setConnected(false);
      setWalletAddress(undefined);
      toast.error('Wallet disconnected with errors');
    }
  }, [connector, wagmiDisconnect, disconnect, resetAuthState, setConnected, setWalletAddress]);

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