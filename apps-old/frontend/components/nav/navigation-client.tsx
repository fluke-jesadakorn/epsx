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

function SignInBanner() {
  const { isConnected } = useAccount();
  const { isAuthenticated, isLoading, openSignInModal } = useSharedAuth();

  if (!isConnected || isAuthenticated || isLoading) { return null; }

  return (
    <div className="sticky top-0 z-40 flex items-center justify-center gap-3 bg-gradient-to-r from-[#5a33b8] via-[#7645d9] to-[#1a9bab] px-6 py-3 text-base text-white shadow-lg dark:from-[#7645d9]/90 dark:via-[#5a33b8] dark:to-[#1fc7d4]/80">
      <span className="font-medium opacity-90">Your wallet is connected —</span>
      <button
        onClick={openSignInModal}
        className="rounded-md bg-white/20 px-4 py-1 font-bold transition-colors hover:bg-white/30"
      >
        Sign In with Wallet
      </button>
      <span className="opacity-70">to access all features</span>
    </div>
  );
}

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
      <header className="sticky top-0 z-50 border-b border-slate-200/60 bg-white/95 backdrop-blur-md dark:border-slate-800 dark:bg-slate-950/95">
        <div className="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 md:px-6">
          <Link href="/" className="flex items-center gap-2.5 group">
            <img src="/logos/epsx-icon.svg" alt="EPSX Icon" className="h-8 w-8 group-active:scale-95 transition-transform" />
            <span className="text-xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#488BFA] to-[#A43FF3] leading-none mt-0.5">
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
    <>
      <header className="sticky top-0 z-50 border-b border-slate-200/60 bg-white/95 backdrop-blur-md dark:border-slate-800 dark:bg-slate-950/95">
        <div className="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 md:px-6">
          {/* Mobile logo (shown <lg, DesktopNav has its own logo >=lg) */}
          <Link href="/" className="lg:hidden flex items-center gap-2.5 group">
            <img src="/logos/epsx-icon.svg" alt="EPSX Icon" className="h-8 w-8 group-active:scale-95 transition-transform" />
            <span className="text-xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#488BFA] to-[#A43FF3] leading-none mt-0.5">
              EPSX
            </span>
          </Link>

          <DesktopNav />
          <NavActions isAuthenticated={fullyAuth} />
        </div>
      </header>
      <SignInBanner />
    </>
  );
}
