'use client';

import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { useWeb3Auth, type Web3AuthState, type Web3AuthActions } from '@/lib/auth/web3';
import { useAccount } from 'wagmi';
import { toast } from 'sonner';

interface Web3AuthContextType extends Web3AuthState, Web3AuthActions {
  isLoading: boolean;
  hasInitialized: boolean;
}

const Web3AuthContext = createContext<Web3AuthContextType | null>(null);

interface Web3AuthProviderProps {
  children: ReactNode;
}

export function Web3AuthProvider({ children }: Web3AuthProviderProps) {
  const { isConnected } = useAccount();
  const web3Auth = useWeb3Auth();
  const [isLoading, setIsLoading] = useState(true);
  const [hasInitialized, setHasInitialized] = useState(false);
  const [isInitializing, setIsInitializing] = useState(false);

  // Initialize authentication state on mount
  useEffect(() => {
    const initializeAuth = async () => {
      // Prevent multiple simultaneous initialization attempts
      if (isInitializing || hasInitialized) {
        console.log('Web3 auth already initializing or initialized, skipping');
        return;
      }

      try {
        setIsInitializing(true);
        setIsLoading(true);
        
        // If wallet is connected, check authentication status
        if (isConnected) {
          await web3Auth.checkAuthStatus();
        }
        
        setHasInitialized(true);
      } catch (error) {
        console.error('Failed to initialize Web3 auth:', error);
        toast.error('Failed to initialize authentication');
      } finally {
        setIsLoading(false);
        setIsInitializing(false);
      }
    };

    initializeAuth();
  }, [isConnected, isInitializing, hasInitialized]);

  // DISABLED: Auto-authentication to prevent conflicts and race conditions
  // Manual authentication only - users must explicitly sign in
  useEffect(() => {
    // Only check auth status for already authenticated users, don't auto-authenticate
    const handleAuthCheck = async () => {
      if (!hasInitialized || !isConnected) return;
      
      // Only check existing session status, don't trigger new authentication
      console.log('🔍 Checking existing Web3 authentication status (no auto-auth)');
      await web3Auth.checkAuthStatus();
    };

    handleAuthCheck();
  }, [isConnected, hasInitialized, web3Auth.checkAuthStatus]);

  const contextValue: Web3AuthContextType = {
    ...web3Auth,
    isLoading,
    hasInitialized,
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
            <div className="h-8 w-8 bg-orange-500 rounded animate-pulse" />
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

// Hook for checking specific permissions
export function useWeb3Permission(permission: string): boolean {
  const { permissions, isAuthenticated } = useWeb3AuthContext();
  
  if (!isAuthenticated) return false;
  
  return permissions.some(p => 
    p.permission === permission || 
    p.permission.includes('*') ||
    permission.startsWith(p.permission.replace('*', ''))
  );
}

// Hook for checking tier requirements
export function useWeb3Tier(requiredTier: 'nft' | 'token' | 'dao' | 'enterprise'): boolean {
  const { userTier, isAuthenticated } = useWeb3AuthContext();
  
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