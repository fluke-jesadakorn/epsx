/* eslint-disable @typescript-eslint/strict-boolean-expressions, @typescript-eslint/no-unnecessary-condition, unicorn/filename-case, max-lines-per-function, complexity, no-constant-binary-expression */
'use client';

import Link from 'next/link';
import { usePathname, useSearchParams } from 'next/navigation';
import { useEffect, useState } from 'react';

import {
  BarChart3,
  Bell,
  BookOpen,
  ChevronRight,
  Code,
  Coins,
  CreditCard,
  FileText,
  Globe,
  Home,
  Key,
  LayoutDashboard,
  Link as LinkIcon,
  Link2,
  Lock,
  MessageCircle,
  Palette,
  Send,
  Settings,
  Shield,
  TrendingUp,
  Users,
  Wallet
} from 'lucide-react';

import { useSharedAuth } from '@/shared/components/auth';

interface NavItem {
  id: string;
  label: string;
  href: string;
  icon: React.ElementType;
  requiresAuth?: boolean;
  children?: NavItem[];
  tab?: string;
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
    label: 'Wallet Mgmt',
    href: '/wallet-management',
    icon: Wallet,
    requiresAuth: true,
    children: [
      { id: 'wm-wallets', label: 'Wallets', href: '/wallet-management/wallets', icon: Wallet },
      { id: 'wm-access', label: 'Access', href: '/wallet-management/access', icon: Shield },
      { id: 'wm-credits', label: 'Credits', href: '/wallet-management/credits', icon: Coins },
    ],
  },
  {
    id: 'payments',
    label: 'Payments',
    href: '/payments',
    icon: CreditCard,
    requiresAuth: true,
    children: [
      { id: 'pay-payments', label: 'Payments', href: '/payments', icon: CreditCard, tab: 'payments' },
      { id: 'pay-access', label: 'User Access', href: '/payments', icon: Users, tab: 'user-access' },
      { id: 'pay-links', label: 'Links', href: '/payments', icon: Link2, tab: 'payment-links' },
    ],
  },
  {
    id: 'chat',
    label: 'Chat Support',
    href: '/chat',
    icon: MessageCircle,
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
    children: [
      { id: 'dev-overview', label: 'Overview', href: '/developer-portal', icon: LayoutDashboard, tab: 'overview' },
      { id: 'dev-keys', label: 'API Keys', href: '/developer-portal', icon: Key, tab: 'keys' },
      { id: 'dev-docs', label: 'Docs', href: '/developer-portal', icon: BookOpen, tab: 'docs' },
      { id: 'dev-usage', label: 'Usage', href: '/developer-portal', icon: TrendingUp, tab: 'usage' },
    ],
  },
  {
    id: 'notifications',
    label: 'Notifications',
    href: '/notifications',
    icon: Bell,
    requiresAuth: true,
    children: [
      { id: 'notif-manage', label: 'Overview', href: '/notifications/manage', icon: Bell },
      { id: 'notif-create', label: 'Send Signal', href: '/notifications/create', icon: Send },
    ],
  },
  {
    id: 'settings',
    label: 'Settings',
    href: '/settings',
    icon: Settings,
    requiresAuth: false,
    children: [
      { id: 'set-general', label: 'Nodes', href: '/settings', icon: Globe, tab: 'general' },
      { id: 'set-notifications', label: 'Signals', href: '/settings', icon: Bell, tab: 'notifications' },
      { id: 'set-security', label: 'Vault', href: '/settings', icon: Lock, tab: 'security' },
      { id: 'set-appearance', label: 'Optics', href: '/settings', icon: Palette, tab: 'appearance' },
    ],
  },
];

/**
 *
 */
export function Sidebar() {
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
  const { isAuthenticated } = useSharedAuth();

  const isWalletConnected = isAuthenticated;

  // Auto-expand sections based on current path
  useEffect(() => {
    setExpandedItems(prev => {
      const next = new Set(prev);
      for (const item of navigationItems) {
        if (item.children && pathname.startsWith(item.href)) {
          next.add(item.id);
        }
      }
      return next;
    });
  }, [pathname]);

  const toggleExpanded = (itemId: string) => {
    setExpandedItems(prev => {
      const next = new Set(prev);
      if (next.has(itemId)) next.delete(itemId);
      else next.add(itemId);
      return next;
    });
  };

  const isChildActive = (child: NavItem) => {
    if (child.tab) {
      return pathname === child.href && searchParams.get('tab') === child.tab;
    }
    return pathname === child.href || pathname.startsWith(`${child.href}/`);
  };

  return (
    <div className="w-56 sm:w-64 min-w-0 max-w-64 bg-white dark:bg-[#13151e] border-r border-gray-200 dark:border-border h-full flex flex-col z-20">
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

            const hasActiveChild = item.children?.some(child => isChildActive(child));
            const isExpanded = expandedItems.has(item.id);
            const needsAuth = item.requiresAuth && !isWalletConnected;
            const isDisabled = needsAuth;
            const Icon = item.icon;
            const hasChildren = !!item.children;

            // Dynamic HREF for auth link
            const href = item.id === 'auth'
              ? `/auth?return_url=${encodeURIComponent(pathname)}`
              : item.href;

            return (
              <div key={item.id} className="mb-1">
                {/* Main Item */}
                <div className="group">
                  {isDisabled ? (
                    <div className="flex items-center gap-3 px-4 py-2.5 rounded-2xl cursor-not-allowed opacity-40 text-muted-foreground grayscale">
                      <Icon className="w-5 h-5 flex-shrink-0" />
                      <span className="text-sm font-semibold truncate">{item.label}</span>
                      <Lock className="w-3.5 h-3.5 flex-shrink-0 ml-auto" />
                    </div>
                  ) : hasChildren ? (
                    <button
                      onClick={() => toggleExpanded(item.id)}
                      className={`w-full flex items-center gap-3 px-4 py-2.5 rounded-2xl transition-all duration-200 active:scale-[0.98] ${isActive || hasActiveChild
                        ? 'bg-gradient-to-r from-[#1fc7d4]/10 to-[#7645d9]/10 text-[#1fc7d4] border border-[#1fc7d4]/20 shadow-sm'
                        : 'text-muted-foreground hover:bg-gray-100 dark:hover:bg-white/5 hover:text-foreground'
                        }`}
                    >
                      <Icon className={`w-5 h-5 flex-shrink-0 ${isActive || hasActiveChild ? 'text-[#1fc7d4]' : ''}`} />
                      <span className="text-sm font-semibold truncate">{item.label}</span>
                      <ChevronRight className={`w-3.5 h-3.5 flex-shrink-0 ml-auto transition-transform duration-200 ${isExpanded ? 'rotate-90' : ''}`} />
                    </button>
                  ) : (
                    <Link href={href}>
                      <div className={`flex items-center gap-3 px-4 py-2.5 rounded-2xl transition-all duration-200 group-active:scale-[0.98] ${isActive
                        ? 'bg-gradient-to-r from-[#1fc7d4]/10 to-[#7645d9]/10 text-[#1fc7d4] border border-[#1fc7d4]/20 shadow-sm'
                        : 'text-muted-foreground hover:bg-gray-100 dark:hover:bg-white/5 hover:text-foreground'
                        }`}>
                        <Icon className={`w-5 h-5 flex-shrink-0 ${isActive ? 'text-[#1fc7d4]' : ''}`} />
                        <span className="text-sm font-semibold truncate">{item.label}</span>
                        {isActive && (
                          <div className="w-1.5 h-1.5 rounded-full bg-[#1fc7d4] ml-auto animate-pulse" />
                        )}
                      </div>
                    </Link>
                  )}
                </div>

                {/* Children */}
                {hasChildren && isExpanded && !isDisabled && (
                  <div className="ml-6 mt-1 space-y-0.5 border-l border-gray-200 dark:border-border pl-2">
                    {item.children?.map(child => {
                      const childActive = isChildActive(child);
                      const ChildIcon = child.icon;
                      const childHref = child.tab ? `${child.href}?tab=${child.tab}` : child.href;

                      return (
                        <Link key={child.id} href={childHref}>
                          <div className={`flex items-center gap-3 px-3 py-2 rounded-xl transition-all ${childActive
                            ? 'text-[#1fc7d4] bg-[#1fc7d4]/5 font-bold'
                            : 'text-muted-foreground hover:text-foreground hover:bg-gray-100 dark:hover:bg-white/5'
                            }`}>
                            <ChildIcon className={`w-4 h-4 flex-shrink-0 ${childActive ? 'text-[#1fc7d4]' : ''}`} />
                            <span className="text-xs font-medium truncate">{child.label}</span>
                            {childActive && (
                              <div className="w-1 h-1 rounded-full bg-[#1fc7d4] ml-auto" />
                            )}
                          </div>
                        </Link>
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
            <div className="bg-gradient-to-br from-[#1fc7d4]/5 to-[#7645d9]/5 rounded-3xl p-5 border border-gray-200 dark:border-border backdrop-blur-sm relative overflow-hidden group">
              <div className="absolute -right-4 -top-4 w-16 h-16 bg-[#1fc7d4]/10 rounded-full blur-2xl group-hover:bg-[#1fc7d4]/20 transition-colors" />
              <div className="relative z-10 text-center">
                <p className="text-sm font-bold text-foreground mb-1">Full Access</p>
                <p className="text-[10px] text-muted-foreground mb-4 px-2">
                  Unlock all features by connecting your wallet.
                </p>
                <Link href={`/auth?return_url=${encodeURIComponent(pathname)}`}>
                  <div className="w-full bg-[#1fc7d4] text-white text-sm font-bold py-2.5 px-4 rounded-2xl shadow-lg shadow-cyan-500/20 hover:shadow-cyan-500/40 hover:scale-[1.02] active:scale-95 transition-all">
                    Connect Wallet
                  </div>
                </Link>
              </div>
            </div>
          </div>
        )}

        {/* User Profile - Always visible at bottom */}
        <div className="bg-white dark:bg-white/[0.04] rounded-3xl p-3 border border-gray-200 dark:border-border">
          <div className="flex items-center gap-3">
            <div className={`w-10 h-10 rounded-2xl flex items-center justify-center text-white text-sm font-bold shadow-lg transition-all ${isWalletConnected
              ? 'bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] shadow-cyan-500/10'
              : 'bg-gray-100 dark:bg-slate-800 text-slate-500 shadow-none'
              }`}>
              {isWalletConnected ? 'AU' : '?'}
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-xs font-bold text-foreground truncate">
                {isWalletConnected ? 'Admin user' : 'Guest'}
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
