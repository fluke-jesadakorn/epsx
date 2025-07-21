'use client';

import { Button } from '@/components/ui/button';
import { useAuth } from '@/context/shared-auth-provider';
import {
  BarChart,
  Database,
  Home,
  LogIn,
  LogOut,
  Menu,
  Moon,
  Sun,
  X,
} from 'lucide-react';
import { useTheme } from 'next-themes';
import Image from 'next/image';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';

const navItems = [
  { href: '/', label: 'Home', icon: Home },
  { href: '/analytics', label: 'Analytics', icon: BarChart },
  { href: '/my-data', label: 'My Data', icon: Database, auth: true },
];

export function Navigation() {
  const { user, signOut, loading } = useAuth();
  const { theme, setTheme } = useTheme();
  const pathname = usePathname();
  const [mobileOpen, setMobileOpen] = useState(false);

  const filteredItems = navItems.filter((item) => !item.auth || user);

  // Don't render navigation during loading to prevent flickering
  if (loading) {
    return (
      <nav className="sticky top-0 z-50 w-full border-b border-border/10 bg-background/90 backdrop-blur-sm supports-[backdrop-filter]:bg-background/70 shadow-sm">
        <div className="container mx-auto px-4 lg:px-6">
          <div className="flex h-20 items-center justify-between">
            <div className="flex items-center gap-3">
              <Image
                src="/logo.png"
                alt="EPSX Logo"
                width={40}
                height={40}
                className="h-10 w-10 object-contain animate-pulse"
                priority
              />
            </div>
            <div className="flex items-center gap-4">
              <div className="animate-pulse bg-gradient-to-r from-orange-500/20 to-yellow-500/20 h-12 w-20 rounded-full"></div>
            </div>
          </div>
        </div>
      </nav>
    );
  }

  console.log(
    'Navigation: User state:',
    user ? 'authenticated' : 'not authenticated',
    'Loading:',
    loading,
  );

  return (
    <nav className="sticky top-0 z-50 w-full border-b border-border/10 bg-background/90 backdrop-blur-sm supports-[backdrop-filter]:bg-background/70 shadow-sm">
      <div className="container mx-auto px-4 lg:px-6">
        <div className="flex h-20 items-center justify-between">
          {/* Enhanced Brand Logo - PancakeSwap Style */}
          <Link href="/" className="flex items-center gap-3 group">
            <Image
              src="/logo.png"
              alt="EPSX Logo"
              width={40}
              height={40}
              className="h-10 w-10 object-contain transition-all group-hover:scale-110"
              priority
            />
          </Link>

          {/* Enhanced Desktop Navigation - PancakeSwap Style */}
          <div className="hidden md:flex items-center gap-2">
            {filteredItems.map((item) => {
              const isActive = pathname === item.href;
              return (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`group relative flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-semibold transition-all duration-200 ${
                    isActive
                      ? 'bg-gradient-to-r from-orange-500 to-yellow-500 text-white shadow-lg pancake-glow'
                      : 'text-muted-foreground hover:text-foreground hover:bg-orange-500/10 hover:text-orange-600'
                  }`}
                >
                  <item.icon className={`h-4 w-4 transition-transform group-hover:scale-110 ${isActive ? '' : 'group-hover:text-orange-500'}`} />
                  <span>{item.label}</span>
                  {isActive && (
                    <div className="absolute inset-0 rounded-full bg-gradient-to-r from-orange-400/20 to-yellow-400/20 -z-10 animate-pulse" />
                  )}
                  {!isActive && (
                    <span className="absolute inset-0 rounded-full bg-gradient-to-r from-orange-500/0 to-yellow-500/0 group-hover:from-orange-500/5 group-hover:to-yellow-500/5 transition-all duration-300" />
                  )}
                </Link>
              );
            })}

            {/* Enhanced Theme Toggle - PancakeSwap Style */}
            <div className="ml-3">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
                className="h-10 w-10 px-0 rounded-lg bg-gradient-to-r from-blue-500/5 to-purple-500/5 hover:from-blue-500/10 hover:to-purple-500/10 transition-all duration-200 border border-blue-200/20 dark:border-purple-400/15"
              >
                <Sun className="h-5 w-5 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0 text-orange-500" />
                <Moon className="absolute h-5 w-5 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100 text-blue-400" />
                <span className="sr-only">Toggle theme</span>
              </Button>
            </div>
          </div>

          {/* Enhanced Right side actions - PancakeSwap Style */}
          <div className="flex items-center gap-3">
            {user ? (
              <div className="flex items-center gap-3">
                <div className="hidden sm:flex items-center gap-3 px-3 py-2 rounded-lg bg-gradient-to-r from-green-500/5 to-emerald-500/5 border border-green-200/20 dark:border-emerald-400/15">
                  <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-green-500/80 to-emerald-500/80 flex items-center justify-center ring-1 ring-white/10 shadow-sm">
                    <span className="text-sm font-bold text-white">
                      {user.email?.charAt(0).toUpperCase()}
                    </span>
                  </div>
                  <span className="text-sm font-medium text-foreground max-w-32 truncate">
                    {user.email}
                  </span>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => signOut()}
                  className="h-8 w-8 px-0 rounded-lg hover:bg-red-500/5 hover:text-red-500 transition-all duration-200 border border-red-200/20 dark:border-red-400/15"
                >
                  <LogOut className="h-4 w-4" />
                  <span className="sr-only">Logout</span>
                </Button>
              </div>
            ) : (
              <Link href="/login">
                <Button
                  variant="outline"
                  size="sm"
                  className="gap-2 px-4 py-2 rounded-lg font-medium bg-gradient-to-r from-orange-500/90 to-yellow-500/90 text-white border-0 hover:from-orange-600 hover:to-yellow-600 transition-all duration-200 shadow-sm"
                >
                  <LogIn className="h-4 w-4" />
                  <span className="hidden sm:inline">Login</span>
                </Button>
              </Link>
            )}

            {/* Enhanced Mobile Menu Button */}
            <Button
              variant="ghost"
              size="sm"
              className="md:hidden h-10 w-10 px-0 rounded-lg bg-gradient-to-r from-purple-500/5 to-pink-500/5 hover:from-purple-500/10 hover:to-pink-500/10 transition-all duration-200 border border-purple-200/20 dark:border-pink-400/15"
              onClick={() => setMobileOpen(!mobileOpen)}
            >
              {mobileOpen ? (
                <X className="h-5 w-5 text-purple-600 dark:text-pink-400" />
              ) : (
                <Menu className="h-5 w-5 text-purple-600 dark:text-pink-400" />
              )}
              <span className="sr-only">Toggle navigation</span>
            </Button>
          </div>
        </div>

        {/* Enhanced Mobile Navigation */}
        {mobileOpen && (
          <div className="md:hidden">
            <div className="absolute inset-x-0 top-20 bg-background/95 backdrop-blur-sm supports-[backdrop-filter]:bg-background/85 border-b border-border/20 shadow-md">
              <div className="container mx-auto px-4 py-6">
                <div className="flex flex-col gap-3">
                  {filteredItems.map((item) => {
                    const isActive = pathname === item.href;
                    return (
                      <Link
                        key={item.href}
                        href={item.href}
                        className={`group flex items-center gap-4 px-4 py-3 rounded-lg text-sm font-semibold transition-all duration-200 ${
                          isActive
                            ? 'bg-gradient-to-r from-orange-500 to-yellow-500 text-white shadow-lg pancake-glow'
                            : 'text-muted-foreground hover:text-foreground hover:bg-orange-500/10 hover:text-orange-600'
                        }`}
                        onClick={() => setMobileOpen(false)}
                      >
                        <item.icon className={`h-5 w-5 transition-transform group-hover:scale-110 ${isActive ? '' : 'group-hover:text-orange-500'}`} />
                        <span>{item.label}</span>
                        {isActive && (
                          <div className="absolute inset-0 rounded-full bg-gradient-to-r from-orange-400/20 to-yellow-400/20 -z-10 animate-pulse" />
                        )}
                      </Link>
                    );
                  })}

                  {/* Enhanced Theme Toggle for Mobile */}
                  <div className="border-t border-border/40 pt-4 mt-3">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() =>
                        setTheme(theme === 'dark' ? 'light' : 'dark')
                      }
                      className="group flex items-center gap-4 px-4 py-3 w-full justify-start rounded-lg text-sm font-semibold text-muted-foreground hover:text-foreground hover:bg-blue-500/5 hover:text-blue-600 transition-all duration-200"
                    >
                      <Sun className="h-5 w-5 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0 text-orange-500" />
                      <Moon className="absolute h-5 w-5 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100 text-blue-400" />
                      <span className="ml-2">Toggle Theme</span>
                    </Button>
                  </div>

                  {/* Enhanced Mobile user info */}
                  {user && (
                    <div className="border-t border-border/40 pt-4 mt-3">
                      <div className="flex items-center gap-4 px-4 py-3 rounded-lg bg-gradient-to-r from-green-500/5 to-emerald-500/5 border border-green-200/20 dark:border-emerald-400/20">
                        <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-green-500/80 to-emerald-500/80 flex items-center justify-center ring-1 ring-white/10 shadow-sm">
                          <span className="text-sm font-bold text-white">
                            {user.email?.charAt(0).toUpperCase()}
                          </span>
                        </div>
                        <span className="text-sm font-medium text-foreground">
                          {user.email}
                        </span>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </nav>
  );
}
