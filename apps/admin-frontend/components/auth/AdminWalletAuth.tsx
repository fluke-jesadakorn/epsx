'use client';

import { ConnectButton } from '@rainbow-me/rainbowkit';
import { AlertTriangle, Crown, LogOut, Shield, Wallet } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React from 'react';

import { Button } from '@/components/ui/button';
import { useAuth } from '@/lib/auth';
import { UserAuthStatus, UserPermissionsDisplay, UserTierBadge, UserWalletDisplay } from '@/shared/components/display/UserDisplay';

interface AdminWalletAuthProps {
  onAuthSuccess?: (walletAddress: string) => void;
  onAuthError?: (error: string) => void;
  className?: string;
}

/**
 *
 * @param root0
 * @param root0.onAuthSuccess
 * @param root0.onAuthError
 * @param root0.className
 */
export function AdminWalletAuth({
  onAuthSuccess,
  onAuthError,
  className = ''
}: AdminWalletAuthProps) {
  const {
    wallet,
    isAuthenticated,
    isLoading,
    error,
    disconnectWallet
  } = useAuth();
  const router = useRouter();

  const walletAddress = wallet?.wallet_address;

  // Call success callback when authenticated with enhanced handling
  React.useEffect(() => {
    if (isAuthenticated && walletAddress && wallet?.is_admin) {
      onAuthSuccess?.(walletAddress);
    }
  }, [isAuthenticated, walletAddress, wallet?.is_admin, onAuthSuccess]);

  // Call error callback when there's an error
  React.useEffect(() => {
    if (error) {
      onAuthError?.(error);
    }
  }, [error, onAuthError]);

  const handleDisconnect = async () => {
    try {
      await disconnectWallet();
      router.push('/auth');
    } catch (_error) {

      console.error('Disconnect error:', _error);
    }
  };

  // All display logic is now handled by shared components

  // Loading state
  if (isLoading) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <div className="flex items-center space-x-2">
          <svg className="h-4 w-4 animate-spin text-gray-500" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span className="text-sm text-gray-500">Authenticating...</span>
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
              if (!mounted) { return null; }

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
                      } catch (_error) {

                        console.error('Admin authentication failed:', _error);
                        onAuthError?.(_error instanceof Error ? _error.message : 'Authentication failed');
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
  if (isAuthenticated && !wallet?.is_admin) {
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