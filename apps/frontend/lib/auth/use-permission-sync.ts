'use client';

import { useCallback } from 'react';

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

function isSessionData(data: unknown): data is SessionData {
  return typeof data === 'object' && data !== null && !Array.isArray(data);
}

interface UsePermissionSyncContext {
  address: string | undefined;
  setState: React.Dispatch<React.SetStateAction<{
    isConnected: boolean;
    isAuthenticated: boolean;
    isAuthenticating: boolean;
    walletAddress?: string;
    permissions: Web3Permission[];
    userTier: 'free' | 'nft' | 'token' | 'dao' | 'enterprise';
    hasApiAccess: boolean;
    error?: string;
  }>>;
}

export function usePermissionSync(ctx: UsePermissionSyncContext) {
  const { address, setState } = ctx;

  const checkAuthStatus = useCallback(async () => {
    if (address === null || address === undefined) {
      return;
    }

    try {
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

      setState(prev => ({
        ...prev,
        isAuthenticated: false,
        error: undefined,
      }));
      return false;
    } catch (_error) {
      return false;
    }
  }, [address, setState]);

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
      setState(prev => ({
        ...prev,
        permissions: [],
        userTier: 'free',
        hasApiAccess: false,
      }));
      return false;
    }
  }, [address, setState]);

  return { checkAuthStatus, refreshPermissions };
}
