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
      <nav className="sticky top-0 z-50 w-full border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 shadow-sm">
        <div className="container mx-auto px-4 lg:px-6">
          <div className="flex h-16 items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="flex h-12 w-12 items-center justify-center">
                <Image
                  src="/logo.png"
                  alt="EPSX Logo"
                  width={48}
                  height={48}
                  className="h-12 w-12 object-contain"
                  priority
                />
              </div>
              <span className="text-xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
                EPSX Admin
              </span>
            </div>
            <div className="flex items-center gap-4">
              <div className="animate-pulse bg-muted h-8 w-16 rounded-md"></div>
            </div>
          </div>
        </div>
      </nav>
    );
  }

  return (
    <nav className="sticky top-0 z-50 w-full border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 shadow-sm">
      <div className="container mx-auto px-4 lg:px-6">
        <div className="flex h-16 items-center justify-between">
          {/* Brand Logo */}
          <Link href="/" className="flex items-center gap-2 group">
            <div className="flex h-12 w-12 items-center justify-center transition-transform group-hover:scale-105">
              <Image
                src="/logo.png"
                alt="EPSX Logo"
                width={48}
                height={48}
                className="h-12 w-12 object-contain"
                priority
              />
            </div>
            <span className="text-xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
              EPSX Admin
            </span>
          </Link>

          <div className="flex items-center gap-4">
            <ThemeSwitch />
            {filteredItems.map((item, idx) =>
              idx === 0 ? (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`flex items-center gap-2 px-3 py-2 rounded hover:bg-accent ${pathname === item.href ? 'bg-accent text-accent-foreground' : ''}`}
                >
                  <span className="w-6 h-6 rounded-full bg-accent flex items-center justify-center mr-2"></span>
                  <item.icon className="w-5 h-5" />
                  <span>{item.label}</span>
                </Link>
              ) : (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`flex items-center gap-2 px-3 py-2 rounded hover:bg-accent ${pathname === item.href ? 'bg-accent text-accent-foreground' : ''}`}
                >
                  <item.icon className="w-5 h-5" />
                  <span>{item.label}</span>
                </Link>
              ),
            )}
            {user ? (
              <ButtonIcon variant="outline" size="sm" onClick={signOut}>
                <LogOut className="w-4 h-4 mr-2" /> Logout
              </ButtonIcon>
            ) : (
              <Link href="/login">
                <ButtonIcon variant="outline" size="sm">
                  <LogIn className="w-4 h-4 mr-2" /> Login
                </ButtonIcon>
              </Link>
            )}
          </div>
        </div>
      </div>
    </nav>
  );
}
