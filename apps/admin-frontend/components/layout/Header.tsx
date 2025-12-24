'use client';

/**
 * ADMIN HEADER
 * Top navigation bar matching frontend design
 * Works alongside Sidebar for admin layout
 */

import { ConnectButton } from '@rainbow-me/rainbowkit';
import {
  ChevronDown,
  Link as LinkIcon,
  LogOut,
  Moon,
  Sun,
  Wallet
} from 'lucide-react';
import Link from 'next/link';
import { useEffect, useState } from 'react';
import { useAccount, useChainId, useDisconnect, useSwitchChain } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { themeUtils } from '@/components/ui/SafeThemeScript';
import { isProduction } from '@/shared/utils';
import { AdminNotificationBell } from './AdminNotificationBellClient';

interface User {
  id: string;
  email: string;
  name?: string;
  role: string;
}

interface HeaderProps {
  user?: User;
}

export function Header({ user }: HeaderProps) {
  const [mounted, setMounted] = useState(false);
  const [cookieWallet, setCookieWallet] = useState<string | null>(null);
  const { logout, isAuthenticated, user: authUser, isLoading: authLoading } = useSharedAuth();

  const chainId = useChainId();
  const { switchChain, isPending: isSwitching } = useSwitchChain();
  const { isConnected, address } = useAccount();
  const { disconnect } = useDisconnect();
  const [currentTheme, setCurrentTheme] = useState<'light' | 'dark'>('light');

  // Read wallet address directly from cookie on mount (before auth hydrates)
  useEffect(() => {
    setMounted(true);
    // Initialize theme from themeUtils
    setCurrentTheme(themeUtils.getTheme());
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
  const walletAddress = address || authUser?.wallet || cookieWallet;

  // Combined connection status: wagmi connected OR cookie-based auth OR have cookie wallet
  const isWalletConnected = isConnected || isAuthenticated || !!cookieWallet;

  const getCurrentChainName = () => {
    if (!mounted) return 'Chain';
    if (chainId === bsc.id) return 'BSC Mainnet';
    if (chainId === bscTestnet.id) return 'BSC Testnet';
    if (chainId === 31337) return 'Hardhat Local';
    return 'Unknown Chain';
  };

  const handleChainSwitch = async (targetChainId: number) => {
    if (!isConnected || isSwitching || targetChainId === chainId) return;
    try {
      await switchChain({ chainId: targetChainId });
    } catch (error) {
      console.error('Failed to switch chain:', error);
    }
  };

  const handleDisconnect = async () => {
    try {
      await logout();
      disconnect();
    } catch (error) {
      console.error('Disconnect error:', error);
    }
  };

  const formatAddress = (addr: string) => `${addr.slice(0, 6)}...${addr.slice(-4)}`;

  // Skeleton during SSR
  if (!mounted) {
    return (
      <header className="sticky top-0 z-40 border-b border-gray-200 dark:border-slate-700/50 bg-white/95 dark:bg-slate-900/95 backdrop-blur-xl">
        <div className="flex h-16 items-center justify-between px-6">
          <div className="flex items-center gap-3">
            <span className="text-xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">EPSX</span>
            <span className="text-xs text-gray-500 dark:text-slate-400 font-medium uppercase tracking-wider">Admin</span>
          </div>
          <div className="h-8 w-24 bg-gray-200 dark:bg-slate-700 rounded-lg animate-pulse" />
        </div>
      </header>
    );
  }

  return (
    <header className="sticky top-0 z-40 border-b border-gray-200 dark:border-slate-700/50 bg-white/95 dark:bg-slate-900/95 backdrop-blur-xl">
      <div className="flex h-16 items-center justify-between px-6">
        {/* Logo / Title */}
        <div className="flex items-center gap-3">
          <Link href="/" className="flex items-center gap-2 hover:opacity-80 transition-opacity">
            <span className="text-xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">EPSX</span>
            <span className="text-xs text-gray-500 dark:text-slate-400 font-medium uppercase tracking-wider">Admin</span>
          </Link>
        </div>

        {/* Right Actions */}
        <div className="flex items-center gap-3">
          {/* Notification Bell */}
          <AdminNotificationBell />

          {/* Theme Toggle */}
          <button
            type="button"
            onClick={() => {
              const newTheme = themeUtils.toggleTheme();
              setCurrentTheme(newTheme);
            }}
            className="p-2 rounded-lg text-orange-500 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors"
            title={`Switch to ${currentTheme === 'dark' ? 'light' : 'dark'} mode`}
            aria-label={`Switch to ${currentTheme === 'dark' ? 'light' : 'dark'} mode`}
          >
            {currentTheme === 'dark' ? (
              <Sun className="h-5 w-5" />
            ) : (
              <Moon className="h-5 w-5" />
            )}
          </button>

          {/* Chain Selector - Only in dev */}
          {!isProduction && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button
                  className="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-gray-600 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-800 hover:text-gray-900 dark:hover:text-white transition-colors"
                  disabled={isSwitching || !isWalletConnected}
                >
                  <LinkIcon className="h-4 w-4 text-orange-500" />
                  <span className="hidden md:inline">{getCurrentChainName()}</span>
                  <ChevronDown className="h-3 w-3 text-slate-400" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                align="end"
                className="w-48 bg-white dark:bg-slate-900 border-gray-200 dark:border-slate-700 p-1"
                style={{ zIndex: 99999 }}
              >
                <DropdownMenuItem
                  onClick={() => handleChainSwitch(bsc.id)}
                  disabled={chainId === bsc.id}
                  className={`flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer ${chainId === bsc.id ? 'bg-orange-500/20 text-orange-400' : 'text-gray-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-800'
                    }`}
                >
                  <div className="w-5 h-5 rounded-full bg-yellow-500 flex items-center justify-center text-[10px] font-bold text-white">56</div>
                  BSC Mainnet
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={() => handleChainSwitch(bscTestnet.id)}
                  disabled={chainId === bscTestnet.id}
                  className={`flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer ${chainId === bscTestnet.id ? 'bg-orange-500/20 text-orange-400' : 'text-gray-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-800'
                    }`}
                >
                  <div className="w-5 h-5 rounded-full bg-purple-500 flex items-center justify-center text-[10px] font-bold text-white">97</div>
                  BSC Testnet
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          )}

          {/* Wallet Connect */}
          {isWalletConnected && walletAddress ? (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="flex items-center gap-2 rounded-lg bg-gradient-to-r from-orange-500 to-yellow-500 px-4 py-2 text-sm font-medium text-white hover:opacity-90 transition-opacity">
                  <Wallet className="h-4 w-4" />
                  <span className="hidden md:inline">{formatAddress(walletAddress)}</span>
                  <ChevronDown className="h-3 w-3" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                align="end"
                className="w-52 bg-white dark:bg-slate-900 border-gray-200 dark:border-slate-700 p-1"
                style={{ zIndex: 99999 }}
              >
                <div className="px-3 py-2 text-xs text-gray-500 dark:text-slate-400 border-b border-gray-200 dark:border-slate-700 mb-1 font-mono">
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
                  className="flex items-center gap-2 rounded-lg bg-gradient-to-r from-orange-500 to-yellow-500 px-4 py-2 text-sm font-medium text-white hover:opacity-90 transition-opacity"
                >
                  <Wallet className="h-4 w-4" />
                  <span className="hidden md:inline">Connect Wallet</span>
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
