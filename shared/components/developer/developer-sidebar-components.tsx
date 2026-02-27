
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
                            ? 'bg-[#7645d9]/10 text-[#7645d9]'
                            : 'text-muted-foreground hover:bg-background hover:text-foreground'
                    )}>
                        <span className="text-xl lg:text-lg flex-shrink-0">{item.icon}</span>
                        <div className="flex flex-col min-w-0">
                            <span className="font-medium text-base lg:text-sm text-ellipsis whitespace-nowrap overflow-hidden">
                                {item.label}
                            </span>
                            {typeof item.description === 'string' && item.description !== '' && (
                                <span className="text-xs text-muted-foreground/60 text-ellipsis whitespace-nowrap overflow-hidden hidden lg:block">
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
                        className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded hover:bg-background"
                    >
                        <span className="text-muted-foreground text-sm">
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
                                        ? 'bg-[#7645d9]/10 text-[#7645d9]'
                                        : 'text-muted-foreground hover:bg-background hover:text-foreground'
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
        <div className="p-4 border-t border-border/10">
            <div className="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-3">
                <div className="flex items-center gap-2 mb-2">
                    <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                    <span className="text-sm font-semibold text-emerald-400">API Status</span>
                </div>
                <span className="text-xs text-emerald-400/80">All systems operational</span>
            </div>
        </div>
    )
}

export function MobileHeader({ title, onOpenSidebar }: { title: string, onOpenSidebar: () => void }) {
    return (
        <div
            className="lg:hidden fixed top-14 left-0 right-0 bg-card border-b border-border/20 shadow-md"
            style={{ zIndex: 60 }}
        >
            <div className="flex items-center justify-between h-12 px-4">
                <button
                    type="button"
                    onClick={onOpenSidebar}
                    className="flex items-center gap-2.5 px-3 py-2 -ml-1 rounded-xl bg-background shadow-sm border border-border/30 hover:bg-accent active:scale-95 transition-all"
                    aria-label="Open developer menu"
                >
                    <span className="text-sm font-semibold text-[#7645d9]">Developer Menu</span>
                    <svg className="w-4 h-4 text-[#7645d9]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                    </svg>
                </button>

                <div className="flex items-center gap-2 px-3 py-1.5 bg-background rounded-full border border-border/30">
                    <div className="w-2 h-2 rounded-full bg-[#7645d9] animate-pulse" />
                    <span className="text-xs font-medium text-muted-foreground">
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
            className="lg:hidden fixed left-0 w-72 max-w-[85vw] bg-card border-r border-border/20 flex flex-col transition-transform duration-300 ease-in-out shadow-2xl"
            style={{
                top: 0,
                bottom: 0,
                zIndex: 9999,
                transform: isOpen ? 'translateX(0)' : 'translateX(-100%)',
                paddingBottom: 'env(safe-area-inset-bottom, 0px)',
                minHeight: '100dvh',
            }}
        >
            <div className="p-4 border-b border-border/10 flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-[#7645d9] to-[#5a33b8]">
                        <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                        </svg>
                    </div>
                    <h1 className="text-xl font-bold text-foreground truncate">{title}</h1>
                </div>
                <button
                    type="button"
                    onClick={onClose}
                    className="p-2 rounded-xl bg-background hover:bg-accent border border-border/30 transition-colors active:scale-95"
                    aria-label="Close menu"
                >
                    <svg className="w-5 h-5 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
