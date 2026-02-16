'use client';

import { useRouter } from 'next/navigation';
import { useCallback } from 'react';
import { toast } from 'sonner';
import type { Connector } from 'wagmi';

interface UseWalletAuthContext {
  address: string | undefined;
  isConnected: boolean;
  connector: Connector | undefined;
  wagmiDisconnect: () => void;
  isAuthenticated: boolean;
  setState: React.Dispatch<React.SetStateAction<{
    isConnected: boolean;
    isAuthenticated: boolean;
    isAuthenticating: boolean;
    walletAddress?: string;
    permissions: unknown[];
    userTier: 'free' | 'nft' | 'token' | 'dao' | 'enterprise';
    hasApiAccess: boolean;
    error?: string;
  }>>;
  refreshPermissions: () => Promise<boolean>;
  hasApiAccess: boolean;
}

export function useWalletAuth(ctx: UseWalletAuthContext) {
  const router = useRouter();
  const {
    address,
    isConnected,
    connector,
    wagmiDisconnect,
    isAuthenticated,
    setState,
    refreshPermissions,
    hasApiAccess
  } = ctx;

  const disconnect = useCallback(async () => {
    try {
      if (connector && typeof connector.disconnect === 'function') {
        try {
          await connector.disconnect();
          await new Promise(resolve => setTimeout(resolve, 150));
        } catch (_connectorError) {
          // Continue
        }
      }

      if (wagmiDisconnect && typeof wagmiDisconnect === 'function') {
        try {
          wagmiDisconnect();

          let attempts = 0;
          const maxAttempts = 10;
          while (attempts < maxAttempts) {
            await new Promise(resolve => setTimeout(resolve, 100));
            attempts++;

            if (attempts >= 5) {break;}
          }
        } catch (_wagmiError) {
          // Continue
        }
      }

      try {
        if (typeof window !== 'undefined') {
          const allKeys = [];
          for (let i = 0; i < window.localStorage.length; i++) {
            const key = window.localStorage.key(i);
            if (key !== null) {allKeys.push(key);}
          }

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

          keysToRemove.forEach(key => {
            try {
              window.localStorage.removeItem(key);
            } catch (_e) {
              // Intentionally empty
            }
          });

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
              // Intentionally empty
            }
          });
        }
      } catch (_storageError) {
        // Intentionally empty
      }

      if (isAuthenticated && address) {
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
          // Continue
        }
      }

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

      router.refresh();
    } catch (_error) {
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

      if (typeof window !== 'undefined') {
        try {
          window.localStorage.removeItem('wagmi.cache');
          window.localStorage.removeItem('wagmi.store');
          window.localStorage.removeItem('wagmi.recentConnector');
        } catch (_storageErr) {
          // Intentionally empty
        }
      }

      toast.error('Wallet disconnected with some errors - refreshing page...');

      router.refresh();
    }
  }, [wagmiDisconnect, isAuthenticated, address, isConnected, connector, router, setState]);

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
        throw new Error(error.message ?? 'Failed to link email');
      }

      toast.success('Email linked successfully');
      await refreshPermissions();
    },
    [address, refreshPermissions]
  );

  const generateApiKey = useCallback(
    async (name: string): Promise<string> => {
      if (!address || !hasApiAccess) {
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
        throw new Error(error.message ?? 'Failed to generate API key');
      }

      const { api_key } = await response.json();
      toast.success('API key generated successfully');

      return api_key;
    },
    [address, hasApiAccess]
  );

  const resetAuthState = useCallback(() => {
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
  }, [address, setState]);

  return { disconnect, linkEmail, generateApiKey, resetAuthState };
}
