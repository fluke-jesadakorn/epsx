
'use client'

import Link from 'next/link'
import { cn } from '../../utils/cn'
import type { NavItem } from './developer-navigation'

export function SidebarNav({
    items,
    pathname,
    expandedItems,
    onToggleExpand,
    onItemClick
}: {
    items: NavItem[],
    pathname: string,
    expandedItems: Set<string>,
    onToggleExpand: (id: string) => void,
    onItemClick?: () => void
}) {
    return (
        <nav className="flex-1 p-4 space-y-2 overflow-y-auto">
            {items.map(item => (
                <SidebarNavItem
                    key={item.id}
                    item={item}
                    pathname={pathname}
                    isExpanded={expandedItems.has(item.id)}
                    onToggle={() => onToggleExpand(item.id)}
                    onClick={onItemClick}
                />
            ))}
        </nav>
    )
}

function SidebarNavItem({
    item,
    pathname,
    isExpanded,
    onToggle,
    onClick
}: {
    item: NavItem,
    pathname: string,
    isExpanded: boolean,
    onToggle: () => void,
    onClick?: () => void
}) {
    const isActive = checkIsActive(pathname, item);
    const hasActiveChild = checkHasActiveChild(pathname, item);

    return (
        <div>
            {/* Main Item */}
            <div className="relative">
                <Link
                    href={item.href}
                    onClick={() => onClick?.()}
                >
                    <div className={cn(
                        "flex items-center gap-3 px-3 py-3 lg:py-2 rounded-lg min-w-0 overflow-hidden transition-all duration-200",
                        (isActive || hasActiveChild)
                            ? 'bg-blue-100 text-blue-700 dark:bg-blue-500/15 dark:text-blue-300'
                            : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-white/5'
                    )}>
                        <span className="text-xl lg:text-lg flex-shrink-0">{item.icon}</span>
                        <div className="flex flex-col min-w-0">
                            <span className="font-medium text-base lg:text-sm text-ellipsis whitespace-nowrap overflow-hidden">
                                {item.label}
                            </span>
                            {typeof item.description === 'string' && item.description !== '' && (
                                <span className="text-xs text-gray-500 dark:text-gray-400 text-ellipsis whitespace-nowrap overflow-hidden hidden lg:block">
                                    {item.description}
                                </span>
                            )}
                        </div>
                    </div>
                </Link>

                {/* Expand button */}
                {item.children && (
                    <button
                        type="button"
                        onClick={(e) => {
                            e.preventDefault()
                            e.stopPropagation()
                            onToggle()
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
                    {item.children.map(child => (
                        <div key={child.id}>
                            <Link href={child.href} onClick={() => onClick?.()}>
                                <div className={cn(
                                    "flex items-center gap-3 px-3 py-3 lg:py-2 rounded-lg min-w-0 overflow-hidden transition-all duration-200",
                                    pathname === child.href || pathname.startsWith(`${child.href}/`)
                                        ? 'bg-blue-50 text-blue-600 dark:bg-blue-500/10 dark:text-blue-300'
                                        : 'text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-white/5'
                                )}>
                                    <span className="text-base lg:text-sm flex-shrink-0">{child.icon}</span>
                                    <span className="text-base lg:text-sm font-medium text-ellipsis whitespace-nowrap overflow-hidden">
                                        {child.label}
                                    </span>
                                </div>
                            </Link>
                        </div>
                    ))}
                </div>
            )}
        </div>
    )
}

/**
 * Internal helper to check if item is active
 */
function checkIsActive(pathname: string, item: NavItem): boolean {
    return pathname === item.href ||
        (item.href !== '/developer' && pathname.startsWith(`${item.href}/`)) ||
        (item.id === 'dashboard' && pathname === '/developer');
}

/**
 * Internal helper to check if item has an active child
 */
function checkHasActiveChild(pathname: string, item: NavItem): boolean {
    if (item.children === undefined) { return false; }
    return item.children.some(child =>
        pathname === child.href || pathname.startsWith(`${child.href}/`)
    );
}

export function SidebarFooter() {
    return (
        <div className="p-4 border-t border-gray-200 dark:border-gray-700">
            <div className="bg-gradient-to-r from-emerald-50 to-blue-50 dark:from-emerald-900/30 dark:to-blue-900/30 rounded-lg p-3 border border-emerald-200 dark:border-emerald-700">
                <div className="flex items-center gap-2 mb-2">
                    <span className="text-lg">🚀</span>
                    <span className="text-sm font-semibold text-emerald-800 dark:text-emerald-200">API Status</span>
                </div>
                <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                    <span className="text-xs text-emerald-600 dark:text-emerald-300">All systems operational</span>
                </div>
            </div>
        </div>
    )
}

export function MobileHeader({ title, onOpenSidebar }: { title: string, onOpenSidebar: () => void }) {
    return (
        <div
            className="lg:hidden fixed top-14 left-0 right-0 bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-gray-800 dark:to-gray-900 border-b border-blue-200 dark:border-gray-700 shadow-md"
            style={{ zIndex: 60 }}
        >
            <div className="flex items-center justify-between h-12 px-4">
                <button
                    type="button"
                    onClick={onOpenSidebar}
                    className="flex items-center gap-2.5 px-3 py-2 -ml-1 rounded-xl bg-white dark:bg-gray-800 shadow-sm border border-blue-200 dark:border-gray-600 hover:bg-blue-50 dark:hover:bg-gray-700 active:scale-95 transition-all"
                    aria-label="Open developer menu"
                >
                    <span className="text-lg">🛠️</span>
                    <span className="text-sm font-semibold text-blue-700 dark:text-blue-300">Developer Menu</span>
                    <svg className="w-4 h-4 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                    </svg>
                </button>

                <div className="flex items-center gap-2 px-3 py-1.5 bg-white/80 dark:bg-gray-800/80 rounded-full border border-blue-200 dark:border-gray-600">
                    <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse" />
                    <span className="text-xs font-medium text-gray-700 dark:text-gray-300">
                        {title}
                    </span>
                </div>
            </div>
        </div>
    )
}

export function MobileOverlay({ isOpen, onClose }: { isOpen: boolean, onClose: () => void }) {
    if (!isOpen) { return null }
    return (
        <div
            className="lg:hidden fixed bg-black/50 backdrop-blur-sm"
            onClick={onClose}
            aria-hidden="true"
            style={{
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                zIndex: 9998,
                paddingBottom: 'env(safe-area-inset-bottom, 0px)',
            }}
        />
    )
}

export function MobileSidebar({
    isOpen,
    title,
    navigationItems,
    pathname,
    expandedItems,
    onToggleExpand,
    onClose
}: {
    isOpen: boolean,
    title: string,
    navigationItems: NavItem[],
    pathname: string,
    expandedItems: Set<string>,
    onToggleExpand: (id: string) => void,
    onClose: () => void
}) {
    return (
        <div
            className="lg:hidden fixed left-0 w-72 max-w-[85vw] bg-white dark:bg-slate-900 border-r border-gray-200 dark:border-gray-700 flex flex-col transition-transform duration-300 ease-in-out shadow-2xl"
            style={{
                top: 0,
                bottom: 0,
                zIndex: 9999,
                transform: isOpen ? 'translateX(0)' : 'translateX(-100%)',
                paddingBottom: 'env(safe-area-inset-bottom, 0px)',
                minHeight: '100dvh',
            }}
        >
            <div className="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/30 dark:to-indigo-900/30">
                <div className="flex items-center gap-3">
                    <span className="text-2xl">🛠️</span>
                    <h1 className="text-xl font-bold text-gray-900 dark:text-white truncate">{title}</h1>
                </div>
                <button
                    type="button"
                    onClick={onClose}
                    className="p-2 rounded-xl bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-800 border border-gray-200 dark:border-gray-700 transition-colors active:scale-95"
                    aria-label="Close menu"
                >
                    <svg className="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>

            <SidebarNav
                items={navigationItems}
                pathname={pathname}
                expandedItems={expandedItems}
                onToggleExpand={onToggleExpand}
                onItemClick={onClose}
            />

            <div style={{ paddingBottom: 'env(safe-area-inset-bottom, 0px)' }}>
                <SidebarFooter />
            </div>
        </div>
    )
}
