'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useAccount } from 'wagmi';

import {
  NavbarProvider,
  useNavbarContext,
} from '@/components/providers/navbar-provider';
import { useSharedAuth } from '@/shared/components/auth';

import { DesktopNav } from './desktop-nav';
import { NavActions } from './nav-actions';

export function NavigationClient() {
  return (
    <NavbarProvider>
      <NavContent />
    </NavbarProvider>
  );
}

function NavContent() {
  const pathname = usePathname();
  const { isHydrated } = useNavbarContext();
  const { isConnected } = useAccount();
  const { isAuthenticated, isLoading } = useSharedAuth();

  const fullyAuth = isConnected && isAuthenticated && !isLoading;

  // Skeleton during hydration
  if (!isHydrated) {
    return (
      <header className="sticky top-0 z-50 border-b border-slate-200/60 bg-white/80 backdrop-blur-lg dark:border-slate-800 dark:bg-slate-950/80">
        <div className="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 md:px-6">
          <Link href="/" className="hover:opacity-80">
            <span className="text-xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
              EPSX
            </span>
          </Link>
          <div className="flex items-center gap-2">
            <div className="hidden md:flex items-center gap-2">
              <div className="h-7 w-16 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse" />
              <div className="h-7 w-20 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse" />
            </div>
            <div className="h-9 w-9 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse lg:hidden" />
          </div>
        </div>
      </header>
    );
  }

  // Hide on auth page
  if (pathname === '/auth') {
    return null;
  }

  return (
    <header className="sticky top-0 z-50 border-b border-slate-200/60 bg-white/80 backdrop-blur-lg dark:border-slate-800 dark:bg-slate-950/80">
      <div className="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 md:px-6">
        {/* Mobile logo (shown <lg, DesktopNav has its own logo >=lg) */}
        <Link href="/" className="lg:hidden hover:opacity-80">
          <span className="text-xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
            EPSX
          </span>
        </Link>

        <DesktopNav />
        <NavActions isAuthenticated={fullyAuth} />
      </div>
    </header>
  );
}
