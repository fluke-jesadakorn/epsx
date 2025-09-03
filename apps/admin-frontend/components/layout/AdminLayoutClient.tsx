'use client';

import { useAuth } from '@/lib/auth';
import {
  BarChart3,
  Bell,
  ChevronRight,
  Coins,
  ExternalLink,
  Home,
  LogOut,
  Menu,
  Moon,
  Package,
  PanelLeft,
  Search,
  Server,
  Settings,
  Shield,
  Sun,
  Users,
  X,
} from 'lucide-react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { ReactNode, useCallback, useEffect, useRef, useState } from 'react';
import { ServerBreadcrumb } from './ServerBreadcrumb';

interface Session {
  user?: {
    id: string;
    email: string;
    name?: string;
    role: string;
    permissions: string[];
    packageTier: string;
  };
  isLoggedIn: boolean;
}

interface AdminLayoutClientProps {
  children: ReactNode;
  session: Session | null;
}

export function AdminLayoutClient({
  children,
  session,
}: AdminLayoutClientProps) {
  const router = useRouter();
  const pathname = usePathname();
  const { logout, isLoading: isSigningOut } = useAuth();
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [isClient, setIsClient] = useState(false);
  const [isMobile, setIsMobile] = useState(false);
  const [expandedMenus, setExpandedMenus] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchFocused, setSearchFocused] = useState(false);
  const [isDarkMode, setIsDarkMode] = useState(false);
  const sidebarRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  const toggleSidebar = useCallback(() => {
    const newCollapsedState = !sidebarCollapsed;
    setSidebarCollapsed(newCollapsedState);

    // When collapsing sidebar, close all expanded menus
    if (newCollapsedState) {
      setExpandedMenus([]);
    }

    // Persist sidebar state
    localStorage.setItem('adminSidebarCollapsed', newCollapsedState.toString());
  }, [sidebarCollapsed]);

  const toggleTheme = useCallback(() => {
    const html = document.documentElement;
    const newTheme = !isDarkMode;

    if (newTheme) {
      html.classList.add('dark');
      localStorage.setItem('theme', 'dark');
    } else {
      html.classList.remove('dark');
      localStorage.setItem('theme', 'light');
    }

    setIsDarkMode(newTheme);
  }, [isDarkMode]);

  // Initialize client-side state
  useEffect(() => {
    setIsClient(true);
    setIsMobile(window.innerWidth < 1024);

    // Restore sidebar state from localStorage
    const savedState = localStorage.getItem('adminSidebarCollapsed');
    if (savedState !== null) {
      setSidebarCollapsed(savedState === 'true');
    }

    // Initialize theme state
    const isDark =
      document.documentElement.classList.contains('dark') ||
      (!('theme' in localStorage) &&
        window.matchMedia('(prefers-color-scheme: dark)').matches);
    setIsDarkMode(isDark);
  }, []);

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && sidebarOpen) {
        setSidebarOpen(false);
      }
    };

    // Listen for custom toggle event from Breadcrumb
    const handleToggleSidebar = () => {
      toggleSidebar();
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

    const handleClickOutside = (e: MouseEvent | TouchEvent) => {
      // Only close sidebar if clicking outside on mobile
      if (
        isMobile &&
        sidebarOpen &&
        sidebarRef.current &&
        !sidebarRef.current.contains(e.target as Node)
      ) {
        setSidebarOpen(false);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    window.addEventListener('toggleSidebar', handleToggleSidebar);

    if (sidebarOpen) {
      document.addEventListener('keydown', handleEscape);
      document.addEventListener('click', handleClickOutside);
      document.addEventListener('touchend', handleClickOutside);

      // Prevent body scroll when sidebar is open on mobile
      if (isMobile) {
        const scrollY = window.scrollY;
        document.body.style.overflow = 'hidden';
        document.body.style.position = 'fixed';
        document.body.style.top = `-${scrollY}px`;
        document.body.style.width = '100%';
      }
    }

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('toggleSidebar', handleToggleSidebar);
      document.removeEventListener('keydown', handleEscape);
      document.removeEventListener('click', handleClickOutside);
      document.removeEventListener('touchend', handleClickOutside);

      // Restore body scroll on cleanup
      if (isMobile) {
        const scrollY = document.body.style.top;
        document.body.style.overflow = 'unset';
        document.body.style.position = 'static';
        document.body.style.top = 'auto';
        document.body.style.width = 'auto';
        if (scrollY) {
          window.scrollTo(0, parseInt(scrollY || '0') * -1);
        }
      }
    };
  }, [sidebarOpen, sidebarCollapsed, toggleSidebar, isMobile]);

  useEffect(() => {
    const handleResize = () => {
      const isMobileNow = window.innerWidth < 1024;
      setIsMobile(isMobileNow);
      if (!isMobileNow) {
        setSidebarOpen(false); // Close mobile sidebar on desktop
      }
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // Handle mobile sidebar visibility with direct DOM manipulation as fallback
  useEffect(() => {
    const sidebar = sidebarRef.current;
    if (sidebar && isMobile) {
      if (sidebarOpen) {
        // When opening on mobile, remove any forced transforms
        sidebar.style.transform = '';
        sidebar.style.pointerEvents = 'auto';
      } else {
        // When closing on mobile, force hide
        sidebar.style.transform = 'translateX(-100%)';
        sidebar.style.pointerEvents = 'none';
      }
    }
  }, [sidebarOpen, isMobile]);

  // Complete menu configuration with structured permissions
  const menuGroups = [
    {
      id: 'dashboard',
      label: 'Dashboard',
      icon: Home,
      href: '/',
      type: 'single' as const,
      requiredPermission: null, // Public to authenticated admin users
    },
    {
      id: 'users',
      label: 'User Management',
      icon: Users,
      type: 'group' as const,
      requiredPermission: 'admin:users:view',
      items: [
        {
          id: 'user-accounts',
          label: 'User Accounts',
          href: '/users',
          icon: Users,
          description: 'Manage user accounts and profiles',
          requiredPermission: 'admin:users:manage',
        },
        {
          id: 'user-create',
          label: 'Create User',
          href: '/users/create',
          icon: Users,
          description: 'Create new user accounts',
          requiredPermission: 'admin:users:create',
        },
        {
          id: 'user-permissions',
          label: 'User Permissions',
          href: '/users/permissions',
          icon: Shield,
          description: 'Manage user permissions globally',
          requiredPermission: 'admin:users:permissions',
        },
      ],
    },
    {
      id: 'system',
      label: 'System Management',
      icon: Server,
      type: 'group' as const,
      requiredPermission: 'admin:system:view',
      items: [
        {
          id: 'permissions',
          label: 'Permissions',
          href: '/permissions',
          icon: Shield,
          description: 'Global permission management',
          requiredPermission: 'admin:permissions:manage',
        },
        {
          id: 'notifications',
          label: 'Notifications',
          href: '/notifications',
          icon: Bell,
          description: 'System notification management',
          requiredPermission: 'admin:notifications:manage',
        },
        {
          id: 'stock-packages',
          label: 'Stock Ranking Packages',
          href: '/stock-ranking-packages',
          icon: Package,
          description: 'Manage stock ranking access packages',
          requiredPermission: 'admin:packages:manage',
        },
      ],
    },
    {
      id: 'analytics',
      label: 'Analytics & Reports',
      icon: BarChart3,
      type: 'group' as const,
      requiredPermission: 'admin:analytics:view',
      items: [
        {
          id: 'analytics-dashboard',
          label: 'Analytics Dashboard',
          href: '/analytics',
          icon: BarChart3,
          description: 'Performance metrics and system insights',
          requiredPermission: 'admin:analytics:view',
        },
      ],
    },
    {
      id: 'config',
      label: 'Configuration',
      icon: Settings,
      type: 'group' as const,
      requiredPermission: 'admin:config:view',
      items: [
        {
          id: 'general-settings',
          label: 'General Settings',
          href: '/settings',
          icon: Settings,
          description: 'Basic system configuration',
          requiredPermission: 'admin:system:configure',
        },
        {
          id: 'developer-portal',
          label: 'Developer Portal',
          href: '/developer-portal',
          icon: ExternalLink,
          description: 'Developer tools and API access',
          requiredPermission: 'admin:developer:access',
        },
      ],
    },
  ];

  // Auto-expand menus that have active children
  useEffect(() => {
    if (!sidebarCollapsed) {
      const activeMenus = menuGroups
        .filter(group => {
          if (group.type === 'single') return false;
          return group.items && group.items.some(item => isActive(item.href));
        })
        .map(group => group.id);

      if (activeMenus.length > 0) {
        setExpandedMenus(activeMenus);
      }
    }
  }, [pathname, sidebarCollapsed]);

  // Filter menu items based on structured permissions
  const hasUserPermission = (permission: string | null) => {
    if (!permission) return true; // No permission required
    
    // Ensure permissions is always treated as an array
    if (!session?.user?.permissions) return false;
    const permissions = Array.isArray(session.user.permissions) 
      ? session.user.permissions 
      : [];

    // Additional safety check
    if (permissions.length === 0) return false;

    // Check for admin wildcard permission
    if (permissions.includes('admin:*:*')) return true;

    // Check for exact permission match
    if (permissions.includes(permission)) return true;

    // Check for broader permissions (e.g., admin:users:* covers admin:users:view)
    if (permission.includes(':')) {
      const [platform, resource] = permission.split(':');
      return permissions.some(
        p => p === `${platform}:${resource}:*` || p === `${platform}:*:*`
      );
    }
    
    return false;
  };

  const filteredMenuGroups = menuGroups
    .filter(group => hasUserPermission(group.requiredPermission))
    .map(group => {
      if (group.type === 'single') {
        return group;
      }
      return {
        ...group,
        items: group.items?.filter(
          item =>
            hasUserPermission(item.requiredPermission) &&
            (item.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
              item.description
                .toLowerCase()
                .includes(searchQuery.toLowerCase()))
        ),
      };
    })
    .filter(group => {
      if (group.type === 'single') {
        return group.label.toLowerCase().includes(searchQuery.toLowerCase());
      }
      return group.items && group.items.length > 0;
    });

  const toggleMenu = (menuId: string) => {
    // Don't allow menu expansion when sidebar is collapsed
    if (sidebarCollapsed) return;

    setExpandedMenus(prev => {
      const isCurrentlyExpanded = prev.includes(menuId);

      if (isCurrentlyExpanded) {
        // If clicking on an expanded menu, collapse it completely
        return [];
      } else {
        // If clicking on a collapsed menu, expand only this one (accordion behavior)
        // This ensures all other menus are collapsed
        return [menuId];
      }
    });
  };

  const isMenuExpanded = (menuId: string) => expandedMenus.includes(menuId);

  const isActive = (href: string) => {
    // Special case for dashboard - only active when exactly on root path
    if (href === '/') {
      return pathname === '/';
    }
    return pathname.startsWith(href);
  };

  // Check if any child item is active for group styling
  const hasActiveChild = (
    items: { href: string; label: string }[] | undefined
  ) => {
    if (!items) return false;
    return items.some(item => isActive(item.href));
  };

  const handleLogout = async () => {
    try {
      // Use Zustand auth logout
      await logout();
    } catch (error) {
      console.error('Logout failed:', error);
      // Fallback to manual redirect
      router.replace('/login');
    }
  };

  const handleSearchKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      setSearchQuery('');
      setSearchFocused(false);
      searchRef.current?.blur();
    }
  };

  return (
    <div className="flex min-h-screen bg-black">
      {/* Mobile Sidebar Scrollbar Styling */}
      {isMobile && (
        <style jsx>{`
          .mobile-nav::-webkit-scrollbar {
            width: 6px;
          }
          .mobile-nav::-webkit-scrollbar-track {
            background: #1f2937;
            border-radius: 3px;
          }
          .mobile-nav::-webkit-scrollbar-thumb {
            background: #eab308;
            border-radius: 3px;
          }
          .mobile-nav::-webkit-scrollbar-thumb:hover {
            background: #f59e0b;
          }
        `}</style>
      )}
      {/* Mobile Overlay */}
      {sidebarOpen && isMobile && (
        <div
          className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm"
          onClick={() => setSidebarOpen(false)}
          onTouchEnd={() => setSidebarOpen(false)}
          style={{
            WebkitTapHighlightColor: 'transparent',
          }}
        />
      )}

      {/* PancakeSwap x Windows Phone Sidebar */}
      <div
        ref={sidebarRef}
        className={`fixed inset-y-0 left-0 z-50 flex transform flex-col border-r border-yellow-500/20 transition-all duration-300 ease-out ${
          sidebarOpen
            ? 'translate-x-0'
            : isMobile
              ? '-translate-x-full'
              : sidebarCollapsed
                ? '-translate-x-full'
                : 'translate-x-0'
        } shadow-2xl ${
          !sidebarOpen && isMobile
            ? 'pointer-events-none'
            : 'touch-manipulation'
        } ${
          isMobile
            ? 'border-r-2 border-yellow-500/40 bg-gray-900'
            : 'bg-gradient-to-b from-gray-900 to-black backdrop-blur-md'
        }`}
        style={{
          width: isMobile ? '280px' : sidebarCollapsed ? '80px' : '280px',
          minWidth: isMobile ? '280px' : sidebarCollapsed ? '80px' : '280px',
          maxWidth: isMobile ? '280px' : sidebarCollapsed ? '80px' : '280px',
        }}
      >
        {/* PancakeSwap x Windows Phone Header */}
        <div
          className={`border-b border-yellow-500/20 p-6 ${sidebarCollapsed && !isMobile ? 'px-3 py-4' : ''}`}
        >
          <div
            className={`flex items-center ${sidebarCollapsed && !isMobile ? 'justify-center' : 'justify-between'} mb-4`}
          >
            {(!sidebarCollapsed || isMobile) && (
              <div className="flex items-center gap-3">
                <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-gradient-to-r from-yellow-400 to-orange-500 shadow-lg">
                  <Coins className="h-6 w-6 text-black" />
                </div>
                <div>
                  <h1 className="text-xl font-extralight tracking-wide text-white">
                    EPSX Admin
                  </h1>
                </div>
              </div>
            )}
            {sidebarCollapsed && !isMobile && (
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-r from-yellow-400 to-orange-500 shadow-lg">
                <Coins className="h-5 w-5 text-black" />
              </div>
            )}
          </div>

          {/* Modern Search Bar */}
          {(!sidebarCollapsed || isMobile) && (
            <div className="relative">
              <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 transform text-gray-400" />
              <input
                ref={searchRef}
                type="text"
                placeholder="Search features..."
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
                onKeyDown={handleSearchKeyDown}
                onFocus={() => setSearchFocused(true)}
                onBlur={() => setSearchFocused(false)}
                className={`min-h-[44px] w-full touch-manipulation border border-gray-600 bg-gradient-to-r from-gray-800 to-gray-700 py-3 pr-4 pl-10 font-light text-white placeholder-gray-400 transition-all duration-300 focus:border-yellow-400 focus:ring-2 focus:ring-yellow-400 ${
                  searchFocused
                    ? 'shadow-lg ring-2 shadow-yellow-400/20 ring-yellow-400'
                    : ''
                }`}
              />
            </div>
          )}
        </div>

        {/* PancakeSwap x Windows Phone Navigation Menu */}
        <nav
          className={`flex-1 space-y-1 overflow-y-auto p-4 ${isMobile ? 'mobile-nav' : ''}`}
          style={
            isMobile
              ? {
                  scrollbarWidth: 'thin',
                  scrollbarColor: '#eab308 #1f2937',
                }
              : {}
          }
        >
          {filteredMenuGroups.map(group => {
            const Icon = group.icon;

            if (group.type === 'single') {
              const isActiveItem = isActive(group.href!);

              return (
                <Link
                  key={group.id}
                  href={group.href!}
                  onClick={() => setSidebarOpen(false)}
                  className={`group relative flex w-full items-center gap-3 overflow-hidden px-4 py-4 text-left transition-all duration-500 ${
                    sidebarCollapsed ? 'justify-center px-2' : ''
                  } ${
                    isActiveItem
                      ? 'border border-yellow-500/20 bg-gradient-to-r from-gray-800/40 to-gray-700/40 text-yellow-300 shadow-sm backdrop-blur-sm'
                      : 'border border-transparent text-gray-300 hover:border-yellow-500/20 hover:bg-gradient-to-r hover:from-gray-800/40 hover:to-gray-700/40 hover:text-yellow-100 hover:shadow-sm hover:backdrop-blur-sm'
                  } min-h-[48px] touch-manipulation hover:scale-[1.01] focus:ring-1 focus:ring-yellow-400/50 focus:outline-none active:scale-[0.99]`}
                  title={sidebarCollapsed ? group.label : undefined}
                >
                  {/* Subtle corner accent */}
                  {isActiveItem && (
                    <div className="absolute top-0 right-0 h-4 w-4 bg-gradient-to-bl from-yellow-300/10 to-transparent"></div>
                  )}

                  <div
                    className={`rounded-lg p-2 transition-all duration-500 ${
                      isActiveItem
                        ? 'bg-gray-600/30 shadow-sm'
                        : 'bg-gray-700/30 group-hover:bg-gray-600/30'
                    }`}
                  >
                    <Icon
                      className={`h-5 w-5 transition-colors duration-500 ${
                        isActiveItem
                          ? 'text-yellow-300'
                          : 'text-gray-400 group-hover:text-yellow-200'
                      }`}
                    />
                  </div>
                  {!sidebarCollapsed && (
                    <div className="min-w-0 flex-1">
                      <div
                        className={`text-sm font-light tracking-wide transition-colors duration-500 ${
                          isActiveItem
                            ? 'font-medium text-yellow-300'
                            : 'text-gray-300 group-hover:text-yellow-100'
                        }`}
                      >
                        {group.label}
                      </div>
                    </div>
                  )}
                </Link>
              );
            }

            const isExpanded = isMenuExpanded(group.id);
            const hasActiveChildItem = hasActiveChild(group.items);

            return (
              <div key={group.id} className="space-y-1">
                {/* PancakeSwap Group Header */}
                <button
                  onClick={() => toggleMenu(group.id)}
                  className={`group relative flex w-full items-center gap-3 overflow-hidden px-4 py-4 text-left transition-all duration-500 ${
                    sidebarCollapsed ? 'justify-center px-2' : ''
                  } ${
                    isExpanded
                      ? 'border border-yellow-500/30 bg-gradient-to-r from-gray-700/60 to-gray-800/60 text-yellow-300 shadow-lg backdrop-blur-sm'
                      : 'border border-transparent text-gray-300 hover:border-yellow-500/20 hover:bg-gradient-to-r hover:from-gray-800/70 hover:to-gray-700/70 hover:text-yellow-100 hover:shadow-md hover:backdrop-blur-sm'
                  } min-h-[48px] touch-manipulation hover:scale-[1.01] focus:ring-1 focus:ring-yellow-400/50 focus:outline-none active:scale-[0.99]`}
                  title={sidebarCollapsed ? group.label : undefined}
                >
                  {/* PancakeSwap corner accent */}
                  {hasActiveChildItem && (
                    <div className="absolute top-0 right-0 h-6 w-6 bg-gradient-to-bl from-yellow-300/20 to-transparent"></div>
                  )}

                  <div
                    className={`rounded-lg p-2 transition-all duration-500 ${
                      isExpanded
                        ? 'bg-yellow-500/20 shadow-sm'
                        : hasActiveChildItem
                          ? 'bg-yellow-500/10 shadow-sm'
                          : 'bg-gray-700/30 group-hover:bg-gray-600/50'
                    }`}
                  >
                    <Icon
                      className={`h-5 w-5 transition-colors duration-500 ${
                        isExpanded
                          ? 'text-yellow-300'
                          : hasActiveChildItem
                            ? 'text-yellow-400'
                            : 'text-gray-400 group-hover:text-yellow-200'
                      }`}
                    />
                  </div>
                  {!sidebarCollapsed && (
                    <>
                      <div className="min-w-0 flex-1">
                        <div
                          className={`text-sm font-light tracking-wide transition-colors duration-500 ${
                            isExpanded
                              ? 'font-medium text-yellow-300'
                              : hasActiveChildItem
                                ? 'font-medium text-yellow-400'
                                : 'text-gray-300 group-hover:text-yellow-100'
                          }`}
                        >
                          {group.label}
                        </div>
                        <div
                          className={`mt-0.5 text-xs font-light transition-colors duration-500 ${
                            isExpanded
                              ? 'text-yellow-400/80'
                              : hasActiveChildItem
                                ? 'text-yellow-500/80'
                                : 'text-gray-500 group-hover:text-gray-300'
                          }`}
                        >
                          {group.items?.length} items
                        </div>
                      </div>
                      <div
                        className={`transition-transform duration-500 ${isExpanded ? 'rotate-90' : ''}`}
                      >
                        <ChevronRight
                          className={`h-4 w-4 transition-colors duration-500 ${
                            isExpanded
                              ? 'text-yellow-300'
                              : hasActiveChildItem
                                ? 'text-yellow-400'
                                : 'text-gray-400 group-hover:text-yellow-200'
                          }`}
                        />
                      </div>
                    </>
                  )}
                </button>

                {/* Expandable Items */}
                {!sidebarCollapsed && isExpanded && (
                  <div className="max-h-96 overflow-hidden opacity-100 transition-all duration-200 ease-out">
                    <div className="space-y-1 pl-6">
                      {group.items?.map(item => {
                        const ItemIcon = item.icon;
                        const isActiveItem = isActive(item.href);

                        return (
                          <Link
                            key={item.id}
                            href={item.href}
                            onClick={() => setSidebarOpen(false)}
                            className={`group flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left transition-all duration-300 ${
                              isActiveItem
                                ? 'border border-yellow-500/40 bg-gradient-to-r from-yellow-500/20 to-yellow-600/20 text-yellow-300 shadow-sm backdrop-blur-sm'
                                : 'border border-transparent text-gray-400 hover:border-yellow-500/20 hover:bg-gradient-to-r hover:from-gray-700/50 hover:to-gray-600/50 hover:text-gray-200 hover:shadow-sm hover:backdrop-blur-sm'
                            } min-h-[44px] touch-manipulation hover:scale-[1.005] focus:ring-1 focus:ring-yellow-400/30 focus:outline-none active:scale-[0.995]`}
                          >
                            <div className="flex w-full items-center gap-2">
                              <div
                                className={`h-6 w-1 rounded-full transition-colors duration-300 ${
                                  isActiveItem
                                    ? 'bg-yellow-400'
                                    : 'bg-gray-600 group-hover:bg-yellow-500/50'
                                }`}
                              ></div>
                              <div
                                className={`rounded-lg p-2 transition-all duration-300 ${
                                  isActiveItem
                                    ? 'bg-yellow-500/20 shadow-sm'
                                    : 'bg-gray-700/50 group-hover:bg-gray-600/70'
                                }`}
                              >
                                <ItemIcon
                                  className={`h-4 w-4 transition-colors duration-300 ${
                                    isActiveItem
                                      ? 'text-yellow-300'
                                      : 'text-gray-400 group-hover:text-gray-200'
                                  }`}
                                />
                              </div>
                              <div className="min-w-0 flex-1">
                                <div
                                  className={`text-sm font-medium transition-colors duration-300 ${
                                    isActiveItem
                                      ? 'text-yellow-300'
                                      : 'text-gray-200 group-hover:text-white'
                                  }`}
                                >
                                  {item.label}
                                </div>
                                <div
                                  className={`mt-0.5 truncate text-xs transition-colors duration-300 ${
                                    isActiveItem
                                      ? 'text-yellow-400/80'
                                      : 'text-gray-500 group-hover:text-gray-300'
                                  }`}
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

        {/* DeFi-Style User Profile & Logout */}
        <div
          className={`space-y-4 border-t border-yellow-500/20 p-4 ${sidebarCollapsed ? 'px-2' : ''}`}
        >
          {/* User Profile */}
          <div
            className={`flex items-center gap-3 rounded-lg border border-yellow-500/10 bg-gradient-to-r from-gray-800/50 to-gray-700/50 p-3 backdrop-blur-sm transition-all duration-150 ${
              sidebarCollapsed ? 'justify-center px-2' : ''
            }`}
          >
            <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-gradient-to-r from-yellow-400 to-orange-500 shadow-lg">
              <span className="text-sm font-bold text-black">
                {session?.user?.email?.charAt(0).toUpperCase() || 'A'}
              </span>
            </div>
            {!sidebarCollapsed && (
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium text-white">
                  {session?.user?.email || 'Admin User'}
                </p>
                <p className="truncate text-xs font-light text-yellow-400">
                  {(session?.user?.permissions || []).length} permissions
                </p>
              </div>
            )}
            {!sidebarCollapsed && (
              <div className="flex items-center gap-1">
                <div className="h-2 w-2 animate-pulse rounded-full bg-green-400"></div>
                <span className="text-xs font-light text-green-400">Live</span>
              </div>
            )}
          </div>

          {/* DeFi Logout Button */}
          <button
            onClick={handleLogout}
            disabled={isSigningOut}
            className={`flex min-h-[44px] w-full transform touch-manipulation items-center gap-3 rounded-lg border border-red-500/20 px-4 py-3 text-sm font-medium text-red-300 transition-all duration-200 hover:scale-[1.02] hover:border-red-500 hover:bg-gradient-to-r hover:from-red-600 hover:to-red-700 hover:text-white focus:ring-2 focus:ring-red-400 focus:ring-offset-2 focus:ring-offset-gray-900 focus:outline-none active:scale-[0.98] disabled:opacity-50 ${
              sidebarCollapsed ? 'justify-center px-2' : ''
            }`}
            title={sidebarCollapsed ? 'Sign out' : undefined}
          >
            <LogOut className="h-4 w-4" />
            {!sidebarCollapsed && (
              <span className="font-light tracking-wide">
                {isSigningOut ? 'Signing out...' : 'Sign out'}
              </span>
            )}
          </button>
        </div>
      </div>

      {/* Main Content Area */}
      <div
        className="flex min-w-0 flex-1 flex-col transition-all duration-200 ease-out"
        style={{
          marginLeft: !isMobile ? (sidebarCollapsed ? '80px' : '280px') : '0px',
        }}
      >
        {/* Windows Phone + PancakeSwap Header Bar */}
        <div className="wp-pancake-header-bg relative overflow-hidden px-4 py-4 shadow-2xl lg:px-6">

          <div className="relative z-10">
            <div className="flex items-center justify-between">
              {/* Enhanced Sidebar Toggle + Breadcrumb */}
              <div className="flex items-center gap-4">
                {/* Mobile Menu Toggle - DeFi Style */}
                <button
                  onClick={() => setSidebarOpen(!sidebarOpen)}
                  className="relative z-[60] flex h-11 min-h-[44px] w-11 min-w-[44px] transform touch-manipulation items-center justify-center rounded-xl bg-gradient-to-r from-yellow-400 to-orange-500 text-black shadow-lg transition-all duration-200 hover:scale-105 hover:from-yellow-500 hover:to-orange-600 focus:ring-2 focus:ring-yellow-400 focus:ring-offset-2 focus:ring-offset-gray-900 focus:outline-none active:scale-95 lg:hidden"
                  aria-label={
                    sidebarOpen ? 'Close mobile menu' : 'Open mobile menu'
                  }
                >
                  {sidebarOpen ? (
                    <X className="h-5 w-5 font-bold" />
                  ) : (
                    <Menu className="h-5 w-5 font-bold" />
                  )}
                </button>

                {/* Desktop Sidebar Toggle - DeFi Style */}
                <button
                  onClick={toggleSidebar}
                  className="group hidden h-11 w-11 items-center justify-center rounded-xl bg-gradient-to-r from-gray-700 to-gray-800 text-white shadow-lg transition-all duration-200 hover:from-yellow-400 hover:to-orange-500 hover:text-black focus:ring-2 focus:ring-yellow-400 focus:ring-offset-2 focus:ring-offset-gray-900 focus:outline-none lg:flex"
                  title="Toggle sidebar (Ctrl+B)"
                >
                  <PanelLeft className="h-5 w-5 transition-transform duration-200 group-hover:scale-110" />
                </button>

                {/* DeFi-Enhanced Breadcrumb Container */}
                <div className="hidden items-center rounded-xl border border-yellow-500/20 bg-black/20 px-4 py-2.5 backdrop-blur-sm sm:flex">
                  <ServerBreadcrumb />
                </div>
              </div>

              {/* DeFi-Style Header Actions */}
              <div className="flex items-center gap-3 lg:gap-5">
                {/* Live Date Display */}
                <div className="hidden items-center gap-3 rounded-xl border border-yellow-500/20 bg-black/30 px-4 py-2 backdrop-blur-sm md:flex">
                  <div className="h-2 w-2 animate-pulse rounded-full bg-green-400"></div>
                  <div className="text-sm font-light text-white">
                    {new Date().toLocaleDateString('en-US', {
                      weekday: 'short',
                      month: 'short',
                      day: 'numeric',
                    })}
                  </div>
                </div>

                {/* DeFi Theme Toggle */}
                <button
                  onClick={toggleTheme}
                  className="group flex h-11 min-h-[44px] w-11 min-w-[44px] touch-manipulation items-center justify-center rounded-xl bg-gradient-to-r from-gray-700 to-gray-800 text-white shadow-lg transition-all duration-200 hover:from-yellow-400 hover:to-orange-500 hover:text-black focus:ring-2 focus:ring-yellow-400 focus:ring-offset-2 focus:ring-offset-gray-900 focus:outline-none"
                  title={
                    isDarkMode ? 'Switch to light mode' : 'Switch to dark mode'
                  }
                >
                  {isDarkMode ? (
                    <Sun className="h-5 w-5 transition-transform group-hover:scale-110" />
                  ) : (
                    <Moon className="h-5 w-5 transition-transform group-hover:scale-110" />
                  )}
                </button>

                {/* Admin Notification Bell - Temporarily disabled */}
                <button
                  className="group flex h-11 min-h-[44px] w-11 min-w-[44px] touch-manipulation items-center justify-center rounded-xl bg-gradient-to-r from-gray-700 to-gray-800 text-white shadow-lg transition-all duration-200 hover:from-yellow-400 hover:to-orange-500 hover:text-black focus:ring-2 focus:ring-yellow-400 focus:ring-offset-2 focus:ring-offset-gray-900 focus:outline-none"
                  title="Notifications"
                >
                  <Bell className="h-5 w-5 transition-transform group-hover:scale-110" />
                </button>
              </div>
            </div>

            {/* Mobile Breadcrumb - Responsive */}
            <div className="mt-3 flex items-center rounded-xl border border-yellow-500/20 bg-black/20 px-4 py-2.5 backdrop-blur-sm sm:hidden">
              <ServerBreadcrumb />
            </div>
          </div>
        </div>

        {/* Page Content with Windows Phone + PancakeSwap Background */}
        <div className="wp-pancake-page-bg relative flex-1 overflow-auto">
          <div className="wp-pancake-content-bg relative z-10 p-4 lg:p-6">
            <div className="mx-auto max-w-7xl">
              <div className="animate-fade-in transition-all duration-300">
                {children}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
