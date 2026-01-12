'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';

import { useSharedAuth } from '@/shared/components/auth/Provider';

interface NavItem {
  id: string;
  label: string;
  href: string;
  icon: string;
  requiresAuth?: boolean;
  children?: NavItem[];
}

const navigationItems: NavItem[] = [
  {
    id: 'dashboard',
    label: 'Dashboard',
    href: '/',
    icon: '🏠',
    requiresAuth: false,
  },
  {
    id: 'auth',
    label: 'Connect Wallet',
    href: '/auth',
    icon: '🔗',
    requiresAuth: false,
  },
  {
    id: 'wallet-management',
    label: 'Wallet Management',
    href: '/wallet-management',
    icon: '👛',
    requiresAuth: true,
  },
  {
    id: 'permissions',
    label: 'Group & Permission',
    href: '/group-and-permission',
    icon: '🔑',
    requiresAuth: true,
  },
  {
    id: 'plans',
    label: 'Plans & Promotions',
    href: '/plans',
    icon: '💳',
    requiresAuth: true,
  },
  {
    id: 'payments',
    label: 'Payments',
    href: '/payments',
    icon: '💰',
    requiresAuth: true,
  },
  {
    id: 'analytics',
    label: 'Analytics',
    href: '/analytics',
    icon: '📊',
    requiresAuth: true,
  },
  {
    id: 'audit-log',
    label: 'Audit Log',
    href: '/audit-log',
    icon: '📜',
    requiresAuth: true,
  },
  {
    id: 'developer',
    label: 'Developer',
    href: '/developer-portal',
    icon: '🧑‍💻',
    requiresAuth: true,
  },
  {
    id: 'notifications',
    label: 'Notifications',
    href: '/notifications',
    icon: '🔔',
    requiresAuth: true,
  },
  {
    id: 'settings',
    label: 'Settings',
    href: '/settings',
    icon: '⚙️',
    requiresAuth: false,
  },
];

/**
 *
 */
export function Sidebar() {
  const pathname = usePathname();
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set(['permissions']));
  const { isAuthenticated } = useSharedAuth();

  // Use isAuthenticated from SharedAuth - it already tracks wallet connection state
  // This avoids depending on wagmi context which may not be available during SSR
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
    <div className="w-64 min-w-0 max-w-64 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 h-full flex flex-col z-20">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <h1 className="text-xl font-bold text-gray-900 dark:text-white truncate">Admin</h1>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto p-4 space-y-2">
        {navigationItems
          .filter(item => {
            // Hide auth item if already connected
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

            return (
              <div key={item.id}>
                {/* Main Item */}
                <div className="relative">
                  {isDisabled ? (
                    <div className={`flex items-center gap-3 px-3 py-2 rounded-lg cursor-not-allowed opacity-50 min-w-0 overflow-hidden ${'text-gray-400 dark:text-gray-500'
                      }`}>
                      <span className="text-lg flex-shrink-0">{item.icon}</span>
                      <span className="font-medium text-ellipsis whitespace-nowrap overflow-hidden hidden sm:inline" style={{ textOverflow: 'ellipsis' }}>{item.label}</span>
                      <span className="font-medium text-ellipsis whitespace-nowrap overflow-hidden sm:hidden" style={{ textOverflow: 'ellipsis' }}>{item.label.replace(/\s+(?:Management|Promotions|Analytics|Notifications)$/, '')}</span>
                      <span className="text-xs flex-shrink-0 bg-muted text-muted-foreground px-2 py-1 rounded hidden sm:inline">🔒</span>
                      <span className="text-xs flex-shrink-0 bg-muted text-muted-foreground px-1 py-1 rounded sm:hidden">🔒</span>
                    </div>
                  ) : (
                    <Link href={item.href}>
                      <div className={`flex items-center gap-3 px-3 py-2 rounded-lg min-w-0 overflow-hidden ${isActive || hasActiveChild
                        ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200'
                        : 'text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800'
                        }`}>
                        <span className="text-lg flex-shrink-0">{item.icon}</span>
                        <span className="font-medium text-ellipsis whitespace-nowrap overflow-hidden hidden sm:inline" style={{ textOverflow: 'ellipsis' }}>{item.label}</span>
                        <span className="font-medium text-ellipsis whitespace-nowrap overflow-hidden sm:hidden" style={{ textOverflow: 'ellipsis' }}>{item.label.replace(/\s+(?:Management|Promotions|Analytics|Notifications)$/, '')}</span>
                      </div>
                    </Link>
                  )}

                  {/* Expand button for items with children */}
                  {item.children && !isDisabled && (
                    <button
                      onClick={() => toggleExpanded(item.id)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
                    >
                      <span className="text-gray-500 text-sm">
                        {isExpanded ? '▼' : '▶'}
                      </span>
                    </button>
                  )}
                </div>

                {/* Children */}
                {item.children && isExpanded && !isDisabled && (
                  <div className="ml-4 mt-1 space-y-1">
                    {item.children.map(child => {
                      const childIsActive = pathname === child.href ||
                        pathname.startsWith(`${child.href}/`);
                      const childNeedsAuth = child.requiresAuth && !isWalletConnected;
                      const childIsDisabled = childNeedsAuth;

                      return (
                        <div key={child.id}>
                          {childIsDisabled ? (
                            <div className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-not-allowed opacity-50 text-gray-400 dark:text-gray-500 min-w-0 overflow-hidden">
                              <span className="text-sm flex-shrink-0">{child.icon}</span>
                              <span className="text-sm font-medium text-ellipsis whitespace-nowrap overflow-hidden hidden sm:inline" style={{ textOverflow: 'ellipsis' }}>{child.label}</span>
                              <span className="text-sm font-medium text-ellipsis whitespace-nowrap overflow-hidden sm:hidden" style={{ textOverflow: 'ellipsis' }}>{child.label.replace(/\s+(?:Management|Promotions|Analytics|Notifications)$/, '')}</span>
                              <span className="text-xs flex-shrink-0 bg-muted text-muted-foreground px-1 py-0.5 rounded">🔒</span>
                            </div>
                          ) : (
                            <Link href={child.href}>
                              <div className={`flex items-center gap-3 px-3 py-2 rounded-lg min-w-0 overflow-hidden ${childIsActive
                                ? 'bg-blue-50 text-blue-600 dark:bg-blue-900/50 dark:text-blue-300'
                                : 'text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800/50'
                                }`}>
                                <span className="text-sm flex-shrink-0">{child.icon}</span>
                                <span className="text-sm font-medium text-ellipsis whitespace-nowrap overflow-hidden hidden sm:inline" style={{ textOverflow: 'ellipsis' }}>{child.label}</span>
                                <span className="text-sm font-medium text-ellipsis whitespace-nowrap overflow-hidden sm:hidden" style={{ textOverflow: 'ellipsis' }}>{child.label.replace(/\s+(?:Management|Promotions|Analytics|Notifications)$/, '')}</span>
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
      <div className="mt-auto">
        {/* Wallet Connection Prompt - shown when not authenticated */}
        {!isWalletConnected && (
          <div className="p-4 border-t border-gray-200 dark:border-gray-700">
            <div className="bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/30 dark:to-indigo-900/30 rounded-lg p-3 border border-blue-200 dark:border-blue-700">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-lg">🔗</span>
                <span className="text-sm font-semibold text-blue-800 dark:text-blue-200">Connect Wallet</span>
              </div>
              <p className="text-xs text-blue-600 dark:text-blue-300 mb-2">
                Connect your wallet to access all admin features and manage permissions.
              </p>
              <Link href="/auth" className="block">
                <div className="w-full bg-blue-600 hover:bg-blue-700 text-white text-xs font-medium py-2 px-3 rounded text-center">
                  Connect Now
                </div>
              </Link>
            </div>
          </div>
        )}

        {/* User Profile - Always visible at bottom */}
        <div className="p-4 border-t border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-white font-semibold flex-shrink-0">
              {isWalletConnected ? 'N' : '?'}
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-gray-900 dark:text-white truncate">
                {isWalletConnected ? 'Admin User' : 'Guest'}
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                {isWalletConnected ? 'Connected' : 'Not connected'}
              </p>
            </div>
          </div>
        </div>
      </div>

    </div>
  );
}