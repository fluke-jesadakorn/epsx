
'use client';

import {
  BarChart3,
  Bell,
  ChartNoAxesColumnIncreasing,
  ChevronDown,
  ChevronRight,
  Code,
  Database,
  File,
  Info,
  Mail,
  LineChart,
  LogIn,
  LogOut,
  Menu,
  Moon,
  Settings,
  Sun,
  TrendingUp,
  User,
  Wallet
} from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useEffect, useState } from 'react';
import { useAccount, useChainId, useSwitchChain } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';

import { WalletProviderIcon } from '@/components/nav/wallet-provider-icon';
import { NotificationBellClient } from '@/components/notifications/notification-bell-client';
import {
  NavbarProvider,
  useNavbarContext,
} from '@/components/providers/navbar-provider';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Sheet,
  SheetContent,
  SheetTrigger
} from '@/components/ui';
import { navigationService } from '@/services/navigation.service';
import { formatAddress } from '@/shared/auth/utils';
import { useSharedAuth } from '@/shared/components/auth';
import { ChainSelector } from '@/shared/components/navigation/chain-selector';
import { devLog, isProduction } from '@/shared/utils';

// Pure Web3 Navigation - no props needed
interface NavigationClientProps { }

const iconMap = {
  docs: <File className="h-5 w-5 text-orange-500" />,
  ranking: <LineChart className="h-5 w-5 text-orange-500" />,
  analytics: <BarChart3 className="h-5 w-5 text-orange-500" />,
  'portfolio': <TrendingUp className="h-5 w-5 text-orange-500" />,
  developer: <Code className="h-5 w-5 text-orange-500" />,
  about: <Info className="h-5 w-5 text-orange-500" />,
  user: <User className="h-5 w-5 text-orange-500" />,
  notification: <Bell className="h-5 w-5 text-orange-500" />,
  theme: <Sun className="h-5 w-5 text-orange-500" />,
  'theme-dark': <Moon className="h-5 w-5 text-orange-500" />,
  login: <LogIn className="h-5 w-5 text-orange-500" />,
  logout: <LogOut className="h-5 w-5 text-orange-500" />,
  menu: <Menu className="h-5 w-5 text-orange-500" />,
  profile: <User className="h-5 w-5 text-orange-500" />,
  market: <ChartNoAxesColumnIncreasing className="h-5 w-5 text-orange-500" />,
  screener: <Database className="h-5 w-5 text-orange-500" />,
  story: <Info className="h-5 w-5 text-orange-500" />,
  team: <User className="h-5 w-5 text-orange-500" />,
  contact: <Settings className="h-5 w-5 text-orange-500" />,
};

export function NavigationClient(_props: NavigationClientProps = {}) {
  return (
    <NavbarProvider>
      <NavigationContent />
    </NavbarProvider>
  );
}

function NavigationContent() {
  const pathname = usePathname();
  const [isOpen, setIsOpen] = useState(false);
  const [expandedItems, setExpandedItems] = useState<string[]>([]);
  const { isHydrated } = useNavbarContext();
  const chainId = useChainId();
  const { switchChain, isPending: isSwitching } = useSwitchChain();
  const { isConnected, address: connectedAddress } = useAccount();

  // Get shared authentication state
  const { isAuthenticated, isLoading: authLoading, user } = useSharedAuth();

  // Comprehensive authentication check
  const isFullyAuthenticated =
    isConnected &&
    isAuthenticated &&
    !authLoading &&
    connectedAddress &&
    user?.wallet_address &&
    connectedAddress.toLowerCase() === user.wallet_address.toLowerCase();

  // Debug authentication state
  useEffect(() => {
    if (!isHydrated) { return; }

    devLog('Notification Bell Display Check:', {
      isConnected,
      isAuthenticated,
      authLoading,
      connectedAddress: connectedAddress?.slice(0, 8),
      isFullyAuthenticated,
    });
  }, [isConnected, isAuthenticated, authLoading, connectedAddress, isFullyAuthenticated, isHydrated]);

  // Get all nav items
  const navItems = navigationService.getNavItems();

  // Toggle mobile submenu
  const toggleExpanded = (key: string) => {
    setExpandedItems(prev =>
      prev.includes(key) ? prev.filter(k => k !== key) : [...prev, key]
    );
  };

  // Get current chain name
  const getCurrentChainName = () => {
    if (!isHydrated) { return 'Chain'; }
    if (chainId === bsc.id) { return 'BSC Mainnet'; }
    if (chainId === bscTestnet.id) { return 'BSC Testnet'; }
    if (chainId === 31337) { return 'Anvil Local'; }
    return 'Unknown Chain';
  };

  // Handle chain switching
  const handleChainSwitch = async (targetChainId: number) => {
    if (!isConnected || isSwitching || targetChainId === chainId) { return; }

    try {
      devLog(`Switching to chain ${targetChainId}...`);
      await switchChain({ chainId: targetChainId });
      devLog(`Successfully switched to chain ${targetChainId}`);
    } catch (error: any) {
      devLog('Failed to switch chain:', error);
      if (error?.code === 4902) {
        devLog('Chain not added to wallet, user needs to add it manually');
      } else if (error?.code === -32002) {
        devLog('Chain switch request pending, user needs to approve in wallet');
      } else if (error?.code === 4001) {
        devLog('User rejected the chain switch request');
      }
    }
  };

  // Skeleton during hydration
  if (!isHydrated) {
    return (
      <header
        className="sticky top-0 z-50 border-b border-slate-200/50 bg-white/90 backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/95"
        suppressHydrationWarning
      >
        <div className="max-w-8xl mx-auto flex h-16 items-center justify-between px-4 md:px-6">
          {/* Logo */}
          <Link href="/" className="flex items-center hover:opacity-80">
            <span className="text-xl md:text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
              EPSX
            </span>
          </Link>

          {/* Skeleton placeholders */}
          <div className="flex items-center gap-2">
            <div className="hidden md:flex items-center gap-2">
              <div className="h-8 w-20 bg-slate-200 dark:bg-slate-700 rounded-lg animate-pulse" />
              <div className="h-8 w-24 bg-slate-200 dark:bg-slate-700 rounded-lg animate-pulse" />
            </div>
            <div className="h-10 w-10 bg-slate-200 dark:bg-slate-700 rounded-full animate-pulse md:hidden" />
          </div>
        </div>
      </header>
    );
  }

  // Hide navigation on auth page for immersive experience
  if (pathname === '/auth') {
    return null;
  }

  return (
    <header
      className="sticky top-0 z-50 border-b border-slate-200/50 bg-white/90 backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/95"
      suppressHydrationWarning
    >
      <div className="max-w-8xl mx-auto flex h-16 items-center justify-between px-4 md:px-6">
        {/* ═══════════════════════════════════════════════════════════════
            LEFT SECTION - Logo + Desktop Navigation
        ═══════════════════════════════════════════════════════════════ */}
        <div className="flex items-center gap-6">
          {/* EPSX Logo */}
          <Link href="/" className="flex items-center hover:opacity-80 transition-opacity">
            <span className="text-xl md:text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
              EPSX
            </span>
          </Link>

          {/* Desktop Navigation - Hidden on mobile/tablet */}
          <nav className="hidden lg:flex items-center gap-1">
            {navItems.filter(item => item.key !== 'about' && item.key !== 'contact').map(item => {
              const IconComponent = iconMap[item.key as keyof typeof iconMap];
              const isActive = pathname === item.href || item.children?.some(child => pathname === child.href);

              if (item.hasDropdown && item.children) {
                return (
                  <DropdownMenu key={item.key}>
                    <DropdownMenuTrigger asChild>
                      <button
                        className={`flex items-center gap-2 rounded-xl px-3 py-2 text-sm font-medium transition-colors ${isActive
                          ? 'bg-orange-50 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300'
                          : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900 dark:text-slate-300 dark:hover:bg-slate-800 dark:hover:text-slate-100'
                          }`}
                      >
                        {IconComponent}
                        <span>{item.label}</span>
                        <ChevronDown className="h-3.5 w-3.5 text-slate-400" />
                      </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent
                      align="start"
                      sideOffset={8}
                      style={{ zIndex: 99999 }}
                      className="w-56 p-2 bg-white/98 backdrop-blur-xl border border-slate-200 shadow-xl rounded-xl dark:bg-slate-900/98 dark:border-slate-700"
                    >
                      {item.children.map(child => {
                        const ChildIconComponent = iconMap[child.key as keyof typeof iconMap];
                        return (
                          <DropdownMenuItem key={child.key} asChild className="p-0">
                            <Link
                              href={child.href}
                              className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm cursor-pointer transition-colors ${pathname === child.href
                                ? 'bg-orange-50 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300'
                                : 'text-slate-700 hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-800'
                                }`}
                            >
                              {ChildIconComponent}
                              <span className="font-medium">{child.label}</span>
                            </Link>
                          </DropdownMenuItem>
                        );
                      })}
                    </DropdownMenuContent>
                  </DropdownMenu>
                );
              }

              return (
                <Link
                  key={item.key}
                  href={item.href}
                  className={`flex items-center gap-2 rounded-xl px-3 py-2 text-sm font-medium transition-colors ${pathname === item.href
                    ? 'bg-orange-50 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300'
                    : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900 dark:text-slate-300 dark:hover:bg-slate-800 dark:hover:text-slate-100'
                    }`}
                >
                  {IconComponent}
                  <span>{item.label}</span>
                </Link>
              );
            })}
          </nav>
        </div>

        {/* ═══════════════════════════════════════════════════════════════
            RIGHT SECTION - Actions + Mobile Menu
        ═══════════════════════════════════════════════════════════════ */}
        <div className="flex items-center gap-2">
          {/* Desktop Actions - Hidden on mobile */}
          <div className="hidden md:flex items-center gap-2">
            {/* Notifications Bell */}
            {isFullyAuthenticated && <NotificationBellClient />}

            {/* About Us - Only on large screens */}
            <Link
              href="/about"
              className={`hidden lg:flex items-center gap-2 rounded-xl px-3 py-2 text-sm font-medium transition-colors ${pathname === '/about'
                ? 'bg-orange-50 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300'
                : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900 dark:text-slate-300 dark:hover:bg-slate-800 dark:hover:text-slate-100'
                }`}
            >
              <Info className="h-4 w-4 text-orange-500" />
              <span>About</span>
            </Link>

            <Link
              href="/contact"
              className={`hidden lg:flex items-center gap-2 rounded-xl px-3 py-2 text-sm font-medium transition-colors ${pathname === '/contact'
                ? 'bg-purple-50 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300'
                : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900 dark:text-slate-300 dark:hover:bg-slate-800 dark:hover:text-slate-100'
                }`}
            >
              <Mail className="h-4 w-4 text-purple-500" />
              <span>Contact</span>
            </Link>

            {/* Chain Selector - Dev only */}
            <ChainSelector />

            {/* Wallet */}
            <WalletProviderIcon compact={false} />
          </div>

          {/* Tablet: Show compact wallet */}
          <div className="hidden sm:flex md:hidden items-center gap-2">
            {isFullyAuthenticated && <NotificationBellClient />}
            <WalletProviderIcon compact={true} />
          </div>

          {/* ═══════════════════════════════════════════════════════════════
              MOBILE MENU BUTTON - Visible on mobile/tablet
          ═══════════════════════════════════════════════════════════════ */}
          <Sheet open={isOpen} onOpenChange={setIsOpen}>
            <SheetTrigger asChild>
              <button
                className="lg:hidden flex items-center justify-center w-10 h-10 rounded-xl bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 transition-colors"
                aria-label="Open navigation menu"
              >
                <Menu className="h-5 w-5 text-slate-700 dark:text-slate-200" />
              </button>
            </SheetTrigger>

            <SheetContent
              side="right"
              className="w-[85vw] max-w-sm p-0 border-l border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900"
            >
              {/* ═══════════════════════════════════════════════════════════
                  MOBILE SHEET HEADER
              ═══════════════════════════════════════════════════════════ */}
              <div className="flex items-center justify-between p-4 border-b border-slate-200 dark:border-slate-700">
                <div className="flex items-center gap-3">
                  <span className="text-xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
                    EPSX
                  </span>
                </div>
              </div>

              {/* ═══════════════════════════════════════════════════════════
                  WALLET CARD (if connected)
              ═══════════════════════════════════════════════════════════ */}
              {isConnected && connectedAddress && (
                <div className="p-4 bg-gradient-to-br from-slate-50 to-orange-50/30 dark:from-slate-800 dark:to-orange-900/10 border-b border-slate-200/50 dark:border-slate-700/50">
                  <div className="flex items-center gap-3">
                    <div className="flex items-center justify-center w-10 h-10 rounded-full bg-gradient-to-br from-orange-400 to-orange-600 shadow-lg">
                      <Wallet className="h-5 w-5 text-white" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                        {formatAddress(connectedAddress)}
                      </div>
                      <div className={`text-xs font-medium ${isAuthenticated ? 'text-emerald-500' : 'text-slate-500'} flex items-center gap-1`}>
                        {isAuthenticated && <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />}
                        {isAuthenticated ? 'Authenticated' : 'Connected'}
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* ═══════════════════════════════════════════════════════════
                  NAVIGATION ITEMS
              ═══════════════════════════════════════════════════════════ */}
              <div className="flex-1 overflow-y-auto p-4">
                <nav className="space-y-1">
                  {navItems.map(item => {
                    const IconComponent = iconMap[item.key as keyof typeof iconMap];
                    const isActive = pathname === item.href || item.children?.some(child => pathname === child.href);
                    const isExpanded = expandedItems.includes(item.key);

                    return (
                      <div key={item.key}>
                        {item.hasDropdown && item.children ? (
                          <>
                            {/* Parent with children */}
                            <button
                              onClick={() => toggleExpanded(item.key)}
                              className={`w-full flex items-center gap-3 px-3 py-3 rounded-xl text-sm font-medium transition-colors ${isActive
                                ? 'bg-orange-50 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300'
                                : 'text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800'
                                }`}
                            >
                              {IconComponent}
                              <span className="flex-1 text-left">{item.label}</span>
                              <ChevronRight className={`h-4 w-4 text-slate-400 transition-transform ${isExpanded ? 'rotate-90' : ''}`} />
                            </button>

                            {/* Submenu */}
                            {isExpanded && (
                              <div className="ml-4 mt-1 space-y-1 border-l-2 border-slate-200 dark:border-slate-700 pl-4">
                                {item.children.map(child => {
                                  const ChildIconComponent = iconMap[child.key as keyof typeof iconMap];
                                  return (
                                    <Link
                                      key={child.key}
                                      href={child.href}
                                      onClick={() => setIsOpen(false)}
                                      className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-colors ${pathname === child.href
                                        ? 'bg-orange-50 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300'
                                        : 'text-slate-600 hover:bg-slate-100 dark:text-slate-400 dark:hover:bg-slate-800'
                                        }`}
                                    >
                                      {ChildIconComponent}
                                      {child.label}
                                    </Link>
                                  );
                                })}
                              </div>
                            )}
                          </>
                        ) : (
                          <Link
                            href={item.href}
                            onClick={() => setIsOpen(false)}
                            className={`flex items-center gap-3 px-3 py-3 rounded-xl text-sm font-medium transition-colors ${isActive
                              ? 'bg-orange-50 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300'
                              : 'text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800'
                              }`}
                          >
                            {IconComponent}
                            {item.label}
                          </Link>
                        )}
                      </div>
                    );
                  })}
                </nav>

                {/* Divider */}
                <div className="my-4 border-t border-slate-200 dark:border-slate-700" />

                {/* Network Selector (dev only) */}
                {!isProduction && (
                  <div className="mb-4">
                    <div className="text-xs font-medium text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-2 px-3">
                      Network
                    </div>
                    <div className="px-3">
                      <ChainSelector compact={false} className="w-full" />
                    </div>
                  </div>
                )}

                {/* Notifications (if authenticated) */}
                {isFullyAuthenticated && (
                  <Link
                    href="/notifications"
                    onClick={() => setIsOpen(false)}
                    className="flex items-center gap-3 px-3 py-3 rounded-xl text-sm font-medium text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800 transition-colors"
                  >
                    <Bell className="h-5 w-5 text-orange-500" />
                    <span>Notifications</span>
                  </Link>
                )}
              </div>

              {/* ═══════════════════════════════════════════════════════════
                  BOTTOM SECTION - Wallet Button
              ═══════════════════════════════════════════════════════════ */}
              <div className="p-4 border-t border-slate-200 dark:border-slate-700 bg-slate-50/50 dark:bg-slate-800/50">
                <WalletProviderIcon compact={false} className="w-full" />
              </div>
            </SheetContent>
          </Sheet>
        </div>
      </div>
    </header>
  );
}
