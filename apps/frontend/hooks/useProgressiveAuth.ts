/**
 * Progressive Authentication Hook
 * Manages three-tier authentication state and provides helper functions
 */
'use client';

import { useMemo } from 'react';
import { useAccount } from 'wagmi';
import { useWeb3AuthContext } from '@/providers/Web3AuthProvider';
import { AuthLevel, type AuthState } from '@/types/progressive-auth';

export function useProgressiveAuth(): AuthState & {
  canAccess: (requiredLevel: AuthLevel) => boolean;
  getAuthMessage: (requiredLevel: AuthLevel, actionName?: string) => string;
  getUpgradeAction: (requiredLevel: AuthLevel) => string;
} {
  const { isConnected } = useAccount();
  const { isAuthenticated, walletAddress } = useWeb3AuthContext();

  // Determine current authentication level
  const currentLevel = useMemo(() => {
    if (isAuthenticated) return AuthLevel.AUTHENTICATED;
    if (isConnected && walletAddress) return AuthLevel.CONNECTED;
    return AuthLevel.PUBLIC;
  }, [isAuthenticated, isConnected, walletAddress]);

  // Check if user can access a feature requiring specific auth level
  const canAccess = (requiredLevel: AuthLevel): boolean => {
    const levelHierarchy = {
      [AuthLevel.PUBLIC]: 0,
      [AuthLevel.CONNECTED]: 1,
      [AuthLevel.AUTHENTICATED]: 2,
    };

    return levelHierarchy[currentLevel] >= levelHierarchy[requiredLevel];
  };

  // Get appropriate message for auth requirement
  const getAuthMessage = (requiredLevel: AuthLevel, actionName = 'access this feature'): string => {
    if (requiredLevel === AuthLevel.CONNECTED && currentLevel === AuthLevel.PUBLIC) {
      return `Connect your wallet to ${actionName}`;
    }
    
    if (requiredLevel === AuthLevel.AUTHENTICATED) {
      if (currentLevel === AuthLevel.PUBLIC) {
        return `Connect your wallet and sign in to ${actionName}`;
      }
      if (currentLevel === AuthLevel.CONNECTED) {
        return `Sign in with your wallet to ${actionName}`;
      }
    }
    
    return `Authentication required to ${actionName}`;
  };

  // Get the action needed to upgrade auth level
  const getUpgradeAction = (requiredLevel: AuthLevel): string => {
    if (requiredLevel === AuthLevel.CONNECTED && currentLevel === AuthLevel.PUBLIC) {
      return 'connect';
    }
    
    if (requiredLevel === AuthLevel.AUTHENTICATED) {
      if (currentLevel === AuthLevel.PUBLIC) {
        return 'connect_and_sign';
      }
      if (currentLevel === AuthLevel.CONNECTED) {
        return 'sign';
      }
    }
    
    return 'upgrade';
  };

  return {
    level: currentLevel,
    walletAddress,
    isAuthenticated,
    isWalletConnected: isConnected,
    canAccess,
    getAuthMessage,
    getUpgradeAction,
  };
}