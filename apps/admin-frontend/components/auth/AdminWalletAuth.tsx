'use client';

import { useState, useCallback, useEffect } from 'react';
import { useAccount, useConnect, useDisconnect, useSignMessage } from 'wagmi';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { SiweMessage } from 'siwe';
import { Wallet, LogOut, Shield, Crown, AlertTriangle, CheckCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useRouter } from 'next/navigation';
import { toast } from 'react-hot-toast';

interface AdminWalletAuthProps {
  onAuthSuccess?: (walletAddress: string) => void;
  onAuthError?: (error: string) => void;
  className?: string;
}

interface AdminPermission {
  permission: string;
  source: 'manual' | 'nft' | 'token' | 'dao';
  expires_at?: string;
  metadata?: Record<string, any>;
}

interface AdminAuthState {
  isConnected: boolean;
  isAuthenticated: boolean;
  isAuthenticating: boolean;
  isAdmin: boolean;
  walletAddress?: string;
  permissions: AdminPermission[];
  adminLevel?: 'super' | 'manager' | 'moderator';
  error?: string;
}

export function AdminWalletAuth({ 
  onAuthSuccess, 
  onAuthError, 
  className = '' 
}: AdminWalletAuthProps) {
  const { address, isConnected } = useAccount();
  const { disconnect } = useDisconnect();
  const { signMessageAsync } = useSignMessage();
  const router = useRouter();

  const [authState, setAuthState] = useState<AdminAuthState>({
    isConnected: false,
    isAuthenticated: false,
    isAuthenticating: false,
    isAdmin: false,
    permissions: [],
  });

  // Check authentication status on mount and address change
  useEffect(() => {
    if (address && isConnected) {
      checkAuthStatus(address);
    } else {
      setAuthState(prev => ({ 
        ...prev, 
        isConnected: false, 
        isAuthenticated: false,
        isAdmin: false,
        walletAddress: undefined 
      }));
    }
  }, [address, isConnected]);

  const checkAuthStatus = async (walletAddress: string) => {
    try {
      const response = await fetch('/api/auth/session', {
        credentials: 'include',
      });
      
      if (response.ok) {
        const session = await response.json();
        if (session.wallet_address === walletAddress) {
          const adminPermissions = await checkAdminPermissions(walletAddress);
          
          setAuthState(prev => ({
            ...prev,
            isConnected: true,
            isAuthenticated: true,
            isAdmin: adminPermissions.isAdmin,
            adminLevel: adminPermissions.level,
            walletAddress,
          }));
          
          await fetchPermissions(walletAddress);
          return;
        }
      }
      
      // Not authenticated, update state
      setAuthState(prev => ({
        ...prev,
        isConnected: true,
        isAuthenticated: false,
        isAdmin: false,
        walletAddress,
      }));
    } catch (error) {
      console.error('Failed to check auth status:', error);
      setAuthState(prev => ({
        ...prev,
        isConnected: true,
        isAuthenticated: false,
        isAdmin: false,
        walletAddress,
        error: 'Failed to check authentication status',
      }));
    }
  };

  const checkAdminPermissions = async (walletAddress: string) => {
    try {
      const response = await fetch('/api/auth/web3/permissions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: walletAddress }),
        credentials: 'include',
      });

      if (response.ok) {
        const data = await response.json();
        const { permissions = [] } = data;
        
        // Check for admin permissions
        const adminPerms = permissions.filter((p: AdminPermission) => 
          p.permission.startsWith('admin:') || p.permission === 'admin:*:*'
        );
        
        let isAdmin = false;
        let level: 'super' | 'manager' | 'moderator' | undefined;

        if (adminPerms.some((p: AdminPermission) => p.permission === 'admin:*:*')) {
          isAdmin = true;
          level = 'super';
        } else if (adminPerms.some((p: AdminPermission) => p.permission.includes('admin:web3:manage'))) {
          isAdmin = true;
          level = 'manager';
        } else if (adminPerms.length > 0) {
          isAdmin = true;
          level = 'moderator';
        }

        return { isAdmin, level };
      }
      
      // Handle non-200 responses
      const errorText = await response.text();
      console.error('Permission check failed:', errorText);
      return { isAdmin: false, level: undefined };
    } catch (error) {
      console.error('Failed to check admin permissions:', error);
      return { isAdmin: false, level: undefined };
    }
  };

  const fetchPermissions = async (walletAddress: string) => {
    try {
      const response = await fetch('/api/auth/web3/permissions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: walletAddress }),
        credentials: 'include',
      });

      if (response.ok) {
        const { permissions } = await response.json();
        setAuthState(prev => ({ ...prev, permissions }));
      }
    } catch (error) {
      console.error('Failed to fetch permissions:', error);
    }
  };

  const handleAuthenticate = async () => {
    if (!address) {
      toast.error('Please connect your wallet first');
      return;
    }

    setAuthState(prev => ({ ...prev, isAuthenticating: true, error: undefined }));

    try {
      // Step 1: Request nonce
      const nonceResponse = await fetch('/api/auth/web3/challenge', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: address }),
      });

      if (!nonceResponse.ok) {
        const errorText = await nonceResponse.text();
        console.error('Nonce request failed:', errorText);
        
        // Try to parse as JSON, fallback to text
        let errorMessage = 'Failed to get authentication nonce';
        try {
          const errorData = JSON.parse(errorText);
          errorMessage = errorData.error || errorMessage;
        } catch {
          // If not JSON, use the status text or a generic message
          errorMessage = `Nonce request failed (${nonceResponse.status})`;
        }
        
        throw new Error(errorMessage);
      }

      const nonceData = await nonceResponse.json();
      const { nonce } = nonceData;

      // Step 2: Create SIWE message with admin context
      const message = new SiweMessage({
        domain: window.location.host,
        address,
        statement: 'Sign in to EPSX Admin Dashboard with admin privileges',
        uri: window.location.origin,
        version: '1',
        chainId: 1,
        nonce,
        issuedAt: new Date().toISOString(),
        requestId: 'admin-auth', // Admin-specific request ID
      });

      const messageString = message.prepareMessage();

      // Step 3: Sign message
      const signature = await signMessageAsync({ message: messageString });

      // Step 4: Verify signature and create admin session
      const verifyResponse = await fetch('/api/auth/web3/verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          wallet_address: address, 
          signature,
          nonce,
          message: messageString,
          admin_context: true // Flag for admin authentication
        }),
        credentials: 'include',
      });

      if (!verifyResponse.ok) {
        const errorText = await verifyResponse.text();
        console.error('Verification failed:', errorText);
        
        // Try to parse as JSON, fallback to text
        let errorMessage = 'Admin authentication failed';
        try {
          const errorData = JSON.parse(errorText);
          errorMessage = errorData.error || errorData.message || errorMessage;
        } catch {
          // If not JSON, use the status text or a generic message
          errorMessage = `Authentication failed (${verifyResponse.status})`;
        }
        
        throw new Error(errorMessage);
      }

      // Check admin permissions
      const adminCheck = await checkAdminPermissions(address);
      
      if (!adminCheck.isAdmin) {
        throw new Error('Wallet does not have admin permissions');
      }

      // Success
      setAuthState(prev => ({
        ...prev,
        isAuthenticated: true,
        isAuthenticating: false,
        isAdmin: true,
        adminLevel: adminCheck.level,
        walletAddress: address,
      }));

      await fetchPermissions(address);
      toast.success(`Successfully authenticated as ${adminCheck.level} admin`);
      onAuthSuccess?.(address);

    } catch (error: any) {
      console.error('Admin authentication error:', error);
      const errorMessage = error.message || 'Admin authentication failed';
      setAuthState(prev => ({ 
        ...prev, 
        isAuthenticating: false, 
        error: errorMessage 
      }));
      toast.error(errorMessage);
      onAuthError?.(errorMessage);
    }
  };

  const handleDisconnect = async () => {
    try {
      // Logout from backend
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'include',
      });

      // Disconnect wallet
      disconnect();
      
      setAuthState({
        isConnected: false,
        isAuthenticated: false,
        isAuthenticating: false,
        isAdmin: false,
        permissions: [],
      });

      toast.success('Disconnected from admin session');
    } catch (error) {
      console.error('Disconnect error:', error);
      toast.error('Failed to disconnect properly');
    }
  };

  const formatAddress = (addr: string) => {
    return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  };

  const getAdminLevelIcon = (level?: string) => {
    switch (level) {
      case 'super': return <Crown className="h-4 w-4 text-yellow-500" />;
      case 'manager': return <Shield className="h-4 w-4 text-blue-500" />;
      case 'moderator': return <CheckCircle className="h-4 w-4 text-green-500" />;
      default: return <Wallet className="h-4 w-4 text-gray-500" />;
    }
  };

  const getAdminLevelColor = (level?: string) => {
    switch (level) {
      case 'super': return 'text-yellow-600 bg-yellow-50 border-yellow-200 dark:text-yellow-300 dark:bg-yellow-900/20 dark:border-yellow-700';
      case 'manager': return 'text-blue-600 bg-blue-50 border-blue-200 dark:text-blue-300 dark:bg-blue-900/20 dark:border-blue-700';
      case 'moderator': return 'text-green-600 bg-green-50 border-green-200 dark:text-green-300 dark:bg-green-900/20 dark:border-green-700';
      default: return 'text-gray-600 bg-gray-50 border-gray-200 dark:text-gray-300 dark:bg-gray-900/20 dark:border-gray-700';
    }
  };

  // Not connected - show connect button
  if (!isConnected) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <ConnectButton.Custom>
          {({ openConnectModal }) => (
            <Button
              onClick={openConnectModal}
              className="flex items-center gap-2 rounded-2xl bg-gradient-to-r from-yellow-400 to-orange-500 px-4 py-2.5 text-sm font-medium text-white hover:from-yellow-500 hover:to-orange-600 dark:from-purple-600 dark:to-pink-600"
            >
              <Wallet className="h-4 w-4" />
              Connect Admin Wallet
            </Button>
          )}
        </ConnectButton.Custom>
      </div>
    );
  }

  // Connected but not authenticated
  if (!authState.isAuthenticated) {
    return (
      <div className={`flex flex-col gap-2 ${className}`}>
        <div className="flex items-center gap-2">
          <Button
            onClick={handleAuthenticate}
            disabled={authState.isAuthenticating}
            className="flex items-center gap-2 rounded-2xl bg-gradient-to-r from-orange-400 to-red-500 px-4 py-2.5 text-sm font-medium text-white hover:from-orange-500 hover:to-red-600 dark:from-orange-600 dark:to-red-700"
          >
            <Shield className="h-4 w-4" />
            {authState.isAuthenticating ? 'Authenticating...' : 'Admin Sign In'}
          </Button>
          
          <ConnectButton.Custom>
            {({ openAccountModal }) => (
              <Button
                variant="outline"
                onClick={openAccountModal}
                className="px-3 py-2.5 text-sm"
              >
                {formatAddress(address!)}
              </Button>
            )}
          </ConnectButton.Custom>
        </div>
        
        {authState.error && (
          <div className="flex items-center gap-2 text-sm text-red-600 dark:text-red-400">
            <AlertTriangle className="h-4 w-4" />
            {authState.error}
          </div>
        )}
      </div>
    );
  }

  // Check if admin privileges exist
  if (authState.isAuthenticated && !authState.isAdmin) {
    return (
      <div className={`flex flex-col gap-2 ${className}`}>
        <div className="flex items-center gap-2 text-sm text-red-600 dark:text-red-400">
          <AlertTriangle className="h-4 w-4" />
          Wallet does not have admin permissions
        </div>
        <Button
          onClick={handleDisconnect}
          variant="outline"
          size="sm"
          className="text-sm"
        >
          Disconnect
        </Button>
      </div>
    );
  }

  // Authenticated admin - show admin info
  return (
    <div className={`flex items-center gap-3 ${className}`}>
      {/* Admin Level Badge */}
      <div className={`flex items-center gap-2 rounded-2xl border px-3 py-2 text-sm font-medium ${getAdminLevelColor(authState.adminLevel)}`}>
        {getAdminLevelIcon(authState.adminLevel)}
        <span className="capitalize">{authState.adminLevel} Admin</span>
      </div>

      {/* Wallet Address */}
      <ConnectButton.Custom>
        {({ openAccountModal }) => (
          <Button
            variant="outline"
            onClick={openAccountModal}
            className="flex items-center gap-2 px-3 py-2 text-sm"
          >
            <Wallet className="h-4 w-4 text-green-500" />
            {formatAddress(authState.walletAddress!)}
          </Button>
        )}
      </ConnectButton.Custom>

      {/* Admin Permissions Count */}
      {authState.permissions.length > 0 && (
        <div className="flex items-center gap-1 rounded-lg bg-blue-50 px-2 py-1 dark:bg-blue-900/20">
          <Shield className="h-3 w-3 text-blue-500" />
          <span className="text-xs font-medium text-blue-600 dark:text-blue-300">
            {authState.permissions.length} perms
          </span>
        </div>
      )}

      {/* Logout Button */}
      <Button
        onClick={handleDisconnect}
        variant="ghost"
        size="sm"
        className="px-2 py-2"
        title="Disconnect Admin Session"
      >
        <LogOut className="h-4 w-4 text-slate-500 hover:text-red-500" />
      </Button>
    </div>
  );
}