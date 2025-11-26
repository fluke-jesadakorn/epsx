'use client';

import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useState, useEffect } from 'react';
import { useAccount, useDisconnect } from 'wagmi';

import { ThemeToggle } from '@/components/ui/ThemeToggle';
import { useSharedAuth } from '@/shared/components/auth/Provider';
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

/**
 *
 * @param root0
 * @param root0.user
 */
export function Header({ user }: HeaderProps) {
  const [showUserMenu, setShowUserMenu] = useState(false);
  const [mounted, setMounted] = useState(false);
  const { logout } = useSharedAuth();
  
  // Only use WAGMI hooks after component mounts to avoid SSR issues
  let address: string | undefined;
  let isConnected = false;
  let disconnect: (() => void) | undefined;

  try {
    // eslint-disable-next-line react-hooks/rules-of-hooks -- Defensive pattern for when WAGMI is not available
    const account = useAccount();
    // eslint-disable-next-line react-hooks/rules-of-hooks -- Defensive pattern for when WAGMI is not available
    const disconnectHook = useDisconnect();
    
    if (mounted) {
      address = account.address;
      isConnected = account.isConnected;
      disconnect = disconnectHook.disconnect;
    }
  } catch (_error) {
    // WAGMI hooks not available, continue with defaults
    // eslint-disable-next-line no-console
    console.warn('WAGMI hooks not available:', _error);
  }
  
  useEffect(() => {
    setMounted(true);
  }, []);

  const handleWalletDisconnect = async () => {
    try {
      // Logout from backend using SharedOpenIDWeb3Provider
      await logout();

      // Disconnect wallet if available
      if (disconnect) {
        disconnect();
      }

      setShowUserMenu(false);
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Disconnect error:', _error);
    }
  };

  const formatAddress = (addr: string) => {
    return `${addr.slice(0, 3)}...${addr.slice(-3)}`;
  };

  return (
    <header className="sticky top-0 z-40 border-b border-yellow-200/50 bg-gradient-to-r from-white via-yellow-50 to-orange-50 shadow-lg backdrop-blur-sm dark:border-slate-700/50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">
      <div className="flex h-16 items-center justify-between px-6">
        {/* Search Section */}
        <div className="flex flex-1 items-center gap-6">
          <div className="relative w-full max-w-md">
            <input
              type="search"
              placeholder="Search users, permissions..."
              className="h-12 w-full rounded-2xl border-2 border-yellow-200/50 bg-gradient-to-r from-white to-yellow-50 pr-4 pl-12 text-gray-900 shadow-lg placeholder:text-gray-500 focus:border-orange-400 focus:ring-2 focus:ring-orange-200 focus:outline-none dark:border-slate-600/50 dark:from-slate-800 dark:to-slate-700 dark:text-slate-100 dark:placeholder:text-slate-400 dark:focus:border-slate-500 dark:focus:ring-slate-500/20"
            />
            <div className="absolute top-1/2 left-4 -translate-y-1/2">
              <span className="text-xl">🔍</span>
            </div>
          </div>

          {/* Quick Actions */}
          <div className="hidden items-center gap-3 md:flex">
            <button className="h-12 rounded-2xl bg-gradient-to-r from-green-400 to-teal-500 px-4 font-semibold text-white shadow-lg">
              <span className="mr-2">➕</span>
              Add User
            </button>
            <button className="h-12 rounded-2xl bg-gradient-to-r from-blue-400 to-purple-500 px-4 font-semibold text-white shadow-lg">
              <span className="mr-2">🔑</span>
              Grant Access
            </button>
          </div>
        </div>

        {/* Right Section */}
        <div className="flex items-center gap-4">
          {/* Notifications - Real data from backend */}
          <AdminNotificationBell />

          {/* Theme Toggle */}
          <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-r from-purple-400 to-pink-500 shadow-lg">
            <ThemeToggle />
          </div>

          {/* User Menu / Connect Wallet */}
          <div className="relative">
            {!mounted ? (
              // Loading state during hydration
              <div className="flex h-12 items-center gap-3 rounded-2xl bg-gradient-to-r from-gray-400 to-gray-500 pr-5 pl-4 font-semibold text-white shadow-lg">
                <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-white/20">
                  <span className="text-lg">⏳</span>
                </div>
                <span className="hidden md:block">Loading...</span>
              </div>
            ) : isConnected ? (
              // Show Admin User Menu when connected
              <button
                onClick={() => setShowUserMenu(!showUserMenu)}
                className="flex h-12 items-center gap-3 rounded-2xl bg-gradient-to-r from-yellow-400 to-orange-500 pr-5 pl-4 font-semibold text-white shadow-lg"
              >
                <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-white/20">
                  <span className="text-lg">👤</span>
                </div>
                <span className="hidden md:block">
                  {isConnected && address ? formatAddress(address) : (user?.name || 'Admin')}
                </span>
                <span className="text-sm">↓</span>
              </button>
            ) : (
              // Show Connect Wallet button when disconnected
              <ConnectButton.Custom>
                {({ openConnectModal }) => (
                  <button
                    onClick={openConnectModal}
                    className="flex h-12 items-center gap-3 rounded-2xl bg-gradient-to-r from-blue-400 to-indigo-500 pr-5 pl-4 font-semibold text-white shadow-lg"
                  >
                    <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-white/20">
                      <span className="text-lg">🔗</span>
                    </div>
                    <span className="hidden md:block">Connect Wallet</span>
                  </button>
                )}
              </ConnectButton.Custom>
            )}

            {showUserMenu && mounted && isConnected && (
              <div className="absolute top-14 right-0 z-50 w-64 rounded-3xl border border-yellow-200 bg-white p-4 shadow-2xl dark:border-slate-700/50 dark:bg-slate-800">
                <div className="space-y-3">
                  <div className="rounded-2xl bg-gradient-to-r from-yellow-50 to-orange-50 p-4 dark:from-yellow-900/20 dark:to-orange-900/20">
                    <div className="flex items-center gap-3">
                      <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-r from-yellow-400 to-orange-500">
                        <span className="text-lg text-white">🔗</span>
                      </div>
                      <div>
                        <div className="font-semibold text-gray-900 dark:text-white">
                          Wallet Connected
                        </div>
                        <div className="text-sm text-gray-600 dark:text-gray-400">
                          {address ? formatAddress(address) : 'No address'}
                        </div>
                        <div className="bg-gradient-to-r from-yellow-600 to-orange-600 bg-clip-text text-xs font-semibold text-transparent">
                          Admin Access
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="space-y-1">
                    <button className="flex w-full items-center gap-3 rounded-2xl p-3 text-left hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 dark:hover:from-gray-700 dark:hover:to-gray-600">
                      <span>👤</span>
                      <span>Profile Settings</span>
                    </button>
                    <button className="flex w-full items-center gap-3 rounded-2xl p-3 text-left hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 dark:hover:from-gray-700 dark:hover:to-gray-600">
                      <span>🔒</span>
                      <span>Security</span>
                    </button>
                    <button className="flex w-full items-center gap-3 rounded-2xl p-3 text-left hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 dark:hover:from-gray-700 dark:hover:to-gray-600">
                      <span>❓</span>
                      <span>Help & Support</span>
                    </button>
                  </div>

                  {/* Logout Section */}
                  <div className="border-t border-gray-200 pt-3 dark:border-gray-600">
                    <button
                      onClick={handleWalletDisconnect}
                      className="flex w-full items-center justify-center gap-2 rounded-2xl bg-gradient-to-r from-orange-400 to-red-500 p-3 font-semibold text-white"
                    >
                      <span>🔌</span>
                      <span>Disconnect Wallet</span>
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </header>
  );
}
