'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';

interface NavItem {
    id: string;
    label: string;
    href: string;
    icon: string;
    description?: string;
    children?: NavItem[];
}

const navigationItems: NavItem[] = [
    {
        id: 'dashboard',
        label: 'API Keys',
        href: '/developer',
        icon: '🔑',
        description: 'Manage your API keys',
    },
    {
        id: 'documentation',
        label: 'Documentation',
        href: '/developer/docs',
        icon: '📚',
        description: 'API Reference',
    },
    {
        id: 'usage',
        label: 'Usage & Monitoring',
        href: '/developer/usage',
        icon: '📊',
        description: 'Monitor your usage',
    },
];

interface DeveloperSidebarProps {
    title?: string;
    className?: string;
}

/**
 * Developer Portal Sidebar - Responsive with mobile support
 * Provides navigation for the Developer Portal section
 */
export function DeveloperSidebar({ title = 'Developer', className = '' }: DeveloperSidebarProps) {
    const pathname = usePathname();
    const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
    const [isMobileOpen, setIsMobileOpen] = useState(false);
    const [isMounted, setIsMounted] = useState(false);

    // Get current page title based on pathname
    const getCurrentPageTitle = () => {
        const currentItem = navigationItems.find(item =>
            pathname === item.href ||
            (item.href !== '/developer' && pathname.startsWith(`${item.href}/`))
        );
        return currentItem?.label || 'Developer Portal';
    };

    // Ensure component is mounted before showing interactive elements
    useEffect(() => {
        setIsMounted(true);
    }, []);

    // Close sidebar on route change (mobile)
    useEffect(() => {
        setIsMobileOpen(false);
    }, [pathname]);

    // Close sidebar on escape key
    useEffect(() => {
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsMobileOpen(false);
            }
        };
        window.addEventListener('keydown', handleEscape);
        return () => window.removeEventListener('keydown', handleEscape);
    }, []);

    // Prevent body scroll when mobile sidebar is open
    useEffect(() => {
        if (isMobileOpen) {
            document.body.style.overflow = 'hidden';
        } else {
            document.body.style.overflow = '';
        }
        return () => {
            document.body.style.overflow = '';
        };
    }, [isMobileOpen]);

    const toggleExpanded = useCallback((itemId: string) => {
        setExpandedItems(prev => {
            const newExpanded = new Set(prev);
            if (newExpanded.has(itemId)) {
                newExpanded.delete(itemId);
            } else {
                newExpanded.add(itemId);
            }
            return newExpanded;
        });
    }, []);

    const handleOpenSidebar = useCallback(() => {
        setIsMobileOpen(true);
    }, []);

    const handleCloseSidebar = useCallback(() => {
        setIsMobileOpen(false);
    }, []);

    // Navigation items renderer
    const renderNavItems = (onItemClick?: () => void) => (
        <nav className="flex-1 p-4 space-y-2 overflow-y-auto">
            {navigationItems.map(item => {
                const isActive = pathname === item.href ||
                    (item.href !== '/developer' && pathname.startsWith(`${item.href}/`)) ||
                    (item.id === 'dashboard' && pathname === '/developer');
                const hasActiveChild = item.children?.some(child =>
                    pathname === child.href || pathname.startsWith(`${child.href}/`)
                );
                const isExpanded = expandedItems.has(item.id);

                return (
                    <div key={item.id}>
                        {/* Main Item */}
                        <div className="relative">
                            <Link
                                href={item.href}
                                onClick={() => onItemClick?.()}
                            >
                                <div className={`flex items-center gap-3 px-3 py-3 lg:py-2 rounded-lg min-w-0 overflow-hidden transition-all duration-200 ${isActive || hasActiveChild
                                    ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200'
                                    : 'text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800'
                                    }`}>
                                    <span className="text-xl lg:text-lg flex-shrink-0">{item.icon}</span>
                                    <div className="flex flex-col min-w-0">
                                        <span className="font-medium text-base lg:text-sm text-ellipsis whitespace-nowrap overflow-hidden" style={{ textOverflow: 'ellipsis' }}>
                                            {item.label}
                                        </span>
                                        {item.description && (
                                            <span className="text-xs text-gray-500 dark:text-gray-400 text-ellipsis whitespace-nowrap overflow-hidden hidden lg:block" style={{ textOverflow: 'ellipsis' }}>
                                                {item.description}
                                            </span>
                                        )}
                                    </div>
                                </div>
                            </Link>

                            {/* Expand button for items with children */}
                            {item.children && (
                                <button
                                    type="button"
                                    onClick={(e) => {
                                        e.preventDefault();
                                        e.stopPropagation();
                                        toggleExpanded(item.id);
                                    }}
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
                                        <div key={child.id}>
                                            <Link
                                                href={child.href}
                                                onClick={() => onItemClick?.()}
                                            >
                                                <div className={`flex items-center gap-3 px-3 py-3 lg:py-2 rounded-lg min-w-0 overflow-hidden transition-all duration-200 ${childIsActive
                                                    ? 'bg-blue-50 text-blue-600 dark:bg-blue-900/50 dark:text-blue-300'
                                                    : 'text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800/50'
                                                    }`}>
                                                    <span className="text-base lg:text-sm flex-shrink-0">{child.icon}</span>
                                                    <span className="text-base lg:text-sm font-medium text-ellipsis whitespace-nowrap overflow-hidden" style={{ textOverflow: 'ellipsis' }}>
                                                        {child.label}
                                                    </span>
                                                </div>
                                            </Link>
                                        </div>
                                    );
                                })}
                            </div>
                        )}
                    </div>
                );
            })}
        </nav>
    );

    // Footer renderer
    const renderFooter = () => (
        <div className="p-4 border-t border-gray-200 dark:border-gray-700">
            <div className="bg-gradient-to-r from-emerald-50 to-blue-50 dark:from-emerald-900/30 dark:to-blue-900/30 rounded-lg p-3 border border-emerald-200 dark:border-emerald-700">
                <div className="flex items-center gap-2 mb-2">
                    <span className="text-lg">🚀</span>
                    <span className="text-sm font-semibold text-emerald-800 dark:text-emerald-200">API Status</span>
                </div>
                <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></div>
                    <span className="text-xs text-emerald-600 dark:text-emerald-300">All systems operational</span>
                </div>
            </div>
        </div>
    );

    // Don't render interactive mobile elements until mounted (prevents hydration mismatch)
    if (!isMounted) {
        return (
            <>
                {/* Desktop sidebar - always render */}
                <div className={`hidden lg:flex w-64 min-w-0 max-w-64 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 h-screen flex-col ${className}`}>
                    <div className="p-4 border-b border-gray-200 dark:border-gray-700">
                        <h1 className="text-xl font-bold text-gray-900 dark:text-white truncate">{title}</h1>
                    </div>
                    {renderNavItems()}
                    {renderFooter()}
                </div>
            </>
        );
    }

    return (
        <>
            {/* ============================================= */}
            {/* MOBILE HEADER - Below main site nav (top-14) */}
            {/* ============================================= */}
            <div
                className="lg:hidden fixed top-14 left-0 right-0 bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-gray-800 dark:to-gray-900 border-b border-blue-200 dark:border-gray-700 shadow-md"
                style={{ zIndex: 60 }}
            >
                <div className="flex items-center justify-between h-12 px-4">
                    {/* Hamburger Menu Button - More prominent */}
                    <button
                        type="button"
                        onClick={handleOpenSidebar}
                        className="flex items-center gap-2.5 px-3 py-2 -ml-1 rounded-xl bg-white dark:bg-gray-800 shadow-sm border border-blue-200 dark:border-gray-600 hover:bg-blue-50 dark:hover:bg-gray-700 active:scale-95 transition-all"
                        aria-label="Open developer menu"
                    >
                        {/* Developer Icon */}
                        <span className="text-lg">🛠️</span>
                        {/* Menu Text */}
                        <span className="text-sm font-semibold text-blue-700 dark:text-blue-300">Developer Menu</span>
                        {/* Arrow Icon */}
                        <svg className="w-4 h-4 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                        </svg>
                    </button>

                    {/* Page Title Badge */}
                    <div className="flex items-center gap-2 px-3 py-1.5 bg-white/80 dark:bg-gray-800/80 rounded-full border border-blue-200 dark:border-gray-600">
                        <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse"></div>
                        <span className="text-xs font-medium text-gray-700 dark:text-gray-300">
                            {getCurrentPageTitle()}
                        </span>
                    </div>
                </div>
            </div>

            {/* Mobile Overlay - clickable to close - extends to cover safe areas */}
            {isMobileOpen && (
                <div
                    className="lg:hidden fixed bg-black/50 backdrop-blur-sm"
                    onClick={handleCloseSidebar}
                    aria-hidden="true"
                    style={{
                        top: 0,
                        left: 0,
                        right: 0,
                        bottom: 0,
                        zIndex: 9998,
                        // Extend beyond safe area on iOS
                        paddingBottom: 'env(safe-area-inset-bottom, 0px)',
                    }}
                />
            )}

            {/* Sidebar - Desktop (always visible) */}
            <div className={`hidden lg:flex w-64 min-w-0 max-w-64 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 h-screen flex-col ${className}`}>
                {/* Desktop Header */}
                <div className="p-4 border-b border-gray-200 dark:border-gray-700">
                    <h1 className="text-xl font-bold text-gray-900 dark:text-white truncate">{title}</h1>
                </div>
                {renderNavItems()}
                {renderFooter()}
            </div>

            {/* Sidebar - Mobile (slide in from left) - Using inline style for transform and safe areas */}
            <div
                className="lg:hidden fixed left-0 w-72 max-w-[85vw] bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 flex flex-col transition-transform duration-300 ease-in-out shadow-2xl"
                style={{
                    top: 0,
                    bottom: 0,
                    zIndex: 9999,
                    transform: isMobileOpen ? 'translateX(0)' : 'translateX(-100%)',
                    // Extend to cover iOS safe areas
                    paddingBottom: 'env(safe-area-inset-bottom, 0px)',
                    minHeight: '100dvh', // Dynamic viewport height for iOS (fallback to 100vh in older browsers)
                }}
            >
                {/* Mobile Sidebar Header with Close Button */}
                <div className="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/30 dark:to-indigo-900/30">
                    <div className="flex items-center gap-3">
                        <span className="text-2xl">🛠️</span>
                        <h1 className="text-xl font-bold text-gray-900 dark:text-white truncate">{title}</h1>
                    </div>
                    <button
                        type="button"
                        onClick={handleCloseSidebar}
                        className="p-2 rounded-xl bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-800 border border-gray-200 dark:border-gray-700 transition-colors active:scale-95"
                        aria-label="Close menu"
                    >
                        <svg className="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>
                {renderNavItems(handleCloseSidebar)}
                {/* Footer with extra padding for iOS safe area */}
                <div style={{ paddingBottom: 'env(safe-area-inset-bottom, 0px)' }}>
                    {renderFooter()}
                </div>
            </div>
        </>
    );
}

/**
 * Mobile Header Component - Optional for pages that need header content
 */
export function DeveloperMobileHeader({ title }: { title?: string }) {
    return (
        <div className="lg:hidden flex items-center justify-center h-14 border-b border-gray-200 dark:border-gray-700 bg-white/80 dark:bg-gray-900/80 backdrop-blur-xl sticky top-0 z-30">
            <h1 className="text-lg font-semibold text-gray-900 dark:text-white">
                {title || 'Developer Portal'}
            </h1>
        </div>
    );
}
