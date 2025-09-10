'use client';

import { LogoutForm } from '@/components/auth/LogoutForm';
import {
  Bell,
  ChartNoAxesColumnIncreasing,
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
} from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';

import { ThemeToggleCSS } from '@/components/features/theme/ThemeToggleCSS';
import { NavbarSkeleton } from '@/components/nav/NavbarSkeleton';
import { NotificationBellSimple } from '@/components/notifications/NotificationBellSimple';
import {
  NavbarProvider,
  useNavbarContext,
} from '@/components/providers/NavbarProvider';
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet';
import { type NotificationData } from '@/lib/actions/notification-actions';
import { navigationService } from '@/services/navigation.service';

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  package_tier: string;
}

interface NavigationClientProps {
  user: AuthUser | null;
  initialNotificationData?: NotificationData | null;
}

const iconMap = {
  docs: <File className="h-4 w-4" />,
  ranking: <LineChart className="h-4 w-4" />,
  analytics: <ChartNoAxesColumnIncreasing className="h-4 w-4" />,
  settings: <Settings className="h-4 w-4" />,
  'my-data': <Database className="h-4 w-4" />,
  about: <Info className="h-4 w-4" />,
  user: <User className="h-4 w-4" />,
  notification: <Bell className="h-4 w-4" />,
  theme: <Sun className="h-4 w-4" />,
  'theme-dark': <Moon className="h-4 w-4" />,
  login: <LogIn className="h-4 w-4" />,
  logout: <LogOut className="h-4 w-4" />,
  menu: <Menu className="h-4 w-4" />,
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

  // Debug logging
  console.log('🔍 NavigationClient Debug:', {
    user: user ? 'LOGGED_IN' : 'NOT_LOGGED_IN',
    email: user?.email,
  });

  // Don't render navigation content until hydrated to prevent mismatch
  if (!isHydrated) {
    return (
      <header
        className="sticky top-0 z-50 border-b border-orange-100/50 bg-white/90 backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/95"
        suppressHydrationWarning
      >
        <div className="max-w-8xl mx-auto flex h-20 items-center px-6">
          {/* Navigation Skeleton - matches hydrated version exactly */}
          <nav className="hidden items-center gap-2 lg:flex">
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
        {/* Logo Section */}
        <div className="mr-8 flex items-center">
          <Link href="/" className="group flex items-center gap-3">
            <span className="bg-gradient-to-r from-orange-500 via-yellow-500 to-pink-500 bg-clip-text text-2xl font-bold text-transparent">
              EPSX
            </span>
          </Link>
        </div>

        {/* Main Navigation */}
        <nav className="hidden items-center gap-2 lg:flex">
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

        {/* Right Actions */}
        <div className="flex items-center gap-3">
          {/* Notifications */}
          {user && (
            <div className="relative flex cursor-pointer items-center gap-2">
              <NotificationBellSimple
                className="!h-5 !w-5 !text-orange-500"
                initialData={initialNotificationData}
              />
              <span className="text-sm font-medium text-slate-600 dark:text-slate-300">
                Notification
              </span>
              {initialNotificationData &&
                initialNotificationData.unread_count > 0 && (
                  <div className="absolute -top-1 left-3 flex h-5 w-5 items-center justify-center rounded-full bg-red-500/90">
                    <span className="text-xs font-bold text-white">
                      {initialNotificationData.unread_count > 99
                        ? '99+'
                        : initialNotificationData.unread_count}
                    </span>
                  </div>
                )}
            </div>
          )}

          {/* User Profile */}
          {user?.email && (
            <Link
              href="/settings"
              className="flex items-center gap-2 text-slate-600 dark:text-slate-300"
            >
              <User className="h-5 w-5 text-orange-500" />
              <span className="text-sm font-medium">User Info</span>
            </Link>
          )}

          {/* Theme Toggle and Auth Actions Group */}
          <div className="flex items-center gap-2">
            {/* Theme Toggle */}
            <ThemeToggleCSS />

            {/* Auth Actions */}
            <div className="hidden md:block">
              {user ? (
                <LogoutForm
                  variant="ghost"
                  className="rounded-2xl border border-slate-200/50 bg-slate-50/80 px-4 py-2.5 text-sm font-medium text-slate-600 hover:border-slate-300/60 hover:bg-slate-100/80 dark:border-slate-700/40 dark:bg-slate-800/40 dark:text-slate-300 dark:hover:bg-slate-700/60"
                />
              ) : (
                <Link
                  href="/login"
                  className="flex items-center gap-2 rounded-2xl border border-slate-200/50 bg-slate-50/80 px-4 py-2.5 text-sm font-medium text-slate-600 hover:border-slate-300/60 hover:bg-slate-100/80 dark:border-slate-700/40 dark:bg-slate-800/40 dark:text-slate-300 dark:hover:bg-slate-700/60"
                >
                  <LogIn className="h-4 w-4 text-orange-500" />
                  Connect
                </Link>
              )}
            </div>
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
                <SheetTitle className="bg-gradient-to-r from-orange-500 to-pink-500 bg-clip-text text-xl font-bold text-transparent">
                  Navigation
                </SheetTitle>
              </SheetHeader>
              <div className="flex flex-col gap-2">
                {navItems.map(item => {
                  const IconComponent =
                    iconMap[item.key as keyof typeof iconMap];
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

                {user ? (
                  <LogoutForm
                    variant="ghost"
                    className="w-full justify-start rounded-2xl border border-slate-200/50 bg-slate-50/80 px-4 py-3 font-medium text-slate-600 hover:border-slate-300/60 hover:bg-slate-100/80 dark:border-slate-700/40 dark:bg-slate-800/40 dark:text-slate-300 dark:hover:bg-slate-700/60"
                  />
                ) : (
                  <Link
                    href="/login"
                    onClick={() => setIsOpen(false)}
                    className="flex items-center gap-4 rounded-2xl border border-slate-200/50 bg-slate-50/80 px-4 py-3 text-sm font-medium text-slate-600 hover:border-slate-300/60 hover:bg-slate-100/80 dark:border-slate-700/40 dark:bg-slate-800/40 dark:text-slate-300 dark:hover:bg-slate-700/60"
                  >
                    <LogIn className="h-4 w-4 text-orange-500" />
                    Connect
                  </Link>
                )}
              </div>
            </SheetContent>
          </Sheet>
        </div>
      </div>
    </header>
  );
}
