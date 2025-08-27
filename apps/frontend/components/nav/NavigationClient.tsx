'use client';

import { LogoutForm } from '@/components/auth/LogoutForm';
import { Badge, Button } from '@/components/ui';
import {
  BarChart,
  Crown,
  Database,
  File,
  LineChart,
  LogIn,
  Menu,
  Settings,
  User,
} from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useEffect, useState } from 'react';

import ThemeToggle from '@/components/features/theme/ThemeToggle';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { NavigationMenu } from '@/components/ui/navigation-menu';
// Notifications completely disabled to fix webpack bundling issues
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { navigationService } from '@/services/navigation.service';
import { formatLevelAsNumber, getLevelColor } from '@/utils/env';

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  package_tier: string;
}

interface NavigationClientProps {
  user: AuthUser | null;
}

const iconMap = {
  docs: <File className="h-4 w-4" />,
  ranking: <LineChart className="h-4 w-4" />,
  analytics: <BarChart className="h-4 w-4" />,
  settings: <Settings className="h-4 w-4" />,
  'my-data': <Database className="h-4 w-4" />,
};

export function NavigationClient({ user }: NavigationClientProps) {
  const pathname = usePathname();
  const [isOpen, setIsOpen] = useState(false);
  const [isMounted, setIsMounted] = useState(false);

  const userLevel = user?.package_tier || 'free';
  const userEmail = user?.email;

  // Prevent hydration mismatch by only rendering after mount
  useEffect(() => {
    setIsMounted(true);
  }, []);

  // Don't render until mounted
  if (!isMounted) {
    return (
      <div className="relative z-50 border-b bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10 backdrop-blur-sm">
        <div className="mx-auto flex h-24 max-w-7xl items-center justify-between px-4 sm:px-6">
          <div className="flex items-center gap-4 sm:gap-8">
            <Link href="/" className="group flex items-center gap-2">
              <span className="bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-2xl font-bold text-transparent transition-transform duration-300 group-hover:scale-105 sm:text-3xl">
                EPSX
              </span>
            </Link>
          </div>
          <div className="flex items-center gap-4 md:gap-6">
            <Button
              variant="ghost"
              disabled
              className="focus-visible:ring-ring hover:bg-primary/10 hover:text-accent-foreground text-muted-foreground hover:text-primary flex flex-col items-center justify-center gap-2 rounded-full px-4 py-2 text-xs font-medium transition-all duration-200 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50"
            >
              <span className="relative block">
                <Settings className="h-4 w-4" />
              </span>
              <span className="mt-1">Theme</span>
            </Button>
          </div>
        </div>
      </div>
    );
  }

  const navItems = navigationService.getNavItems(!!user);

  return (
    <div className="relative z-50 border-b bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10 backdrop-blur-sm">
      <div className="mx-auto flex h-24 max-w-7xl items-center justify-between px-4 sm:px-6">
        <div className="flex items-center gap-4 sm:gap-8">
          <Link href="/" className="group flex items-center gap-2">
            <span className="bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-2xl font-bold text-transparent transition-transform duration-300 group-hover:scale-105 sm:text-3xl">
              EPSX
            </span>
          </Link>
          {/* Tablet Navigation Pills (md breakpoint) */}
          <nav className="bg-muted/50 hidden gap-1 rounded-full p-1 md:flex lg:hidden">
            {navItems.slice(0, 4).map(item => (
              <Link
                key={item.key}
                href={item.href}
                className={`relative flex h-10 w-10 items-center justify-center rounded-full transition-all duration-200 hover:scale-110 active:scale-95 ${
                  pathname === item.href
                    ? 'bg-primary text-primary-foreground shadow-lg'
                    : 'hover:bg-muted text-muted-foreground hover:text-foreground'
                } `}
                title={item.label}
              >
                {iconMap[item.key as keyof typeof iconMap]}
                {/* Active indicator */}
                {pathname === item.href && (
                  <div className="bg-primary absolute -bottom-2 left-1/2 h-1 w-1 -translate-x-1/2 transform rounded-full" />
                )}
              </Link>
            ))}

            {/* More items indicator for tablet */}
            {navItems.length > 4 && (
              <button className="hover:bg-muted text-muted-foreground flex h-10 w-10 items-center justify-center rounded-full">
                <svg
                  className="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <circle cx="12" cy="12" r="1" />
                  <circle cx="19" cy="12" r="1" />
                  <circle cx="5" cy="12" r="1" />
                </svg>
              </button>
            )}
          </nav>

          {/* Desktop Navigation (lg+ breakpoint) */}
          <nav className="hidden gap-2 lg:flex">
            {navItems.map(item => (
              <Link
                key={item.key}
                href={item.href}
                className={`focus-visible:ring-ring hover:bg-primary/10 hover:text-accent-foreground flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-all duration-200 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 ${
                  pathname === item.href
                    ? 'bg-primary/10 text-primary'
                    : 'text-muted-foreground hover:text-primary'
                } `}
              >
                <span className="flex items-center justify-center">
                  {iconMap[item.key as keyof typeof iconMap]}
                </span>
                <span className="hidden xl:block">{item.label}</span>
              </Link>
            ))}
          </nav>
          <NavigationMenu className="hidden">
            {/* Keep for structure, but hidden since we use custom nav above */}
          </NavigationMenu>
        </div>

        <div className="flex items-center gap-2 sm:gap-3 lg:gap-4">
          {/* Notifications - Only for authenticated users */}
          {user && (
            <div className="relative">
              {/* Notifications disabled due to webpack bundling issues */}
            </div>
          )}
          
          {/* Theme Toggle - Responsive */}
          <div className="hidden sm:block">
            <ThemeToggle />
          </div>

          {/* User Avatar & Settings - Tablet Enhanced */}
          {user?.email && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Link
                    href="/settings"
                    className="hover:bg-muted/50 flex items-center gap-2 rounded-lg p-1 transition-all duration-200 hover:scale-105 md:p-2"
                  >
                    <Avatar className="border-primary/20 h-8 w-8 border-2 md:h-10 md:w-10">
                      <AvatarFallback>
                        <User className="h-4 w-4 md:h-5 md:w-5" />
                      </AvatarFallback>
                    </Avatar>
                    {/* Desktop: Full email, Tablet: Shortened, Mobile: Hidden */}
                    <div className="hidden lg:block">
                      <div className="text-foreground text-sm font-medium">
                        {userEmail}
                      </div>
                      <div className="text-muted-foreground text-xs">
                        Settings
                      </div>
                    </div>
                    <div className="hidden md:block lg:hidden">
                      <div className="text-foreground text-sm font-medium">
                        {userEmail.split('@')[0]}
                      </div>
                    </div>
                  </Link>
                </TooltipTrigger>
                <TooltipContent>
                  <div className="text-center">
                    <p className="font-medium">{userEmail}</p>
                    <p className="text-muted-foreground text-xs">
                      Click to open settings
                    </p>
                  </div>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}

          {/* Mobile/Tablet Menu */}
          <Sheet open={isOpen} onOpenChange={setIsOpen}>
            <SheetTrigger asChild className="md:hidden">
              <Button
                variant="ghost"
                size="icon"
                className="bg-background h-8 w-8 shadow-md transition-all hover:scale-110 active:scale-95 sm:h-10 sm:w-10"
              >
                <Menu className="h-4 w-4 sm:h-5 sm:w-5" />
              </Button>
            </SheetTrigger>
            <SheetContent side="right" className="w-[280px] sm:w-[400px]">
              <SheetHeader>
                <SheetTitle className="text-left">Menu</SheetTitle>
              </SheetHeader>
              <div className="mt-6 flex flex-col gap-4">
                {/* Notifications - Mobile */}
                {user && (
                  <Link
                    href="/notifications"
                    onClick={() => setIsOpen(false)}
                    className="bg-primary/5 flex items-center justify-between rounded-lg p-3 hover:bg-primary/10 transition-colors"
                  >
                    <span className="text-sm font-medium">Notifications</span>
                    {/* Notifications disabled due to webpack bundling issues */}
                  </Link>
                )}
                
                {/* Theme Toggle - Mobile */}
                <div className="bg-primary/5 flex items-center justify-between rounded-lg p-3">
                  <span className="text-sm font-medium">Theme</span>
                  <ThemeToggle />
                </div>

                {/* User Level Display - Mobile */}
                {user && (
                  <div className="bg-primary/5 flex items-center justify-center gap-2 rounded-lg p-3">
                    <Badge
                      variant="secondary"
                      className={`${getLevelColor(userLevel)} border-current bg-current/10 px-3 py-1 text-sm font-bold text-current`}
                    >
                      <Crown className="mr-1 h-3 w-3" />
                      {formatLevelAsNumber(userLevel)}
                    </Badge>
                  </div>
                )}

                {navItems.map(item => (
                  <Link
                    key={item.key}
                    href={item.href}
                    onClick={() => setIsOpen(false)}
                    className={`focus-visible:ring-ring hover:bg-primary/10 hover:text-accent-foreground flex flex-col items-center gap-1 rounded-full px-4 py-2 text-xs font-medium transition-all duration-200 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 ${
                      pathname === item.href
                        ? 'bg-primary/10 text-primary'
                        : 'text-muted-foreground hover:text-primary'
                    }`}
                  >
                    {iconMap[item.key as keyof typeof iconMap]}
                    <span className="mt-1">{item.label}</span>
                  </Link>
                ))}

                {user ? (
                  <div onClick={() => setIsOpen(false)} className="mt-4">
                    <LogoutForm
                      className="focus-visible:ring-ring hover:bg-primary/10 hover:text-accent-foreground text-muted-foreground hover:text-primary flex flex-col items-center gap-1 rounded-full px-4 py-2 text-xs font-medium transition-all duration-200 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50"
                      variant="ghost"
                    />
                  </div>
                ) : (
                  <Link
                    href="/login"
                    onClick={() => setIsOpen(false)}
                    className="focus-visible:ring-ring hover:bg-primary/10 hover:text-accent-foreground text-muted-foreground hover:text-primary mt-4 flex flex-col items-center gap-1 rounded-full px-4 py-2 text-xs font-medium transition-all duration-200 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50"
                  >
                    <span className="block">
                      <LogIn className="h-4 w-4" />
                    </span>
                    <span className="mt-1">Login</span>
                  </Link>
                )}
              </div>
            </SheetContent>
          </Sheet>

          {/* Desktop/Large Tablet Login/Logout */}
          <div className="hidden lg:block">
            {user ? (
              <LogoutForm
                className="focus-visible:ring-ring hover:bg-primary/10 hover:text-accent-foreground text-muted-foreground hover:text-primary flex items-center justify-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-all duration-200 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 lg:px-4"
                variant="ghost"
              />
            ) : (
              <Link href="/login">
                <Button
                  variant="ghost"
                  className="focus-visible:ring-ring hover:bg-primary/10 hover:text-accent-foreground text-muted-foreground hover:text-primary flex items-center justify-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-all duration-200 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 lg:px-4"
                >
                  <LogIn className="h-4 w-4" />
                  <span className="hidden xl:block">Login</span>
                </Button>
              </Link>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
