// Copied and adapted from frontend/components/nav.tsx
'use client';

import { ButtonIcon } from '@/components/ui/button-icon';
import { ThemeSwitch } from '@/components/ui/ThemeSwitch';
import { useAdminAuth } from '@/context/admin-auth';
import { BarChart, Database, Home, LogIn, LogOut } from 'lucide-react';
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
      <nav className="relative z-50 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container mx-auto px-4">
          <div className="flex h-16 items-center justify-between">
            <Link href="/" className="text-xl font-bold">
              ESPX
            </Link>
            <div className="flex items-center gap-4">
              <div className="animate-pulse bg-gray-300 h-8 w-16 rounded"></div>
            </div>
          </div>
        </div>
      </nav>
    );
  }

  return (
    <nav className="relative z-50 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container mx-auto px-4">
        <div className="flex h-16 items-center justify-between">
          <div className="flex items-center gap-4">
            <ThemeSwitch />
          </div>
          <div className="flex items-center gap-4">
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
