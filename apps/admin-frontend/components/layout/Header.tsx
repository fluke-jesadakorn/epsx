'use client';

/**
 * ADMIN HEADER
 * Top navigation bar matching frontend design
 * Works alongside Sidebar for admin layout
 */

import { ConnectButton } from '@rainbow-me/rainbowkit';
import {
  ChevronDown,
  LogOut,
  Wallet
} from 'lucide-react';
import { useTheme } from 'next-themes';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { useAccount, useDisconnect } from 'wagmi';

import { AdminNotificationBell } from './AdminNotificationBellClient';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { themeUtils } from '@/components/ui/SafeThemeScript';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { ChainSelector } from '@/shared/components/navigation/ChainSelector';
import { UnifiedThemeToggle } from '@/shared/components/ui/UnifiedThemeToggle';
import { isProduction } from '@/shared/utils';

interface User {
  id: string;
  email: string;
  name?: string;
  role: string;
}

interface HeaderProps {
  user?: User;
}

/**
 *
 * @param root0
 * @param root0.user
 */
export function Header({ user }: HeaderProps) {
  const { logout, isAuthenticated, user: authUser, isLoading: authLoading } = useSharedAuth();
  const router = useRouter();
  const { isConnected, address } = useAccount();
  const { disconnect } = useDisconnect();
  const { theme, resolvedTheme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const [cookieWallet, setCookieWallet] = useState<string | null>(null);

  // Read wallet address directly from cookie on mount (before auth hydrates)
  useEffect(() => {
    setMounted(true);
    // Sync theme on mount if needed
    const current = themeUtils.getTheme();
    if (current && (resolvedTheme ?? theme) !== current) {
      setTheme(current);
    }
    // Try to read wallet from epsx.user cookie directly
    try {
      const userCookie = document.cookie
        .split('; ')
        .find(row => row.startsWith('epsx.user=') || row.startsWith('__Host-epsx.user='));
      if (userCookie) {
        const value = decodeURIComponent(userCookie.split('=')[1] || '');
        const parsed = JSON.parse(value);
        if (parsed?.wallet || parsed?.wallet_address) {
          setCookieWallet(parsed.wallet || parsed.wallet_address);
        }
      }
    } catch (e) {
      // Ignore parsing errors
    }
  }, []);

  // Use wagmi address first, then auth user, then direct cookie read
  const walletAddress = address || authUser?.wallet_address || cookieWallet;

  // Combined connection status: wagmi connected OR cookie-based auth OR have cookie wallet
  const isWalletConnected = isConnected || isAuthenticated || !!cookieWallet;

  const handleDisconnect = async () => {
    try {
      // Clear local state first to prevent UI flicker
      setCookieWallet(null);
      await logout();
      disconnect();
      router.push('/auth');
    } catch (error) {
      console.error('Disconnect error:', error);
    }
  };

  const formatAddress = (addr: string) => `${addr.slice(0, 6)}...${addr.slice(-4)}`;

  // Skeleton during SSR
  if (!mounted) {
    return (
      <header className="sticky top-0 z-40 border-b border-border bg-background">
        <div className="flex h-16 items-center justify-between px-3 sm:px-4 lg:px-6 gap-2 sm:gap-3">
          <div className="flex items-center gap-2 sm:gap-3 min-w-0">
            <span className="text-lg sm:text-xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent whitespace-nowrap">EPSX</span>
            <span className="text-[10px] sm:text-xs text-muted-foreground font-medium uppercase tracking-wider whitespace-nowrap">Admin</span>
          </div>
          <div className="h-6 sm:h-8 w-16 sm:w-24 bg-muted rounded-lg animate-pulse flex-shrink-0" />
        </div>
      </header>
    );
  }

  return (
    <header className="sticky top-0 z-40 border-b border-border bg-background/95 backdrop-blur-xl">
      <div className="flex h-16 items-center justify-between px-3 sm:px-4 lg:px-6 gap-2 sm:gap-3">
        {/* Logo / Title */}
        <div className="flex items-center gap-2 sm:gap-3 min-w-0 flex-shrink">
          <Link href="/" className="flex items-center gap-1 sm:gap-2 hover:opacity-80 transition-opacity min-w-0">
            <span className="text-lg sm:text-xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent whitespace-nowrap">EPSX</span>
            <span className="text-[10px] sm:text-xs text-muted-foreground font-medium uppercase tracking-wider whitespace-nowrap">Admin</span>
          </Link>
        </div>

        {/* Right Actions */}
        <div className="flex items-center gap-1.5 sm:gap-2 lg:gap-3 flex-shrink-0">
          {/* Notification Bell */}
          <div className="hidden sm:block">
            <AdminNotificationBell />
          </div>

          {/* Theme Toggle */}
          <UnifiedThemeToggle variant="default" size="md" showTooltip={true} />

          {/* Chain Selector - Only in dev */}
          {!isProduction && (
            <div className="hidden lg:block">
              <ChainSelector />
            </div>
          )}

          {/* Wallet Connect */}
          {isWalletConnected && walletAddress ? (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="flex items-center gap-1.5 sm:gap-2 rounded-lg bg-primary px-2 sm:px-3 lg:px-4 py-1.5 sm:py-2 text-xs sm:text-sm font-medium text-primary-foreground hover:opacity-90 transition-opacity">
                  <Wallet className="h-3.5 w-3.5 sm:h-4 sm:w-4 flex-shrink-0" />
                  <span className="hidden lg:inline">{formatAddress(walletAddress)}</span>
                  <ChevronDown className="h-3 w-3 flex-shrink-0" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                align="end"
                className="w-52 bg-popover border-border p-1"
                style={{ zIndex: 99999 }}
              >
                <div className="px-3 py-2 text-xs text-muted-foreground border-b border-border mb-1 font-mono break-all">
                  {walletAddress}
                </div>
                <DropdownMenuItem
                  onClick={handleDisconnect}
                  className="flex items-center gap-2 px-3 py-2 rounded-md text-red-400 hover:bg-red-500/20 cursor-pointer"
                >
                  <LogOut className="h-4 w-4" />
                  Disconnect
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          ) : (
            <ConnectButton.Custom>
              {({ openConnectModal }) => (
                <button
                  onClick={openConnectModal}
                  className="flex items-center gap-1.5 sm:gap-2 rounded-lg bg-primary px-2 sm:px-3 lg:px-4 py-1.5 sm:py-2 text-xs sm:text-sm font-medium text-primary-foreground hover:opacity-90 transition-opacity"
                >
                  <Wallet className="h-3.5 w-3.5 sm:h-4 sm:w-4 flex-shrink-0" />
                  <span className="hidden lg:inline">Connect Wallet</span>
                </button>
              )}
            </ConnectButton.Custom>
          )}
        </div>
      </div>
    </header>
  );
}

export default Header;
