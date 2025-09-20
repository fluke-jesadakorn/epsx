'use client';

import { WalletConnectAuth } from '@/components/auth/WalletConnectAuth';
import {
  Bell,
  ChartNoAxesColumnIncreasing,
  ChevronDown,
  Code,
  Database,
  File,
  Info,
  LineChart,
  LogIn,
  LogOut,
  Menu,
  Moon,
  Settings,
  Sun,
  User,
  Wallet,
} from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';

import { UnifiedThemeToggle } from '@/components/ui';
import { NavbarSkeleton } from '@/components/nav/NavbarSkeleton';
import { NotificationBellSimple } from '@/components/notifications/NotificationBellSimple';
import {
  NavbarProvider,
  useNavbarContext,
} from '@/components/providers/NavbarProvider';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet';
import { type NotificationData } from '@/lib/actions/notifications';
import { navigationService } from '@/services/navigation.service';
import { type User } from '../../../../shared/types/auth';

interface NavigationClientProps {
  user: User | null;
  initialNotificationData?: NotificationData | null;
}

const iconMap = {
  docs: <File className="h-4 w-4" />,
  ranking: <LineChart className="h-4 w-4" />,
  analytics: <ChartNoAxesColumnIncreasing className="h-4 w-4" />,
  settings: <Settings className="h-4 w-4" />,
  'my-data': <Database className="h-4 w-4" />,
  developer: <Code className="h-4 w-4" />,
  about: <Info className="h-4 w-4" />,
  user: <User className="h-4 w-4" />,
  notification: <Bell className="h-4 w-4" />,
  theme: <Sun className="h-4 w-4" />,
  'theme-dark': <Moon className="h-4 w-4" />,
  login: <LogIn className="h-4 w-4" />,
  logout: <LogOut className="h-4 w-4" />,
  menu: <Menu className="h-4 w-4" />,
  profile: <User className="h-4 w-4" />,
};

export function NavigationClient({
  user,
  initialNotificationData,
}: NavigationClientProps) {
  return (
    <NavbarProvider>
      <NavigationContent
        user={user}
        initialNotificationData={initialNotificationData}
      />
    </NavbarProvider>
  );
}

function NavigationContent({
  user,
  initialNotificationData,
}: NavigationClientProps) {
  const pathname = usePathname();
  const [isOpen, setIsOpen] = useState(false);
  const { isHydrated } = useNavbarContext();

  // Use stable nav items to prevent hydration issues
  const navItems = navigationService.getNavItems(!!user);


  // Don't render navigation content until hydrated to prevent mismatch
  if (!isHydrated) {
    return (
      <header
        className="sticky top-0 z-50 border-b border-orange-100/50 bg-white/90 backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/95"
        suppressHydrationWarning
      >
        <div className="max-w-8xl mx-auto flex h-20 items-center px-6">
          {/* EPSX Logo - Skeleton */}
          <Link href="/" className="flex items-center mr-8 hover:opacity-80">
            <span className="text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
              EPSX
            </span>
          </Link>

          {/* Navigation Skeleton - matches hydrated version exactly */}
          <nav className="hidden items-center gap-2 lg:flex mr-8">
            {navItems.map(item => {
              const IconComponent = iconMap[item.key as keyof typeof iconMap];
              return (
                <Link
                  key={item.key}
                  href={item.href}
                  className="flex items-center gap-2 rounded-2xl px-4 py-2.5 text-sm font-medium text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200"
                >
                  <span className="text-orange-500">
                    {IconComponent || <Info className="h-4 w-4" />}
                  </span>
                  <span>{item.label}</span>
                </Link>
              );
            })}
          </nav>

          {/* Spacer */}
          <div className="flex-1" />

          {/* Right Actions Skeleton */}
          <div className="flex items-center gap-3">
            <NavbarSkeleton
              showNotifications={!!user}
              showUser={!!user?.email}
            />
          </div>
        </div>
      </header>
    );
  }

  return (
    <header
      className="sticky top-0 z-50 border-b border-orange-100/50 bg-white/90 backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/95"
      suppressHydrationWarning
    >
      <div className="max-w-8xl mx-auto flex h-20 items-center px-6">
        {/* EPSX Logo */}
        <Link href="/" className="flex items-center mr-8 hover:opacity-80">
          <span className="text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
            EPSX
          </span>
        </Link>

        {/* Main Navigation */}
        <nav className="hidden items-center gap-2 lg:flex mr-8">
          {navItems.map(item => {
            const IconComponent = iconMap[item.key as keyof typeof iconMap];
            return (
              <Link
                key={item.key}
                href={item.href}
                className={`flex items-center gap-2 rounded-2xl px-4 py-2.5 text-sm font-medium ${
                  pathname === item.href
                    ? 'border border-orange-200/50 bg-orange-50/80 text-orange-700 dark:border-orange-700/30 dark:bg-orange-900/20 dark:text-orange-300'
                    : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
                }`}
              >
                <span
                  className={
                    pathname === item.href
                      ? 'text-orange-600 dark:text-orange-300'
                      : 'text-orange-500'
                  }
                >
                  {IconComponent || <Info className="h-4 w-4" />}
                </span>
                {item.label}
              </Link>
            );
          })}
        </nav>

        {/* Spacer */}
        <div className="flex-1" />

        {/* Right Actions - Hidden on mobile */}
        <div className="hidden items-center gap-3 lg:flex">
          {/* Notifications */}
          {user && (
            <NotificationBellSimple
              className="!h-5 !w-5 !text-orange-500"
              initialData={initialNotificationData}
            />
          )}

          {/* Theme Toggle */}
          <UnifiedThemeToggle variant="minimal" />

          {/* Web3-First Authentication */}
          <WalletConnectAuth variant="compact" className="flex items-center gap-2" />

          {/* Profile Link - Show for both Web3 and traditional auth */}
          {user?.email && (
            <Link
              href="/profile"
              className="flex items-center gap-2 rounded-2xl px-3 py-2 text-slate-600 hover:bg-slate-50/80 hover:text-orange-600 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-orange-400"
            >
              <User className="h-5 w-5 text-orange-500" />
              <span className="text-sm font-medium">Profile</span>
            </Link>
          )}
        </div>

        {/* Mobile Menu */}
        <Sheet open={isOpen} onOpenChange={setIsOpen}>
          <SheetTrigger asChild className="lg:hidden">
            <button className="rounded-2xl bg-orange-50 p-3 text-orange-500 shadow-sm transition-all duration-300 hover:scale-105 hover:bg-orange-100 hover:shadow-md dark:bg-slate-800 dark:text-orange-400 dark:hover:bg-slate-700">
              <Menu className="h-5 w-5" />
            </button>
          </SheetTrigger>
          <SheetContent
            side="right"
            className="w-80 border-l border-orange-100/50 bg-white/95 backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/95"
          >
            <SheetHeader className="pb-6">
              <div className="flex items-center gap-3 mb-4">
                <Link href="/" onClick={() => setIsOpen(false)} className="flex items-center">
                  <span className="text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
                    EPSX
                  </span>
                </Link>
              </div>
              <SheetTitle className="text-xl font-bold text-slate-700 dark:text-slate-300">
                Navigation
              </SheetTitle>
            </SheetHeader>
            <div className="my-2 border-t border-orange-100 dark:border-slate-700" />
            <div className="flex flex-col gap-2">
              {navItems.map(item => {
                const IconComponent = iconMap[item.key as keyof typeof iconMap];
                return (
                  <Link
                    key={item.key}
                    href={item.href}
                    onClick={() => setIsOpen(false)}
                    className={`flex items-center gap-4 rounded-2xl px-4 py-3 text-sm font-medium ${
                      pathname === item.href
                        ? 'border border-orange-200/50 bg-orange-50/80 text-orange-700 dark:border-orange-700/30 dark:bg-orange-900/20 dark:text-orange-300'
                        : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
                    }`}
                  >
                    <span
                      className={
                        pathname === item.href
                          ? 'text-orange-600 dark:text-orange-300'
                          : 'text-orange-500'
                      }
                    >
                      {IconComponent || <Info className="h-4 w-4" />}
                    </span>
                    {item.label}
                  </Link>
                );
              })}

              <div className="my-4 border-t border-orange-100 dark:border-slate-700" />

              {/* Profile link for mobile */}
              {user && (
                <>
                  <Link
                    href="/profile"
                    onClick={() => setIsOpen(false)}
                    className={`flex items-center gap-4 rounded-2xl px-4 py-3 text-sm font-medium ${
                      pathname === '/profile'
                        ? 'border border-orange-200/50 bg-orange-50/80 text-orange-700 dark:border-orange-700/30 dark:bg-orange-900/20 dark:text-orange-300'
                        : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
                    }`}
                  >
                    <User className="h-4 w-4 text-orange-500" />
                    Profile
                  </Link>
                  <div className="my-2 border-t border-orange-100 dark:border-slate-700" />
                </>
              )}

              {/* Theme Toggle */}
              <div className="w-full">
                <UnifiedThemeToggle 
                  variant="minimal" 
                  showLabel={true} 
                  showTooltip={false}
                  className="w-full justify-start gap-4 rounded-2xl px-4 py-3 h-auto text-sm font-medium text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200"
                />
              </div>
              <div className="my-2 border-t border-orange-100 dark:border-slate-700" />

              {/* Web3-First Authentication in Mobile */}
              <div className="rounded-2xl border border-orange-200/50 bg-gradient-to-r from-orange-50/80 to-purple-50/80 p-4 dark:border-orange-700/40 dark:from-orange-900/20 dark:to-purple-900/20">
                <div className="flex items-center gap-2 mb-3">
                  <Wallet className="h-4 w-4 text-orange-500" />
                  <span className="text-sm font-medium text-orange-900 dark:text-orange-100">
                    Web3 Authentication
                  </span>
                </div>
                <WalletConnectAuth variant="default" className="w-full" />
              </div>

            </div>
          </SheetContent>
        </Sheet>
      </div>
    </header>
  );
}
