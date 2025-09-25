'use client';

import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { useAccount, useConnect, useDisconnect } from 'wagmi';
import { toast } from 'react-hot-toast';
import { usePermissionCache } from '@/lib/auth/permission-cache-service';

interface Web3AuthContextType {
  isAuthenticated: boolean;
  isLoading: boolean;
  walletAddress?: string;
  permissions?: string[];
  adminLevel?: 'none' | 'moderator' | 'manager' | 'super';
  hasAdminAccess?: boolean;
  login: () => Promise<void>;
  logout: () => Promise<void>;
  refreshSession: () => Promise<void>;
}

const Web3AuthContext = createContext<Web3AuthContextType | undefined>(undefined);

export const useWeb3Auth = () => {
  const context = useContext(Web3AuthContext);
  if (context === undefined) {
    throw new Error('useWeb3Auth must be used within a Web3AuthProvider');
  }
  return context;
};

interface Web3AuthProviderProps {
  children: ReactNode;
}

export function Web3AuthProvider({ children }: Web3AuthProviderProps) {
  const { address, isConnected } = useAccount();
  const { connect, connectors } = useConnect();
  const { disconnect } = useDisconnect();
  const { getPermissions, getCachedPermissions, clearCache } = usePermissionCache();
  
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [permissions, setPermissions] = useState<string[]>([]);
  const [adminLevel, setAdminLevel] = useState<'none' | 'moderator' | 'manager' | 'super'>('none');
  const [hasAdminAccess, setHasAdminAccess] = useState(false);

  // Check authentication when wallet connection changes
  useEffect(() => {
    if (isConnected && address) {
      verifyWalletSession(address);
    } else {
      setIsAuthenticated(false);
      setPermissions([]);
      setAdminLevel('none');
      setHasAdminAccess(false);
      setIsLoading(false);
    }
  }, [isConnected, address]);

  // Verify wallet session and permissions
  const verifyWalletSession = async (walletAddress: string) => {
    setIsLoading(true);
    try {
      console.log('🔍 Web3 Auth: Verifying session for wallet:', walletAddress);

      // Use cache service to prevent heavy loops
      const cached = getCachedPermissions(walletAddress);
      if (cached) {
        console.log('🚀 Web3 Auth: Using cached permissions for', walletAddress.slice(0, 6));
        updateAuthState(cached.permissions, cached.admin_level, cached.has_admin_access, walletAddress);
        setIsLoading(false);
        return;
      }

      // Fetch fresh permissions using cache service
      const data = await getPermissions(walletAddress);
      if (data) {
        updateAuthState(data.permissions, data.admin_level, data.has_admin_access, walletAddress);
      } else {
        console.warn('⚠️ Web3 Auth: Permission check failed');
        setIsAuthenticated(false);
        setHasAdminAccess(false);
      }
    } catch (error) {
      console.error('💥 Web3 Auth: Session verification failed:', error);
      setIsAuthenticated(false);
      setHasAdminAccess(false);
      toast.error('Failed to verify wallet permissions');
    } finally {
      setIsLoading(false);
    }
  };

  // Helper to update auth state
  const updateAuthState = async (allPermissions: string[], level: string, hasAdmin: boolean, walletAddress: string) => {
    setPermissions(allPermissions);
    setAdminLevel(level as 'none' | 'moderator' | 'manager' | 'super');
    setHasAdminAccess(hasAdmin);
    setIsAuthenticated(hasAdmin);

    if (hasAdmin) {
      console.log('✅ Web3 Auth: Admin session verified for wallet:', walletAddress, 'Level:', level);
      
      // Set wallet session cookie
      try {
        await fetch('/api/auth/web3/verify', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ 
            wallet_address: walletAddress,
            admin_context: true 
          }),
          credentials: 'include',
        });
      } catch (error) {
        console.warn('Failed to set session cookie:', error);
      }
      
      toast.success(`Welcome, Admin ${walletAddress.slice(0, 6)}...${walletAddress.slice(-4)}`);
    } else {
      console.warn('❌ Web3 Auth: Wallet has no admin permissions:', walletAddress);
      toast.error('This wallet does not have admin permissions');
      setIsAuthenticated(false);
    }
  };

  // Login with wallet connection
  const login = async () => {
    try {
      setIsLoading(true);
      
      if (!isConnected) {
        // Try to connect with the first available connector (usually injected)
        const connector = connectors[0];
        if (connector) {
          connect({ connector });
        } else {
          toast.error('No wallet connector available');
        }
      }
      
      // Authentication will be triggered by the useEffect when wallet connects
    } catch (error) {
      console.error('💥 Web3 Auth: Login failed:', error);
      toast.error('Failed to connect wallet');
      setIsLoading(false);
    }
  };

  // Logout and disconnect wallet
  const logout = async () => {
    try {
      setIsLoading(true);
      
      // Clear permission cache
      if (address) {
        clearCache(address);
      }
      
      // Clear session cookie
      await fetch('/api/auth/web3/logout', {
        method: 'POST',
        credentials: 'include',
      });

      // Disconnect wallet
      disconnect();
      
      // Reset state
      setIsAuthenticated(false);
      setPermissions([]);
      setAdminLevel('none');
      setHasAdminAccess(false);
      
      toast.success('Logged out successfully');
    } catch (error) {
      console.error('💥 Web3 Auth: Logout failed:', error);
      toast.error('Failed to logout');
    } finally {
      setIsLoading(false);
    }
  };

  // Refresh session (re-verify permissions)
  const refreshSession = async () => {
    if (address) {
      await verifyWalletSession(address);
    }
  };

  const value: Web3AuthContextType = {
    isAuthenticated,
    isLoading,
    walletAddress: address,
    permissions,
    adminLevel,
    hasAdminAccess,
    login,
    logout,
    refreshSession,
  };

  return (
    <Web3AuthContext.Provider value={value}>
      {children}
    </Web3AuthContext.Provider>
  );
}