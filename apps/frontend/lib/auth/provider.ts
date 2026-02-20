'use client';

import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';
import { useAccount, useDisconnect, useSignMessage } from 'wagmi';
import { useWeb3Session } from './use-web3-session';
import { usePermissionSync, type Web3Permission } from './use-permission-sync';
import { useWalletAuth } from './use-wallet-auth';

export type { Web3Permission };

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

export function useWeb3Auth(): Web3AuthState & Web3AuthActions {
  const { address, isConnected, connector } = useAccount();
  const { disconnect: wagmiDisconnect } = useDisconnect();
  const { signMessageAsync } = useSignMessage();

  const [state, setState] = useState<Web3AuthState>({
    isConnected: false,
    isAuthenticated: false,
    isAuthenticating: false,
    permissions: [],
    userTier: 'free',
    hasApiAccess: false,
  });

  const [isHydrated, setIsHydrated] = useState(false);

  useEffect(() => {
    setIsHydrated(true);
  }, []);

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

          if (event.data.type === 'SESSION_INVALIDATED' && event.data.source === 'web3_disconnect') {
            const shouldInvalidate = event.data.walletAddress === null || event.data.walletAddress === undefined || event.data.walletAddress === address;
            if (shouldInvalidate) {
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

              try {
                window.localStorage.removeItem('oidc_session');
                window.sessionStorage.removeItem('oidc_session');
              } catch (_error) {
                // Intentionally empty
              }

              toast.info('Session was ended in another tab');
            }
          }
        } catch (_error) {
          // Intentionally empty
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
          // Intentionally empty
        }
      };
    } catch (_error) {
      return () => { };
    }
  }, [address]);

  const { checkAuthStatus, refreshPermissions } = usePermissionSync({ address, setState });

  useEffect(() => {
    if (!isHydrated) {return;}

    if (address && isConnected) {
      setState(prev => ({
        ...prev,
        isConnected: true,
        walletAddress: address,
        error: undefined,
      }));

      void (async () => {
        try {
          const response = await fetch('/api/auth/session', {
            credentials: 'include',
            cache: 'no-cache',
          });

          if (!response.ok) {
            if (response.status !== 401 && response.status !== 500) {
              // Handle other errors
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
            // Intentionally empty
          }
        } catch (_error) {
          setState(prev => ({
            ...prev,
            isConnected: true,
            isAuthenticated: false,
            walletAddress: address,
          }));
        }
      })();
    } else if (!address || !isConnected) {
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

  const { authenticate } = useWeb3Session({
    address,
    connector,
    signMessageAsync: signMessageAsync as ((...args: unknown[]) => Promise<string>) | undefined,
    isAuthenticating: state.isAuthenticating,
    refreshPermissions,
    setState
  });

  const { disconnect, linkEmail, generateApiKey, resetAuthState } = useWalletAuth({
    address,
    isConnected,
    connector,
    wagmiDisconnect,
    isAuthenticated: state.isAuthenticated,
    setState,
    refreshPermissions,
    hasApiAccess: state.hasApiAccess
  });

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

export { formatAddress } from '@/shared/auth/utils';
