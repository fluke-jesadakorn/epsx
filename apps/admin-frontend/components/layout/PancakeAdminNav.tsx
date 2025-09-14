'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState, useEffect } from 'react';

interface NavItem {
  id: string;
  label: string;
  href: string;
  icon: string;
  gradient: string;
  description: string;
}

const navigationItems: NavItem[] = [ // Updated navigation with Remote Config
  {
    id: 'dashboard',
    label: 'Dashboard',
    href: '/',
    icon: '🏠',
    gradient: 'from-yellow-400 to-orange-500 dark:from-purple-500 dark:to-pink-500',
    description: 'Overview & Stats',
  },
  {
    id: 'users',
    label: 'Users',
    href: '/users',
    icon: '👥',
    gradient: 'from-blue-400 to-purple-500 dark:from-blue-500 dark:to-purple-600',
    description: 'Manage Users',
  },
  {
    id: 'permissions',
    label: 'Permissions',
    href: '/permissions',
    icon: '🔑',
    gradient: 'from-green-400 to-teal-500 dark:from-green-500 dark:to-teal-600',
    description: 'Access Control',
  },
  {
    id: 'plans',
    label: 'Plans',
    href: '/plans',
    icon: '💳',
    gradient: 'from-emerald-400 to-green-500 dark:from-emerald-500 dark:to-green-600',
    description: 'Pricing Plans',
  },
  {
    id: 'promotions',
    label: 'Promotions',
    href: '/promotions',
    icon: '🎯',
    gradient: 'from-pink-400 to-rose-500 dark:from-pink-500 dark:to-rose-600',
    description: 'Marketing Campaigns',
  },
  {
    id: 'affiliates',
    label: 'Affiliates',
    href: '/affiliates',
    icon: '🤝',
    gradient: 'from-indigo-400 to-violet-500 dark:from-indigo-500 dark:to-violet-600',
    description: 'Partner Program',
  },
  {
    id: 'remote-config',
    label: 'Remote Config',
    href: '/remote-config',
    icon: '🎛️',
    gradient: 'from-teal-400 to-cyan-500 dark:from-teal-500 dark:to-cyan-600',
    description: 'Settings & Flags',
  },
  {
    id: 'analytics',
    label: 'Analytics',
    href: '/analytics',
    icon: '📊',
    gradient: 'from-purple-400 to-pink-500 dark:from-purple-600 dark:to-pink-600',
    description: 'Data Insights',
  },
  {
    id: 'notifications',
    label: 'Notifications',
    href: '/notifications',
    icon: '🔔',
    gradient: 'from-orange-400 to-red-500 dark:from-orange-600 dark:to-red-600',
    description: 'Alerts & Messages',
  },
  {
    id: 'settings',
    label: 'Settings',
    href: '/settings',
    icon: '⚙️',
    gradient: 'from-gray-400 to-gray-600 dark:from-gray-600 dark:to-gray-700',
    description: 'System Config',
  },
];

export function PancakeAdminNav() {
  const pathname = usePathname();
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    // Check dark mode on mount and listen for changes
    const checkDarkMode = () => {
      setIsDark(document.documentElement.classList.contains('dark'));
    };
    
    checkDarkMode();
    
    // Listen for theme changes
    const observer = new MutationObserver(checkDarkMode);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class']
    });
    
    return () => observer.disconnect();
  }, []);

  return (
    <div
      className={`${isCollapsed ? 'w-20' : 'w-72'} relative border-r border-yellow-200/50 bg-gradient-to-b from-white via-yellow-50 to-orange-50 shadow-2xl backdrop-blur-sm transition-all duration-300 ease-in-out dark:border-slate-700/50`}
      style={{
        background: isDark 
          ? 'linear-gradient(to bottom, rgb(15 23 42), rgb(30 41 59), rgb(15 23 42))' 
          : undefined
      }}
    >
      {/* Logo Section */}
      <div className="border-b border-yellow-200/50 p-6 dark:border-slate-700/50">
        <div className="flex items-center gap-3">
          {isCollapsed ? (
            <h1 className="bg-gradient-to-r from-yellow-600 to-orange-600 dark:from-purple-400 dark:to-pink-400 bg-clip-text text-lg font-black text-transparent">
              EPSX
            </h1>
          ) : (
            <h1 className="bg-gradient-to-r from-yellow-600 to-orange-600 dark:from-purple-400 dark:to-pink-400 bg-clip-text text-xl font-black text-transparent">
              EPSX Admin
            </h1>
          )}
        </div>

        {/* Collapse Toggle */}
        <button
          onClick={() => setIsCollapsed(!isCollapsed)}
          className="absolute top-8 -right-4 z-50 flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-r from-yellow-400 to-orange-500 shadow-lg transition-transform hover:scale-110 hover:shadow-xl"
          style={{
            background: isDark 
              ? 'linear-gradient(to right, rgb(51 65 85), rgb(71 85 105))' 
              : undefined
          }}
        >
          <span className="text-sm font-bold text-white">{isCollapsed ? '→' : '←'}</span>
        </button>
      </div>

      {/* Navigation Items */}
      <div className="space-y-3 p-4">
        {navigationItems.map(item => {
          const isActive = 
            pathname === item.href || 
            (item.href !== '/' && pathname.startsWith(`${item.href}/`));
          
          const isSubroute = pathname.startsWith(`${item.href}/`) && pathname !== item.href;

          return (
            <Link key={item.id} href={item.href}>
              <div
                className={`group relative overflow-hidden rounded-2xl transition-all duration-300 ${
                  isActive
                    ? `bg-gradient-to-r ${item.gradient} scale-105 shadow-2xl`
                    : 'hover:scale-105 hover:bg-gradient-to-r hover:from-yellow-100 hover:to-orange-100 hover:shadow-lg dark:hover:from-slate-700/50 dark:hover:to-slate-600/50'
                }`}
              >
                {/* Background Pattern */}
                <div className="absolute inset-0 opacity-10">
                  <div className="absolute inset-0 bg-gradient-to-br from-white/20 via-transparent to-black/10"></div>
                  <div className="absolute top-0 right-0 -mt-10 -mr-10 h-20 w-20 rounded-full bg-white/5"></div>
                  <div className="absolute bottom-0 left-0 -mb-8 -ml-8 h-16 w-16 rounded-full bg-white/5"></div>
                </div>

                <div className="relative flex items-center gap-4 p-4">
                  <div
                    className={`flex h-12 w-12 items-center justify-center rounded-2xl transition-all duration-300 ${
                      isActive
                        ? 'bg-white/20 shadow-inner'
                        : 'bg-gradient-to-r from-gray-100 to-gray-200 group-hover:from-white/20 group-hover:to-white/10 dark:from-slate-700 dark:to-slate-600'
                    }`}
                  >
                    <span
                      className={`text-2xl ${isActive ? 'text-white' : 'transition-transform group-hover:scale-110'}`}
                    >
                      {item.icon}
                    </span>
                  </div>

                  {!isCollapsed && (
                    <div className="min-w-0 flex-1">
                      <div
                        className={`text-lg font-bold ${
                          isActive
                            ? 'text-white'
                            : 'text-gray-900 group-hover:text-gray-800 dark:text-slate-100 dark:group-hover:text-slate-50'
                        }`}
                      >
                        {item.label}
                      </div>
                      <div
                        className={`text-sm ${
                          isActive
                            ? 'text-white/80'
                            : 'text-gray-500 group-hover:text-gray-600 dark:text-slate-400 dark:group-hover:text-slate-300'
                        }`}
                      >
                        {item.description}
                      </div>
                    </div>
                  )}

                  {/* Active Indicator */}
                  {isActive && (
                    <div className="absolute right-2 h-8 w-2 rounded-full bg-white/40 shadow-inner"></div>
                  )}
                  
                  {/* Subroute Indicator */}
                  {isSubroute && !isCollapsed && (
                    <div className="absolute right-6 top-1/2 transform -translate-y-1/2">
                      <div className="h-2 w-2 rounded-full bg-white/60 animate-pulse"></div>
                    </div>
                  )}
                </div>
              </div>
            </Link>
          );
        })}
      </div>

      {/* Bottom Section */}
      <div className="absolute right-0 bottom-0 left-0 border-t border-yellow-200/50 p-4 dark:border-slate-700/50">
        <div className="rounded-2xl bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 dark:from-purple-600 dark:via-purple-500 dark:to-pink-600 p-4 shadow-xl">
          <div className="text-center">
            <div className="mb-1 text-lg font-bold text-white">⚡ EPSX!</div>
            {!isCollapsed && (
              <div className="text-sm text-white/80">
                Advanced analytics and trading platform
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
