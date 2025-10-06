'use client';

import React from 'react';
import { SharedOpenIDWeb3Provider, useSharedAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider';
import { createFrontendClient } from '@/shared/auth/openid-web3-client';

// Create frontend client instance
const frontendClient = createFrontendClient();

// Web3 Auth Provider Component
// Now uses the shared OpenID + Web3 authentication system
export function Web3AuthProvider({ children }: { children: React.ReactNode }) {
  return (
    <SharedOpenIDWeb3Provider 
      clientId="epsx-frontend"
      onAuthError={(error) => {
        console.error('Frontend auth error:', error);
      }}
    >
      {children}
    </SharedOpenIDWeb3Provider>
  );
}

// Legacy compatibility hooks that wrap the shared authentication
export function useWeb3Auth() {
  const auth = useSharedAuth();
  
  return {
    // Map shared auth to legacy interface for backward compatibility
    authState: {
      walletAddress: auth.user?.wallet_address || null,
      isConnected: auth.isAuthenticated,
      isAuthenticated: auth.isAuthenticated,
      isLoading: auth.isLoading,
      user: auth.user ? {
        wallet_address: auth.user.wallet_address,
        user_id: auth.user.sub,
        permissions: auth.user.permissions,
        tier: auth.user.tier_level
      } : null,
      permissions: auth.user?.permissions || [],
      userTier: auth.user?.tier_level || null,
      error: auth.error,
      expiresAt: null, // Not used in new system
      permissionInfo: [], // Not used in new system
      groupMemberships: [], // Not used in new system
      web3Permissions: [], // Not used in new system
      
      // Legacy action methods (mapped to new system)
      connect: async () => {
        // Wallet connection handled by WAGMI/RainbowKit
      },
      disconnect: auth.logout,
      authenticate: async () => {
        // Auto-triggered in new system
        return { success: true };
      },
      logout: auth.logout,
      can: auth.hasPermissionForDisplay,
      hasPermission: auth.hasPermissionForDisplay,
      hasAnyPermission: (permissions: string[]) => permissions.some(auth.hasPermissionForDisplay),
      hasAllPermissions: (permissions: string[]) => permissions.every(auth.hasPermissionForDisplay),
      refreshPermissions: auth.refreshUser,
      getPermissionStats: async () => ({ total_permissions: 0, permanent_permissions: 0, temporary_permissions: 0, expired_permissions: 0 }),
      getGroupMemberships: async () => [],
      checkPermissions: async () => ({})
    },
    walletState: {
      isConnecting: false,
      isReconnecting: false,
      connector: undefined,
      chain: undefined,
      error: undefined
    },
    connectWallet: async () => {},
    disconnectWallet: auth.logout,
    authenticate: async () => ({ success: true }),
    logout: auth.logout,
    checkPermission: async (permission: string) => auth.hasPermissionForDisplay(permission),
    checkPermissions: async () => ({}),
    refreshPermissions: auth.refreshUser,
    getGroupMemberships: async () => [],
    getPermissionStats: async () => ({ total_permissions: 0, permanent_permissions: 0, temporary_permissions: 0, expired_permissions: 0 }),
    hasPermission: auth.hasPermissionForDisplay,
    isAdmin: () => auth.hasPermissionForDisplay('admin:*:*'),
    getCurrentTier: () => auth.user?.tier_level || 'free'
  };
}

// Legacy convenience hooks for backward compatibility
export function useWeb3AuthState() {
  const { authState } = useWeb3Auth();
  return authState;
}

export function useWeb3WalletState() {
  const { walletState } = useWeb3Auth();
  return walletState;
}

export function useWeb3Permission(permission: string): boolean {
  const { hasPermissionForDisplay } = useSharedAuth();
  return hasPermissionForDisplay(permission);
}

export function useWeb3Admin(): boolean {
  const { hasPermissionForDisplay } = useSharedAuth();
  return hasPermissionForDisplay('admin:*:*');
}

// Re-export shared authentication hooks for modern usage
export { useSharedAuth, useSharedAuth as useAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider';