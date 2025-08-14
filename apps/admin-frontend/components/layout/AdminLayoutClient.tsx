'use client';

import { ReactNode, useState, useCallback, useEffect, useRef } from 'react';
import { usePathname, useRouter } from 'next/navigation';
import { useAuth } from '@/lib/auth';

interface Session {
  user?: {
    id: string;
    email: string;
    name?: string;
    role: string;
    adminModules: string[];
    permissions: string[];
    packageTier: string;
  };
  isLoggedIn: boolean;
}
import Link from 'next/link';
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

interface AdminLayoutClientProps {
  children: ReactNode;
  session: Session | null;
}

export function AdminLayoutClient({ children, session }: AdminLayoutClientProps) {
  const router = useRouter();
  const pathname = usePathname();
  const { logout, isLoading: isSigningOut } = useAuth();
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

    document.addEventListener('keydown', handleKeyDown);

    if (sidebarOpen) {
      document.addEventListener('keydown', handleEscape);
      document.addEventListener('click', handleClickOutside);
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

  // Menu configuration with admin module checks
  const menuGroups = [
    {
      id: 'dashboard',
      label: 'Dashboard',
      icon: Home,
      href: '/',
      type: 'single' as const,
      requiredModule: null,
    },
    {
      id: 'users',
      label: 'User Management',
      icon: Users,
      type: 'group' as const,
      requiredModule: 'user_operations',
      items: [
        {
          id: 'user-list',
          label: 'User Accounts',
          href: '/users',
          icon: Users,
          description: 'Unified user management hub with profiles and permissions',
          requiredModule: 'user_operations',
        },
      ],
    },
    {
      id: 'security',
      label: 'Security & Access',
      icon: Shield,
      type: 'group' as const,
      requiredModule: null,
      items: [
        {
          id: 'permission-profiles',
          label: 'Permission Profiles',
          href: '/permission-profiles',
          icon: Shield,
          description: 'Manage permission profiles and assignments',
          requiredModule: 'permission_admin',
        },
        {
          id: 'developer-portal',
          label: 'Developer Portal',
          href: '/developer-portal',
          icon: ExternalLink,
          description: 'Access developer tools and API documentation',
          requiredModule: 'developer_relations',
        },
      ],
    },
    {
      id: 'analytics',
      label: 'Analytics & Reports',
      icon: BarChart3,
      type: 'group' as const,
      requiredModule: 'analytics_specialist',
      items: [
        {
          id: 'analytics-overview',
          label: 'Analytics Dashboard',
          href: '/analytics',
          icon: BarChart3,
          description: 'Performance metrics and insights',
          requiredModule: 'analytics_specialist',
        },
      ],
    },
    {
      id: 'system',
      label: 'System Management',
      icon: Server,
      type: 'group' as const,
      requiredModule: 'system_admin',
      items: [
        {
          id: 'database',
          label: 'Database',
          href: '/database',
          icon: Database,
          description: 'Database management and monitoring',
          requiredModule: 'system_admin',
        },
        {
          id: 'servers',
          label: 'Server Status',
          href: '/servers',
          icon: Server,
          description: 'Server health and performance',
          requiredModule: 'system_admin',
        },
      ],
    },
    {
      id: 'settings',
      label: 'Configuration',
      icon: Settings,
      type: 'group' as const,
      requiredModule: null,
      items: [
        {
          id: 'general-settings',
          label: 'General Settings',
          href: '/settings',
          icon: Settings,
          description: 'Basic system configuration',
          requiredModule: 'system_admin',
        },
      ],
    },
  ];

  // Filter menu items based on session's admin modules
  const hasUserAdminModule = (module: string | null) => {
    if (!module) return true; // No module required
    const adminModules = session?.user?.admin_modules || [];
    return adminModules.includes(module);
  };

  const filteredMenuGroups = menuGroups
    .filter(group => hasUserAdminModule(group.requiredModule))
    .map(group => {
      if (group.type === 'single') {
        return group;
      }
      return {
        ...group,
        items: group.items?.filter(
          item => 
            hasUserAdminModule(item.requiredModule) &&
            (item.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
             item.description.toLowerCase().includes(searchQuery.toLowerCase()))
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
    setExpandedMenus(prev =>
      prev.includes(menuId)
        ? prev.filter(id => id !== menuId)
        : [...prev, menuId]
    );
  };

  const isMenuExpanded = (menuId: string) => expandedMenus.includes(menuId);

  const isActive = (href: string) => {
    return pathname.startsWith(href);
  };

  const isGroupActive = (items: { href: string; label: string }[]) => {
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
    <div className="flex min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Mobile Sidebar Toggle Button */}
      <div className="lg:hidden fixed top-4 left-4 z-50">
        <button
          onClick={() => setSidebarOpen(!sidebarOpen)}
          className="p-3 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-all duration-150 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900 min-h-[44px] min-w-[44px] touch-manipulation"
          aria-label={sidebarOpen ? 'Close mobile menu' : 'Open mobile menu'}
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
          className="p-3 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-all duration-150 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900 min-h-[44px] min-w-[44px] touch-manipulation"
          aria-label={sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
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
        />
      )}

      {/* Sidebar */}
      <div
        ref={sidebarRef}
        className={`bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col fixed inset-y-0 left-0 z-50 transform transition-all duration-200 ease-out ${
          sidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'
        } ${sidebarCollapsed ? 'w-16' : 'w-80'} ${
          sidebarCollapsed ? 'lg:w-16' : 'lg:w-80'
        } shadow-lg lg:shadow-none`}
      >
        {/* Header */}
        <div className={`p-6 border-b border-gray-200 dark:border-gray-700 ${sidebarCollapsed ? 'px-2 py-4' : ''}`}>
          <div className={`flex items-center ${sidebarCollapsed ? 'justify-center' : 'justify-between'} mb-4`}>
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
                onFocus={() => setSearchFocused(true)}
                onBlur={() => setSearchFocused(false)}
                className={`w-full pl-10 pr-4 py-3 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-150 min-h-[44px] touch-manipulation ${
                  searchFocused ? 'ring-2 ring-blue-500 shadow-sm' : ''
                }`}
              />
            </div>
          )}
        </div>

        {/* Navigation Menu */}
        <nav className="flex-1 p-4 space-y-2 overflow-y-auto">
          {filteredMenuGroups.map((group, index) => {
            const Icon = group.icon;

            if (group.type === 'single') {
              const isActiveItem = isActive(group.href!);

              return (
                <Link
                  key={group.id}
                  href={group.href!}
                  onClick={() => setSidebarOpen(false)}
                  className={`group w-full flex items-center gap-3 px-4 py-3 rounded-xl text-left transition-all duration-150 ${
                    sidebarCollapsed ? 'justify-center px-2' : ''
                  } ${
                    isActiveItem
                      ? 'bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 text-blue-700 dark:text-blue-300 border border-blue-200/50 dark:border-blue-800/50 shadow-sm'
                      : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50 border border-transparent hover:border-gray-200 dark:hover:border-gray-600'
                  } transform hover:scale-[1.01] active:scale-[0.99] focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800 min-h-[44px] touch-manipulation`}
                  title={sidebarCollapsed ? group.label : undefined}
                >
                  <div
                    className={`p-2.5 rounded-lg transition-all duration-200 ${
                      isActiveItem
                        ? 'bg-blue-100 dark:bg-blue-800/30 shadow-sm'
                        : 'bg-gray-100 dark:bg-gray-600 group-hover:bg-gray-200 dark:group-hover:bg-gray-500'
                    }`}
                  >
                    <Icon
                      className={`h-5 w-5 transition-colors ${
                        isActiveItem ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'
                      }`}
                    />
                  </div>
                  {!sidebarCollapsed && (
                    <div className="flex-1 min-w-0">
                      <div
                        className={`font-medium text-sm transition-colors ${
                          isActiveItem ? 'text-blue-700 dark:text-blue-300' : 'text-gray-900 dark:text-white'
                        }`}
                      >
                        {group.label}
                      </div>
                    </div>
                  )}
                </Link>
              );
            }

            const isGroupActiveState = group.items ? isGroupActive(group.items) : false;
            const isExpanded = isMenuExpanded(group.id);

            return (
              <div key={group.id} className="space-y-1">
                {/* Group Header */}
                <button
                  onClick={() => toggleMenu(group.id)}
                  className={`group w-full flex items-center gap-3 px-4 py-3 rounded-xl text-left transition-all duration-150 ${
                    sidebarCollapsed ? 'justify-center px-2' : ''
                  } ${
                    isGroupActiveState
                      ? 'bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 text-blue-700 dark:text-blue-300 border border-blue-200/50 dark:border-blue-800/50 shadow-sm'
                      : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50 border border-transparent hover:border-gray-200 dark:hover:border-gray-600'
                  } transform hover:scale-[1.01] active:scale-[0.99] focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800 min-h-[44px] touch-manipulation`}
                  title={sidebarCollapsed ? group.label : undefined}
                >
                  <div
                    className={`p-2.5 rounded-lg transition-all duration-200 ${
                      isGroupActiveState
                        ? 'bg-blue-100 dark:bg-blue-800/30 shadow-sm'
                        : 'bg-gray-100 dark:bg-gray-600 group-hover:bg-gray-200 dark:group-hover:bg-gray-500'
                    }`}
                  >
                    <Icon
                      className={`h-5 w-5 transition-colors ${
                        isGroupActiveState ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'
                      }`}
                    />
                  </div>
                  {!sidebarCollapsed && (
                    <>
                      <div className="flex-1 min-w-0">
                        <div
                          className={`font-medium text-sm transition-colors ${
                            isGroupActiveState ? 'text-blue-700 dark:text-blue-300' : 'text-gray-900 dark:text-white'
                          }`}
                        >
                          {group.label}
                        </div>
                        <div
                          className={`text-xs mt-0.5 transition-colors ${
                            isGroupActiveState
                              ? 'text-blue-600/80 dark:text-blue-400/80'
                              : 'text-gray-500 dark:text-gray-400'
                          }`}
                        >
                          {group.items?.length} items
                        </div>
                      </div>
                      <div className={`transition-transform duration-200 ${isExpanded ? 'rotate-90' : ''}`}>
                        <ChevronRight
                          className={`h-4 w-4 transition-colors ${
                            isGroupActiveState ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400'
                          }`}
                        />
                      </div>
                    </>
                  )}
                </button>

                {/* Expandable Items */}
                {!sidebarCollapsed && (
                  <div
                    className={`transition-all duration-200 ease-out overflow-hidden ${
                      isExpanded ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0'
                    }`}
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
                            className={`group w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-all duration-150 ${
                              isActiveItem
                                ? 'bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 text-blue-700 dark:text-blue-300 border border-blue-200/50 dark:border-blue-800/50 shadow-sm'
                                : 'text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700/50 border border-transparent hover:border-gray-200 dark:hover:border-gray-600'
                            } transform hover:scale-[1.005] active:scale-[0.995] focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800 min-h-[44px] touch-manipulation`}
                          >
                            <div className="flex items-center gap-2 w-full">
                              <div
                                className={`w-1 h-6 rounded-full transition-colors ${
                                  isActiveItem ? 'bg-blue-500' : 'bg-gray-300 dark:bg-gray-600'
                                }`}
                              ></div>
                              <div
                                className={`p-2 rounded-lg transition-all duration-200 ${
                                  isActiveItem
                                    ? 'bg-blue-100 dark:bg-blue-800/30 shadow-sm'
                                    : 'bg-gray-100 dark:bg-gray-600 group-hover:bg-gray-200 dark:group-hover:bg-gray-500'
                                }`}
                              >
                                <ItemIcon
                                  className={`h-4 w-4 transition-colors ${
                                    isActiveItem ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'
                                  }`}
                                />
                              </div>
                              <div className="flex-1 min-w-0">
                                <div
                                  className={`font-medium text-sm transition-colors ${
                                    isActiveItem ? 'text-blue-700 dark:text-blue-300' : 'text-gray-900 dark:text-white'
                                  }`}
                                >
                                  {item.label}
                                </div>
                                <div
                                  className={`text-xs mt-0.5 truncate transition-colors ${
                                    isActiveItem
                                      ? 'text-blue-600/80 dark:text-blue-400/80'
                                      : 'text-gray-500 dark:text-gray-400'
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

        {/* User Profile & Logout */}
        <div className={`border-t border-gray-200 dark:border-gray-700 p-4 space-y-4 ${sidebarCollapsed ? 'px-2' : ''}`}>
          {/* User Profile */}
          <div
            className={`flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg transition-all duration-150 ${
              sidebarCollapsed ? 'justify-center px-2' : ''
            }`}
          >
            <div className="h-10 w-10 rounded-full bg-gradient-to-r from-blue-500 to-indigo-500 flex items-center justify-center flex-shrink-0">
              <span className="text-sm font-bold text-white">{session?.user?.email?.charAt(0).toUpperCase() || 'A'}</span>
            </div>
            {!sidebarCollapsed && (
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-gray-900 dark:text-white truncate">
                  {session?.user?.email || 'Admin User'}
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                  {(session?.user?.admin_modules || []).length} modules
                </p>
              </div>
            )}
            {!sidebarCollapsed && (
              <div className="flex items-center gap-1">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                <span className="text-xs text-green-600 dark:text-green-400">Online</span>
              </div>
            )}
          </div>

          {/* Logout Button */}
          <button
            onClick={handleLogout}
            disabled={isSigningOut}
            className={`w-full flex items-center gap-3 px-4 py-3 text-sm font-medium text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-all duration-150 border border-transparent hover:border-red-200 dark:hover:border-red-800 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800 min-h-[44px] touch-manipulation disabled:opacity-50 ${
              sidebarCollapsed ? 'justify-center px-2' : ''
            }`}
            title={sidebarCollapsed ? 'Sign out' : undefined}
          >
            <LogOut className="h-4 w-4" />
            {!sidebarCollapsed && <span>{isSigningOut ? 'Signing out...' : 'Sign out'}</span>}
          </button>
        </div>
      </div>

      {/* Main Content Area */}
      <div
        className={`flex-1 flex flex-col min-w-0 transition-all duration-200 ease-out ${
          sidebarCollapsed ? 'lg:ml-16' : 'lg:ml-80'
        }`}
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
              <button className="p-3 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 relative rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-150 min-h-[44px] min-w-[44px] touch-manipulation focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800">
                <Bell className="h-5 w-5" />
                <span className="absolute -top-1 -right-1 h-3 w-3 bg-red-500 rounded-full"></span>
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