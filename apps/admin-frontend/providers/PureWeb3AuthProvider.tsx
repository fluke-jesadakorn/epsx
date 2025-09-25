/**
 * Pure Web3 Authentication Provider for Admin Frontend
 * Integrates with wagmi/wallet connection and provides pure Web3 auth to the admin app
 * No sessions, no cookies - pure signature-based authentication with admin permissions
 */

'use client';

import React, { createContext, useContext, useEffect, useCallback } from 'react';
import { useAccount, useSignMessage, useChainId, useDisconnect } from 'wagmi';
import { toast } from 'react-hot-toast';
import { usePureWeb3Auth, usePureWeb3AuthStore } from '@/lib/auth/pure-web3-service';

// Context for Pure Web3 Auth
interface PureWeb3AuthContextValue {
  // Connection state
  isWalletConnected: boolean;
  isAuthenticated: boolean;
  isAuthenticating: boolean;
  isLoading: boolean;
  error?: string;
  
  // Wallet info
  walletAddress?: string;
  chainId: number;
  
  // Permissions
  permissions: string[];
  hasPermission: (permission: string) => boolean;
  hasAnyPermission: (permissions: string[]) => boolean;
  hasAllPermissions: (permissions: string[]) => boolean;
  isAdmin: () => boolean;
  
  // Actions
  authenticate: () => Promise<void>;
  refreshAuth: () => Promise<void>;
  disconnect: () => Promise<void>;
  
  // API access
  api: any; // PureWeb3ApiClient
}

const PureWeb3AuthContext = createContext<PureWeb3AuthContextValue | null>(null);

// Provider component
export function PureWeb3AuthProvider({ children }: { children: React.ReactNode }) {
  // Wagmi hooks
  const { address, isConnected: isWalletConnected } = useAccount();
  const { signMessageAsync } = useSignMessage();
  const chainId = useChainId();
  const { disconnect: disconnectWallet } = useDisconnect();
  
  // Pure Web3 auth store
  const auth = usePureWeb3Auth();

  // Inject signing function into global scope for the service
  useEffect(() => {
    if (typeof window !== 'undefined') {
      window.__pureWeb3_signMessage = signMessageAsync;
      window.__pureWeb3_getAccount = () => {
        if (address && chainId) {
          return { address, chainId };
        }
        return null;
      };
    }
    
    return () => {
      if (typeof window !== 'undefined') {
        delete window.__pureWeb3_signMessage;
        delete window.__pureWeb3_getAccount;
      }
    };
  }, [signMessageAsync, address, chainId]);

  // Handle wallet connection changes
  useEffect(() => {
    if (isWalletConnected && address && chainId && !auth.isConnected) {
      // Wallet connected, set up pure Web3 auth
      auth.connect(address, chainId);
    } else if (!isWalletConnected && auth.isConnected) {
      // Wallet disconnected, clear auth
      auth.disconnect();
    }
  }, [isWalletConnected, address, chainId, auth.isConnected]);

  // Update chain ID when it changes
  useEffect(() => {
    if (auth.isConnected && chainId && chainId !== auth.chainId) {
      auth.setConnected(true, auth.walletAddress, chainId);
    }
  }, [chainId, auth.chainId, auth.isConnected]);

  // Authenticate with backend (verify signature)
  const authenticate = useCallback(async () => {
    if (!isWalletConnected || !address) {
      toast.error('Please connect your wallet first');
      return;
    }

    if (auth.isAuthenticating) {
      toast('Authentication already in progress');
      return;
    }

    try {
      await auth.verifyConnection();
      
      // Check if user has admin permissions
      if (auth.permissions.some(p => p.startsWith('admin:'))) {
        toast.success('Successfully authenticated as admin');
      } else {
        toast.error('Admin permissions required for this application');
        return;
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Authentication failed';
      toast.error(errorMsg);
      console.error('Pure Web3 admin authentication failed:', error);
    }
  }, [isWalletConnected, address, auth]);

  // Refresh authentication (re-verify and fetch permissions)
  const refreshAuth = useCallback(async () => {
    if (!auth.isConnected) {
      return;
    }

    try {
      await auth.verifyConnection();
      await auth.refreshPermissions();
    } catch (error) {
      console.warn('Auth refresh failed:', error);
    }
  }, [auth.isConnected]);

  // Disconnect wallet and clear auth
  const disconnect = useCallback(async () => {
    try {
      await auth.signOut();
      disconnectWallet();
      toast.success('Wallet disconnected');
    } catch (error) {
      console.error('Disconnect failed:', error);
      toast.error('Failed to disconnect cleanly');
    }
  }, [auth, disconnectWallet]);

  // Auto-authenticate on wallet connection (if not already authenticated)
  useEffect(() => {
    let timeoutId: NodeJS.Timeout;
    
    if (isWalletConnected && address && auth.isReady && !auth.isAuthorized && !auth.isAuthenticating) {
      // Auto-authenticate after a short delay
      timeoutId = setTimeout(() => {
        authenticate();
      }, 1000);
    }

    return () => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    };
  }, [isWalletConnected, address, auth.isReady, auth.isAuthorized, auth.isAuthenticating, authenticate]);

  // Monitor for admin permission changes and show notifications
  useEffect(() => {
    if (auth.permissions.length > 0) {
      const adminPermissions = auth.permissions.filter(p => p.startsWith('admin:'));
      
      console.log('Pure Web3 Admin Auth - Permissions loaded:', {
        total: auth.permissions.length,
        admin: adminPermissions.length,
        wallet: auth.walletAddress
      });

      // Warn if no admin permissions
      if (adminPermissions.length === 0) {
        toast.error('No admin permissions found. Contact system administrator.');
      }
    }
  }, [auth.permissions, auth.walletAddress]);

  const contextValue: PureWeb3AuthContextValue = {
    // Connection state
    isWalletConnected,
    isAuthenticated: auth.isAuthorized && auth.permissions.some(p => p.startsWith('admin:')),
    isAuthenticating: auth.isAuthenticating,
    isLoading: auth.isLoading,
    error: auth.error,
    
    // Wallet info
    walletAddress: address,
    chainId: chainId || 1,
    
    // Permissions
    permissions: auth.permissions,
    hasPermission: auth.hasPermission,
    hasAnyPermission: auth.hasAnyPermission, 
    hasAllPermissions: auth.hasAllPermissions,
    isAdmin: auth.isAdmin,
    
    // Actions
    authenticate,
    refreshAuth,
    disconnect,
    
    // API access
    api: auth.api,
  };

  return (
    <PureWeb3AuthContext.Provider value={contextValue}>
      {children}
    </PureWeb3AuthContext.Provider>
  );
}

// Hook to use Pure Web3 Auth context
export function usePureWeb3AuthContext(): PureWeb3AuthContextValue {
  const context = useContext(PureWeb3AuthContext);
  if (!context) {
    throw new Error('usePureWeb3AuthContext must be used within PureWeb3AuthProvider');
  }
  return context;
}

// Admin permission guard component
interface PureWeb3AdminGuardProps {
  children: React.ReactNode;
  permission?: string;
  anyPermissions?: string[];
  allPermissions?: string[];
  requireSuperAdmin?: boolean;
  fallback?: React.ReactNode;
}

export function PureWeb3AdminGuard({ 
  children, 
  permission,
  anyPermissions,
  allPermissions,
  requireSuperAdmin = false,
  fallback = <div className="text-center text-red-500 py-8">Access denied - insufficient admin permissions</div>
}: PureWeb3AdminGuardProps) {
  const { hasPermission, hasAnyPermission, hasAllPermissions, isAuthenticated, permissions } = usePureWeb3AuthContext();

  if (!isAuthenticated) {
    return <div className="text-center text-gray-500 py-8">Please authenticate with your admin wallet</div>;
  }

  // Check for basic admin access
  if (!permissions.some(p => p.startsWith('admin:'))) {
    return <div className="text-center text-red-500 py-8">Admin permissions required</div>;
  }

  let hasAccess = true;

  if (requireSuperAdmin && !hasPermission('admin:super:all')) {
    hasAccess = false;
  }

  if (permission && !hasPermission(permission)) {
    hasAccess = false;
  }

  if (anyPermissions && !hasAnyPermission(anyPermissions)) {
    hasAccess = false;
  }

  if (allPermissions && !hasAllPermissions(allPermissions)) {
    hasAccess = false;
  }

  if (!hasAccess) {
    return <>{fallback}</>;
  }

  return <>{children}</>;
}

// Admin authentication status component
export function PureWeb3AdminAuthStatus() {
  const { 
    isWalletConnected, 
    isAuthenticated, 
    isAuthenticating, 
    walletAddress, 
    chainId, 
    permissions,
    error,
    authenticate 
  } = usePureWeb3AuthContext();

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <h3 className="text-red-800 font-medium">Admin Authentication Error</h3>
        <p className="text-red-600 text-sm mt-1">{error}</p>
        <button 
          onClick={authenticate}
          className="mt-2 text-sm bg-red-600 text-white px-3 py-1 rounded hover:bg-red-700"
        >
          Retry Authentication
        </button>
      </div>
    );
  }

  if (!isWalletConnected) {
    return (
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
        <h3 className="text-yellow-800 font-medium">Admin Wallet Not Connected</h3>
        <p className="text-yellow-600 text-sm mt-1">Please connect your admin wallet to continue</p>
      </div>
    );
  }

  if (isAuthenticating) {
    return (
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h3 className="text-blue-800 font-medium">Authenticating Admin Access...</h3>
        <p className="text-blue-600 text-sm mt-1">Please sign the message in your wallet</p>
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <div className="bg-orange-50 border border-orange-200 rounded-lg p-4">
        <h3 className="text-orange-800 font-medium">Admin Authentication Required</h3>
        <p className="text-orange-600 text-sm mt-1">Click to authenticate with your admin wallet</p>
        <button 
          onClick={authenticate}
          className="mt-2 text-sm bg-orange-600 text-white px-3 py-1 rounded hover:bg-orange-700"
        >
          Authenticate Admin Wallet
        </button>
      </div>
    );
  }

  const adminPermissions = permissions.filter(p => p.startsWith('admin:'));

  return (
    <div className="bg-green-50 border border-green-200 rounded-lg p-4">
      <h3 className="text-green-800 font-medium">Admin Wallet Authenticated</h3>
      <div className="text-green-600 text-sm mt-1 space-y-1">
        <p>Address: {walletAddress?.slice(0, 8)}...{walletAddress?.slice(-6)}</p>
        <p>Chain ID: {chainId}</p>
        <p>Admin Permissions: {adminPermissions.length}</p>
      </div>
    </div>
  );
}