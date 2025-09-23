'use client';

import { useState, useEffect, useCallback } from 'react';
import { formatAddress } from '@/lib/auth/web3-store';
import { useWeb3AuthContext } from '@/providers/Web3AuthProvider';
import { useWeb3Context } from '@/providers/Web3Provider';
import { useProgressiveAuth } from '@/hooks/useProgressiveAuth';
import { AuthLevel } from '@/types/progressive-auth';
import { ConnectedWalletDropdown } from './ConnectedWalletDropdown';
import { WalletConnectionModal } from './WalletConnectionModal';
import { Wallet, Link, Loader2, AlertCircle, Shield, Eye, RefreshCw } from 'lucide-react';
import { useAccount } from 'wagmi';

interface WalletConnectAuthProps {
  onAuthSuccess?: (walletAddress: string) => void;
  onAuthError?: (error: string) => void;
  className?: string;
  
  /**
   * Preferred authentication level for this component instance
   * - CONNECTED: Show "Connect Wallet" for personalization
   * - AUTHENTICATED: Show "Sign In" for full access
   */
  preferredLevel?: AuthLevel;
  
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
  preferredLevel = AuthLevel.CONNECTED,
  compact = false
}: WalletConnectAuthProps) {
  const [isHydrated, setIsHydrated] = useState(false);
  const [autoAuthFailed, setAutoAuthFailed] = useState(false);
  
  const { address, isConnected: wagmiConnected } = useAccount();
  const { isInitialized } = useWeb3Context();
  const {
    isConnected,
    isAuthenticated,
    isAuthenticating,
    walletAddress,
    error,
    authenticate,
    disconnect,
    resetAuthState,
    hasInitialized,
    isLoading,
  } = useWeb3AuthContext();
  
  // Progressive authentication state
  const progressiveAuth = useProgressiveAuth();

  // Handle hydration to prevent SSR/client mismatch
  useEffect(() => {
    setIsHydrated(true);
  }, []);

  // DISABLED: Auto-authentication causing conflicts and initialization issues
  // Manual authentication only to prevent race conditions and wallet bugs
  useEffect(() => {
    // Only check if wallet is connected, but don't auto-authenticate
    if (
      isHydrated && 
      hasInitialized && 
      !isLoading && 
      wagmiConnected && 
      address && 
      !isAuthenticated && 
      !isAuthenticating
    ) {
      console.log('🔗 Wallet connected but auto-authentication disabled - manual sign-in required');
      // Always require manual authentication to prevent conflicts
      setAutoAuthFailed(true);
    }
  }, [isHydrated, hasInitialized, isLoading, wagmiConnected, address, isAuthenticated, isAuthenticating]);

  // Reset autoAuthFailed when wallet disconnects
  useEffect(() => {
    if (!wagmiConnected) {
      setAutoAuthFailed(false);
    }
  }, [wagmiConnected]);

  // Handle callbacks
  if (isAuthenticated && walletAddress && onAuthSuccess) {
    onAuthSuccess(walletAddress);
  }

  if (error && onAuthError) {
    onAuthError(error);
  }


  // Loading state during hydration, initialization, or validation
  if (!isHydrated || !hasInitialized || isLoading) {
    return <LoadingButton message="Loading..." className={className} />;
  }

  // Get button info based on progressive auth level and preferred level
  const getButtonInfo = () => {
    switch (progressiveAuth.level) {
      case AuthLevel.PUBLIC:
        return {
          icon: <Wallet className="h-4 w-4" />,
          text: compact ? 'Connect' : 'Connect Wallet',
          description: compact ? '' : 'for personalization',
          gradient: 'from-blue-500 to-cyan-600 hover:from-blue-600 hover:to-cyan-700',
        };
      case AuthLevel.CONNECTED:
        if (preferredLevel === AuthLevel.AUTHENTICATED) {
          return {
            icon: <Shield className="h-4 w-4" />,
            text: compact ? 'Sign In' : 'Sign In',
            description: compact ? '' : 'for full access',
            gradient: 'from-orange-500 to-purple-600 hover:from-orange-600 hover:to-purple-700',
          };
        }
        return {
          icon: <div className="w-2 h-2 bg-blue-400 rounded-full"></div>,
          text: compact ? formatAddress(walletAddress!) : formatAddress(walletAddress!),
          description: compact ? '' : 'Connected',
          gradient: 'from-blue-500 to-cyan-600 hover:from-blue-600 hover:to-cyan-700',
        };
      case AuthLevel.AUTHENTICATED:
        return {
          icon: <div className="w-2 h-2 bg-green-400 rounded-full"></div>,
          text: compact ? formatAddress(walletAddress!) : formatAddress(walletAddress!),
          description: compact ? '' : 'Authenticated',
          gradient: 'from-green-500 to-emerald-600 hover:from-green-600 hover:to-emerald-700',
        };
      default:
        return {
          icon: <Eye className="h-4 w-4" />,
          text: compact ? 'Connect' : 'Connect Wallet',
          description: '',
          gradient: 'from-slate-500 to-slate-600 hover:from-slate-600 hover:to-slate-700',
        };
    }
  };

  const buttonInfo = getButtonInfo();

  // Progressive authentication flow with custom components
  // AUTHENTICATED or CONNECTED: Show connected wallet dropdown
  if (progressiveAuth.level === AuthLevel.AUTHENTICATED || progressiveAuth.level === AuthLevel.CONNECTED) {
    // If this component prefers authentication and we're only connected, show sign-in button
    if (progressiveAuth.level === AuthLevel.CONNECTED && preferredLevel === AuthLevel.AUTHENTICATED) {
      return (
        <div className="flex items-center gap-2">
          {isAuthenticating ? (
            <LoadingButton message="Signing..." className={className} />
          ) : (
            <button
              onClick={async () => {
                try {
                  await authenticate();
                } catch (error: any) {
                  console.error('Authentication failed:', error);
                  onAuthError?.(error.message || 'Authentication failed');
                }
              }}
              disabled={isAuthenticating}
              className={`flex items-center gap-2 bg-gradient-to-r ${buttonInfo.gradient} text-white px-4 py-2 rounded-lg text-sm font-medium disabled:opacity-50 ${className}`}
            >
              {buttonInfo.icon}
              <span>{buttonInfo.text}</span>
              {!compact && buttonInfo.description && (
                <span className="text-xs opacity-75">({buttonInfo.description})</span>
              )}
            </button>
          )}
          
          {error && (
            <ErrorDisplay 
              error={error} 
              onReset={() => {
                resetAuthState();
                setAutoAuthFailed(false);
              }}
            />
          )}
        </div>
      );
    }

    // Show connected wallet dropdown
    return <ConnectedWalletDropdown className={className} />;
  }

  // PUBLIC: Show connect wallet modal
  return (
    <div className="flex items-center gap-2">
      {isAuthenticating ? (
        <LoadingButton message="Connecting..." className={className} />
      ) : (
        <WalletConnectionModal className={className} />
      )}
      
      {error && (
        <ErrorDisplay 
          error={error} 
          onReset={() => {
            resetAuthState();
            setAutoAuthFailed(false);
          }}
        />
      )}
    </div>
  );
}