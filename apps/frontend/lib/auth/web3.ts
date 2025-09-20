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
}

export function useWeb3Auth(): Web3AuthState & Web3AuthActions {
  const { address, isConnected } = useAccount();
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

  // Auto-check auth status when wallet connects
  useEffect(() => {
    if (address && isConnected) {
      checkAuthStatus();
    } else {
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
  }, [address, isConnected]);

  const checkAuthStatus = useCallback(async () => {
    if (!address) return;

    try {
      const response = await fetch('/api/auth/session', {
        credentials: 'include',
      });

      if (response.ok) {
        const session = await response.json();
        if (session.wallet_address === address) {
          setState(prev => ({
            ...prev,
            isConnected: true,
            isAuthenticated: true,
            walletAddress: address,
          }));
          await refreshPermissions();
          return;
        }
      }

      // Not authenticated but connected
      setState(prev => ({
        ...prev,
        isConnected: true,
        isAuthenticated: false,
        walletAddress: address,
      }));
    } catch (error) {
      console.error('Failed to check auth status:', error);
      setState(prev => ({
        ...prev,
        isConnected: true,
        isAuthenticated: false,
        walletAddress: address,
        error: 'Failed to check authentication status',
      }));
    }
  }, [address]);

  const refreshPermissions = useCallback(async () => {
    if (!address) return;

    try {
      const response = await fetch('/api/auth/web3/permissions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: address }),
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
      }
    } catch (error) {
      console.error('Failed to fetch permissions:', error);
    }
  }, [address]);

  const authenticate = useCallback(async () => {
    if (!address) {
      toast.error('Please connect your wallet first');
      return;
    }

    setState(prev => ({ ...prev, isAuthenticating: true, error: undefined }));

    try {
      // Step 1: Get challenge nonce
      const challengeResponse = await fetch('/api/auth/web3/challenge', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: address }),
      });

      if (!challengeResponse.ok) {
        throw new Error('Failed to get authentication challenge');
      }

      const { nonce } = await challengeResponse.json();

      // Step 2: Create SIWE message
      const message = new SiweMessage({
        domain: window.location.host,
        address,
        statement: 'Sign in to EPSX - Web3 Trading Platform',
        uri: window.location.origin,
        version: '1',
        chainId: 1,
        nonce,
        issuedAt: new Date().toISOString(),
        expirationTime: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(), // 24 hours
      });

      const messageString = message.prepareMessage();

      // Step 3: Sign message with wallet
      const signature = await signMessageAsync({ message: messageString });

      // Step 4: Verify signature and establish session
      const verifyResponse = await fetch('/api/auth/web3/verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wallet_address: address,
          signature,
          nonce,
          message: messageString,
        }),
        credentials: 'include',
      });

      if (!verifyResponse.ok) {
        const error = await verifyResponse.json();
        throw new Error(error.message || 'Authentication failed');
      }

      // Success
      setState(prev => ({
        ...prev,
        isAuthenticated: true,
        isAuthenticating: false,
        walletAddress: address,
      }));

      await refreshPermissions();
      toast.success('Successfully authenticated with Web3 wallet');

    } catch (error: any) {
      console.error('Web3 authentication error:', error);
      const errorMessage = error.message || 'Authentication failed';
      setState(prev => ({
        ...prev,
        isAuthenticating: false,
        error: errorMessage,
      }));
      toast.error(errorMessage);
    }
  }, [address, signMessageAsync, refreshPermissions]);

  const disconnect = useCallback(async () => {
    try {
      // Logout from backend
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'include',
      });

      // Disconnect wallet
      wagmiDisconnect();

      setState({
        isConnected: false,
        isAuthenticated: false,
        isAuthenticating: false,
        permissions: [],
        userTier: 'free',
        hasApiAccess: false,
      });

      toast.success('Disconnected from Web3 wallet');
    } catch (error) {
      console.error('Disconnect error:', error);
      toast.error('Failed to disconnect properly');
    }
  }, [wagmiDisconnect]);

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

  return {
    ...state,
    authenticate,
    disconnect,
    checkAuthStatus,
    refreshPermissions,
    linkEmail,
    generateApiKey,
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