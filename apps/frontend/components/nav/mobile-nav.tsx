'use client';

import { Bell, ChevronRight, Menu, Wallet } from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';
import { useAccount } from 'wagmi';

import { WalletProviderIcon } from '@/components/nav/wallet-provider-icon';
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui';
import { formatAddress } from '@/shared/auth/utils';
import { useSharedAuth } from '@/shared/components/auth';
import { ChainSelector } from '@/shared/components/navigation/chain-selector';
import { isProduction } from '@/shared/utils';

import { type NavGroup, NAV_GROUPS, isGroupActive, isItemActive } from './nav-config';

function MobileGroup({
  group,
  onNavigate,
}: {
  group: NavGroup;
  onNavigate: () => void;
}) {
  const pathname = usePathname();
  const active = isGroupActive(group, pathname);
  const [open, setOpen] = useState(active);

  return (
    <div>
      <button
        onClick={() => setOpen(prev => !prev)}
        className={`w-full flex items-center justify-between px-3 py-2.5 rounded-md text-sm font-medium transition-colors ${active
            ? 'text-slate-900 dark:text-white'
            : 'text-slate-600 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white'
          }`}
      >
        <span className="flex items-center gap-2">
          {group.icon != null && <group.icon className="h-4 w-4 text-orange-500" />}
          {group.label}
        </span>
        <ChevronRight className={`h-4 w-4 text-orange-500 transition-transform ${open ? 'rotate-90' : ''}`} />
      </button>
      {open && (
        <div className="ml-3 border-l border-slate-200 dark:border-slate-700 pl-3 space-y-0.5">
          {group.items.map(item => {
            const Icon = item.icon;
            const itemActive = isItemActive(item, pathname);
            return (
              <Link
                key={item.key}
                href={item.href}
                onClick={onNavigate}
                className={`flex items-center gap-2.5 px-3 py-2 rounded-md text-sm transition-colors ${itemActive
                    ? 'bg-slate-100 text-slate-900 dark:bg-slate-800 dark:text-white'
                    : 'text-slate-500 hover:text-slate-900 hover:bg-slate-50 dark:text-slate-400 dark:hover:text-white dark:hover:bg-slate-800'
                  }`}
              >
                {Icon != null && <Icon className="h-4 w-4 shrink-0 text-orange-500" />}
                {item.label}
              </Link>
            );
          })}
        </div>
      )}
    </div>
  );
}

interface MobileNavProps {
  isAuthenticated: boolean;
}

export function MobileNav({ isAuthenticated }: MobileNavProps) {
  const [open, setOpen] = useState(false);
  const { isConnected, address } = useAccount();
  const { isAuthenticated: authStatus } = useSharedAuth();
  const close = () => setOpen(false);

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger asChild>
        <button
          className="lg:hidden flex items-center justify-center w-9 h-9 rounded-md hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          aria-label="Open menu"
        >
          <Menu className="h-5 w-5 text-orange-500" />
        </button>
      </SheetTrigger>

      <SheetContent
        side="right"
        className="w-[85vw] max-w-sm p-0 border-l border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900"
      >
        {/* Header */}
        <div className="flex items-center p-4 border-b border-slate-200 dark:border-slate-700">
          <img src="/epsx-logo.svg" alt="EPSX" className="h-8 w-auto" />
        </div>

        {/* Wallet card */}
        {isConnected && address != null && (
          <div className="p-4 border-b border-slate-100 dark:border-slate-800">
            <div className="flex items-center gap-3">
              <div className="flex items-center justify-center w-9 h-9 rounded-full bg-slate-100 dark:bg-slate-800">
                <Wallet className="h-4 w-4 text-orange-500" />
              </div>
              <div className="min-w-0 flex-1">
                <div className="text-sm font-medium text-slate-900 dark:text-white truncate">
                  {formatAddress(address)}
                </div>
                <div className={`text-xs ${authStatus ? 'text-emerald-500' : 'text-slate-400'} flex items-center gap-1`}>
                  {authStatus && <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />}
                  {authStatus ? 'Authenticated' : 'Connected'}
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Nav groups */}
        <div className="flex-1 overflow-y-auto p-4 space-y-1">
          {NAV_GROUPS.map(group => (
            <MobileGroup key={group.key} group={group} onNavigate={close} />
          ))}

          <div className="my-3 border-t border-slate-100 dark:border-slate-800" />

          {/* Chain selector (dev only) */}
          {!isProduction && (
            <div className="px-3 mb-3">
              <div className="text-xs font-medium text-slate-400 uppercase tracking-wider mb-2">Network</div>
              <ChainSelector compact={false} className="w-full" />
            </div>
          )}

          {/* Notifications link */}
          {isAuthenticated && (
            <Link
              href="/notifications"
              onClick={close}
              className="flex items-center gap-2.5 px-3 py-2.5 rounded-md text-sm font-medium text-slate-600 hover:text-slate-900 hover:bg-slate-50 dark:text-slate-400 dark:hover:text-white dark:hover:bg-slate-800 transition-colors"
            >
              <Bell className="h-4 w-4 text-orange-500" />
              Notifications
            </Link>
          )}
        </div>

        {/* Bottom wallet button */}
        <div className="p-4 border-t border-slate-200 dark:border-slate-700">
          <WalletProviderIcon compact={false} className="w-full" />
        </div>
      </SheetContent>
    </Sheet>
  );
}
