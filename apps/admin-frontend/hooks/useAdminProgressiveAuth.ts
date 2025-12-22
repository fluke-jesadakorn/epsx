/**
 * Progressive Authentication Hook for Admin Frontend
 * Manages three-tier authentication state with admin permission validation
 */
'use client';

import { useSharedAuth } from '@/shared/components/auth/Provider';
import { AuthLevel, type AuthLevelType, type AuthState } from '@/types/progressive-auth';
import { useMemo } from 'react';
import { useAccount } from 'wagmi';

export function useAdminProgressiveAuth(): AuthState & {
  canAccess: (requiredLevel: AuthLevelType, requiredPermissions?: string[]) => boolean;
  getAuthMessage: (requiredLevel: AuthLevelType, actionName?: string) => string;
  getUpgradeAction: (requiredLevel: AuthLevelType) => string;
  hasPermission: (permission: string) => boolean;
  hasAnyPermission: (permissions: string[]) => boolean;
} {
  const { isConnected } = useAccount();
  const { isAuthenticated, getWalletAddress, getUserPermissions } = useSharedAuth();
  const walletAddress = getWalletAddress();
  const permissions = getUserPermissions();

  // Determine current authentication level
  const currentLevel = useMemo(() => {
    if (isAuthenticated && permissions && permissions.length > 0) return AuthLevel.AUTHENTICATED;
    if (isConnected && walletAddress) return AuthLevel.CONNECTED;
    return AuthLevel.PUBLIC;
  }, [isAuthenticated, isConnected, walletAddress, permissions]);

  // ============================================================================
  // DEPRECATED: Permission check functions
  // Backend handles all permission enforcement via JWT middleware
  // These are kept for backward compatibility but always return true
  // ============================================================================

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  const hasPermission = (_permission: string): boolean => {
    console.warn('[DEPRECATED] useAdminProgressiveAuth.hasPermission() - Permission enforcement moved to backend. This always returns true.');
    return true;
  };

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  const hasAnyPermission = (_permissions: string[]): boolean => {
    console.warn('[DEPRECATED] useAdminProgressiveAuth.hasAnyPermission() - Permission enforcement moved to backend. This always returns true.');
    return true;
  };

  // Check if user can access a feature requiring specific auth level and permissions
  const canAccess = (requiredLevel: AuthLevelType, requiredPermissions?: string[]): boolean => {
    const levelHierarchy = {
      [AuthLevel.PUBLIC]: 0,
      [AuthLevel.CONNECTED]: 1,
      [AuthLevel.AUTHENTICATED]: 2,
    };

    // Check auth level requirement
    const hasRequiredLevel = levelHierarchy[currentLevel] >= levelHierarchy[requiredLevel];
    if (!hasRequiredLevel) return false;

    // Check permission requirement
    if (requiredPermissions && requiredPermissions.length > 0) {
      return hasAnyPermission(requiredPermissions);
    }

    return true;
  };

  // Get appropriate message for auth requirement
  const getAuthMessage = (requiredLevel: AuthLevelType, actionName = 'access this feature'): string => {
    if (requiredLevel === AuthLevel.CONNECTED && currentLevel === AuthLevel.PUBLIC) {
      return `Connect your admin wallet to ${actionName}`;
    }

    if (requiredLevel === AuthLevel.AUTHENTICATED) {
      if (currentLevel === AuthLevel.PUBLIC) {
        return `Connect your admin wallet and authenticate to ${actionName}`;
      }
      if (currentLevel === AuthLevel.CONNECTED) {
        return `Sign in with your admin wallet to ${actionName}`;
      }
    }

    return `Admin authentication required to ${actionName}`;
  };

  // Get the action needed to upgrade auth level
  const getUpgradeAction = (requiredLevel: AuthLevelType): string => {
    if (requiredLevel === AuthLevel.CONNECTED && currentLevel === AuthLevel.PUBLIC) {
      return 'connect';
    }

    if (requiredLevel === AuthLevel.AUTHENTICATED) {
      if (currentLevel === AuthLevel.PUBLIC) {
        return 'connect_and_authenticate';
      }
      if (currentLevel === AuthLevel.CONNECTED) {
        return 'authenticate';
      }
    }

    return 'upgrade';
  };

  return {
    level: currentLevel,
    walletAddress: walletAddress ?? undefined,
    isAuthenticated,
    isWalletConnected: isConnected,
    adminPermissions: permissions || [],
    adminLevel: undefined, // Not provided by PureWeb3AuthProvider
    canAccess,
    getAuthMessage,
    getUpgradeAction,
    hasPermission,
    hasAnyPermission,
  };
}