'use client';

/**
 * ADMIN HEADER
 * Top navigation bar matching frontend design
 * Works alongside Sidebar for admin layout
 */

import { useTheme } from 'next-themes';
import Link from 'next/link';
import { useEffect, useState } from 'react';

import { AdminNotificationBell } from './AdminNotificationBellClient';
import { AdminWalletConnectAuth } from '@/components/auth/AdminWalletConnectAuth';

import { themeUtils } from '@/components/ui/SafeThemeScript';
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
  const { theme, resolvedTheme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  // Read wallet address directly from cookie on mount (before auth hydrates)
  useEffect(() => {
    setMounted(true);
    // Sync theme on mount if needed
    const current = themeUtils.getTheme();
    if (current && (resolvedTheme ?? theme) !== current) {
      setTheme(current);
    }
  }, [theme, resolvedTheme, setTheme]);

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
          <AdminWalletConnectAuth className="rounded-lg bg-primary px-2 sm:px-3 lg:px-4 py-1.5 sm:py-2 text-xs sm:text-sm font-medium hover:opacity-90 transition-opacity" />
        </div>
      </div>
    </header>
  );
}

export default Header;
