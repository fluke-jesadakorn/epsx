'use client';

import { ProgressiveAuthStatus } from '@/components/auth/ProgressiveAuthStatus';
import {
  Bell,
  ChartNoAxesColumnIncreasing,
  ChevronDown,
  Code,
  Database,
  TrendingUp,
  File,
  Info,
  LineChart,
  LogIn,
  LogOut,
  Menu,
  Moon,
  Settings,
  Sun,
  User,
  Wallet,
  BarChart3,
  Link as LinkIcon,
} from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState, useEffect } from 'react';
import { useChainId, useSwitchChain, useAccount } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';

import { NavbarSkeleton } from '@/components/nav/NavbarSkeleton';
import { ChainSelector } from '@/components/nav/ChainSelector';
import { WalletProviderIcon } from '@/components/nav/WalletProviderIcon';
import { NotificationBellClient } from '@/components/notifications/NotificationBellClient';
import { UserManagementDropdown } from '@/components/nav/UserManagementDropdown';
import {
  NavbarProvider,
  useNavbarContext,
} from '@/components/providers/NavbarProvider';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui';
import { navigationService } from '@/services/navigation.service';
import { useSharedAuth } from '@/shared/components/auth/Provider';

// Pure Web3 Navigation - no props needed
interface NavigationClientProps {}

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
  // Pure Web3 authentication data will be accessed via context hooks
  return (
    <NavbarProvider>
      <NavigationContent />
    </NavbarProvider>
  );
}

function NavigationContent() {
  const pathname = usePathname();
  const [isOpen, setIsOpen] = useState(false);
  const { isHydrated } = useNavbarContext();
  const chainId = useChainId();
  const { switchChain, isPending: isSwitching } = useSwitchChain();
  const { isConnected, address: connectedAddress } = useAccount();

  // Get shared authentication state
  const { isAuthenticated, isLoading: authLoading, user } = useSharedAuth();

  // Comprehensive authentication check: wallet must be connected AND authenticated AND addresses match
  const isFullyAuthenticated =
    isConnected &&
    isAuthenticated &&
    !authLoading &&
    connectedAddress &&
    user?.wallet_address &&
    connectedAddress.toLowerCase() === user.wallet_address.toLowerCase();

  // Debug authentication state
  useEffect(() => {
    if (!isHydrated) return;

    console.log('🔔 Notification Bell Display Check:', {
      isConnected,
      isAuthenticated,
      authLoading,
      connectedAddress: connectedAddress?.slice(0, 8),
      authenticatedAddress: user?.wallet_address?.slice(0, 8),
      addressesMatch: connectedAddress?.toLowerCase() === user?.wallet_address?.toLowerCase(),
      isFullyAuthenticated,
      shouldShowBell: isFullyAuthenticated
    });
  }, [isConnected, isAuthenticated, authLoading, connectedAddress, user?.wallet_address, isFullyAuthenticated, isHydrated]);

  // Get all nav items - no permission filtering
  const navItems = navigationService.getNavItems();

  // Get current chain name
  const getCurrentChainName = () => {
    if (!isHydrated) return 'Chain';
    if (chainId === bsc.id) return 'BSC Mainnet';
    if (chainId === bscTestnet.id) return 'BSC Testnet';
    return 'Unknown Chain';
  };

  // Handle chain switching with better error handling
  const handleChainSwitch = async (targetChainId: number) => {
    if (!isConnected || isSwitching || targetChainId === chainId) return;
    
    try {
      console.log(`🔄 Switching to chain ${targetChainId}...`);
      await switchChain({ chainId: targetChainId });
      console.log(`✅ Successfully switched to chain ${targetChainId}`);
    } catch (error: any) {
      console.error('❌ Failed to switch chain:', error);
      
      // Handle specific error cases
      if (error?.code === 4902) {
        console.log('🔧 Chain not added to wallet, user needs to add it manually');
      } else if (error?.code === -32002) {
        console.log('⏳ Chain switch request pending, user needs to approve in wallet');
      } else if (error?.code === 4001) {
        console.log('🚫 User rejected the chain switch request');
      }
    }
  };


  // Don't render navigation content until hydrated to prevent mismatch
  if (!isHydrated) {
    return (
      <header
        className="sticky top-0 z-50 border-b border-orange-100/50 bg-white/90 backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/95"
        suppressHydrationWarning
      >
        <div className="max-w-8xl mx-auto flex h-20 items-center px-6">
          {/* EPSX Logo - Skeleton */}
          <Link href="/" className="flex items-center mr-8 hover:opacity-80">
            <span className="text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
              EPSX
            </span>
          </Link>

          {/* Navigation Skeleton - matches hydrated version exactly */}
          <nav className="hidden items-center gap-2 lg:flex mr-8">
            {navigationService.getNavItems().filter(item => item.key !== 'about').map(item => {
              const IconComponent = iconMap[item.key as keyof typeof iconMap];
              return (
                <div
                  key={item.key}
                  className="flex items-center gap-3 rounded-2xl px-4 py-2.5 text-sm font-medium text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200"
                >
                  {IconComponent || (
                    <div className="flex items-center gap-2">
                      <div className="w-6 h-6 rounded-full bg-slate-500 flex items-center justify-center">
                        <Info className="h-3 w-3 text-white" />
                      </div>
                      <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
                    </div>
                  )}
                  <span>{item.label}</span>
                  {item.hasDropdown && (
                    <ChevronDown className="h-3 w-3 ml-1 text-slate-400" />
                  )}
                </div>
              );
            })}
          </nav>

          {/* Spacer */}
          <div className="flex-1" />

          {/* Right Actions Skeleton */}
          <div className="hidden items-center gap-2 lg:flex">
            {/* Notifications Skeleton - Commented out */}
            {/* <div className="flex items-center gap-2 rounded-2xl px-3 py-2">
                <div className="h-5 w-5 bg-slate-200 dark:bg-slate-700 rounded animate-pulse" />
                <div className="h-4 w-20 bg-slate-200 dark:bg-slate-700 rounded animate-pulse" />
                <div className="h-3 w-3 bg-slate-200 dark:bg-slate-700 rounded animate-pulse" />
              </div> */}
            
            {/* About Us Skeleton */}
            <div className="flex items-center gap-2 rounded-2xl px-3 py-2">
              <Info className="h-5 w-5 text-orange-500" />
              <span className="text-sm font-medium text-slate-600 dark:text-slate-300">About Us</span>
            </div>
            
            
            {/* Chain Selector Skeleton */}
            <div className="flex items-center gap-2 rounded-2xl px-3 py-2">
              <LinkIcon className="h-5 w-5 text-orange-500" />
              <span className="text-sm font-medium text-slate-600 dark:text-slate-300">Chain</span>
              <ChevronDown className="h-3 w-3 text-slate-400" />
            </div>
            
            {/* Wallet Connect Skeleton */}
            <div className="flex items-center gap-2 rounded-2xl px-3 py-2 ml-2">
              <div className="h-10 w-20 bg-slate-200 dark:bg-slate-700 rounded-full animate-pulse" />
            </div>
          </div>

          {/* Mobile Menu Trigger Skeleton */}
          <div className="lg:hidden">
            <div className="h-10 w-10 rounded-full bg-orange-50 dark:bg-slate-800 animate-pulse" />
          </div>
        </div>
      </header>
    );
  }

  return (
    <header
      className="sticky top-0 z-50 border-b border-orange-100/50 bg-white/90 backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/95"
      suppressHydrationWarning
    >
      <div className="max-w-8xl mx-auto flex h-20 items-center px-6">
        {/* EPSX Logo */}
        <Link href="/" className="flex items-center mr-8 hover:opacity-80">
          <span className="text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
            EPSX
          </span>
        </Link>

        {/* Main Navigation */}
        <nav className="hidden items-center gap-2 lg:flex mr-8">
          {navItems.filter(item => item.key !== 'about').map(item => {
            const IconComponent = iconMap[item.key as keyof typeof iconMap];
            const isActive = pathname === item.href || item.children?.some(child => pathname === child.href);
            
            if (item.hasDropdown && item.children) {
              return (
                <DropdownMenu key={item.key}>
                  <DropdownMenuTrigger asChild>
                    <button
                      className={`flex items-center gap-3 rounded-2xl px-4 py-2.5 text-sm font-medium ${
                        isActive
                          ? 'border border-orange-200/50 bg-orange-50/80 text-orange-700 dark:border-orange-700/30 dark:bg-orange-900/20 dark:text-orange-300'
                          : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
                      }`}
                    >
                      {IconComponent || <Info className="h-5 w-5 text-orange-500" />}
                      {item.label}
                      <ChevronDown className="h-3 w-3 ml-1 text-slate-400" />
                    </button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="start" className="w-48 bg-white/95 backdrop-blur-xl border border-orange-100/50 dark:bg-slate-900/95 dark:border-slate-700/50">
                    {item.children.map(child => {
                      const ChildIconComponent = iconMap[child.key as keyof typeof iconMap];
                      return (
                        <DropdownMenuItem key={child.key} asChild>
                          <Link
                            href={child.href}
                            className={`flex items-center gap-3 px-3 py-2 text-sm cursor-pointer ${
                              pathname === child.href
                                ? 'bg-orange-50/80 text-orange-700 dark:bg-orange-900/20 dark:text-orange-300'
                                : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
                            }`}
                          >
                            {ChildIconComponent || <Info className="h-4 w-4 text-orange-500" />}
                            {child.label}
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
                className={`flex items-center gap-3 rounded-2xl px-4 py-2.5 text-sm font-medium ${
                  pathname === item.href
                    ? 'border border-orange-200/50 bg-orange-50/80 text-orange-700 dark:border-orange-700/30 dark:bg-orange-900/20 dark:text-orange-300'
                    : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
                }`}
              >
                {IconComponent || (
                  <div className="flex items-center gap-2">
                    <div className="w-6 h-6 rounded-full bg-slate-500 flex items-center justify-center">
                      <Info className="h-3 w-3 text-white" />
                    </div>
                    <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
                  </div>
                )}
                {item.label}
              </Link>
            );
          })}
        </nav>

        {/* Spacer */}
        <div className="flex-1" />

        {/* Right Actions - Hidden on mobile */}
        <div className="hidden items-center gap-2 lg:flex">
          {/* Notifications Bell - Display only if wallet connected AND authenticated */}
          {isFullyAuthenticated && <NotificationBellClient />}

          {/* About Us Link */}
          <Link
            href="/about"
            className={`flex items-center gap-2 rounded-2xl px-3 py-2 text-sm font-medium ${
              pathname === '/about'
                ? 'bg-orange-50/80 text-orange-700 dark:bg-orange-900/20 dark:text-orange-300'
                : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
            }`}
          >
            <Info className="h-5 w-5 text-orange-500" />
            <span>About Us</span>
          </Link>


          {/* Chain Selection Dropdown */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button 
                className="flex items-center gap-2 rounded-2xl px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200"
                disabled={isSwitching || !isConnected}
              >
                <LinkIcon className="h-5 w-5 text-orange-500" />
                <span>{getCurrentChainName()}</span>
                <ChevronDown className="h-3 w-3 ml-1 text-slate-400" />
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-52 bg-white/95 backdrop-blur-xl border border-orange-100/50 dark:bg-slate-900/95 dark:border-slate-700/50">
              <DropdownMenuItem 
                onClick={() => handleChainSwitch(bsc.id)}
                disabled={isSwitching || chainId === bsc.id}
                className={`flex items-center gap-3 px-3 py-2 text-sm cursor-pointer ${
                  chainId === bsc.id 
                    ? 'bg-orange-50/80 text-orange-700 dark:bg-orange-900/20 dark:text-orange-300' 
                    : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
                } disabled:opacity-50 disabled:cursor-not-allowed`}
              >
                <div className="w-5 h-5 rounded-full bg-yellow-500 flex items-center justify-center">
                  <span className="text-xs font-bold text-white">56</span>
                </div>
                BSC Mainnet
                {chainId === bsc.id && (
                  <LinkIcon className="h-4 w-4 text-orange-500 ml-auto" />
                )}
              </DropdownMenuItem>
              <DropdownMenuItem 
                onClick={() => handleChainSwitch(bscTestnet.id)}
                disabled={isSwitching || chainId === bscTestnet.id}
                className={`flex items-center gap-3 px-3 py-2 text-sm cursor-pointer ${
                  chainId === bscTestnet.id 
                    ? 'bg-orange-50/80 text-orange-700 dark:bg-orange-900/20 dark:text-orange-300' 
                    : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
                } disabled:opacity-50 disabled:cursor-not-allowed`}
              >
                <div className="w-5 h-5 rounded-full bg-purple-500 flex items-center justify-center">
                  <span className="text-xs font-bold text-white">97</span>
                </div>
                BSC Testnet
                {chainId === bscTestnet.id && (
                  <LinkIcon className="h-4 w-4 text-orange-500 ml-auto" />
                )}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              {!isConnected && (
                <div className="px-3 py-2 text-xs text-slate-500 dark:text-slate-400">
                  Connect your wallet to switch networks
                </div>
              )}
            </DropdownMenuContent>
          </DropdownMenu>

          {/* PancakeSwap-style Connect/Disconnect Button */}
          <WalletProviderIcon compact={false} className="ml-2" />
        </div>

        {/* Mobile Menu */}
        <Sheet open={isOpen} onOpenChange={setIsOpen}>
          <SheetTrigger asChild className="lg:hidden">
            <button className="rounded-2xl bg-orange-50 p-3 text-orange-500 shadow-sm transition-all duration-300 hover:scale-105 hover:bg-orange-100 hover:shadow-md dark:bg-slate-800 dark:text-orange-400 dark:hover:bg-slate-700">
              <Menu className="h-5 w-5" />
            </button>
          </SheetTrigger>
          <SheetContent
            side="right"
            className="w-80 border-l border-orange-100/50 bg-white/95 backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/95"
          >
            <SheetHeader className="pb-6">
              <div className="flex items-center gap-3 mb-4">
                <Link href="/" onClick={() => setIsOpen(false)} className="flex items-center">
                  <span className="text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
                    EPSX
                  </span>
                </Link>
              </div>
              <SheetTitle className="text-xl font-bold text-slate-700 dark:text-slate-300">
                Navigation
              </SheetTitle>
            </SheetHeader>
            <div className="my-2 border-t border-orange-100 dark:border-slate-700" />
            <div className="flex flex-col gap-2">
              {navItems.map(item => {
                const IconComponent = iconMap[item.key as keyof typeof iconMap];
                const isActive = pathname === item.href || item.children?.some(child => pathname === child.href);
                
                return (
                  <div key={item.key}>
                    <Link
                      href={item.href}
                      onClick={() => setIsOpen(false)}
                      className={`flex items-center gap-4 rounded-2xl px-4 py-3 text-sm font-medium ${
                        isActive
                          ? 'border border-orange-200/50 bg-orange-50/80 text-orange-700 dark:border-orange-700/30 dark:bg-orange-900/20 dark:text-orange-300'
                          : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
                      }`}
                    >
                      {IconComponent || <Info className="h-5 w-5 text-orange-500" />}
                      {item.label}
                      {item.hasDropdown && (
                        <ChevronDown className="h-3 w-3 ml-auto text-slate-400" />
                      )}
                    </Link>
                    
                    {/* Mobile Submenu */}
                    {item.hasDropdown && item.children && (
                      <div className="ml-6 mt-2 space-y-1">
                        {item.children.map(child => {
                          const ChildIconComponent = iconMap[child.key as keyof typeof iconMap];
                          return (
                            <Link
                              key={child.key}
                              href={child.href}
                              onClick={() => setIsOpen(false)}
                              className={`flex items-center gap-3 rounded-xl px-3 py-2 text-sm ${
                                pathname === child.href
                                  ? 'bg-orange-50/80 text-orange-700 dark:bg-orange-900/20 dark:text-orange-300'
                                  : 'text-slate-500 hover:bg-slate-50/80 hover:text-slate-600 dark:text-slate-400 dark:hover:bg-slate-800/40 dark:hover:text-slate-300'
                              }`}
                            >
                              {ChildIconComponent || <Info className="h-4 w-4 text-orange-500" />}
                              {child.label}
                            </Link>
                          );
                        })}
                      </div>
                    )}
                  </div>
                );
              })}

              <div className="my-4 border-t border-orange-100 dark:border-slate-700" />

              {/* Web3 & User Controls in Mobile */}
              <div className="space-y-3">
                {/* Chain Selection */}
                <div className="rounded-2xl border border-orange-200/50 bg-gradient-to-r from-orange-50/80 to-yellow-50/80 p-3 dark:border-orange-700/40 dark:from-orange-900/20 dark:to-yellow-900/20">
                  <div className="flex items-center gap-2 mb-2">
                    <span className="text-sm">🔗</span>
                    <span className="text-sm font-medium text-orange-900 dark:text-orange-100">
                      Network
                    </span>
                  </div>
                  <ChainSelector compact={false} className="w-full" />
                </div>

              </div>

              {/* PancakeSwap-style Connect Button at Bottom */}
              <div className="mt-6 pt-4 border-t border-orange-100 dark:border-slate-700">
                <WalletProviderIcon compact={false} className="w-full" />
              </div>

            </div>
          </SheetContent>
        </Sheet>
      </div>
    </header>
  );
}
