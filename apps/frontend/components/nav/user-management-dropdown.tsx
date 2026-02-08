'use client';

import { WalletConnectAuth } from '@/components/auth/WalletConnectauth';
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui';
import {
  BarChart3,
  CreditCard,
  Gift,
  Settings,
  User,
} from 'lucide-react';
import Link from 'next/link';
import { useEffect, useState } from 'react';

interface UserManagementDropdownProps {
  className?: string;
  compact?: boolean;
}

export function UserManagementDropdown({ className = '', compact = false }: UserManagementDropdownProps) {
  const [isHydrated, setIsHydrated] = useState(false);
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    setIsHydrated(true);
  }, []);

  if (!isHydrated) {
    return (
      <Button
        variant="ghost"
        size="icon"
        className={`h-10 w-10 rounded-full bg-orange-50 text-orange-500 hover:bg-orange-100 dark:bg-slate-800 dark:text-orange-400 dark:hover:bg-slate-700 ${className}`}
        disabled
      >
        <User className="h-4 w-4" />
      </Button>
    );
  }

  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className={`h-10 w-10 rounded-full bg-orange-50 text-orange-500 hover:bg-orange-100 dark:bg-slate-800 dark:text-orange-400 dark:hover:bg-slate-700 relative ${className}`}
        >
          <User className="h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent className="w-80 p-0" align="end" side="bottom" sideOffset={5}>
        {/* Header */}
        <div className="p-4 border-b">
          <div className="flex items-center gap-3">
            <div className="h-12 w-12 rounded-full bg-gradient-to-r from-orange-400 to-purple-500 flex items-center justify-center">
              <User className="h-6 w-6 text-white" />
            </div>
            <div className="flex-1">
              <div className="text-sm font-medium text-slate-700 dark:text-slate-300">
                User Menu
              </div>
              <div className="text-xs text-slate-500">
                Backend handles access control
              </div>
            </div>
          </div>
        </div>

        {/* Menu Items */}
        <div className="p-2">
          {[
            {
              icon: User,
              label: 'Profile',
              href: '/profile',
              description: 'Manage your account settings'
            },
            {
              icon: Settings,
              label: 'Account',
              href: '/account',
              description: 'Manage your account settings'
            },
            {
              icon: BarChart3,
              label: 'Analytics',
              href: '/analytics',
              description: 'View your trading data'
            },
            {
              icon: CreditCard,
              label: 'Billing',
              href: '/payment',
              description: 'Manage subscription and payments'
            },
            {
              icon: Gift,
              label: 'Notifications',
              href: '/notifications',
              description: 'Manage alerts and notifications'
            }
          ].map((item) => {
            const Icon = item.icon;

            return (
              <DropdownMenuItem
                key={item.href}
                asChild
                className="p-3 cursor-pointer"
              >
                <Link href={item.href}>
                  <div className="flex items-center gap-3 w-full">
                    <div className="h-8 w-8 rounded-lg bg-slate-100 dark:bg-slate-800 flex items-center justify-center">
                      <Icon className="h-4 w-4" />
                    </div>
                    <div className="flex-1">
                      <div className="text-sm font-medium">{item.label}</div>
                      <div className="text-xs text-slate-500">{item.description}</div>
                    </div>
                  </div>
                </Link>
              </DropdownMenuItem>
            );
          })}
        </div>

        <DropdownMenuSeparator />

        {/* Wallet Connect */}
        <div className="p-2">
          <WalletConnectAuth
            compact={false}
            className="w-full"
          />
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}