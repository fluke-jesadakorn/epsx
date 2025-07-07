'use client';

import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

import { getAuthStatus } from '@/app/actions/getAuthStatus';

import { LineChart, User, LogOut, LogIn, CreditCard, Menu } from 'lucide-react';

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

import { useAuth } from '../context/auth-context';

import ThemeToggle from './ThemeToggle';

interface NavbarProps {
  userEmail: string | null;
}

const getNavItems = () => {
  return [
    {
      label: 'Analytics',
      href: '/analytics',
      key: 'analytics',
      icon: <LineChart className="h-4 w-4" />,
    },
    {
      label: 'You Market Data Sync',
      href: '/market-data-sync',
      key: 'you-market-data-sync',
      icon: <CreditCard className="h-4 w-4" />,
    },
  ];
};

export default function Navbar({ userEmail }: NavbarProps) {
  const pathname = usePathname() || '';
  const [isOpen, setIsOpen] = useState(false);
  const [isAuth, setIsAuth] = useState(false);
  const router = useRouter();

  const { signOut } = useAuth();

  useEffect(() => {
    const checkAuth = async () => {
      const authStatus = await getAuthStatus();
      setIsAuth(authStatus.isAuthenticated);
    };
    checkAuth();
  }, [pathname]);

  const handleLogout = async () => {
    try {
      setIsOpen(false);
      await signOut();
      router.push('/login');
    } catch (error) {
      console.error('Error signing out:', error);
    }
  };

  return (
    <div className="navbar-bg backdrop-blur-lg border-b navbar-border relative z-50 transition-all duration-300">
      <div className="flex h-20 items-center px-6 justify-between max-w-7xl mx-auto">
        <div className="flex items-center gap-8">
          <Link
            href="/"
            className="flex items-center gap-2 font-bold text-xl group"
          >
            <span className="bg-gradient-to-r from-purple-500 to-pink-500 bg-clip-text text-transparent group-hover:animate-pulse-glow transition-all duration-300">
              EPSX
            </span>
          </Link>
          <NavigationMenu className="hidden md:block">
            <NavigationMenuList className="flex space-x-2">
              {getNavItems().map((item) => (
                <NavigationMenuItem key={item.key}>
                  <NavigationMenuLink asChild>
                    <Link href={item.href}>
                      <Button
                        variant="ghost"
                        className={`flex items-center gap-2 rounded-full nav-item-hover
                        ${
                          pathname === item.href
                            ? 'bg-primary/10 text-primary'
                            : 'text-muted-foreground hover:text-primary'
                        }`}
                      >
                        {item.icon}
                        {item.label}
                      </Button>
                    </Link>
                  </NavigationMenuLink>
                </NavigationMenuItem>
              ))}
            </NavigationMenuList>
          </NavigationMenu>
        </div>

        <div className="flex items-center gap-4 md:gap-6 justify-end min-w-[200px]">
          <ThemeToggle />

          {isAuth && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Link
                    href="/settings"
                    className="flex items-center gap-2 nav-item-hover rounded-full hover:bg-primary/10 hover:cursor-pointer"
                  >
                    <Avatar className="h-8 w-8">
                      <AvatarFallback>
                        <User className="h-4 w-4" />
                      </AvatarFallback>
                    </Avatar>
                    <span className="hidden md:inline text-muted-foreground hover:text-primary">
                      Setting
                    </span>
                  </Link>
                </TooltipTrigger>
                <TooltipContent>
                  <p>{userEmail || 'User'}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}

          {/* Mobile Menu */}
          <Sheet open={isOpen} onOpenChange={setIsOpen}>
            <SheetTrigger asChild className="md:hidden">
              <Button variant="ghost" size="icon">
                <Menu className="h-5 w-5" />
              </Button>
            </SheetTrigger>
            <SheetContent side="right">
              <SheetHeader>
                <SheetTitle className="text-left">Menu</SheetTitle>
              </SheetHeader>
              <div className="flex flex-col gap-4 mt-6 hover:cursor-pointer">
                {getNavItems().map((item) => (
                  <Link
                    key={item.key}
                    href={item.href}
                    onClick={() => setIsOpen(false)}
                    className={`flex items-center gap-2 p-2 rounded-lg hover:bg-primary/10
                      ${pathname === item.href ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-primary'}`}
                  >
                    {item.icon}
                    {item.label}
                  </Link>
                ))}

                <div className="border-t pt-4 mt-2">
                  {isAuth ? (
                    <Button
                      variant="ghost"
                      onClick={() => {
                        handleLogout();
                        setIsOpen(false);
                      }}
                      className="flex items-center gap-2 w-full justify-start p-2 rounded-lg nav-item-hover hover:bg-primary/10 hover:cursor-pointer text-muted-foreground hover:text-primary"
                    >
                      <LogOut className="h-4 w-4" />
                      Logout
                    </Button>
                  ) : (
                    <Link
                      href="/login"
                      onClick={() => setIsOpen(false)}
                      className="flex items-center gap-2 p-2 rounded-lg nav-item-hover hover:bg-primary/10 hover:cursor-pointer text-muted-foreground hover:text-primary"
                    >
                      <LogIn className="h-4 w-4" />
                      Login
                    </Link>
                  )}
                </div>
              </div>
            </SheetContent>
          </Sheet>

          <div className="hidden md:block">
            {isAuth ? (
              <Button
                variant="ghost"
                onClick={handleLogout}
                className="flex items-center gap-2 rounded-full nav-item-hover hover:bg-primary/10 hover:cursor-pointer"
              >
                <LogOut className="h-4 w-4" />
                Logout
              </Button>
            ) : (
              <Link href="/login">
                <Button
                  variant="ghost"
                  className="flex items-center gap-2 rounded-full nav-item-hover hover:bg-primary/10 hover:cursor-pointer"
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
