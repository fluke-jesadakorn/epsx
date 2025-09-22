'use client';

import { useState, useEffect } from 'react';
import { useProgressiveAuth } from '@/hooks/useProgressiveAuth';
import { AuthLevel } from '@/types/progressive-auth';
import { formatAddress } from '@/lib/auth/web3';
import {
  User,
  Settings,
  CreditCard,
  Shield,
  LogOut,
  Eye,
  Wallet,
  ChevronRight,
  Bell,
} from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import Link from 'next/link';

interface UserManagementDropdownProps {
  className?: string;
  compact?: boolean;
}

export function UserManagementDropdown({ className = '', compact = false }: UserManagementDropdownProps) {
  const [isHydrated, setIsHydrated] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const auth = useProgressiveAuth();

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

  const getAuthLevelInfo = () => {
    switch (auth.level) {
      case AuthLevel.PUBLIC:
        return {
          icon: <Eye className="h-4 w-4" />,
          label: 'Public',
          badge: 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400',
          description: 'Browse only',
        };
      case AuthLevel.CONNECTED:
        return {
          icon: <Wallet className="h-4 w-4" />,
          label: 'Connected',
          badge: 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400',
          description: 'Wallet connected',
        };
      case AuthLevel.AUTHENTICATED:
        return {
          icon: <Shield className="h-4 w-4" />,
          label: 'Authenticated',
          badge: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
          description: 'Full access',
        };
      default:
        return {
          icon: <Eye className="h-4 w-4" />,
          label: 'Unknown',
          badge: 'bg-slate-100 text-slate-600',
          description: '',
        };
    }
  };

  const levelInfo = getAuthLevelInfo();

  const getTriggerContent = () => {
    if (compact) {
      return (
        <div className="flex items-center gap-1">
          {levelInfo.icon}
          <div className={`w-2 h-2 rounded-full ${
            auth.level === AuthLevel.AUTHENTICATED ? 'bg-green-400' : 
            auth.level === AuthLevel.CONNECTED ? 'bg-blue-400' : 'bg-slate-400'
          }`} />
        </div>
      );
    }

    return (
      <div className="flex items-center gap-2">
        {levelInfo.icon}
        <div className="flex flex-col items-start">
          <span className="text-xs font-medium">{levelInfo.label}</span>
          <span className="text-xs text-slate-500 dark:text-slate-400">
            {levelInfo.description}
          </span>
        </div>
      </div>
    );
  };

  const menuItems = [
    // Always available
    {
      icon: <User className="h-4 w-4" />,
      label: 'Profile',
      href: '/profile',
      description: 'Account settings',
      minLevel: AuthLevel.PUBLIC,
    },
    {
      icon: <Settings className="h-4 w-4" />,
      label: 'Settings',
      href: '/settings',
      description: 'Preferences',
      minLevel: AuthLevel.PUBLIC,
    },
    
    // Connected wallet required
    {
      icon: <Wallet className="h-4 w-4" />,
      label: 'Wallet',
      href: '/wallet',
      description: 'Manage wallets',
      minLevel: AuthLevel.CONNECTED,
    },
    
    // Authenticated required
    {
      icon: <CreditCard className="h-4 w-4" />,
      label: 'Billing',
      href: '/billing',
      description: 'Plans & payments',
      minLevel: AuthLevel.AUTHENTICATED,
    },
    {
      icon: <Bell className="h-4 w-4" />,
      label: 'Notifications',
      href: '/notifications',
      description: 'Manage alerts',
      minLevel: AuthLevel.AUTHENTICATED,
    },
    {
      icon: <Shield className="h-4 w-4" />,
      label: 'Security',
      href: '/security',
      description: 'Auth & privacy',
      minLevel: AuthLevel.AUTHENTICATED,
    },
  ];

  const availableItems = menuItems.filter(item => auth.level >= item.minLevel);

  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size={compact ? "icon" : "sm"}
          className={`
            ${compact ? 'h-10 w-10 rounded-full' : 'h-10 px-3 rounded-lg'} 
            bg-orange-50 text-orange-500 hover:bg-orange-100 
            dark:bg-slate-800 dark:text-orange-400 dark:hover:bg-slate-700
            ${className}
          `}
        >
          {getTriggerContent()}
        </Button>
      </DropdownMenuTrigger>
      
      <DropdownMenuContent 
        align="end" 
        className="w-64 p-2 bg-white/95 backdrop-blur-xl border border-orange-100/50 dark:bg-slate-900/95 dark:border-slate-700/50"
      >
        {/* User Status Header */}
        <div className="px-3 py-3 mb-2">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-full bg-orange-100 dark:bg-orange-900/30">
              {levelInfo.icon}
            </div>
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-1">
                <span className="text-sm font-medium text-slate-900 dark:text-slate-100">
                  {levelInfo.label}
                </span>
                <Badge variant="secondary" className={`text-xs ${levelInfo.badge}`}>
                  {levelInfo.description}
                </Badge>
              </div>
              
              {auth.walletAddress && (
                <div className="text-xs text-slate-500 dark:text-slate-400 font-mono">
                  {formatAddress(auth.walletAddress)}
                </div>
              )}
              
              {auth.level === AuthLevel.PUBLIC && (
                <div className="text-xs text-slate-500 dark:text-slate-400">
                  Not signed in
                </div>
              )}
            </div>
          </div>
        </div>

        <DropdownMenuSeparator className="bg-orange-100/50 dark:bg-slate-700/50" />

        {/* Menu Items */}
        <div className="space-y-1">
          {availableItems.map((item) => (
            <DropdownMenuItem key={item.href} asChild>
              <Link
                href={item.href}
                onClick={() => setIsOpen(false)}
                className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-orange-50/80 dark:hover:bg-slate-800/40 w-full"
              >
                <span className="text-slate-400">{item.icon}</span>
                <div className="flex-1">
                  <div className="text-sm font-medium text-slate-900 dark:text-slate-100">
                    {item.label}
                  </div>
                  <div className="text-xs text-slate-500 dark:text-slate-400">
                    {item.description}
                  </div>
                </div>
                <ChevronRight className="h-3 w-3 text-slate-400" />
              </Link>
            </DropdownMenuItem>
          ))}
        </div>

        {/* Authentication Actions */}
        {auth.level !== AuthLevel.AUTHENTICATED && (
          <>
            <DropdownMenuSeparator className="bg-orange-100/50 dark:bg-slate-700/50" />
            <div className="px-3 py-2">
              <div className="text-xs text-slate-600 dark:text-slate-400 mb-2">
                Upgrade Access
              </div>
              
              {auth.level === AuthLevel.PUBLIC && (
                <Link
                  href="/auth/connect"
                  onClick={() => setIsOpen(false)}
                  className="flex items-center gap-2 text-xs text-blue-600 dark:text-blue-400 hover:underline"
                >
                  <Wallet className="h-3 w-3" />
                  Connect Wallet
                </Link>
              )}
              
              {auth.level === AuthLevel.CONNECTED && (
                <Link
                  href="/auth/signin"
                  onClick={() => setIsOpen(false)}
                  className="flex items-center gap-2 text-xs text-green-600 dark:text-green-400 hover:underline"
                >
                  <Shield className="h-3 w-3" />
                  Sign In for Full Access
                </Link>
              )}
            </div>
          </>
        )}

        {/* Logout - Only show if authenticated */}
        {auth.level === AuthLevel.AUTHENTICATED && (
          <>
            <DropdownMenuSeparator className="bg-orange-100/50 dark:bg-slate-700/50" />
            <DropdownMenuItem
              onClick={() => {
                // Handle logout logic here
                console.log('Logout clicked');
                setIsOpen(false);
              }}
              className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-red-50/80 dark:hover:bg-red-900/20 text-red-600 dark:text-red-400"
            >
              <LogOut className="h-4 w-4" />
              <span className="text-sm font-medium">Sign Out</span>
            </DropdownMenuItem>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}