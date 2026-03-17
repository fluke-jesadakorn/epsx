'use client';

import { useCallback } from 'react';
import { toast } from 'sonner';
import type { Connector } from 'wagmi';
import type { Web3Permission } from './use-permission-sync';

interface EthProvider {
  request?: (args: { method: string; params?: unknown[] }) => Promise<unknown>;
}

function isEthProvider(data: unknown): data is EthProvider {
  return typeof data === 'object' && data !== null && !Array.isArray(data);
}

function isValidAccountList(data: unknown): data is string[] {
  return Array.isArray(data) && data.length > 0 && typeof data[0] === 'string';
}

const validateWalletAccess = async (provider: EthProvider, walletAddress: string): Promise<string[]> => {
  const accounts = await provider.request?.({
    method: 'eth_requestAccounts',
  });

  if (!isValidAccountList(accounts)) {
    throw new Error('No accounts returned from wallet');
  }

  const normalizedAddress = walletAddress.toLowerCase();
  const hasMatchingAccount = accounts.some(
    (acc: string) => acc.toLowerCase() === normalizedAddress
  );

  if (!hasMatchingAccount) {
    throw new Error('Connected wallet address not found in authorized accounts');
  }

  return accounts;
};

const handleWalletAuthError = (error: unknown): string => {
  const err = error as { code?: number; message?: string };
  if (err.code === 4001) {
    return 'Wallet access denied by user';
  }
  if (err.code === 4100) {
    return 'Wallet not authorized - please connect your wallet first';
  }
  if (typeof err.message === 'string' && err.message.includes('User rejected')) {
    return 'Wallet access was rejected';
  }
  return `Wallet authorization failed: ${err.message ?? 'Unknown error'}`;
};

interface UseWeb3SessionContext {
  address: string | undefined;
  connector: Connector | undefined;
  signMessageAsync: ((...args: unknown[]) => Promise<string>) | undefined;
  isAuthenticating: boolean;
  refreshPermissions: () => Promise<boolean>;
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

export function useWeb3Session(ctx: UseWeb3SessionContext) {
  const { address, connector, signMessageAsync, isAuthenticating, refreshPermissions, setState } = ctx;

  const authenticate = useCallback(async () => {
    if (address === null || address === undefined) {
      toast.error('Please connect your wallet first');
      return;
    }

    if (!connector) {
      toast.error('Wallet connector not found. Please reconnect your wallet.');
      return;
    }

    if (connector.ready === false) {
      toast.error('Wallet is not ready. Please check your wallet connection.');
      return;
    }

    try {
      const provider = await connector.getProvider?.();
      if (!isEthProvider(provider)) {
        throw new Error('Wallet provider not available');
      }

      try {
        await validateWalletAccess(provider, address);
      } catch (authError: unknown) {
        const errorMessage = handleWalletAuthError(authError);
        throw new Error(errorMessage, { cause: authError });
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

    if (isAuthenticating) {
      toast.error('Authentication already in progress. Please wait.');
      return;
    }

    setState(prev => ({ ...prev, isAuthenticating: true, error: undefined }));

    try {
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

      if (!signMessageAsync) {
        throw new Error(
          'Wallet signing function not available. Please reconnect your wallet.'
        );
      }

      let signature: string;
      try {
        signature = await signMessageAsync({ message: messageString, account: address as `0x${string}` });
      } catch (error: unknown) {
        const err = error as { code?: number; message?: string };

        if (
          err.code === 4001 ||
          err.message?.includes('User rejected') ||
          err.message?.includes('User denied')
        ) {
          throw new Error('Signature was cancelled by user', { cause: error });
        } else if (err.message?.includes('Method not found') ?? false) {
          throw new Error('Wallet does not support message signing', { cause: error });
        } else if (err.message?.includes('Connection lost') ?? false) {
          throw new Error('Wallet connection lost - please reconnect', { cause: error });
        } else {
          throw new Error(
            `Wallet signing failed: ${err.message ?? 'Unknown wallet error'}`,
            { cause: error }
          );
        }
      }

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

        throw new Error(errorData.error ?? 'Authentication failed');
      }

      const authData = await authResponse.json() as { access_token?: string; refresh_token?: string; user?: Record<string, unknown> };

      const { loginAction } = await import('@/shared/auth/actions');
      if (authData.access_token !== undefined && authData.access_token !== '') {
        await loginAction(authData.access_token, authData.user ?? {}, authData.refresh_token);
      }

      setState(prev => ({
        ...prev,
        isAuthenticated: true,
        isAuthenticating: false,
        walletAddress: address,
      }));

      if (typeof window !== 'undefined') {
        try {
          window.localStorage.setItem('oidc_session', '1');
          document.cookie = 'oidc_session=1; path=/; SameSite=Lax';
        } catch (_error) {
          // Intentionally empty
        }
      }

      await refreshPermissions();
      toast.success('Successfully authenticated with Web3 wallet');
    } catch (error: unknown) {
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
  }, [address, signMessageAsync, refreshPermissions, connector, isAuthenticating, setState]);

  return { authenticate };
}
