'use client';

/**
 * SHARED OPENID + WEB3 AUTHENTICATION PROVIDER
 * Unified React provider for both frontend and admin-frontend
 */

import React, {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState
} from 'react';
import type {
  UnifiedApiResponse,
  UserInfoResponse
} from '../../auth/client';
import {
  SharedWeb3AuthClient
} from '../../auth/client';
import { logger } from '../../utils/logger';
import { useApiHelpers } from './hooks/use-api-helpers';
import { useAuthInitialization } from './hooks/use-auth-initialization';
import { useSessionActions } from './hooks/use-session-actions';
import { useWalletAuth } from './hooks/use-wallet-auth';

// Shared authentication context value
export interface SharedAuthContextValue {
  // Authentication state
  user: UserInfoResponse | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  isSigningChallenge: boolean;
  error: string | null;

  // Authentication actions
  requestChallenge: (
    walletAddress: string
  ) => Promise<{ nonce: string; message: string; wallet_address: string }>;
  authenticateWithWallet: (params: {
    walletAddress: string;
    signature: string;
    message: string;
    nonce: string;
  }) => Promise<{ success: boolean; user?: UserInfoResponse; error?: string }>;
  authenticateWithDirectApi: (result: {
    wallet_address: string;
    permissions: string[];
    tier_level?: string;
    is_new_user: boolean;
    access_token?: string;
  }) => Promise<void>;
  logout: () => Promise<void>;
  refreshUser: () => Promise<void>;
  refreshSession: () => Promise<boolean>;

  // Display helpers
  getWalletAddress: () => string | null;
  getUserTier: () => string;
  getUserPermissions: () => string[];

  makeApiRequest: (
    endpoint: string,
    options?: RequestInit
  ) => Promise<UnifiedApiResponse>;

  // Modal state management
  showSignInModal: boolean;
  openSignInModal: () => void;
  closeSignInModal: () => void;
}

// Create context
const SharedAuthContext = createContext<SharedAuthContextValue | undefined>(undefined);

// Provider props
interface SharedOpenIDWeb3ProviderProps {
  children: React.ReactNode;
  clientId?: string;
  backendUrl?: string;
  onAuthError?: (error: string) => void;
  initialUser?: UserInfoResponse | null;
}

// Provider component
export function SharedOpenIDWeb3Provider({
  children,
  clientId = 'epsx-frontend',
  backendUrl,
  onAuthError,
  initialUser = null,
}: SharedOpenIDWeb3ProviderProps) {
  const [isSigningChallenge, setIsSigningChallenge] = useState(false);
  const [showSignInModal, setShowSignInModal] = useState(false);

  const [client] = useState(() => {
    // Use centralized URL resolver as fallback when prop is empty/undefined
    // Bracket notation bypasses webpack build-time inlining for NEXT_PUBLIC_* vars
    const envBackendUrl = typeof window !== 'undefined'
      ? (process.env['NEXT_PUBLIC_BACKEND_URL'] ?? '')
      : (process.env['BACKEND_URL'] ?? process.env['NEXT_PUBLIC_BACKEND_URL'] ?? '');
    const resolvedBackendUrl =
      (backendUrl !== undefined && backendUrl !== '' ? backendUrl : null) ??
      (envBackendUrl !== '' ? envBackendUrl : null) ??
      (typeof window !== 'undefined'
        ? window.location.origin.replace(/:300[0-9]/, ':8080')
        : 'https://api.epsx.io');

    logger.info('[AUTH] Provider: Configuration', {
      provided: backendUrl,
      resolved: resolvedBackendUrl,
      clientId,
    });

    return new SharedWeb3AuthClient(clientId, resolvedBackendUrl);
  });

  const {
    user,
    setUser,
    isLoading,
    setIsLoading,
    error,
    setError
  } = useAuthInitialization({
    client,
    initialUser,
    clientId,
    onAuthError
  });

  const {
    requestChallenge,
    authenticateWithWallet,
    authenticateWithDirectApi,
  } = useWalletAuth({
    client,
    variant: clientId === 'epsx-admin' ? 'admin' : 'user',
    setUser,
    setIsLoading,
    setError,
    setIsSigningChallenge,
    onAuthError,
  });

  const { logout, refreshUser, refreshSession } = useSessionActions({
    client,
    clientId,
    setError,
    onAuthError,
  });

  const {
    getWalletAddress,
    getUserTier,
    getUserPermissions,
    makeApiRequest,
  } = useApiHelpers({ client });

  const openSignInModal = useCallback(() => setShowSignInModal(true), []);
  const closeSignInModal = useCallback(() => setShowSignInModal(false), []);

  const isAuthenticated = user !== null;

  const contextValue: SharedAuthContextValue = useMemo(() => ({
    user,
    isAuthenticated,
    isLoading,
    isSigningChallenge,
    error,
    requestChallenge,
    authenticateWithWallet,
    authenticateWithDirectApi,
    logout,
    refreshUser,
    refreshSession,
    getWalletAddress,
    getUserTier,
    getUserPermissions,
    makeApiRequest,
    showSignInModal,
    openSignInModal,
    closeSignInModal,
  }), [
    user,
    isAuthenticated,
    isLoading,
    isSigningChallenge,
    error,
    requestChallenge,
    authenticateWithWallet,
    authenticateWithDirectApi,
    logout,
    refreshUser,
    refreshSession,
    getWalletAddress,
    getUserTier,
    getUserPermissions,
    makeApiRequest,
    showSignInModal,
    openSignInModal,
    closeSignInModal,
  ]);

  return (
    <SharedAuthContext.Provider value={contextValue}>
      {children}
    </SharedAuthContext.Provider>
  );
}

export function useSharedAuth(): SharedAuthContextValue {
  const context = useContext(SharedAuthContext);
  if (!context) {
    throw new Error('useSharedAuth must be used within SharedOpenIDWeb3Provider');
  }
  return context;
}

export const useAuth = useSharedAuth;
