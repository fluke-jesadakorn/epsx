'use client';

import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { useWeb3Auth } from '@/lib/auth/use-web3-auth';
import { useAccount } from 'wagmi';
import { toast } from 'sonner';

interface Web3AuthContextType {
  isConnected: boolean;
  isAuthenticated: boolean;
  isAuthenticating: boolean;
  isLoading: boolean;
  hasInitialized: boolean;
  walletAddress?: string;
  permissions: any[];
  userTier: 'free' | 'nft' | 'token' | 'dao' | 'enterprise';
  hasApiAccess: boolean;
  error?: string;
  authenticate: () => Promise<void>;
  disconnect: () => Promise<void>;
  checkAuthStatus: () => Promise<boolean | undefined>;
  refreshPermissions: () => Promise<boolean>;
  linkEmail: (email: string, password: string) => Promise<void>;
  generateApiKey: (name: string) => Promise<string>;
  resetAuthState: () => void;
}

const Web3AuthContext = createContext<Web3AuthContextType | null>(null);

interface Web3AuthProviderProps {
  children: ReactNode;
}

export function Web3AuthProvider({ children }: Web3AuthProviderProps) {
  const { isConnected } = useAccount();
  const web3Auth = useWeb3Auth();

  // Initialize authentication when wallet connects
  useEffect(() => {
    const initializeAuth = async () => {
      if (!isConnected) {
        console.log('🔄 Wallet disconnected, Web3 auth state managed by store');
        return;
      }

      if (web3Auth.hasInitialized) {
        console.log('Web3 auth already initialized');
        return;
      }

      try {
        console.log('🚀 Initializing Web3 auth for connected wallet');
        await web3Auth.checkAuthStatus();
        console.log('✅ Web3 auth initialization completed');
      } catch (error) {
        console.error('❌ Failed to initialize Web3 auth:', error);
        toast.error('Failed to initialize authentication');
      }
    };

    initializeAuth();
  }, [isConnected, web3Auth.hasInitialized, web3Auth.checkAuthStatus]);

  const contextValue: Web3AuthContextType = {
    isConnected: web3Auth.isConnected,
    isAuthenticated: web3Auth.isAuthenticated,
    isAuthenticating: web3Auth.isAuthenticating,
    isLoading: web3Auth.isLoading,
    hasInitialized: web3Auth.hasInitialized,
    walletAddress: web3Auth.walletAddress,
    permissions: web3Auth.permissions,
    userTier: web3Auth.userTier,
    hasApiAccess: web3Auth.hasApiAccess,
    error: web3Auth.error,
    authenticate: web3Auth.authenticate,
    disconnect: web3Auth.disconnect,
    checkAuthStatus: web3Auth.checkAuthStatus,
    refreshPermissions: web3Auth.refreshPermissions,
    linkEmail: web3Auth.linkEmail,
    generateApiKey: web3Auth.generateApiKey,
    resetAuthState: web3Auth.resetAuthState,
  };

  return (
    <Web3AuthContext.Provider value={contextValue}>
      {children}
    </Web3AuthContext.Provider>
  );
}

export function useWeb3AuthContext(): Web3AuthContextType {
  const context = useContext(Web3AuthContext);
  if (!context) {
    throw new Error('useWeb3AuthContext must be used within Web3AuthProvider');
  }
  return context;
}

// Higher-order component for pages that require Web3 authentication
export function withWeb3Auth<P extends object>(
  Component: React.ComponentType<P>,
  options: {
    requireAuth?: boolean;
    requireTier?: 'nft' | 'token' | 'dao' | 'enterprise';
    fallbackComponent?: React.ComponentType;
  } = {}
) {
  const { requireAuth = false, requireTier, fallbackComponent: Fallback } = options;

  return function WrappedComponent(props: P) {
    const { isAuthenticated, userTier, isLoading, hasInitialized } = useWeb3AuthContext();

    // Show loading while initializing
    if (!hasInitialized || isLoading) {
      return (
        <div className="flex items-center justify-center min-h-[200px]">
          <div className="flex flex-col items-center gap-3">
            <div className="h-8 w-8 bg-orange-500 rounded" />
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Initializing Web3 authentication...
            </p>
          </div>
        </div>
      );
    }

    // Check authentication requirement
    if (requireAuth && !isAuthenticated) {
      if (Fallback) {
        return <Fallback />;
      }
      return (
        <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
          <h2 className="text-xl font-semibold text-slate-900 dark:text-slate-100">
            Web3 Authentication Required
          </h2>
          <p className="text-slate-600 dark:text-slate-400 text-center max-w-md">
            Please connect your wallet and sign in to access this feature.
          </p>
        </div>
      );
    }

    // Check tier requirement
    if (requireTier && isAuthenticated) {
      const tierHierarchy = {
        nft: 1,
        token: 2,
        dao: 3,
        enterprise: 4,
      };

      const userLevel = tierHierarchy[userTier as keyof typeof tierHierarchy] || 0;
      const requiredLevel = tierHierarchy[requireTier];

      if (userLevel < requiredLevel) {
        if (Fallback) {
          return <Fallback />;
        }
        return (
          <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
            <h2 className="text-xl font-semibold text-slate-900 dark:text-slate-100">
              Higher Tier Required
            </h2>
            <p className="text-slate-600 dark:text-slate-400 text-center max-w-md">
              This feature requires {requireTier} tier access or higher. Your current tier: {userTier}.
            </p>
          </div>
        );
      }
    }

    return <Component {...props} />;
  };
}

// Re-export hooks from the store-based implementation
export { useWeb3Permission, useWeb3Tier } from '@/lib/auth/use-web3-auth';