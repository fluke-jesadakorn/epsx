'use client';

import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

import { getAuthStatus } from '@/app/actions/getAuthStatus';

interface NavbarProps {
  userEmail: string | null;
}
import {
  LineChart,
  User,
  LogOut,
  LogIn,
  CreditCard,
  // File,
  Menu,
} from 'lucide-react';

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

const getNavItems = () => {
  return [
    // {
    //   label: "Docs",
    //   href: "https://your-git-book-url.com",
    //   key: "docs",
    //   icon: <File className="h-4 w-4" />,
    // },
    {
      label: 'Analytics',
      href: '/analytics',
      key: 'analytics',
      icon: <LineChart className="h-4 w-4" />,
    },
    {
      label: 'Data Collection',
      href: '/data-collection',
      key: 'data-collection',
      icon: <CreditCard className="h-4 w-4" />,
    },
  ];
};

export default function Navbar({ userEmail }: NavbarProps) {
  const pathname = usePathname() || '';
  const [isOpen, setIsOpen] = useState(false);
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const router = useRouter();

  const { signOut } = useAuth();

  useEffect(() => {
    const checkAuthStatus = async () => {
      const authStatus = await getAuthStatus();
      setIsLoggedIn(authStatus.isAuthenticated);
    };
    checkAuthStatus();
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
    <div className="bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10 border-b backdrop-blur-sm relative z-50">
      <div className="flex h-20 items-center px-6 justify-between max-w-7xl mx-auto">
        <div className="flex items-center gap-8">
          <Link href="/" className="flex items-center gap-2 font-bold text-xl">
            <span className="bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent">
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
                        className={`flex items-center gap-2 rounded-full hover:bg-primary/10
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

          {isLoggedIn && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Link href="/settings" className="flex items-center gap-2">
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
                  {isLoggedIn ? (
                    <Button
                      variant="ghost"
                      onClick={() => {
                        handleLogout();
                        setIsOpen(false);
                      }}
                      className="flex items-center gap-2 w-full justify-start p-2 rounded-lg hover:bg-primary/10 text-muted-foreground hover:text-primary"
                    >
                      <LogOut className="h-4 w-4" />
                      Logout
                    </Button>
                  ) : (
                    <Link
                      href="/login"
                      onClick={() => setIsOpen(false)}
                      className="flex items-center gap-2 p-2 rounded-lg hover:bg-primary/10 text-muted-foreground hover:text-primary"
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
            {isLoggedIn ? (
              <Button
                variant="ghost"
                onClick={handleLogout}
                className="flex items-center gap-2 rounded-full hover:bg-primary/10 hover:cursor-pointer"
              >
                <LogOut className="h-4 w-4" />
                Logout
              </Button>
            ) : (
              <Link href="/login">
                <Button
                  variant="ghost"
                  className="flex items-center gap-2 rounded-full hover:bg-primary/10 hover:cursor-pointer"
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
