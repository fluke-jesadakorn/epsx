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
  children?: NavItem[];
}

const navigationItems: NavItem[] = [ // Updated navigation with new features
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
    children: [
      {
        id: 'permissions-bulk',
        label: 'Bulk Operations',
        href: '/permissions?mode=bulk',
        icon: '🔄',
        gradient: 'from-orange-400 to-red-500 dark:from-orange-500 dark:to-red-600',
        description: 'Bulk Permission Management',
      },
      {
        id: 'permissions-grant',
        label: 'Grant Permissions',
        href: '/permissions?mode=grant',
        icon: '✋',
        gradient: 'from-green-400 to-emerald-500 dark:from-green-500 dark:to-emerald-600',
        description: 'Grant User Permissions',
      },
      {
        id: 'permissions-hierarchy',
        label: 'Permission Hierarchy',
        href: '/permissions/hierarchy',
        icon: '🌳',
        gradient: 'from-blue-400 to-cyan-500 dark:from-blue-500 dark:to-cyan-600',
        description: 'Inheritance & Structure',
      },
      {
        id: 'permissions-policies',
        label: 'Dynamic Policies',
        href: '/permissions/policies',
        icon: '🛡️',
        gradient: 'from-purple-400 to-violet-500 dark:from-purple-500 dark:to-violet-600',
        description: 'ABAC & Monitoring',
      },
    ]
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
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());

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

  useEffect(() => {
    // Auto-expand parent items when child is active
    const newExpanded = new Set<string>();
    navigationItems.forEach(item => {
      if (item.children) {
        const hasActiveChild = item.children.some(child => 
          pathname === child.href || pathname.startsWith(`${child.href}/`)
        );
        if (hasActiveChild) {
          newExpanded.add(item.id);
        }
      }
    });
    setExpandedItems(newExpanded);
  }, [pathname]);

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
    <div
      className={`${isCollapsed ? 'w-20' : 'w-72'} relative border-r border-yellow-200/50 bg-gradient-to-b from-white via-yellow-50 to-orange-50 shadow-2xl backdrop-blur-sm dark:border-slate-700/50`}
      style={{
        background: isDark 
          ? 'linear-gradient(to bottom, rgb(15 23 42), rgb(30 41 59), rgb(15 23 42))' 
          : undefined
      }}
    >
      {/* Logo Section */}
      <div className="border-b border-yellow-200/50 p-6 dark:border-slate-700/50">
        <div className="flex items-center gap-3">
          <img 
            src="/logo.png" 
            alt="EPSX" 
            className="h-8 w-auto object-contain"
          />
          {!isCollapsed && (
            <h1 className="bg-gradient-to-r from-yellow-600 to-orange-600 dark:from-purple-400 dark:to-pink-400 bg-clip-text text-xl font-black text-transparent">
              Admin
            </h1>
          )}
        </div>

        {/* Collapse Toggle */}
        <button
          onClick={() => setIsCollapsed(!isCollapsed)}
          className="absolute top-8 -right-4 z-50 flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-r from-yellow-400 to-orange-500 shadow-lg hover:shadow-xl"
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
          
          const hasActiveChild = item.children?.some(child => 
            pathname === child.href || pathname.startsWith(`${child.href}/`)
          );
          
          const isSubroute = pathname.startsWith(`${item.href}/`) && pathname !== item.href;
          const isExpanded = expandedItems.has(item.id);

          return (
            <div key={item.id}>
              {/* Parent Item */}
              <div className="relative">
                <Link href={item.href}>
                  <div
                    className={`group relative overflow-hidden rounded-2xl ${
                      isActive || hasActiveChild
                        ? `bg-gradient-to-r ${item.gradient} shadow-2xl`
                        : 'bg-white/10 dark:bg-slate-700/20'
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
                        className={`flex h-12 w-12 items-center justify-center rounded-2xl ${
                          isActive || hasActiveChild
                            ? 'bg-white/20 shadow-inner'
                            : 'bg-gradient-to-r from-gray-100 to-gray-200 dark:from-slate-700 dark:to-slate-600'
                        }`}
                      >
                        <span
                          className={`text-2xl ${isActive || hasActiveChild ? 'text-white' : ''}`}
                        >
                          {item.icon}
                        </span>
                      </div>

                      {!isCollapsed && (
                        <div className="min-w-0 flex-1">
                          <div
                            className={`text-lg font-bold ${
                              isActive || hasActiveChild
                                ? 'text-white'
                                : 'text-gray-900 dark:text-slate-100'
                            }`}
                          >
                            {item.label}
                          </div>
                          <div
                            className={`text-sm ${
                              isActive || hasActiveChild
                                ? 'text-white/80'
                                : 'text-gray-500 dark:text-slate-400'
                            }`}
                          >
                            {item.description}
                          </div>
                        </div>
                      )}

                      {/* Active Indicator */}
                      {(isActive || hasActiveChild) && (
                        <div className="absolute right-2 h-8 w-2 rounded-full bg-white/40 shadow-inner"></div>
                      )}
                      
                      {/* Subroute Indicator */}
                      {isSubroute && !isCollapsed && (
                        <div className="absolute right-6 top-1/2 transform -translate-y-1/2">
                          <div className="h-2 w-2 rounded-full bg-white/60"></div>
                        </div>
                      )}
                    </div>
                  </div>
                </Link>

                {/* Expand/Collapse Button (separate from link) */}
                {item.children && !isCollapsed && (
                  <button
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      toggleExpanded(item.id);
                    }}
                    className="absolute right-3 top-1/2 z-30 p-2 rounded-lg"
                    style={{ transform: 'translateY(-50%)' }}
                  >
                    <span className={`text-sm ${isActive || hasActiveChild ? 'text-white' : 'text-gray-500'}`}>
                      {isExpanded ? '▼' : '▶'}
                    </span>
                  </button>
                )}
              </div>

              {/* Children Items */}
              {item.children && isExpanded && !isCollapsed && (
                <div className="ml-4 mt-2 space-y-2">
                  {item.children.map(child => {
                    const childIsActive = 
                      pathname === child.href || 
                      (child.href !== '/' && pathname.startsWith(`${child.href}/`));

                    return (
                      <Link key={child.id} href={child.href}>
                        <div
                          className={`group relative overflow-hidden rounded-xl ${
                            childIsActive
                              ? `bg-gradient-to-r ${child.gradient} shadow-lg`
                              : 'bg-gray-50 dark:bg-slate-600/30'
                          }`}
                        >
                          <div className="relative flex items-center gap-3 p-3">
                            <div
                              className={`flex h-8 w-8 items-center justify-center rounded-xl ${
                                childIsActive
                                  ? 'bg-white/20 shadow-inner'
                                  : 'bg-gradient-to-r from-gray-100 to-gray-200 dark:from-slate-600 dark:to-slate-500'
                              }`}
                            >
                              <span
                                className={`text-sm ${childIsActive ? 'text-white' : ''}`}
                              >
                                {child.icon}
                              </span>
                            </div>

                            <div className="min-w-0 flex-1">
                              <div
                                className={`text-sm font-medium ${
                                  childIsActive
                                    ? 'text-white'
                                    : 'text-gray-800 dark:text-slate-200'
                                }`}
                              >
                                {child.label}
                              </div>
                              <div
                                className={`text-xs ${
                                  childIsActive
                                    ? 'text-white/80'
                                    : 'text-gray-500 dark:text-slate-400'
                                }`}
                              >
                                {child.description}
                              </div>
                            </div>

                            {/* Child Active Indicator */}
                            {childIsActive && (
                              <div className="absolute right-1 h-6 w-1 rounded-full bg-white/40 shadow-inner"></div>
                            )}
                          </div>
                        </div>
                      </Link>
                    );
                  })}
                </div>
              )}
            </div>
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