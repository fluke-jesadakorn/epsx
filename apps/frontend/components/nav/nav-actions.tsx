'use client';

import { WalletProviderIcon } from '@/components/nav/wallet-provider-icon';
import { NotificationBellClient } from '@/components/notifications/notification-bell-client';
import { ChainSelector } from '@/shared/components/navigation/chain-selector';
import { UnifiedThemeToggle } from '@/shared/components/ui/theme-toggle';

import { MobileNav } from './mobile-nav';

interface NavActionsProps {
  isAuthenticated: boolean;
}

export function NavActions({ isAuthenticated }: NavActionsProps) {
  return (
    <div className="flex items-center gap-2">
      {/* Desktop actions */}
      <div className="hidden md:flex items-center gap-1.5">
        {isAuthenticated && <NotificationBellClient />}
        <UnifiedThemeToggle variant="minimal" size="md" showTooltip={true} />
        <ChainSelector />
        <WalletProviderIcon compact={false} />
      </div>

      {/* Tablet: compact wallet */}
      <div className="hidden sm:flex md:hidden items-center gap-1.5">
        {isAuthenticated && <NotificationBellClient />}
        <UnifiedThemeToggle variant="minimal" size="md" showTooltip={true} />
        <WalletProviderIcon compact />
      </div>

      {/* Mobile hamburger */}
      <MobileNav isAuthenticated={isAuthenticated} />
    </div>
  );
}
