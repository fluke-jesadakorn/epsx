'use client';

import {
  LineChart,
  BarChart,
  User,
  LogOut,
  LogIn,
  File,
  Menu,
  Settings,
  Database,
} from 'lucide-react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { useState, memo, useEffect } from 'react';

import ThemeToggle from '@/components/features/theme/ThemeToggle';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { Button } from '@/components/ui/button';
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from '@/components/ui/navigation-menu';
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
import { useAuth } from '@/context/auth-context-improved';
import { navigationService } from '@/services/navigation.service';

const iconMap = {
  docs: <File className="h-4 w-4" />,
  ranking: <LineChart className="h-4 w-4" />,
  analytics: <BarChart className="h-4 w-4" />,
  settings: <Settings className="h-4 w-4" />,
  'my-data': <Database className="h-4 w-4" />,
};

function NavbarComponent() {
  const pathname = usePathname();
  const { user, signOut, loading } = useAuth();
  // Get user email and admin status from user object
  const userEmail = user?.email;
  const [isOpen, setIsOpen] = useState(false);
  const [isMounted, setIsMounted] = useState(false);
  const router = useRouter();

  // Prevent hydration mismatch by only rendering after mount
  useEffect(() => {
    setIsMounted(true);
  }, []);

  // Don't render until mounted and auth is not loading
  if (!isMounted || loading) {
    return (
      <div className="relative z-50 bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10 border-b backdrop-blur-sm">
        <div className="flex h-20 items-center px-6 justify-between max-w-7xl mx-auto">
          <div className="flex items-center gap-8">
            <Link href="/" className="flex items-center gap-3 group">
              <span className="font-bold text-2xl bg-gradient-to-r from-blue-600 via-purple-600 to-blue-700 bg-clip-text text-transparent group-hover:scale-105 transition-transform duration-300">
                EPSX
              </span>
            </Link>
          </div>
          <div className="flex items-center gap-4 md:gap-6">
            <ThemeToggle />
          </div>
        </div>
      </div>
    );
  }

  const navItems = navigationService.getNavItems(!!user);

  const handleLogout = async () => {
    try {
      await signOut();
      router.push('/login');
    } catch (error) {
      console.error('Error signing out:', error);
    }
  };

  const renderNavItem = (
    item: ReturnType<typeof navigationService.getNavItems>[0],
  ) => (
    <NavigationMenuItem key={item.key}>
      <NavigationMenuLink
        asChild
        href={item.href}
        className={`${
          pathname === item.href
            ? 'bg-primary/10 text-primary'
            : 'text-muted-foreground hover:text-primary'
        }`}
      >
        <Link href={item.href} className="flex flex-col items-center gap-1">
          {iconMap[item.key as keyof typeof iconMap]}
          <span className="mt-1">{item.label}</span>
        </Link>
      </NavigationMenuLink>
    </NavigationMenuItem>
  );

  return (
    <div className="relative z-50 bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10 border-b backdrop-blur-sm">
      <div className="flex h-20 items-center px-6 justify-between max-w-7xl mx-auto">
        <div className="flex items-center gap-8">
          <Link href="/" className="flex items-center gap-3 group">
            <span className="font-bold text-2xl bg-gradient-to-r from-blue-600 via-purple-600 to-blue-700 bg-clip-text text-transparent">
              EPSX
            </span>
          </Link>
          <nav className="hidden md:flex gap-2">
            {navItems.map(item => (
              <Link
                key={item.key}
                href={item.href}
                className={`flex flex-col items-center justify-center gap-2 rounded-full px-4 py-2 text-xs font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 ${
                  pathname === item.href ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-primary'
                }`}
              >
                <span className="block">{iconMap[item.key as keyof typeof iconMap]}</span>
                <span className="mt-1">{item.label}</span>
              </Link>
            ))}
          </nav>
          <NavigationMenu className="hidden">
            {/* Keep for structure, but hidden since we use custom nav above */}
          </NavigationMenu>
        </div>

        <div className="flex items-center gap-4 md:gap-6">
          <ThemeToggle />

          {user?.email && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Link href="/settings" className="flex items-center gap-2">
                    <Avatar className="h-9 w-9">
                      <AvatarFallback>
                        <User className="h-4 w-4" />
                      </AvatarFallback>
                    </Avatar>
                    <span className="hidden md:inline text-muted-foreground hover:text-primary">
                      Settings
                    </span>
                  </Link>
                </TooltipTrigger>
                <TooltipContent>
                  <p>{userEmail}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}

          {/* Mobile Menu */}
          <Sheet open={isOpen} onOpenChange={setIsOpen}>
            <SheetTrigger asChild className="md:hidden">
              <Button
                variant="ghost"
                size="icon"
                className="bg-background shadow-md"
              >
                <Menu className="h-4 w-4" />
              </Button>
            </SheetTrigger>
            <SheetContent side="right">
              <SheetHeader>
                <SheetTitle className="text-left">Menu</SheetTitle>
              </SheetHeader>
              <div className="flex flex-col gap-4 mt-6">
                {navItems.map((item) => (
                  <Link
                    key={item.key}
                    href={item.href}
                    onClick={() => setIsOpen(false)}
                    className={`flex flex-col items-center gap-1 rounded-full px-4 py-2 text-xs font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 ${
                      pathname === item.href ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-primary'
                    }`}
                  >
                    {iconMap[item.key as keyof typeof iconMap]}
                    <span className="mt-1">{item.label}</span>
                  </Link>
                ))}

                {user ? (
                  <Button
                    variant="ghost"
                    onClick={() => {
                      setIsOpen(false);
                      handleLogout();
                    }}
                    className="flex items-center gap-2 w-full justify-start p-2 rounded-lg hover:bg-primary/10 text-muted-foreground hover:text-primary mt-4"
                  >
                    <LogOut className="h-4 w-4" />
                    Logout
                  </Button>
                ) : (
                  <Link
                    href="/login"
                    onClick={() => setIsOpen(false)}
                    className="flex items-center gap-2 p-2 rounded-lg hover:bg-primary/10 text-muted-foreground hover:text-primary mt-4"
                  >
                    <LogIn className="h-4 w-4" />
                    Login
                  </Link>
                )}
              </div>
            </SheetContent>
          </Sheet>

          <div className="hidden md:block">
            {user ? (
              <Button
                variant="ghost"
                onClick={handleLogout}
                className="flex items-center gap-2 rounded-full hover:bg-primary/10"
              >
                <LogOut className="h-4 w-4" />
                Logout
              </Button>
            ) : (
              <Link href="/login">
                <Button
                  variant="ghost"
                  className="flex items-center gap-2 rounded-full hover:bg-primary/10"
                >
                  <LogIn className="h-4 w-4" />
                  Login
                </Button>
              </Link>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// Memoize the component to prevent unnecessary re-renders
export default memo(NavbarComponent);
