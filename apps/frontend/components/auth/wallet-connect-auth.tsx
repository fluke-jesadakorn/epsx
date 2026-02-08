'use client';

import { useSharedAuth } from '@/shared/components/auth/Provider';
import { AlertCircle, Loader2, RefreshCw, Shield } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useAccount, useSignMessage } from 'wagmi';
import { ConnectedWalletDropdown } from './connected-wallet-dropdown';
import { WalletConnectionModal } from './wallet-connection-modal';

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
  const { signMessageAsync } = useSignMessage();
  const {
    user,
    isLoading,
    error,
    requestChallenge,
    authenticateWithWallet,
    refreshUser,
    logout,
  } = useSharedAuth();

  const isAuthenticated = !!user;

  // Local state for authentication flow
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [isSigningChallenge, setIsSigningChallenge] = useState(false);

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
      {(isSigningChallenge || isAuthenticating) ? (
        <LoadingButton
          message={isAuthenticating ? "Authenticating..." : wagmiConnected ? "Signing..." : "Connecting..."}
          className={className}
        />
      ) : wagmiConnected && address ? (
        <button
          onClick={async () => {
            let safetyTimeout: NodeJS.Timeout | null = null;

            try {
              setIsAuthenticating(true);
              setIsSigningChallenge(true);

              // Safety timeout: reset state if stuck for too long
              safetyTimeout = setTimeout(() => {
                if (isAuthenticating || isSigningChallenge) {
                  console.warn('[AUTH] UI: Authentication timed out, resetting state');
                  setIsAuthenticating(false);
                  setIsSigningChallenge(false);
                  onAuthError?.('Process timed out. Please try again.');
                }
              }, 15000); // 15s max waiting time

              // Use the shared authentication service
              const challenge = await requestChallenge(address);

              // Clear signing state once challenge is received, now waiting for signature
              // But we keep isAuthenticating true until full completion

              const signature = await signMessageAsync({ message: challenge.message, account: address });

              const result = await authenticateWithWallet(
                address,
                signature,
                challenge.message,
                challenge.nonce
              );

              if (result.success) {
                // Trigger success callback
                if (onAuthSuccess) {
                  onAuthSuccess(address);
                }
              } else {
                throw new Error(result.error || 'Authentication verification failed');
              }

            } catch (error: unknown) {
              console.error('❌ Authentication failed:', error);
              const errorMessage = error instanceof Error ? error.message : 'Authentication failed';
              onAuthError?.(errorMessage);
            } finally {
              if (safetyTimeout) {clearTimeout(safetyTimeout);}
              setIsAuthenticating(false);
              setIsSigningChallenge(false);
            }
          }}
          disabled={isSigningChallenge || isAuthenticating}
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
            refreshUser();
          }}
          onFullReset={async () => {
            // Full reset - logout and reload
            await logout();
          }}
        />
      )}
    </div>
  );
}