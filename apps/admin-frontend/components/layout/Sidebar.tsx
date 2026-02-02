'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';

import {
  BarChart3,
  Bell,
  Code,
  CreditCard,
  FileText,
  Home,
  Link as LinkIcon,
  Lock,
  Settings,
  Wallet
} from 'lucide-react';

import { useSharedAuth } from '@/shared/components/auth/Provider';

interface NavItem {
  id: string;
  label: string;
  href: string;
  icon: React.ElementType;
  requiresAuth?: boolean;
  children?: NavItem[];
}

const navigationItems: NavItem[] = [
  {
    id: 'dashboard',
    label: 'Dashboard',
    href: '/',
    icon: Home,
    requiresAuth: false,
  },
  {
    id: 'auth',
    label: 'Connect Wallet',
    href: '/auth',
    icon: LinkIcon,
    requiresAuth: false,
  },
  {
    id: 'wallet-management',
    label: 'Wallet Management',
    href: '/wallet-management',
    icon: Wallet,
    requiresAuth: true,
  },
  {
    id: 'payments',
    label: 'Payments',
    href: '/payments',
    icon: CreditCard,
    requiresAuth: true,
  },
  {
    id: 'analytics',
    label: 'Analytics',
    href: '/analytics',
    icon: BarChart3,
    requiresAuth: true,
  },
  {
    id: 'audit-log',
    label: 'Audit Log',
    href: '/audit-log',
    icon: FileText,
    requiresAuth: true,
  },
  {
    id: 'developer',
    label: 'Developer',
    href: '/developer-portal',
    icon: Code,
    requiresAuth: true,
  },
  {
    id: 'notifications',
    label: 'Notifications',
    href: '/notifications',
    icon: Bell,
    requiresAuth: true,
  },
  {
    id: 'settings',
    label: 'Settings',
    href: '/settings',
    icon: Settings,
    requiresAuth: false,
  },
];

/**
 *
 */
export function Sidebar() {
  const pathname = usePathname();
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set([]));
  const { isAuthenticated } = useSharedAuth();

  const isWalletConnected = isAuthenticated;

  const toggleExpanded = (itemId: string) => {
    const newExpanded = new Set(expandedItems);
    if (newExpanded.has(itemId)) {
      newExpanded.delete(itemId);
    } else {
      newExpanded.add(itemId);
    }
    setExpandedItems(newExpanded);
  };

  return (
    <div className="w-56 sm:w-64 min-w-0 max-w-64 bg-slate-900/40 backdrop-blur-2xl border-r border-white/5 h-full flex flex-col z-20">
      {/* Header */}
      <div className="p-6">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-xl bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] flex items-center justify-center text-white shadow-lg shadow-cyan-500/20">
            <Home className="w-5 h-5" />
          </div>
          <div>
            <h1 className="text-xl font-bold tracking-tight text-foreground">EPSX</h1>
            <p className="text-[10px] uppercase tracking-[0.2em] font-bold text-cyan-400 -mt-1">Admin</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto px-4 space-y-1">
        {navigationItems
          .filter(item => {
            if (item.id === 'auth' && isWalletConnected) { return false; }
            return true;
          })
          .map(item => {
            const isActive = pathname === item.href ||
              (item.href !== '/' && pathname.startsWith(`${item.href}/`));
            const hasActiveChild = item.children?.some(child =>
              pathname === child.href || pathname.startsWith(`${child.href}/`)
            );
            const isExpanded = expandedItems.has(item.id);
            const needsAuth = item.requiresAuth && !isWalletConnected;
            const isDisabled = needsAuth;
            const Icon = item.icon;

            return (
              <div key={item.id} className="mb-1">
                {/* Main Item */}
                <div className="relative group">
                  {isDisabled ? (
                    <div className="flex items-center gap-3 px-4 py-2.5 rounded-2xl cursor-not-allowed opacity-40 text-muted-foreground grayscale">
                      <Icon className="w-5 h-5 flex-shrink-0" />
                      <span className="text-sm font-semibold truncate">{item.label}</span>
                      <Lock className="w-3.5 h-3.5 flex-shrink-0 ml-auto" />
                    </div>
                  ) : (
                    <Link href={item.href}>
                      <div className={`flex items-center gap-3 px-4 py-2.5 rounded-2xl transition-all duration-200 group-active:scale-[0.98] ${isActive || hasActiveChild
                        ? 'bg-gradient-to-r from-[#1fc7d4]/10 to-[#7645d9]/10 text-[#1fc7d4] border border-[#1fc7d4]/20 shadow-sm'
                        : 'text-muted-foreground hover:bg-white/5 hover:text-foreground'
                        }`}>
                        <Icon className={`w-5 h-5 flex-shrink-0 ${isActive || hasActiveChild ? 'text-[#1fc7d4]' : ''}`} />
                        <span className="text-sm font-semibold truncate">{item.label}</span>
                        {(isActive || hasActiveChild) && (
                          <div className="w-1.5 h-1.5 rounded-full bg-[#1fc7d4] ml-auto animate-pulse" />
                        )}
                      </div>
                    </Link>
                  )}

                  {/* Expand button for items with children */}
                  {item.children && !isDisabled && (
                    <button
                      onClick={() => toggleExpanded(item.id)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded-xl hover:bg-white/5"
                    >
                      <span className="text-muted-foreground text-xs">
                        {isExpanded ? '▼' : '▶'}
                      </span>
                    </button>
                  )}
                </div>

                {/* Children */}
                {item.children && isExpanded && !isDisabled && (
                  <div className="ml-6 mt-1 space-y-1 border-l border-white/5 pl-2">
                    {item.children.map(child => {
                      const childIsActive = pathname === child.href ||
                        pathname.startsWith(`${child.href}/`);
                      const childNeedsAuth = child.requiresAuth && !isWalletConnected;
                      const childIsDisabled = childNeedsAuth;
                      const ChildIcon = child.icon;

                      return (
                        <div key={child.id}>
                          {childIsDisabled ? (
                            <div className="flex items-center gap-3 px-4 py-2 rounded-2xl cursor-not-allowed opacity-40 text-muted-foreground">
                              <ChildIcon className="w-4 h-4 flex-shrink-0" />
                              <span className="text-sm font-medium truncate">{child.label}</span>
                              <Lock className="w-3 h-3 flex-shrink-0 ml-auto" />
                            </div>
                          ) : (
                            <Link href={child.href}>
                              <div className={`flex items-center gap-3 px-4 py-2 rounded-2xl transition-all ${childIsActive
                                ? 'text-[#ed4b9e] font-bold'
                                : 'text-muted-foreground hover:text-foreground'
                                }`}>
                                <ChildIcon className="w-4 h-4 flex-shrink-0" />
                                <span className="text-sm font-medium truncate">{child.label}</span>
                              </div>
                            </Link>
                          )}
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}
      </nav>

      {/* Footer Area */}
      <div className="mt-auto p-4">
        {/* Wallet Connection Prompt - shown when not authenticated */}
        {!isWalletConnected && (
          <div className="mb-4">
            <div className="bg-gradient-to-br from-[#1fc7d4]/5 to-[#7645d9]/5 rounded-3xl p-5 border border-white/5 backdrop-blur-sm relative overflow-hidden group">
              <div className="absolute -right-4 -top-4 w-16 h-16 bg-[#1fc7d4]/10 rounded-full blur-2xl group-hover:bg-[#1fc7d4]/20 transition-colors" />
              <div className="relative z-10 text-center">
                <p className="text-sm font-bold text-foreground mb-1">Full Access</p>
                <p className="text-[10px] text-muted-foreground mb-4 px-2">
                  Unlock all features by connecting your wallet.
                </p>
                <Link href="/auth">
                  <div className="w-full bg-[#1fc7d4] text-white text-sm font-bold py-2.5 px-4 rounded-2xl shadow-lg shadow-cyan-500/20 hover:shadow-cyan-500/40 hover:scale-[1.02] active:scale-95 transition-all">
                    Connect Wallet
                  </div>
                </Link>
              </div>
            </div>
          </div>
        )}

        {/* User Profile - Always visible at bottom */}
        <div className="bg-white/5 rounded-3xl p-3 border border-white/5">
          <div className="flex items-center gap-3">
            <div className={`w-10 h-10 rounded-2xl flex items-center justify-center text-white text-sm font-bold shadow-lg transition-all ${isWalletConnected
              ? 'bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] shadow-cyan-500/10'
              : 'bg-slate-800 text-slate-500 shadow-none'
              }`}>
              {isWalletConnected ? 'AU' : '?'}
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-xs font-bold text-foreground truncate">
                {isWalletConnected ? 'Admin User' : 'Guest'}
              </p>
              <div className="flex items-center gap-1.5">
                <div className={`w-1.5 h-1.5 rounded-full ${isWalletConnected ? 'bg-emerald-500' : 'bg-slate-500'}`} />
                <p className="text-[10px] font-bold text-muted-foreground tracking-wide uppercase">
                  {isWalletConnected ? 'Connected' : 'Offline'}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

    </div>
  );
}
