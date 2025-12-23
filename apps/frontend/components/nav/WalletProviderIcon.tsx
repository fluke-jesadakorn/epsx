import { useUnifiedWeb3 } from '@/shared/components/providers/UnifiedWeb3Provider';

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  UnifiedThemeToggle,
} from '@/components/ui';
import { useWeb3AuthStore } from '@/lib/auth/store';
import { formatAddress } from '@/shared/auth/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { Check, Code, Copy, ExternalLink, Settings, Wallet } from 'lucide-react';
import Link from 'next/link';
import { useEffect, useState } from 'react';
import { useAccount, useDisconnect, useSignMessage } from 'wagmi';

interface WalletProviderIconProps {
  className?: string;
  compact?: boolean;
}

interface WalletProviderInfo {
  name: string;
  icon: string;
  color: string;
}

const walletProviders: Record<string, WalletProviderInfo> = {
  metaMask: {
    name: 'MetaMask',
    icon: '🦊',
    color: 'bg-orange-500',
  },
  walletConnect: {
    name: 'WalletConnect',
    icon: '🔗',
    color: 'bg-blue-500',
  },
  injected: {
    name: 'Browser Wallet',
    icon: '🌐',
    color: 'bg-purple-500',
  },
  coinbase: {
    name: 'Coinbase',
    icon: '🔵',
    color: 'bg-blue-600',
  },
  rainbow: {
    name: 'Rainbow',
    icon: '🌈',
    color: 'bg-gradient-to-r from-pink-500 to-violet-500',
  },
};

export function WalletProviderIcon({ className = '', compact = false }: WalletProviderIconProps) {
  const [isHydrated, setIsHydrated] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [isDisconnecting, setIsDisconnecting] = useState(false);
  const [authRetryCount, setAuthRetryCount] = useState(0);
  const [lastAuthError, setLastAuthError] = useState<string | null>(null);
  const { address, isConnected, connector } = useAccount();
  const { disconnect } = useDisconnect();
  const { isInitialized } = useUnifiedWeb3();
  const { isAuthenticated, requestChallenge, authenticateWithWallet, logout } = useSharedAuth();
  const { signMessageAsync } = useSignMessage();

  // Get web3 auth store for syncing authentication state
  const {
    setConnected: setWeb3Connected,
    setAuthenticated: setWeb3Authenticated,
    setWalletAddress: setWeb3WalletAddress
  } = useWeb3AuthStore();

  // Sync wallet connection state to Web3AuthStore immediately
  useEffect(() => {
    if (isConnected && address) {
      // ✅ Set isConnected and walletAddress IMMEDIATELY when wallet connects
      setWeb3Connected(true);
      setWeb3WalletAddress(address.toLowerCase());
      console.log('✅ Wallet connected - synced to Web3 auth store:', address);
    }
  }, [isConnected, address, setWeb3Connected, setWeb3WalletAddress]);

  useEffect(() => {
    setIsHydrated(true);
    console.log('🔍 WalletProviderIcon Debug:', {
      isConnected,
      address,
      connector: connector?.id,
      isHydrated: true,
      isInitialized,
      isAuthenticated
    });
  }, [isConnected, address, connector, isInitialized, isAuthenticated]);

  // Sync disconnect state: Clear web3 auth store when wallet disconnects
  useEffect(() => {
    if (!isConnected || !address) {
      // Wallet disconnected - clear web3 auth store completely
      setWeb3Connected(false);
      setWeb3Authenticated(false);
      setWeb3WalletAddress(undefined);
      setAuthRetryCount(0); // Reset retry count
      setLastAuthError(null); // Clear last error
      console.log('🔌 Wallet disconnected - cleared web3 auth store');
    }
  }, [isConnected, address, setWeb3Connected, setWeb3Authenticated, setWeb3WalletAddress]);

  // NOTE: Auto-authentication is DISABLED
  // Users only need to connect wallet. SIWE sign-in is triggered:
  // 1. Manually via "Sign In" button in dropdown
  // 2. Automatically when backend returns 401/403 on protected API calls
  // Frontend does NOT enforce permissions - backend handles all authorization

  const handleCopyAddress = async () => {
    if (!address) return;

    try {
      await navigator.clipboard.writeText(address);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy address:', error);
    }
  };

  // Show loading state during hydration or before RainbowKit is ready
  if (!isHydrated || !isInitialized) {
    return (
      <Button
        variant="ghost"
        size="icon"
        className={`h-10 w-10 rounded-full bg-orange-50 text-orange-500 hover:bg-orange-100 dark:bg-slate-800 dark:text-orange-400 dark:hover:bg-slate-700 ${className}`}
        disabled
      >
        <Wallet className="h-4 w-4" />
      </Button>
    );
  }

  // Only render connect button when RainbowKit is fully initialized
  if (!isConnected || !address) {
    return (
      <ConnectButton.Custom>
        {({ openConnectModal, connectModalOpen, mounted }) => {
          // Build className safely to avoid Promise issues
          const baseClasses = 'flex items-center gap-2 rounded-2xl font-medium transition-colors';
          const sizeClasses = compact ? 'h-8 px-3 text-xs' : 'h-10 px-4 text-sm';
          const colorClasses = 'bg-gradient-to-r from-orange-400 to-orange-600 hover:from-orange-500 hover:to-orange-700 text-white shadow-lg hover:shadow-xl border-0';
          const extraClasses = typeof className === 'string' ? className : '';

          const finalClassName = [baseClasses, sizeClasses, colorClasses, extraClasses].filter(Boolean).join(' ');

          return (
            <button
              onClick={openConnectModal}
              type="button"
              className={finalClassName}
            >
              <Wallet className={compact ? 'h-3 w-3 text-white' : 'h-4 w-4 text-white'} />
              <span>Connect</span>
            </button>
          );
        }}
      </ConnectButton.Custom>
    );
  }

  // Detect wallet provider
  const connectorId = connector?.id?.toLowerCase() || 'injected';
  const providerInfo = walletProviders[connectorId] || walletProviders.injected;

  const getTriggerContent = () => {
    // Show error state when max retries reached
    if (authRetryCount >= 3 && lastAuthError) {
      if (compact) {
        return (
          <div className="flex items-center gap-2">
            <Wallet className="h-4 w-4 text-red-500" />
            <span className="text-xs font-medium text-red-600 dark:text-red-400">
              Auth Failed
            </span>
          </div>
        );
      }
      return (
        <div className="flex items-center gap-2">
          <Wallet className="h-5 w-5 text-red-500" />
          <span className="text-sm font-medium text-red-600 dark:text-red-400">
            Authentication Failed
          </span>
        </div>
      );
    }

    // Show authenticating state
    if (isAuthenticating) {
      if (compact) {
        return (
          <div className="flex items-center gap-2">
            <Wallet className="h-4 w-4 text-orange-500" />
            <span className="text-xs font-medium text-orange-600 dark:text-orange-400">
              Signing... ({authRetryCount + 1}/3)
            </span>
          </div>
        );
      }
      return (
        <div className="flex items-center gap-2">
          <Wallet className="h-5 w-5 text-orange-500" />
          <span className="text-sm font-medium text-orange-600 dark:text-orange-400">
            Signing... ({authRetryCount + 1}/3)
          </span>
        </div>
      );
    }

    if (compact) {
      return (
        <div className="flex items-center gap-2">
          <Wallet className="h-4 w-4 text-orange-500" />
          <span className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {formatAddress(address)}
          </span>
        </div>
      );
    }

    return (
      <div className="flex items-center gap-2">
        <Wallet className="h-5 w-5 text-orange-500" />
        <span className="text-sm font-medium text-slate-700 dark:text-slate-300">
          {formatAddress(address)}
        </span>
      </div>
    );
  };

  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <button
          className="flex items-center gap-2 rounded-2xl px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200 bg-transparent border-0 transition-colors"
        >
          {getTriggerContent()}
        </button>
      </DropdownMenuTrigger>

      <DropdownMenuContent
        align="end"
        sideOffset={5}
        style={{ zIndex: 99999 }}
        className="w-64 p-2 bg-white/95 backdrop-blur-xl border border-orange-100/50 dark:bg-slate-900/95 dark:border-slate-700/50"
      >
        {/* Wallet Header */}
        <div className="px-3 py-2 mb-2">
          <div className="flex items-center gap-3">
            <Wallet className="h-5 w-5 text-orange-500" />
            <div>
              <div className="text-sm font-medium text-slate-900 dark:text-slate-100">
                {providerInfo.name}
              </div>
              <div className="text-xs text-slate-500 dark:text-slate-400">
                {authRetryCount >= 3 && lastAuthError
                  ? `Authentication Failed (${authRetryCount} attempts)`
                  : isAuthenticating
                    ? `Signing... (${authRetryCount + 1}/3)`
                    : isAuthenticated
                      ? 'Authenticated'
                      : 'Connected'}
              </div>
            </div>
          </div>
        </div>

        <DropdownMenuSeparator className="bg-orange-100/50 dark:bg-slate-700/50" />

        {/* Wallet Address */}
        <DropdownMenuItem
          onClick={handleCopyAddress}
          className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-orange-50/80 dark:hover:bg-slate-800/40"
        >
          <div className="flex items-center gap-2 flex-1">
            {copied ? (
              <Check className="h-4 w-4 text-orange-500" />
            ) : (
              <Copy className="h-4 w-4 text-orange-500" />
            )}
            <div>
              <div className="text-sm font-medium">
                {copied ? 'Copied!' : 'Copy Address'}
              </div>
              <div className="text-xs text-slate-500 dark:text-slate-400 font-mono">
                {formatAddress(address)}
              </div>
            </div>
          </div>
        </DropdownMenuItem>

        {/* Sign In - only show when connected but not authenticated */}
        {!isAuthenticated && !isAuthenticating && (
          <DropdownMenuItem
            onClick={async () => {
              if (!address) return;
              try {
                setIsAuthenticating(true);
                console.log('🔐 Manual sign-in started for:', address);

                // Step 1: Request challenge from backend
                const challenge = await requestChallenge(address);
                console.log('✅ Challenge received, prompting user to sign...');

                // Step 2: Prompt user to sign the SIWE message
                const signature = await signMessageAsync({
                  message: challenge.message,
                });
                console.log('✅ Message signed by user');

                // Step 3: Complete authentication with signature
                const result = await authenticateWithWallet(
                  challenge.wallet_address,
                  signature,
                  challenge.message,
                  challenge.nonce
                );

                if (result.success) {
                  console.log('✅ Manual sign-in successful!');
                  setAuthRetryCount(0);
                  setLastAuthError(null);
                  setWeb3Authenticated(true);
                  setWeb3WalletAddress(address.toLowerCase());
                } else {
                  console.error('❌ Sign-in failed:', result.error);
                  setLastAuthError(result.error ?? null);
                }
              } catch (error: any) {
                console.error('❌ Sign-in error:', error);
                if (error?.code === 4001 || error?.message?.includes('User rejected')) {
                  console.log('ℹ️ User rejected signature request');
                } else {
                  setLastAuthError(error?.message || 'Sign-in failed');
                }
              } finally {
                setIsAuthenticating(false);
              }
            }}
            className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-green-50/80 dark:hover:bg-green-900/20 text-green-600 dark:text-green-400"
          >
            <Wallet className="h-4 w-4 text-green-600 dark:text-green-400" />
            <div>
              <div className="text-sm font-medium">Sign In</div>
              <div className="text-xs opacity-75">
                Authenticate with your wallet
              </div>
            </div>
          </DropdownMenuItem>
        )}

        {/* Retry Authentication - only show when auth failed */}
        {authRetryCount >= 3 && lastAuthError && (
          <DropdownMenuItem
            onClick={() => {
              setAuthRetryCount(0);
              setLastAuthError(null);
              console.log('🔄 Cleared auth error state');
            }}
            className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-orange-50/80 dark:hover:bg-slate-800/40"
          >
            <Wallet className="h-4 w-4 text-orange-500" />
            <div>
              <div className="text-sm font-medium">Clear Error</div>
              <div className="text-xs text-slate-500 dark:text-slate-400">
                Reset authentication state
              </div>
            </div>
          </DropdownMenuItem>
        )}

        {/* View on Explorer */}
        <DropdownMenuItem
          onClick={() => {
            const explorerUrl = `https://bscscan.com/address/${address}`;
            window.open(explorerUrl, '_blank', 'noopener,noreferrer');
            setIsOpen(false);
          }}
          className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-orange-50/80 dark:hover:bg-slate-800/40"
        >
          <ExternalLink className="h-4 w-4 text-orange-500" />
          <div>
            <div className="text-sm font-medium">View on Explorer</div>
            <div className="text-xs text-slate-500 dark:text-slate-400">
              Open in BSCScan
            </div>
          </div>
        </DropdownMenuItem>

        <DropdownMenuSeparator className="bg-orange-100/50 dark:bg-slate-700/50" />

        {/* Developer Portal */}
        <DropdownMenuItem asChild className="px-3 py-2 rounded-lg cursor-pointer hover:bg-orange-50/80 dark:hover:bg-slate-800/40">
          <Link href="/developer" className="flex items-center gap-3">
            <Code className="h-4 w-4 text-orange-500" />
            <div>
              <div className="text-sm font-medium">Developer Portal</div>
              <div className="text-xs text-slate-500 dark:text-slate-400">
                Manage API keys and access
              </div>
            </div>
          </Link>
        </DropdownMenuItem>

        {/* Settings */}
        <DropdownMenuItem asChild className="px-3 py-2 rounded-lg cursor-pointer hover:bg-orange-50/80 dark:hover:bg-slate-800/40">
          <Link href="/settings" className="flex items-center gap-3">
            <Settings className="h-4 w-4 text-orange-500" />
            <div>
              <div className="text-sm font-medium">Settings</div>
              <div className="text-xs text-slate-500 dark:text-slate-400">
                App preferences and configuration
              </div>
            </div>
          </Link>
        </DropdownMenuItem>

        {/* Theme Toggle */}
        <div className="px-3 py-2">
          <UnifiedThemeToggle
            variant="minimal"
            showLabel={true}
            showTooltip={false}
            className="w-full justify-start gap-3 rounded-lg px-0 py-1 text-sm font-medium text-slate-600 hover:bg-orange-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200"
          />
        </div>

        <DropdownMenuSeparator className="bg-orange-100/50 dark:bg-slate-700/50" />

        {/* Disconnect Wallet */}
        <DropdownMenuItem
          onClick={async () => {
            // Prevent auto-auth during manual disconnect
            setIsDisconnecting(true);

            try {
              // 1. OIDC Session Termination (clear access_token, id_token, refresh_token, session cookies)
              await logout();

              // 2. Wallet Disconnection (wagmi)
              disconnect();

              // 3. Local State Cleanup
              setWeb3Connected(false);
              setWeb3Authenticated(false);
              setWeb3WalletAddress(undefined);
              setIsOpen(false);
            } catch (error) {
              console.error('OIDC logout error:', error);
              // Ensure wallet disconnects even if OIDC logout fails
              disconnect();
              setWeb3Connected(false);
              setWeb3Authenticated(false);
              setWeb3WalletAddress(undefined);
              setIsOpen(false);
            } finally {
              // Reset flag after disconnect completes (500ms ensures wagmi cleanup finishes)
              setTimeout(() => setIsDisconnecting(false), 500);
            }
          }}
          className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-red-50/80 dark:hover:bg-red-900/20 text-red-600 dark:text-red-400"
        >
          <Wallet className="h-4 w-4 text-red-600 dark:text-red-400" />
          <div>
            <div className="text-sm font-medium">Disconnect</div>
            <div className="text-xs opacity-75">
              Disconnect your wallet
            </div>
          </div>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu >
  );
}