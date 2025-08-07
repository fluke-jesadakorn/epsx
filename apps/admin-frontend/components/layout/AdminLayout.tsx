'use client';

import { useAdminAuth } from '@/lib/auth/ctx';
import {
  Activity,
  BarChart3,
  Bell,
  ChevronRight,
  Database,
  ExternalLink,
  Globe,
  HardDrive,
  Home,
  Key,
  LogOut,
  Menu,
  Package,
  Palette,
  PanelLeft,
  Search,
  Server,
  Settings,
  Shield,
  Users,
  X,
} from 'lucide-react';
import { Breadcrumb } from './Breadcrumb';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { useCallback, useEffect, useRef, useState } from 'react';

interface AdminLayoutProps {
  children: React.ReactNode;
}

export function AdminLayout({ children }: AdminLayoutProps) {
  const { user, loading, initialized, signOut } = useAdminAuth();
  const router = useRouter();
  const pathname = usePathname();
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(() => {
    // Load collapsed state from localStorage
    if (typeof window !== 'undefined') {
      return localStorage.getItem('adminSidebarCollapsed') === 'true';
    }
    return false;
  });
  const [expandedMenus, setExpandedMenus] = useState<string[]>(['dashboard']);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchFocused, setSearchFocused] = useState(false);
  const sidebarRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  const toggleSidebar = useCallback(() => {
    const newCollapsedState = !sidebarCollapsed;
    setSidebarCollapsed(newCollapsedState);
    // Persist sidebar state
    localStorage.setItem('adminSidebarCollapsed', newCollapsedState.toString());
  }, [sidebarCollapsed]);

  useEffect(() => {
    // Only redirect if auth context is fully initialized and no user is found
    if (initialized && !loading && !user) {
      router.replace('/login');
    }
  }, [user, loading, initialized, router]);

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && sidebarOpen) {
        setSidebarOpen(false);
      }
    };

    // Global keyboard shortcuts
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+B or Cmd+B to toggle sidebar
      if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault();
        toggleSidebar();
      }
      // Ctrl+/ or Cmd+/ to focus search
      if ((e.ctrlKey || e.metaKey) && e.key === '/') {
        e.preventDefault();
        if (!sidebarCollapsed) {
          searchRef.current?.focus();
        }
      }
      // Alt+M to toggle mobile sidebar
      if (e.altKey && e.key === 'm' && window.innerWidth < 1024) {
        e.preventDefault();
        setSidebarOpen(!sidebarOpen);
      }
    };

    const handleClickOutside = (e: MouseEvent) => {
      if (
        sidebarRef.current &&
        !sidebarRef.current.contains(e.target as Node)
      ) {
        setSidebarOpen(false);
      }
    };

    // Always listen for global shortcuts
    document.addEventListener('keydown', handleKeyDown);

    if (sidebarOpen) {
      document.addEventListener('keydown', handleEscape);
      document.addEventListener('click', handleClickOutside);
      // Improved mobile scroll lock - preserve scroll position
      const scrollY = window.scrollY;
      document.body.style.overflow = 'hidden';
      document.body.style.position = 'fixed';
      document.body.style.top = `-${scrollY}px`;
      document.body.style.width = '100%';
    }

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('keydown', handleEscape);
      document.removeEventListener('click', handleClickOutside);
      // Restore scroll position
      const scrollY = document.body.style.top;
      document.body.style.overflow = 'unset';
      document.body.style.position = 'static';
      document.body.style.top = 'auto';
      document.body.style.width = 'auto';
      if (scrollY) {
        window.scrollTo(0, parseInt(scrollY || '0') * -1);
      }
    };
  }, [sidebarOpen, sidebarCollapsed, toggleSidebar]);

  useEffect(() => {
    const handleResize = () => {
      if (window.innerWidth >= 1024) {
        setSidebarOpen(false);
      }
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  if (loading || !initialized) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (!user) {
    return null;
  }
  
  // Provide a mock user for testing
  const mockUser = user || { email: 'test@example.com', displayName: 'Test User' };

  const menuGroups = [
    {
      id: 'dashboard',
      label: 'Dashboard',
      icon: Home,
      href: '/',
      type: 'single' as const,
    },
    {
      id: 'users',
      label: 'User Management',
      icon: Users,
      type: 'group' as const,
      items: [
        {
          id: 'user-list',
          label: 'User Accounts',
          href: '/users',
          icon: Users,
          description: 'Unified user management hub with profiles and permissions',
        },
      ],
    },
    {
      id: 'security',
      label: 'Security & Access',
      icon: Shield,
      type: 'group' as const,
      items: [
        {
          id: 'module-management',
          label: 'Module Management',
          href: '/modules',
          icon: Settings,
          description: 'Manage system modules and their configurations',
        },
        {
          id: 'developer-portal',
          label: 'Developer Portal',
          href: '/developer-portal',
          icon: ExternalLink,
          description: 'Access developer tools and API documentation',
        },
        {
          id: 'stock-ranking-packages',
          label: 'Stock Ranking Packages',
          href: '/stock-ranking-packages',
          icon: Package,
          description: 'Assign stock ranking access packages to users',
        },
        {
          id: 'auth',
          label: 'Authentication',
          href: '/auth',
          icon: Key,
          description: 'Login and authentication settings',
        },
      ],
    },
    {
      id: 'analytics',
      label: 'Analytics & Reports',
      icon: BarChart3,
      type: 'group' as const,
      items: [
        {
          id: 'analytics-overview',
          label: 'Analytics Dashboard',
          href: '/analytics',
          icon: BarChart3,
          description: 'Performance metrics and insights',
        },
        // {
        //   id: 'user-analytics',
        //   label: 'User Analytics',
        //   href: '/analytics/users',
        //   icon: Eye,
        //   description: 'User behavior and engagement',
        // },
        // {
        //   id: 'activity-logs',
        //   label: 'Activity Logs',
        //   href: '/logs',
        //   icon: Activity,
        //   description: 'System and user activity monitoring',
        // },
      ],
    },
    {
      id: 'system',
      label: 'System Management',
      icon: Server,
      type: 'group' as const,
      items: [
        {
          id: 'database',
          label: 'Database',
          href: '/database',
          icon: Database,
          description: 'Database management and monitoring',
        },
        {
          id: 'servers',
          label: 'Server Status',
          href: '/servers',
          icon: Server,
          description: 'Server health and performance',
        },
        {
          id: 'storage',
          label: 'Storage',
          href: '/storage',
          icon: HardDrive,
          description: 'File storage and backups',
        },
      ],
    },
    {
      id: 'settings',
      label: 'Configuration',
      icon: Settings,
      type: 'group' as const,
      items: [
        {
          id: 'general-settings',
          label: 'General Settings',
          href: '/settings',
          icon: Settings,
          description: 'Basic system configuration',
        },
        {
          id: 'appearance',
          label: 'Appearance',
          href: '/settings/appearance',
          icon: Palette,
          description: 'Theme and display settings',
        },
        {
          id: 'notifications',
          label: 'Notifications',
          href: '/settings/notifications',
          icon: Bell,
          description: 'Alert and notification preferences',
        },
        {
          id: 'integrations',
          label: 'Integrations',
          href: '/settings/integrations',
          icon: Globe,
          description: 'Third-party integrations',
        },
      ],
    },
  ];

  const toggleMenu = (menuId: string) => {
    setExpandedMenus(prev =>
      prev.includes(menuId)
        ? prev.filter(id => id !== menuId)
        : [...prev, menuId]
    );
  };

  const handleSearchKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      setSearchQuery('');
      setSearchFocused(false);
      searchRef.current?.blur();
    }
    // Enhanced keyboard navigation
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      const firstMenuItem = document.querySelector('[data-menu-item]');
      if (firstMenuItem) {
        (firstMenuItem as HTMLElement).focus();
      }
    }
    if (e.key === 'Tab' && e.shiftKey) {
      e.preventDefault();
      const lastMenuItem = document.querySelector(
        '[data-menu-item]:last-child'
      );
      if (lastMenuItem) {
        (lastMenuItem as HTMLElement).focus();
      }
    }
  };

  // Enhanced keyboard navigation for menu items
  const handleMenuKeyDown = (e: React.KeyboardEvent, menuId?: string) => {
    const currentElement = e.currentTarget as HTMLElement;
    const menuItems = document.querySelectorAll('[data-menu-item]');
    const currentIndex = Array.from(menuItems).indexOf(currentElement);

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        const nextItem = menuItems[currentIndex + 1];
        if (nextItem) {
          (nextItem as HTMLElement).focus();
        } else {
          // Loop to first item
          (menuItems[0] as HTMLElement).focus();
        }
        break;
      case 'ArrowUp':
        e.preventDefault();
        const prevItem = menuItems[currentIndex - 1];
        if (prevItem) {
          (prevItem as HTMLElement).focus();
        } else {
          // Loop to last item
          (menuItems[menuItems.length - 1] as HTMLElement).focus();
        }
        break;
      case 'Home':
        e.preventDefault();
        (menuItems[0] as HTMLElement).focus();
        break;
      case 'End':
        e.preventDefault();
        (menuItems[menuItems.length - 1] as HTMLElement).focus();
        break;
      case 'Enter':
      case ' ':
        if (menuId) {
          e.preventDefault();
          toggleMenu(menuId);
        }
        break;
    }
  };

  const handleSearchFocus = () => {
    setSearchFocused(true);
  };

  const isMenuExpanded = (menuId: string) => expandedMenus.includes(menuId);

  const isActive = (href: string) => {
    return pathname.startsWith(href);
  };

  const isGroupActive = (items: any[]) => {
    return items.some(item => isActive(item.href));
  };

  const handleLogout = async () => {
    try {
      await signOut();
      router.replace('/login');
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  const getCurrentPageTitle = () => {
    for (const group of menuGroups) {
      if (group.type === 'single' && isActive(group.href!)) {
        return group.label;
      }
      if (group.type === 'group') {
        const activeItem = group.items?.find(item => isActive(item.href));
        if (activeItem) {
          return activeItem.label;
        }
      }
    }
    return 'Dashboard';
  };

  const filteredMenuGroups = menuGroups
    .map(group => {
      if (group.type === 'single') {
        return group;
      }
      return {
        ...group,
        items: group.items?.filter(
          item =>
            item.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
            item.description.toLowerCase().includes(searchQuery.toLowerCase())
        ),
      };
    })
    .filter(group => {
      if (group.type === 'single') {
        return group.label.toLowerCase().includes(searchQuery.toLowerCase());
      }
      return group.items && group.items.length > 0;
    });

  return (
    <div className="flex min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Mobile Sidebar Toggle Button */}
      <div className="lg:hidden fixed top-4 left-4 z-50">
        <button
          onClick={() => setSidebarOpen(!sidebarOpen)}
          className="
            p-3 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700
            hover:bg-gray-50 dark:hover:bg-gray-700 transition-all duration-150
            focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900
            min-h-[44px] min-w-[44px] touch-manipulation
          "
          aria-label={sidebarOpen ? 'Close mobile menu (Alt+M)' : 'Open mobile menu (Alt+M)'}
          aria-expanded={sidebarOpen}
          title={sidebarOpen ? 'Close menu' : 'Open menu'}
        >
          {sidebarOpen ? (
            <X className="h-5 w-5 text-gray-600 dark:text-gray-400" />
          ) : (
            <Menu className="h-5 w-5 text-gray-600 dark:text-gray-400" />
          )}
        </button>
      </div>

      {/* Desktop Sidebar Toggle Button */}
      <div className="hidden lg:block fixed top-4 left-4 z-50">
        <button
          onClick={toggleSidebar}
          className="
            p-3 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700
            hover:bg-gray-50 dark:hover:bg-gray-700 transition-all duration-150
            focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900
            min-h-[44px] min-w-[44px] touch-manipulation
          "
          aria-label={sidebarCollapsed ? 'Expand sidebar (Ctrl+B)' : 'Collapse sidebar (Ctrl+B)'}
          aria-expanded={!sidebarCollapsed}
          title={sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          <PanelLeft
            className={`h-5 w-5 text-gray-600 dark:text-gray-400 transition-transform duration-150 ${sidebarCollapsed ? 'rotate-180' : ''}`}
          />
        </button>
      </div>

      {/* Mobile Overlay */}
      {sidebarOpen && (
        <div
          className="lg:hidden fixed inset-0 z-40 bg-black/50 backdrop-blur-sm"
          onClick={() => setSidebarOpen(false)}
          aria-hidden="true"
        />
      )}

      {/* Sidebar */}
      <div
        ref={sidebarRef}
        className={`
          bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col
          fixed inset-y-0 left-0 z-50 transform transition-all duration-200 ease-out
          ${sidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
          ${sidebarCollapsed ? 'w-16' : 'w-80'}
          ${sidebarCollapsed ? 'lg:w-16' : 'lg:w-80'}
          shadow-lg lg:shadow-none
        `}
      >
        {/* Header */}
        <div
          className={`p-6 border-b border-gray-200 dark:border-gray-700 ${sidebarCollapsed ? 'px-2 py-4' : ''}`}
        >
          <div
            className={`flex items-center ${sidebarCollapsed ? 'justify-center' : 'justify-between'} mb-4`}
          >
            {!sidebarCollapsed && (
              <div className="flex items-center gap-3">
                <div className="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
                  <Shield className="h-8 w-8 text-blue-600 dark:text-blue-400" />
                </div>
                <div>
                  <h1 className="text-xl font-bold text-gray-900 dark:text-white">
                    EPSX Admin
                  </h1>
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    v2.0.1
                  </p>
                </div>
              </div>
            )}
            {sidebarCollapsed && (
              <div className="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
                <Shield className="h-8 w-8 text-blue-600 dark:text-blue-400" />
              </div>
            )}
            {/* Mobile close button */}
            <button
              onClick={() => setSidebarOpen(false)}
              className="lg:hidden p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
              aria-label="Close menu"
            >
              <X className="h-6 w-6" />
            </button>
          </div>

          {/* Search Bar */}
          {!sidebarCollapsed && (
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
              <input
                ref={searchRef}
                type="text"
                placeholder="Search features..."
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
                onKeyDown={handleSearchKeyDown}
                onFocus={handleSearchFocus}
                onBlur={() => setSearchFocused(false)}
                className={`
                  w-full pl-10 pr-4 py-3 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 
                  rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent 
                  transition-all duration-150 min-h-[44px] touch-manipulation
                  ${searchFocused ? 'ring-2 ring-blue-500 shadow-sm' : ''}
                `}
                aria-label="Search admin features. Use arrow keys to navigate results."
                role="searchbox"
                aria-expanded={searchQuery.length > 0}
                aria-autocomplete="list"
              />
            </div>
          )}
        </div>

        {/* Navigation Menu */}
        <nav
          className="flex-1 p-4 space-y-2 overflow-y-auto scrollbar-thin scrollbar-thumb-gray-300 dark:scrollbar-thumb-gray-600 scrollbar-track-transparent"
          role="navigation"
          aria-label="Admin navigation menu"
        >
          {filteredMenuGroups.map((group, index) => {
            const Icon = group.icon;

            if (group.type === 'single') {
              const isActiveItem = isActive(group.href!);

              return (
                <Link
                  key={group.id}
                  href={group.href!}
                  onClick={() => setSidebarOpen(false)}
                  onKeyDown={e => handleMenuKeyDown(e)}
                  className={`
                    submenu-item group w-full flex items-center gap-3 px-4 py-3 rounded-xl text-left transition-all duration-150
                    ${sidebarCollapsed ? 'justify-center px-2' : ''}
                    ${
                      isActiveItem
                        ? 'bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 text-blue-700 dark:text-blue-300 border border-blue-200/50 dark:border-blue-800/50 shadow-sm'
                        : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50 border border-transparent hover:border-gray-200 dark:hover:border-gray-600'
                    }
                    transform hover:scale-[1.01] active:scale-[0.99] focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800
                    min-h-[44px] touch-manipulation
                  `}
                  data-menu-item
                  style={{
                    animationDelay: `${index * 25}ms`,
                  }}
                  title={sidebarCollapsed ? group.label : undefined}
                >
                  <div
                    className={`
                      p-2.5 rounded-lg transition-all duration-200
                      ${
                        isActiveItem
                          ? 'bg-blue-100 dark:bg-blue-800/30 shadow-sm'
                          : 'bg-gray-100 dark:bg-gray-600 group-hover:bg-gray-200 dark:group-hover:bg-gray-500'
                      }
                    `}
                  >
                    <Icon
                      className={`h-5 w-5 transition-colors ${isActiveItem ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'}`}
                    />
                  </div>
                  {!sidebarCollapsed && (
                    <div className="flex-1 min-w-0">
                      <div
                        className={`font-medium text-sm transition-colors ${isActiveItem ? 'text-blue-700 dark:text-blue-300' : 'text-gray-900 dark:text-white'}`}
                      >
                        {group.label}
                      </div>
                    </div>
                  )}
                </Link>
              );
            }

            const isGroupActiveState = group.items
              ? isGroupActive(group.items)
              : false;
            const isExpanded = isMenuExpanded(group.id);

            return (
              <div key={group.id} className="space-y-1">
                {/* Group Header */}
                <button
                  onClick={() => toggleMenu(group.id)}
                  onKeyDown={e => handleMenuKeyDown(e, group.id)}
                  className={`
                    submenu-item group w-full flex items-center gap-3 px-4 py-3 rounded-xl text-left transition-all duration-150
                    ${sidebarCollapsed ? 'justify-center px-2' : ''}
                    ${
                      isGroupActiveState
                        ? 'bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 text-blue-700 dark:text-blue-300 border border-blue-200/50 dark:border-blue-800/50 shadow-sm'
                        : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50 border border-transparent hover:border-gray-200 dark:hover:border-gray-600'
                    }
                    transform hover:scale-[1.01] active:scale-[0.99] focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800
                    min-h-[44px] touch-manipulation
                  `}
                  data-menu-item
                  style={{
                    animationDelay: `${index * 25}ms`,
                  }}
                  title={sidebarCollapsed ? group.label : undefined}
                  aria-expanded={isExpanded}
                  aria-controls={`menu-${group.id}`}
                  aria-label={`${group.label} menu${isExpanded ? ', expanded' : ', collapsed'}`}
                >
                  <div
                    className={`
                      p-2.5 rounded-lg transition-all duration-200
                      ${
                        isGroupActiveState
                          ? 'bg-blue-100 dark:bg-blue-800/30 shadow-sm'
                          : 'bg-gray-100 dark:bg-gray-600 group-hover:bg-gray-200 dark:group-hover:bg-gray-500'
                      }
                    `}
                  >
                    <Icon
                      className={`h-5 w-5 transition-colors ${isGroupActiveState ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'}`}
                    />
                  </div>
                  {!sidebarCollapsed && (
                    <>
                      <div className="flex-1 min-w-0">
                        <div
                          className={`font-medium text-sm transition-colors ${isGroupActiveState ? 'text-blue-700 dark:text-blue-300' : 'text-gray-900 dark:text-white'}`}
                        >
                          {group.label}
                        </div>
                        <div
                          className={`text-xs mt-0.5 transition-colors ${isGroupActiveState ? 'text-blue-600/80 dark:text-blue-400/80' : 'text-gray-500 dark:text-gray-400'}`}
                        >
                          {group.items?.length} items
                        </div>
                      </div>
                      <div
                        className={`transition-transform duration-200 ${isExpanded ? 'rotate-90' : ''}`}
                      >
                        <ChevronRight
                          className={`h-4 w-4 transition-colors ${isGroupActiveState ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400'}`}
                        />
                      </div>
                    </>
                  )}
                </button>

                {/* Expandable Items */}
                {!sidebarCollapsed && (
                  <div
                    id={`menu-${group.id}`}
                    className={`
                      transition-all duration-200 ease-out overflow-hidden
                      ${isExpanded ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0'}
                    `}
                  >
                    <div className="pl-6 space-y-1">
                      {group.items?.map((item, itemIndex) => {
                        const ItemIcon = item.icon;
                        const isActiveItem = isActive(item.href);

                        return (
                          <Link
                            key={item.id}
                            href={item.href}
                            onClick={() => setSidebarOpen(false)}
                            onKeyDown={e => handleMenuKeyDown(e)}
                            className={`
                              submenu-item group w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-all duration-150
                              ${
                                isActiveItem
                                  ? 'bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 text-blue-700 dark:text-blue-300 border border-blue-200/50 dark:border-blue-800/50 shadow-sm'
                                  : 'text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700/50 border border-transparent hover:border-gray-200 dark:hover:border-gray-600'
                              }
                              transform hover:scale-[1.005] active:scale-[0.995] focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800
                              min-h-[44px] touch-manipulation
                            `}
                            data-menu-item
                            style={{
                              animationDelay: `${index * 25 + itemIndex * 15}ms`,
                            }}
                            aria-label={`${item.label}: ${item.description}`}
                          >
                            <div className="flex items-center gap-2 w-full">
                              <div
                                className={`w-1 h-6 rounded-full transition-colors ${isActiveItem ? 'bg-blue-500' : 'bg-gray-300 dark:bg-gray-600'}`}
                              ></div>
                              <div
                                className={`
                                  p-2 rounded-lg transition-all duration-200
                                  ${
                                    isActiveItem
                                      ? 'bg-blue-100 dark:bg-blue-800/30 shadow-sm'
                                      : 'bg-gray-100 dark:bg-gray-600 group-hover:bg-gray-200 dark:group-hover:bg-gray-500'
                                  }
                                `}
                              >
                                <ItemIcon
                                  className={`h-4 w-4 transition-colors ${isActiveItem ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'}`}
                                />
                              </div>
                              <div className="flex-1 min-w-0">
                                <div
                                  className={`font-medium text-sm transition-colors ${isActiveItem ? 'text-blue-700 dark:text-blue-300' : 'text-gray-900 dark:text-white'}`}
                                >
                                  {item.label}
                                </div>
                                <div
                                  className={`text-xs mt-0.5 truncate transition-colors ${isActiveItem ? 'text-blue-600/80 dark:text-blue-400/80' : 'text-gray-500 dark:text-gray-400'}`}
                                >
                                  {item.description}
                                </div>
                              </div>
                            </div>
                          </Link>
                        );
                      })}
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </nav>

        {/* User Profile & Logout */}
        <div
          className={`border-t border-gray-200 dark:border-gray-700 p-4 space-y-4 ${sidebarCollapsed ? 'px-2' : ''}`}
        >
          {/* User Profile */}
          <div
            className={`flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg transition-all duration-150 ${sidebarCollapsed ? 'justify-center px-2' : ''}`}
            role="img"
            aria-label={`User profile: ${mockUser.displayName || 'Admin User'}, ${mockUser.email}`}
          >
            <div className="h-10 w-10 rounded-full bg-gradient-to-r from-blue-500 to-indigo-500 flex items-center justify-center flex-shrink-0">
              <span className="text-sm font-bold text-white">
                {mockUser.email?.charAt(0).toUpperCase()}
              </span>
            </div>
            {!sidebarCollapsed && (
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-gray-900 dark:text-white truncate">
                  {mockUser.displayName || 'Admin User'}
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                  {mockUser.email}
                </p>
              </div>
            )}
            {!sidebarCollapsed && (
              <div className="flex items-center gap-1">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                <span className="text-xs text-green-600 dark:text-green-400">
                  Online
                </span>
              </div>
            )}
          </div>

          {/* Logout Button */}
          <button
            onClick={handleLogout}
            className={`
              w-full flex items-center gap-3 px-4 py-3 text-sm font-medium text-red-600 dark:text-red-400 
              hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-all duration-150 
              border border-transparent hover:border-red-200 dark:hover:border-red-800
              focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800
              min-h-[44px] touch-manipulation
              ${sidebarCollapsed ? 'justify-center px-2' : ''}
            `}
            title={sidebarCollapsed ? 'Sign out' : undefined}
          >
            <LogOut className="h-4 w-4" />
            {!sidebarCollapsed && <span>Sign out</span>}
          </button>

          {/* Footer Info & Keyboard Shortcuts */}
          {!sidebarCollapsed && (
            <div className="text-center pt-2 border-t border-gray-200 dark:border-gray-700 space-y-2">
              <div className="flex items-center justify-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                <Shield className="h-3 w-3" />
                <span>EPSX Admin Console v2.0</span>
              </div>
              <div className="text-xs text-gray-400 dark:text-gray-500 space-y-1">
                <div className="flex items-center justify-center gap-1">
                  <kbd className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-xs">
                    Ctrl+B
                  </kbd>
                  <span>Toggle sidebar</span>
                </div>
                <div className="flex items-center justify-center gap-1">
                  <kbd className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-xs">
                    Ctrl+/
                  </kbd>
                  <span>Search</span>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Main Content Area */}
      <div
        className={`
          flex-1 flex flex-col min-w-0 transition-all duration-200 ease-out
          ${sidebarCollapsed ? 'lg:ml-16' : 'lg:ml-80'}
        `}
      >
        {/* Header Bar */}
        <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4 shadow-sm">
          <div className="flex items-center justify-between">
            {/* Breadcrumb */}
            <Breadcrumb />

            {/* Header Actions */}
            <div className="flex items-center gap-4">
              <div className="text-sm text-gray-500 dark:text-gray-400">
                {new Date().toLocaleDateString('en-US', {
                  weekday: 'short',
                  year: 'numeric',
                  month: 'short',
                  day: 'numeric',
                })}
              </div>
              <div className="h-6 w-px bg-gray-300 dark:bg-gray-600"></div>
              <button
                className="
                  p-3 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 
                  relative rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 
                  transition-colors duration-150 min-h-[44px] min-w-[44px] touch-manipulation
                  focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800
                "
                aria-label="Notifications (1 unread)"
                title="View notifications"
              >
                <Bell className="h-5 w-5" />
                <span
                  className="absolute -top-1 -right-1 h-3 w-3 bg-red-500 rounded-full"
                  aria-hidden="true"
                ></span>
              </button>
              <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
                <Activity className="h-4 w-4" />
                <span className="hidden sm:inline">Live</span>
              </div>
            </div>
          </div>
        </div>

        {/* Page Content */}
        <div className="flex-1 p-6 overflow-auto bg-gray-50 dark:bg-gray-900">
          <div className="max-w-7xl mx-auto">
            <div className="animate-fade-in">{children}</div>
          </div>
        </div>
      </div>
    </div>
  );
}
