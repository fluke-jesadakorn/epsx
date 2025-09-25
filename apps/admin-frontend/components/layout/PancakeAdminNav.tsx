'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';
import { AdminWalletAuth } from '@/components/auth/AdminWalletAuth';

interface NavItem {
  id: string;
  label: string;
  href: string;
  icon: string;
  children?: NavItem[];
}

const navigationItems: NavItem[] = [
  {
    id: 'dashboard',
    label: 'Dashboard',
    href: '/',
    icon: '🏠',
  },
  {
    id: 'users',
    label: 'Users',
    href: '/users',
    icon: '👥',
  },
  {
    id: 'permissions',
    label: 'Permissions',
    href: '/permissions',
    icon: '🔑',
    children: [
      {
        id: 'permissions-web3',
        label: 'Web3 Permissions',
        href: '/permissions/web3',
        icon: '🌐',
      },
      {
        id: 'permissions-policies',
        label: 'Policies',
        href: '/permissions/policies',
        icon: '🛡️',
      },
    ]
  },
  {
    id: 'analytics',
    label: 'Analytics',
    href: '/analytics',
    icon: '📊',
  },
  {
    id: 'notifications',
    label: 'Notifications',
    href: '/notifications',
    icon: '🔔',
  },
  {
    id: 'settings',
    label: 'Settings',
    href: '/settings',
    icon: '⚙️',
  },
];

export function PancakeAdminNav() {
  const pathname = usePathname();
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set(['permissions']));

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
        {navigationItems.map(item => {
          const isActive = pathname === item.href || 
            (item.href !== '/' && pathname.startsWith(`${item.href}/`));
          const hasActiveChild = item.children?.some(child => 
            pathname === child.href || pathname.startsWith(`${child.href}/`)
          );
          const isExpanded = expandedItems.has(item.id);

          return (
            <div key={item.id}>
              {/* Main Item */}
              <div className="relative">
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
                
                {/* Expand button for items with children */}
                {item.children && (
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
              {item.children && isExpanded && (
                <div className="ml-4 mt-1 space-y-1">
                  {item.children.map(child => {
                    const childIsActive = pathname === child.href || 
                      pathname.startsWith(`${child.href}/`);

                    return (
                      <Link key={child.id} href={child.href}>
                        <div className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                          childIsActive
                            ? 'bg-blue-50 text-blue-600 dark:bg-blue-900/50 dark:text-blue-300'
                            : 'text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800/50'
                        }`}>
                          <span className="text-sm">{child.icon}</span>
                          <span className="text-sm font-medium">{child.label}</span>
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

      {/* Auth Section */}
      <div className="p-4 border-t border-gray-200 dark:border-gray-700">
        <div className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Authentication
        </div>
        <AdminWalletAuth className="w-full" />
      </div>
    </div>
  );
}