'use client';

/**
 * ADMIN HEADER
 * Minimal top navigation bar
 */

import { useTheme } from 'next-themes';

import { useEffect } from 'react';

import { AdminWalletConnectAuth } from '@/components/auth/admin-wallet-connect-auth';
import { AdminNotificationBell } from './admin-notification-bell-client';
import { Breadcrumb } from './breadcrumb';

import { themeUtils } from '@/components/ui/safe-theme-script';
import { ChainSelector } from '@/shared/components/navigation/chain-selector';
import { UnifiedThemeToggle } from '@/shared/components/ui/theme-toggle';
import { isProduction } from '@/shared/utils';
import type { Notification as ApiNotification } from '@/shared/api/notifications';

interface User {
  id: string;
  email: string;
  name?: string;
  role: string;
}

interface HeaderProps {
  user?: User;
  initialNotifications?: ApiNotification[];
  initialUnreadCount?: number;
}

/**
 *
 * @param root0
 * @param root0.user
 */
export function Header({ user: _user, initialNotifications, initialUnreadCount }: HeaderProps) {
  const { setTheme } = useTheme();

  useEffect(() => {
    const current = themeUtils.getTheme();
    setTheme(current);
  }, [setTheme]);

  return (
    <header className="sticky top-0 z-40 border-b border-border/40 bg-card">
      <div className="flex h-16 items-center justify-between px-6 gap-3">
        {/* Breadcrumb */}
        <div className="flex items-center gap-2 min-w-0 flex-shrink">
          <Breadcrumb />
        </div>

        {/* Right Actions */}
        <div className="flex items-center gap-3 flex-shrink-0">
          {/* Notification Bell */}
          <div className="hidden sm:block">
            <AdminNotificationBell initialNotifications={initialNotifications} initialUnreadCount={initialUnreadCount} />
          </div>

          <div className="w-[1px] h-6 bg-border hidden sm:block" />

          {/* Theme Toggle */}
          <UnifiedThemeToggle variant="minimal" size="md" showTooltip={true} />

          {/* Chain Selector - Only in dev */}
          {!isProduction && (
            <div className="hidden lg:block">
              <ChainSelector />
            </div>
          )}

          {/* Wallet Connect */}
          <AdminWalletConnectAuth className="rounded-2xl bg-[#1fc7d4] px-5 py-2 text-sm font-bold text-white shadow-lg shadow-cyan-500/10 hover:shadow-cyan-500/30 hover:scale-[1.02] active:scale-95 transition-all" />
        </div>
      </div>
    </header>
  );
}

export default Header;
