/**
 * Admin Wallet Connect Component
 * Uses shared authentication system for consistency
 */
'use client';

import React from 'react';

import { useSharedAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider';
import { UserWalletDisplay, UserTierBadge, UserAuthStatus } from '@/shared/components/display/UserDisplay';

interface AdminWalletConnectProps {
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
export function AdminWalletConnect({ onAuthSuccess, onAuthError, className }: AdminWalletConnectProps) {
  const { isAuthenticated, user, error } = useSharedAuth();

  // Handle callbacks
  React.useEffect(() => {
    if (isAuthenticated && user?.wallet_address) {
      onAuthSuccess?.(user.wallet_address);
    }
  }, [isAuthenticated, user?.wallet_address, onAuthSuccess]);

  React.useEffect(() => {
    if (error) {
      onAuthError?.(error);
    }
  }, [error, onAuthError]);

  return (
    <div className={`flex items-center gap-3 ${className}`}>
      <UserTierBadge className="px-3 py-2" />
      <UserWalletDisplay showFullAddress={false} className="text-sm" />
      <UserAuthStatus />
    </div>
  );
}