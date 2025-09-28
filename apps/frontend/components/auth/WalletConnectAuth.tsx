'use client';

import { useState, useEffect } from 'react';
import { useSharedAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider';
import { ConnectedWalletDropdown } from './ConnectedWalletDropdown';
import { WalletConnectionModal } from './WalletConnectionModal';
import { Wallet, Loader2, AlertCircle, Shield, RefreshCw } from 'lucide-react';
import { useAccount } from 'wagmi';

interface WalletConnectAuthProps {
  onAuthSuccess?: (walletAddress: string) => void;
  onAuthError?: (error: string) => void;
  className?: string;
  
  /**
   * Show compact mode for navigation bars
   */
  compact?: boolean;
}

// Simple loading button for 2-step flow
function LoadingButton({ message, className = '' }: { 
  message: string; 
  className?: string; 
}) {
  return (
    <button
      disabled
      className={`flex items-center gap-2 bg-orange-500 text-white opacity-75 px-4 py-2 rounded-lg text-sm font-medium ${className}`}
    >
      <Loader2 className="h-4 w-4 animate-pulse" />
      {message}
    </button>
  );
}

// Enhanced error display with reset options
function ErrorDisplay({ error, onReset, onFullReset }: { 
  error: string; 
  onReset: () => void;
  onFullReset?: () => void;
}) {
  return (
    <div className="flex items-center gap-2 text-xs">
      <div className="flex items-center gap-1 text-red-500 max-w-32 truncate" title={error}>
        <AlertCircle className="h-3 w-3" />
        <span>{error}</span>
      </div>
      <button
        onClick={onReset}
        className="px-2 py-1 bg-red-500 text-white rounded hover:bg-red-600"
        title="Try again"
      >
        Retry
      </button>
      {onFullReset && (
        <button
          onClick={onFullReset}
          className="flex items-center gap-1 px-2 py-1 bg-gray-500 text-white rounded hover:bg-gray-600"
          title="Full reset"
        >
          <RefreshCw className="h-3 w-3" />
          Reset
        </button>
      )}
    </div>
  );
}

export function WalletConnectAuth({ 
  onAuthSuccess, 
  onAuthError, 
  className = '',
  compact = false
}: WalletConnectAuthProps) {
  const [isHydrated, setIsHydrated] = useState(false);
  
  const { address, isConnected: wagmiConnected } = useAccount();
  const {
    isAuthenticated,
    isLoading,
    user,
    error,
    authenticateWithWallet,
    logout,
    isSigningChallenge
  } = useSharedAuth();

  // Handle hydration to prevent SSR/client mismatch
  useEffect(() => {
    setIsHydrated(true);
  }, []);

  // Handle callbacks
  useEffect(() => {
    if (isAuthenticated && user?.wallet_address && onAuthSuccess) {
      onAuthSuccess(user.wallet_address);
    }
  }, [isAuthenticated, user?.wallet_address, onAuthSuccess]);

  useEffect(() => {
    if (error && onAuthError) {
      onAuthError(error);
    }
  }, [error, onAuthError]);

  // Loading state during hydration or authentication
  if (!isHydrated || isLoading) {
    return <LoadingButton message="Loading..." className={className} />;
  }

  // Show connected wallet dropdown if authenticated
  if (isAuthenticated && user?.wallet_address) {
    return <ConnectedWalletDropdown className={className} />;
  }

  // Show connect/sign in button
  return (
    <div className="flex items-center gap-2">
      {isSigningChallenge ? (
        <LoadingButton message={wagmiConnected ? "Signing..." : "Connecting..."} className={className} />
      ) : wagmiConnected && address ? (
        <button
          onClick={async () => {
            try {
              // TODO: Implement full Web3 challenge/sign flow
              // await authenticateWithWallet(address, signature, message, nonce);
              console.log('Authentication triggered for:', address);
            } catch (error: unknown) {
              console.error('Authentication failed:', error);
              const errorMessage = error instanceof Error ? error.message : 'Authentication failed';
              onAuthError?.(errorMessage);
            }
          }}
          disabled={isSigningChallenge}
          className={`flex items-center gap-2 bg-gradient-to-r from-orange-500 to-purple-600 hover:from-orange-600 hover:to-purple-700 text-white px-4 py-2 rounded-lg text-sm font-medium disabled:opacity-50 ${className}`}
        >
          <Shield className="h-4 w-4" />
          <span>{compact ? 'Sign In' : 'Sign In'}</span>
        </button>
      ) : (
        <WalletConnectionModal className={className} />
      )}
      
      {error && (
        <ErrorDisplay 
          error={error} 
          onReset={() => {
            // Reset error state by refreshing user
            if (typeof window !== 'undefined') {
              window.location.reload();
            }
          }}
          onFullReset={() => {
            // Full reset - logout and reload
            logout();
            if (typeof window !== 'undefined') {
              window.location.reload();
            }
          }}
        />
      )}
    </div>
  );
}