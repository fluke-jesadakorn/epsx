'use client';

import { useUnifiedWeb3 } from '@/shared/components/providers/UnifiedWeb3Provider';

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui';
import { themeUtils } from '@/components/ui/SafeThemeScript';
import { formatAddress } from '@/shared/auth/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { copyToClipboard } from '@/utils/util';
import { Check, ChevronRight, Code, Copy, ExternalLink, LogOut, Moon, Settings, Sun, Wallet } from 'lucide-react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
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
  const router = useRouter();
  const pathname = usePathname();
  const [isHydrated, setIsHydrated] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [isDisconnecting, setIsDisconnecting] = useState(false);
  const [authRetryCount, setAuthRetryCount] = useState(0);
  const [lastAuthError, setLastAuthError] = useState<string | null>(null);
  const [currentTheme, setCurrentTheme] = useState<'light' | 'dark'>('light');
  const { address, isConnected, connector, status: accountStatus } = useAccount();
  const { disconnect } = useDisconnect();
  const { isInitialized } = useUnifiedWeb3();
  const { isAuthenticated, requestChallenge, authenticateWithWallet, logout } = useSharedAuth();
  const { signMessageAsync } = useSignMessage();

  useEffect(() => {
    if (!isHydrated) {
      setIsHydrated(true);
      setCurrentTheme(themeUtils.getTheme());
    }
  }, [isHydrated]);

  // Reset auth retry count on disconnect
  useEffect(() => {
    if (!isConnected || !address) {
      setAuthRetryCount(0);
      setLastAuthError(null);
    }
  }, [isConnected, address]);

  const handleCopyAddress = async () => {
    if (!address) {return;}
    const success = await copyToClipboard(address);
    if (success) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleViewExplorer = () => {
    const explorerUrl = `https://bscscan.com/address/${address}`;
    window.open(explorerUrl, '_blank', 'noopener,noreferrer');
  };

  const handleSignIn = async () => {
    if (!address) {return;}
    try {
      setIsAuthenticating(true);
      const challenge = await requestChallenge(address);
      const signature = await signMessageAsync({ message: challenge.message });
      const result = await authenticateWithWallet(
        challenge.wallet_address,
        signature,
        challenge.message,
        challenge.nonce
      );

      if (result.success) {
        setAuthRetryCount(0);
        setLastAuthError(null);
      } else {
        console.error('❌ Sign-in failed:', result.error);
        setLastAuthError(result.error ?? null);
      }
    } catch (error: any) {
      console.error('❌ Sign-in error:', error);
      if (error?.code !== 4001 && !error?.message?.includes('User rejected')) {
        setLastAuthError(error?.message || 'Sign-in failed');
      }
    } finally {
      setIsAuthenticating(false);
    }
  };

  const handleDisconnect = async () => {
    setIsDisconnecting(true);
    try {
      await logout();
      await disconnect();
    } catch (error) {
      console.error('OIDC logout error:', error);
      await disconnect();
    } finally {
      setIsDisconnecting(false);
    }
  };

  const handleThemeToggle = () => {
    const newTheme = themeUtils.toggleTheme();
    setCurrentTheme(newTheme);
  };

  // Loading state - only wait for hydration and initialization
  // Let wagmi reconnect silently in background without blocking UI
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

  // Not connected - show connect button
  if (!isConnected || !address) {
    const handleConnectRedirect = () => {
      router.push(`/auth?return_url=${encodeURIComponent(pathname)}`);
    };

    const baseClasses = 'flex items-center gap-2 rounded-2xl font-medium transition-all duration-200';
    const sizeClasses = compact ? 'h-8 px-3 text-xs' : 'h-10 px-4 text-sm';
    const colorClasses = 'bg-gradient-to-r from-orange-400 to-orange-600 hover:from-orange-500 hover:to-orange-700 text-white shadow-lg hover:shadow-xl border-0';
    const extraClasses = typeof className === 'string' ? className : '';
    const finalClassName = [baseClasses, sizeClasses, colorClasses, extraClasses].filter(Boolean).join(' ');

    return (
      <button onClick={handleConnectRedirect} type="button" className={finalClassName}>
        <Wallet className={compact ? 'h-3 w-3 text-white' : 'h-4 w-4 text-white'} />
        <span>Sign In</span>
      </button>
    );
  }

  // Detect wallet provider
  const connectorId = connector?.id?.toLowerCase() || 'injected';
  const providerInfo = walletProviders[connectorId] || walletProviders.injected;

  const getStatus = () => {
    if (authRetryCount >= 3 && lastAuthError) {return { text: 'Auth Failed', color: 'text-red-500' };}
    if (isAuthenticating) {return { text: `Signing... (${authRetryCount + 1}/3)`, color: 'text-orange-500' };}
    if (isAuthenticated) {return { text: 'Authenticated', color: 'text-emerald-500' };}
    return { text: 'Connected', color: 'text-slate-500' };
  };

  const status = getStatus();

  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <button
          className={`flex items-center gap-2 rounded-2xl px-3 py-2 text-sm font-medium transition-all duration-200
            text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 
            dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200 
            bg-transparent border-0 ${className}`}
        >
          <Wallet className={`h-4 w-4 ${status.color === 'text-red-500' ? 'text-red-500' : 'text-orange-500'}`} />
          <span className="text-sm font-medium">{formatAddress(address)}</span>
        </button>
      </DropdownMenuTrigger>

      <DropdownMenuContent
        align="end"
        sideOffset={8}
        style={{ zIndex: 99999 }}
        className="w-72 p-0 bg-white/98 backdrop-blur-xl border border-slate-200 dark:bg-slate-900/98 dark:border-slate-700 shadow-2xl rounded-2xl overflow-hidden"
      >
        {/* ═══════════════════════════════════════════════════════════════
            HEADER - Wallet Info Card
        ═══════════════════════════════════════════════════════════════ */}
        <div className="p-4 bg-gradient-to-br from-slate-50 to-orange-50/50 dark:from-slate-800 dark:to-orange-900/20 border-b border-slate-200/50 dark:border-slate-700/50">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-full bg-white dark:bg-slate-700 shadow-sm">
              <span className="text-xl">{providerInfo.icon}</span>
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                {providerInfo.name}
              </div>
              <div className="text-xs font-mono text-slate-500 dark:text-slate-400 truncate">
                {address}
              </div>
              <div className={`text-xs font-medium ${status.color} flex items-center gap-1 mt-0.5`}>
                {status.text === 'Authenticated' && <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />}
                {status.text}
              </div>
            </div>
          </div>
        </div>

        {/* ═══════════════════════════════════════════════════════════════
            QUICK ACTIONS - Copy & Explorer Buttons
        ═══════════════════════════════════════════════════════════════ */}
        <div className="p-2 flex gap-2">
          <button
            onClick={handleCopyAddress}
            className="flex-1 flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl
              bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700
              text-slate-700 dark:text-slate-300 text-sm font-medium transition-all duration-150"
          >
            {copied ? <Check className="h-4 w-4 text-emerald-500" /> : <Copy className="h-4 w-4" />}
            {copied ? 'Copied!' : 'Copy'}
          </button>
          <button
            onClick={handleViewExplorer}
            className="flex-1 flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl
              bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700
              text-slate-700 dark:text-slate-300 text-sm font-medium transition-all duration-150"
          >
            <ExternalLink className="h-4 w-4" />
            Explorer
          </button>
        </div>

        <DropdownMenuSeparator className="bg-slate-200/50 dark:bg-slate-700/50 my-0" />

        {/* ═══════════════════════════════════════════════════════════════
            SIGN IN - Only when connected but not authenticated
        ═══════════════════════════════════════════════════════════════ */}
        {!isAuthenticated && !isAuthenticating && (
          <>
            <div className="p-2">
              <button
                onClick={handleSignIn}
                className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl
                  bg-gradient-to-r from-emerald-50 to-green-50 hover:from-emerald-100 hover:to-green-100
                  dark:from-emerald-900/30 dark:to-green-900/30 dark:hover:from-emerald-900/50 dark:hover:to-green-900/50
                  border border-emerald-200/50 dark:border-emerald-700/30
                  text-emerald-700 dark:text-emerald-300 transition-all duration-150"
              >
                <Wallet className="h-4 w-4" />
                <div className="flex-1 text-left">
                  <div className="text-sm font-semibold">Sign In with Wallet</div>
                  <div className="text-xs opacity-75">Authenticate to access all features</div>
                </div>
              </button>
            </div>
            <DropdownMenuSeparator className="bg-slate-200/50 dark:bg-slate-700/50 my-0" />
          </>
        )}

        {/* ═══════════════════════════════════════════════════════════════
            RETRY AUTH - Only when auth failed
        ═══════════════════════════════════════════════════════════════ */}
        {authRetryCount >= 3 && lastAuthError && (
          <>
            <div className="p-2">
              <button
                onClick={() => {
                  setAuthRetryCount(0);
                  setLastAuthError(null);
                }}
                className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl
                  bg-orange-50 hover:bg-orange-100 dark:bg-orange-900/20 dark:hover:bg-orange-900/30
                  text-orange-700 dark:text-orange-300 transition-all duration-150"
              >
                <Wallet className="h-4 w-4" />
                <div className="flex-1 text-left">
                  <div className="text-sm font-medium">Retry Authentication</div>
                  <div className="text-xs opacity-75">Clear error and try again</div>
                </div>
              </button>
            </div>
            <DropdownMenuSeparator className="bg-slate-200/50 dark:bg-slate-700/50 my-0" />
          </>
        )}

        {/* ═══════════════════════════════════════════════════════════════
            NAVIGATION LINKS
        ═══════════════════════════════════════════════════════════════ */}
        <div className="p-2 space-y-1">
          <DropdownMenuItem asChild className="p-0">
            <Link
              href="/account"
              className="flex items-center gap-3 px-3 py-2.5 rounded-xl cursor-pointer
                hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
            >
              <Settings className="h-4 w-4 text-slate-500" />
              <span className="flex-1 text-sm font-medium text-slate-700 dark:text-slate-300">Account Settings</span>
              <ChevronRight className="h-4 w-4 text-slate-400" />
            </Link>
          </DropdownMenuItem>

          <DropdownMenuItem asChild className="p-0">
            <Link
              href="/developer"
              className="flex items-center gap-3 px-3 py-2.5 rounded-xl cursor-pointer
                hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
            >
              <Code className="h-4 w-4 text-slate-500" />
              <span className="flex-1 text-sm font-medium text-slate-700 dark:text-slate-300">Developer Portal</span>
              <ChevronRight className="h-4 w-4 text-slate-400" />
            </Link>
          </DropdownMenuItem>
        </div>

        <DropdownMenuSeparator className="bg-slate-200/50 dark:bg-slate-700/50 my-0" />

        {/* ═══════════════════════════════════════════════════════════════
            THEME TOGGLE
        ═══════════════════════════════════════════════════════════════ */}
        <div className="p-2">
          <button
            onClick={handleThemeToggle}
            className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl
              hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          >
            {currentTheme === 'dark' ? (
              <Moon className="h-4 w-4 text-slate-500" />
            ) : (
              <Sun className="h-4 w-4 text-amber-500" />
            )}
            <span className="flex-1 text-left text-sm font-medium text-slate-700 dark:text-slate-300">
              {currentTheme === 'dark' ? 'Dark Mode' : 'Light Mode'}
            </span>
            {/* Toggle Switch */}
            <div className={`relative w-10 h-5 rounded-full transition-colors ${currentTheme === 'dark' ? 'bg-orange-500' : 'bg-slate-300'
              }`}>
              <div className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow-sm transition-transform ${currentTheme === 'dark' ? 'translate-x-5' : 'translate-x-0.5'
                }`} />
            </div>
          </button>
        </div>

        <DropdownMenuSeparator className="bg-slate-200/50 dark:bg-slate-700/50 my-0" />

        {/* ═══════════════════════════════════════════════════════════════
            DISCONNECT - Danger Zone
        ═══════════════════════════════════════════════════════════════ */}
        <div className="p-2">
          <button
            onClick={handleDisconnect}
            disabled={isDisconnecting}
            className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl
              bg-red-50 hover:bg-red-100 dark:bg-red-900/20 dark:hover:bg-red-900/30
              text-red-600 dark:text-red-400 transition-all duration-150
              disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <LogOut className="h-4 w-4" />
            <span className="text-sm font-medium">
              {isDisconnecting ? 'Disconnecting...' : 'Disconnect Wallet'}
            </span>
          </button>
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}