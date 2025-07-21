// Copied and adapted from frontend/components/nav.tsx
'use client';

import { ButtonIcon } from '@/components/ui/button-icon';
import { ThemeSwitch } from '@/components/ui/ThemeSwitch';
import { useAdminAuth } from '@/context/admin-auth';
import { BarChart, Database, Home, LogIn, LogOut } from 'lucide-react';
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
  const { user, signOut, loading } = useAdminAuth();
  const pathname = usePathname();
  const [mobileOpen, setMobileOpen] = useState(false);

  const filteredItems = navItems.filter((item) => !item.auth || user);

  if (loading) {
    return (
      <nav className="sticky top-0 z-50 w-full border-b border-border/20 bg-background/80 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60 shadow-lg pancake-card-gradient">
        <div className="container mx-auto px-4 lg:px-6">
          <div className="flex h-20 items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-14 w-14 items-center justify-center rounded-full bg-gradient-to-r from-orange-500 to-yellow-500 p-1">
                <div className="flex h-full w-full items-center justify-center rounded-full bg-background">
                  <Image
                    src="/logo.png"
                    alt="EPSX Logo"
                    width={40}
                    height={40}
                    className="h-10 w-10 object-contain"
                    priority
                  />
                </div>
              </div>
              <div className="flex flex-col">
                <span className="text-2xl font-bold bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent">
                  EPSX
                </span>
                <span className="text-xs text-muted-foreground font-medium">
                  Admin Dashboard
                </span>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <div className="animate-pulse bg-muted h-10 w-20 rounded-full"></div>
            </div>
          </div>
        </div>
      </nav>
    );
  }

  return (
    <nav className="sticky top-0 z-50 w-full border-b border-border/20 bg-background/80 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60 shadow-lg pancake-card-gradient">
      <div className="container mx-auto px-4 lg:px-6">
        <div className="flex h-20 items-center justify-between">
          {/* Enhanced Brand Logo */}
          <Link href="/" className="flex items-center gap-3 group">
            <div className="flex h-14 w-14 items-center justify-center rounded-full bg-gradient-to-r from-orange-500 to-yellow-500 p-1 transition-all group-hover:scale-110 group-hover:rotate-6">
              <div className="flex h-full w-full items-center justify-center rounded-full bg-background transition-all group-hover:bg-background/90">
                <Image
                  src="/logo.png"
                  alt="EPSX Logo"
                  width={40}
                  height={40}
                  className="h-10 w-10 object-contain transition-all group-hover:scale-110"
                  priority
                />
              </div>
            </div>
            <div className="flex flex-col">
              <span className="text-2xl font-bold bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent group-hover:from-yellow-500 group-hover:to-orange-500 transition-all">
                EPSX
              </span>
              <span className="text-xs text-muted-foreground font-medium">
                Admin Dashboard
              </span>
            </div>
          </Link>

          <div className="flex items-center gap-4">
            <ThemeSwitch />
            {filteredItems.map((item, idx) =>
              idx === 0 ? (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`group flex items-center gap-2 px-4 py-2 rounded-full font-medium transition-all duration-300 hover:scale-105 ${
                    pathname === item.href 
                      ? 'bg-gradient-to-r from-orange-500 to-yellow-500 text-white shadow-lg' 
                      : 'hover:bg-orange-500/10 hover:text-orange-500'
                  }`}
                >
                  <span className={`w-2 h-2 rounded-full transition-all ${
                    pathname === item.href ? 'bg-white' : 'bg-orange-500 group-hover:scale-125'
                  }`}></span>
                  <item.icon className="w-5 h-5" />
                  <span>{item.label}</span>
                </Link>
              ) : (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`group flex items-center gap-2 px-4 py-2 rounded-full font-medium transition-all duration-300 hover:scale-105 ${
                    pathname === item.href 
                      ? 'bg-gradient-to-r from-orange-500 to-yellow-500 text-white shadow-lg' 
                      : 'hover:bg-orange-500/10 hover:text-orange-500'
                  }`}
                >
                  <item.icon className="w-5 h-5 transition-transform group-hover:scale-110" />
                  <span>{item.label}</span>
                </Link>
              ),
            )}
            {user ? (
              <button 
                onClick={signOut}
                className="pancake-button-secondary flex items-center gap-2 text-sm font-medium"
              >
                <LogOut className="w-4 h-4" /> 
                Logout
              </button>
            ) : (
              <Link href="/login">
                <button className="pancake-button flex items-center gap-2 text-sm font-medium">
                  <LogIn className="w-4 h-4" /> 
                  Login
                </button>
              </Link>
            )}
          </div>
        </div>
      </div>
    </nav>
  );
}
