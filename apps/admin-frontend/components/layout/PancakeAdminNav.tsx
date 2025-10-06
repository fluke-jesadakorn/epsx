'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';
import { useAccount } from 'wagmi';

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
    label: 'Permissions',
    href: '/permissions',
    icon: '🔑',
    requiresAuth: true,
    children: [
      {
        id: 'permissions-web3',
        label: 'Web3 Permissions',
        href: '/permissions/web3',
        icon: '🌐',
        requiresAuth: true,
      },
      {
        id: 'permissions-policies',
        label: 'Policies',
        href: '/permissions/policies',
        icon: '🛡️',
        requiresAuth: true,
      },
    ]
  },
  {
    id: 'plans',
    label: 'Plans & Promotions',
    href: '/plans',
    icon: '💳',
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
export function PancakeAdminNav() {
  const pathname = usePathname();
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set(['permissions']));
  const { isConnected } = useAccount();

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
    <div className="w-64 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 h-screen flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">Admin</h1>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-4 space-y-2">
        {navigationItems
          .filter(item => {
            // Hide auth item if already connected
            if (item.id === 'auth' && isConnected) {return false;}
            return true;
          })
          .map(item => {
          const isActive = pathname === item.href || 
            (item.href !== '/' && pathname.startsWith(`${item.href}/`));
          const hasActiveChild = item.children?.some(child => 
            pathname === child.href || pathname.startsWith(`${child.href}/`)
          );
          const isExpanded = expandedItems.has(item.id);
          const needsAuth = item.requiresAuth && !isConnected;
          const isDisabled = needsAuth;

          return (
            <div key={item.id}>
              {/* Main Item */}
              <div className="relative">
                {isDisabled ? (
                  <div className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-colors cursor-not-allowed opacity-50 ${
                    'text-gray-400 dark:text-gray-500'
                  }`}>
                    <span className="text-lg">{item.icon}</span>
                    <span className="font-medium">{item.label}</span>
                    <span className="text-xs ml-auto bg-orange-100 text-orange-600 px-2 py-1 rounded dark:bg-orange-900 dark:text-orange-300">🔒 Auth Required</span>
                  </div>
                ) : (
                  <Link href={item.href}>
                    <div className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                      isActive || hasActiveChild
                        ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200'
                        : 'text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800'
                    }`}>
                      <span className="text-lg">{item.icon}</span>
                      <span className="font-medium">{item.label}</span>
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
                    const childNeedsAuth = child.requiresAuth && !isConnected;
                    const childIsDisabled = childNeedsAuth;

                    return (
                      <div key={child.id}>
                        {childIsDisabled ? (
                          <div className="flex items-center gap-3 px-3 py-2 rounded-lg transition-colors cursor-not-allowed opacity-50 text-gray-400 dark:text-gray-500">
                            <span className="text-sm">{child.icon}</span>
                            <span className="text-sm font-medium">{child.label}</span>
                            <span className="text-xs ml-auto bg-orange-100 text-orange-600 px-1 py-0.5 rounded dark:bg-orange-900 dark:text-orange-300">🔒</span>
                          </div>
                        ) : (
                          <Link href={child.href}>
                            <div className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                              childIsActive
                                ? 'bg-blue-50 text-blue-600 dark:bg-blue-900/50 dark:text-blue-300'
                                : 'text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800/50'
                            }`}>
                              <span className="text-sm">{child.icon}</span>
                              <span className="text-sm font-medium">{child.label}</span>
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

      {/* Footer - Wallet Connection Prompt */}
      {!isConnected && (
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

    </div>
  );
}