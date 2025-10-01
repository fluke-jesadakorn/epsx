'use client';

import React from 'react';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { Wallet, LogOut, Shield, Crown, AlertTriangle, CheckCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useAuth } from '@/lib/auth';
import { SharedWeb3SignIn } from '@/shared/components/auth/SharedWeb3SignIn';
import { UserWalletDisplay, UserTierBadge, UserAuthStatus, UserPermissionsDisplay } from '@/shared/components/display/UserDisplay';

interface AdminWalletAuthProps {
  onAuthSuccess?: (walletAddress: string) => void;
  onAuthError?: (error: string) => void;
  className?: string;
}

export function AdminWalletAuth({ 
  onAuthSuccess, 
  onAuthError, 
  className = '' 
}: AdminWalletAuthProps) {
  const { 
    wallet, 
    user, // For backward compatibility
    isAuthenticated, 
    isLoading, 
    error, 
    isAdmin,
    disconnectWallet
  } = useAuth();
  
  const currentWallet = wallet || user; // Support both new and legacy
  const walletAddress = currentWallet?.wallet_address;

  // Call success callback when authenticated with enhanced handling
  React.useEffect(() => {
    if (isAuthenticated && walletAddress && isAdmin()) {
      console.log('✅ Admin wallet auth: Admin user authenticated', { wallet_address: walletAddress });
      onAuthSuccess?.(walletAddress);
    } else if (isAuthenticated && walletAddress && !isAdmin()) {
      console.log('⚠️ Admin wallet auth: User authenticated but lacks admin permissions');
    }
  }, [isAuthenticated, walletAddress, isAdmin, onAuthSuccess]);
  
  // Call error callback when there's an error
  React.useEffect(() => {
    if (error) {
      onAuthError?.(error);
    }
  }, [error, onAuthError]);

  const handleDisconnect = async () => {
    try {
      await disconnectWallet();
    } catch (error) {
      console.error('Disconnect error:', error);
    }
  };

  // All display logic is now handled by shared components

  // Loading state
  if (isLoading) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <div className="flex items-center space-x-2">
          <div className="w-4 h-4 bg-gray-300 rounded-full animate-pulse"></div>
          <span className="text-sm text-gray-500">Loading...</span>
        </div>
      </div>
    );
  }

  // Not authenticated - show authentication interface
  if (!isAuthenticated) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <div className="flex flex-col gap-4 p-6 border rounded-lg">
          <div className="flex items-center gap-2 text-lg font-semibold">
            <Shield className="h-5 w-5" />
            Admin Authentication
          </div>
          <p className="text-sm text-gray-600">Secure admin access with Web3 wallet</p>
          
          <ConnectButton.Custom>
            {({ account, chain, openAccountModal, openChainModal, openConnectModal, mounted }) => {
              if (!mounted) return null;
              
              if (!account) {
                return (
                  <Button onClick={openConnectModal} className="w-full">
                    <Wallet className="h-4 w-4 mr-2" />
                    Connect Admin Wallet
                  </Button>
                );
              }
              
              return (
                <div className="flex flex-col gap-2">
                  <div className="text-sm text-gray-600">
                    Connected: {account.address}
                  </div>
                  <Button 
                    onClick={async () => {
                      try {
                        await useAuth.getState().authenticateAdmin();
                      } catch (error) {
                        console.error('Admin authentication failed:', error);
                        onAuthError?.(error instanceof Error ? error.message : 'Authentication failed');
                      }
                    }}
                    className="w-full"
                  >
                    <Crown className="h-4 w-4 mr-2" />
                    Authenticate as Admin
                  </Button>
                </div>
              );
            }}
          </ConnectButton.Custom>
        </div>
      </div>
    );
  }

  // Authenticated but not admin - show error
  if (isAuthenticated && !isAdmin()) {
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

  // Authenticated admin - show admin info using shared components
  return (
    <div className={`flex items-center gap-3 ${className}`}>
      {/* Admin Tier Badge */}
      <UserTierBadge className="px-3 py-2" />

      {/* Wallet Address Display */}
      <UserWalletDisplay showFullAddress={false} className="text-sm" />

      {/* Auth Status */}
      <UserAuthStatus />

      {/* Permissions Summary */}
      <UserPermissionsDisplay maxDisplay={0} />

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