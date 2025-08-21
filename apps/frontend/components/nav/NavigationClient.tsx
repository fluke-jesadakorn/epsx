'use client';

import {
  BarChart,
  Crown,
  Database,
  File,
  LineChart,
  LogIn,
  LogOut,
  Menu,
  Settings,
  User,
} from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useEffect, useState } from 'react';
import { LogoutForm } from '@/components/auth/LogoutForm';
import { Badge, Button } from '@/components/ui';

import ThemeToggle from '@/components/features/theme/ThemeToggle';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { NavigationMenu } from '@/components/ui/navigation-menu';
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
      <div className="relative z-50 bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10 border-b backdrop-blur-sm">
        <div className="flex h-24 items-center px-4 sm:px-6 justify-between max-w-7xl mx-auto">
          <div className="flex items-center gap-4 sm:gap-8">
            <Link href="/" className="flex items-center gap-2 group">
              <span className="text-2xl sm:text-3xl font-bold bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent group-hover:scale-105 transition-transform duration-300">
                EPSX
              </span>
            </Link>
          </div>
          <div className="flex items-center gap-4 md:gap-6">
            <Button
              variant="ghost"
              disabled
              className="flex flex-col items-center justify-center gap-2 rounded-full px-4 py-2 text-xs font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 text-muted-foreground hover:text-primary"
            >
              <span className="block relative">
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
    <div className="relative z-50 bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10 border-b backdrop-blur-sm">
      <div className="flex h-24 items-center px-4 sm:px-6 justify-between max-w-7xl mx-auto">
        <div className="flex items-center gap-4 sm:gap-8">
          <Link href="/" className="flex items-center gap-2 group">
            <span className="text-2xl sm:text-3xl font-bold bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent group-hover:scale-105 transition-transform duration-300">
              EPSX
            </span>
          </Link>
          {/* Tablet Navigation Pills (md breakpoint) */}
          <nav className="hidden md:flex lg:hidden gap-1 bg-muted/50 rounded-full p-1">
            {navItems.slice(0, 4).map((item) => (
              <Link
                key={item.key}
                href={item.href}
                className={`
                  relative flex items-center justify-center w-10 h-10 rounded-full
                  transition-all duration-200 hover:scale-110 active:scale-95
                  ${pathname === item.href
                    ? 'bg-primary text-primary-foreground shadow-lg'
                    : 'hover:bg-muted text-muted-foreground hover:text-foreground'
                  }
                `}
                title={item.label}
              >
                {iconMap[item.key as keyof typeof iconMap]}
                {/* Active indicator */}
                {pathname === item.href && (
                  <div className="absolute -bottom-2 left-1/2 transform -translate-x-1/2 w-1 h-1 bg-primary rounded-full" />
                )}
              </Link>
            ))}
            
            {/* More items indicator for tablet */}
            {navItems.length > 4 && (
              <button className="flex items-center justify-center w-10 h-10 rounded-full hover:bg-muted text-muted-foreground">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <circle cx="12" cy="12" r="1"/>
                  <circle cx="19" cy="12" r="1"/>
                  <circle cx="5" cy="12" r="1"/>
                </svg>
              </button>
            )}
          </nav>

          {/* Desktop Navigation (lg+ breakpoint) */}
          <nav className="hidden lg:flex gap-2">
            {navItems.map((item) => (
              <Link
                key={item.key}
                href={item.href}
                className={`
                  flex items-center justify-center gap-2 rounded-lg px-4 py-2 
                  text-sm font-medium transition-all duration-200 
                  focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 
                  hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] 
                  disabled:pointer-events-none disabled:opacity-50 
                  ${pathname === item.href
                    ? 'bg-primary/10 text-primary'
                    : 'text-muted-foreground hover:text-primary'
                  }
                `}
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
          {/* Theme Toggle - Responsive */}
          <div className="hidden sm:block">
            <ThemeToggle />
          </div>

          {/* User Level Display - Enhanced for Tablet */}
          {user && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex items-center gap-2">
                    <Badge
                      variant="secondary"
                      className={`
                        ${getLevelColor(userLevel)} border-current bg-current/10 text-current 
                        font-bold text-xs px-2 py-1 md:px-3 md:py-1.5
                        transition-all duration-200 hover:scale-105
                      `}
                    >
                      <Crown className="h-3 w-3 mr-1" />
                      <span className="hidden md:inline">Level </span>
                      {formatLevelAsNumber(userLevel)}
                    </Badge>
                  </div>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Your current level: {formatLevelAsNumber(userLevel)}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}

          {/* User Avatar & Settings - Tablet Enhanced */}
          {user?.email && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Link 
                    href="/settings" 
                    className="flex items-center gap-2 hover:bg-muted/50 rounded-lg p-1 md:p-2 transition-all duration-200 hover:scale-105"
                  >
                    <Avatar className="h-8 w-8 md:h-10 md:w-10 border-2 border-primary/20">
                      <AvatarFallback>
                        <User className="h-4 w-4 md:h-5 md:w-5" />
                      </AvatarFallback>
                    </Avatar>
                    {/* Desktop: Full email, Tablet: Shortened, Mobile: Hidden */}
                    <div className="hidden lg:block">
                      <div className="text-sm font-medium text-foreground">{userEmail}</div>
                      <div className="text-xs text-muted-foreground">Settings</div>
                    </div>
                    <div className="hidden md:block lg:hidden">
                      <div className="text-sm font-medium text-foreground">
                        {userEmail.split('@')[0]}
                      </div>
                    </div>
                  </Link>
                </TooltipTrigger>
                <TooltipContent>
                  <div className="text-center">
                    <p className="font-medium">{userEmail}</p>
                    <p className="text-xs text-muted-foreground">Click to open settings</p>
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
                className="bg-background shadow-md h-8 w-8 sm:h-10 sm:w-10 hover:scale-110 active:scale-95 transition-all"
              >
                <Menu className="h-4 w-4 sm:h-5 sm:w-5" />
              </Button>
            </SheetTrigger>
            <SheetContent side="right" className="w-[280px] sm:w-[400px]">
              <SheetHeader>
                <SheetTitle className="text-left">Menu</SheetTitle>
              </SheetHeader>
              <div className="flex flex-col gap-4 mt-6">
                {/* Theme Toggle - Mobile */}
                <div className="flex items-center justify-between p-3 bg-primary/5 rounded-lg">
                  <span className="text-sm font-medium">Theme</span>
                  <ThemeToggle />
                </div>

                {/* User Level Display - Mobile */}
                {user && (
                  <div className="flex items-center justify-center gap-2 p-3 bg-primary/5 rounded-lg">
                    <Badge
                      variant="secondary"
                      className={`${getLevelColor(userLevel)} border-current bg-current/10 text-current font-bold text-sm px-3 py-1`}
                    >
                      <Crown className="h-3 w-3 mr-1" />
                      {formatLevelAsNumber(userLevel)}
                    </Badge>
                  </div>
                )}

                {navItems.map((item) => (
                  <Link
                    key={item.key}
                    href={item.href}
                    onClick={() => setIsOpen(false)}
                    className={`flex flex-col items-center gap-1 rounded-full px-4 py-2 text-xs font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 ${
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
                  <div 
                    onClick={() => setIsOpen(false)}
                    className="mt-4"
                  >
                    <LogoutForm 
                      className="flex flex-col items-center gap-1 rounded-full px-4 py-2 text-xs font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 text-muted-foreground hover:text-primary"
                      variant="ghost"
                    />
                  </div>
                ) : (
                  <Link
                    href="/login"
                    onClick={() => setIsOpen(false)}
                    className="flex flex-col items-center gap-1 rounded-full px-4 py-2 text-xs font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 text-muted-foreground hover:text-primary mt-4"
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
                className="flex items-center justify-center gap-2 rounded-lg px-3 lg:px-4 py-2 text-sm font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 text-muted-foreground hover:text-primary"
                variant="ghost"
              />
            ) : (
              <Link href="/login">
                <Button
                  variant="ghost"
                  className="flex items-center justify-center gap-2 rounded-lg px-3 lg:px-4 py-2 text-sm font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 text-muted-foreground hover:text-primary"
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