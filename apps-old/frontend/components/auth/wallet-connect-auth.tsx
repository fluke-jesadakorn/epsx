
'use client';

import { useSharedAuth } from '@/shared/components/auth';
import { AlertCircle, Loader2, RefreshCw, Shield, X } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { useAccount, useSignMessage } from 'wagmi';
import { ConnectedWalletDropdown } from './connected-wallet-dropdown';
import { WalletConnectionModal } from './wallet-connection-modal';

interface WalletConnectAuthProps {
  onAuthSuccess?: (walletAddress: string) => void;
  onAuthError?: (error: string) => void;
  className?: string;
  compact?: boolean;
}

function LoadingButton({ message, onCancel, className = '' }: {
  message: string;
  onCancel?: () => void;
  className?: string;
}) {
  return (
    <div className="flex flex-col gap-2 w-full">
      <button
        disabled
        className={`flex items-center justify-center gap-2 sm:gap-3 bg-orange-500 text-white opacity-75 px-6 py-4 sm:px-4 sm:py-2 rounded-xl sm:rounded-lg text-base sm:text-sm font-bold sm:font-medium min-h-[56px] sm:min-h-0 ${className}`}
      >
        <Loader2 className="h-5 w-5 sm:h-4 sm:w-4 animate-spin" />
        <span>{message}</span>
      </button>
      {onCancel && (
        <button
          onClick={onCancel}
          className="flex items-center justify-center gap-1.5 text-xs text-slate-400 hover:text-white transition-colors py-1"
        >
          <X className="h-3 w-3" />
          <span>Cancel</span>
        </button>
      )}
    </div>
  );
}

function ErrorDisplay({ error, onReset, onFullReset }: {
  error: string;
  onReset: () => void;
  onFullReset?: () => void;
}) {
  return (
    <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 w-full text-xs sm:text-sm mt-3">
      <div className="flex items-center gap-2 text-red-400 px-3 py-2 bg-red-500/10 rounded-lg border border-red-500/20" title={error}>
        <AlertCircle className="h-4 w-4 sm:h-3 sm:w-3 shrink-0" />
        <span className="flex-1 truncate">{error}</span>
      </div>
      <div className="flex gap-2">
        <button
          onClick={onReset}
          className="flex-1 sm:flex-none px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 active:scale-[0.98] transition-all font-medium min-h-[44px] sm:min-h-0"
        >
          Retry
        </button>
        {onFullReset && (
          <button
            onClick={onFullReset}
            className="flex items-center justify-center gap-1.5 px-4 py-2 bg-slate-600 text-white rounded-lg hover:bg-slate-700 active:scale-[0.98] transition-all font-medium min-h-[44px] sm:min-h-0"
          >
            <RefreshCw className="h-4 w-4 sm:h-3 sm:w-3" />
            <span>Reset</span>
          </button>
        )}
      </div>
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

  const { address, isConnected: wagmiConnected, connector } = useAccount();
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

  const isAuthenticated = Boolean(user);

  // Auth flow state
  const [authStep, setAuthStep] = useState<'idle' | 'challenge' | 'signing' | 'verifying'>('idle');
  const [localError, setLocalError] = useState<string | null>(null);

  // Track whether initial provider load completed (separate from auth-flow loading)
  const [initDone, setInitDone] = useState(false);

  // Track current auth attempt so we can abort stale ones
  const authAttemptRef = useRef(0);

  // Reset all auth state to idle
  const resetAuth = () => {
    authAttemptRef.current += 1; // invalidate any in-flight attempt
    setAuthStep('idle');
    setLocalError(null);
  };

  // Handle hydration
  useEffect(() => {
    setIsHydrated(true);
  }, []);

  // Mark initial load done (provider isLoading goes false after cookie restore)
  // After init, provider isLoading from authenticateWithWallet won't block the UI
  useEffect(() => {
    if (!isLoading && !initDone) { setInitDone(true); }
  }, [isLoading, initDone]);

  // ROOT FIX: Watch wallet disconnection during auth - immediately reset
  useEffect(() => {
    if (authStep !== 'idle' && (!wagmiConnected || !address)) {
      setLocalError('Wallet disconnected');
      setAuthStep('idle');
      authAttemptRef.current += 1;
    }
  }, [wagmiConnected, address, authStep]);

  // Sync shared auth errors
  useEffect(() => {
    if (error) { setLocalError(error); }
  }, [error]);

  useEffect(() => {
    if (error && onAuthError) { onAuthError(error); }
  }, [error, onAuthError]);

  // The actual sign-in flow
  const handleSignIn = async () => {
    if (!address || authStep !== 'idle') { return; }

    const attempt = ++authAttemptRef.current;
    const isStale = () => attempt !== authAttemptRef.current;

    setLocalError(null);

    try {
      // Step 1: Request challenge
      setAuthStep('challenge');
      const challenge = await requestChallenge(address);
      if (isStale()) { return; }

      // Step 2: Sign message in wallet
      setAuthStep('signing');
      const signature = await signMessageAsync({ message: challenge.message, account: address, connector });
      if (isStale()) { return; }

      // Step 3: Verify with backend
      setAuthStep('verifying');
      const result = await authenticateWithWallet({
        walletAddress: address,
        signature,
        message: challenge.message,
        nonce: challenge.nonce,
      });
      if (isStale()) { return; }

      if (result.success) {
        setAuthStep('idle');
        onAuthSuccess?.(address);
      } else {
        setLocalError(result.error ?? 'Authentication verification failed');
        setAuthStep('idle');
      }
    } catch (err: unknown) {
      if (isStale()) { return; }

      let msg = 'Authentication failed';
      if (err instanceof Error) {
        // Common wallet rejection messages
        if (err.message.includes('User rejected') || err.message.includes('User denied') || err.message.includes('user rejected')) {
          msg = 'Signature cancelled. Click Sign Message to try again.';
        } else {
          msg = err.message;
        }
      }

      setLocalError(msg);
      setAuthStep('idle');
      onAuthError?.(msg);
    }
  };

  const isBusy = authStep !== 'idle';
  const stepLabel = authStep === 'challenge' ? 'Requesting...' : authStep === 'signing' ? 'Sign in wallet...' : authStep === 'verifying' ? 'Verifying...' : '';

  // Loading state during hydration or initial provider load only
  // Once initDone, don't block on provider isLoading (set during authenticateWithWallet)
  if (!isHydrated || (!initDone && isLoading)) {
    return <LoadingButton message="Loading..." className={className} />;
  }

  // Authenticated AND wallet connected - show connected state
  // If only cookies exist but wagmi disconnected, show connect flow instead
  if (isAuthenticated && user?.wallet_address && wagmiConnected) {
    return <ConnectedWalletDropdown className={className} />;
  }

  return (
    <div className="flex items-center gap-2">
      {isBusy ? (
        <LoadingButton
          message={stepLabel}
          className={className}
          onCancel={resetAuth}
        />
      ) : wagmiConnected && address ? (
        <div className="flex flex-col gap-2 sm:gap-3 w-full">
          <button
            onClick={() => void handleSignIn()}
            disabled={isBusy}
            className={`flex items-center justify-center gap-2 sm:gap-3 bg-gradient-to-r from-orange-500 to-purple-600 hover:from-orange-600 hover:to-purple-700 text-white px-6 py-4 sm:px-4 sm:py-2 rounded-xl sm:rounded-lg text-base sm:text-sm font-bold sm:font-medium transition-all disabled:opacity-50 shadow-lg hover:shadow-xl active:scale-[0.98] min-h-[56px] sm:min-h-0 ${className}`}
          >
            <Shield className="h-5 w-5 sm:h-4 sm:w-4" />
            <span>Sign Message</span>
          </button>
          {!compact && (
            <p className="text-xs sm:text-sm text-slate-400 text-center font-medium">
              Connected: {address.slice(0, 6)}...{address.slice(-4)}
            </p>
          )}
        </div>
      ) : (
        <WalletConnectionModal className={className} />
      )}

      {localError && !isBusy && (
        <ErrorDisplay
          error={localError}
          onReset={() => {
            setLocalError(null);
            void refreshUser();
          }}
          onFullReset={async () => {
            setLocalError(null);
            await logout();
          }}
        />
      )}
    </div>
  );
}
