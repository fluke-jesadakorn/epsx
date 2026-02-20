'use client'

import { usePathname } from 'next/navigation'
import { useCallback, useEffect, useState } from 'react'
import { navigationItems } from './developer-navigation'
import {
    MobileHeader,
    MobileOverlay,
    MobileSidebar,
    SidebarFooter,
    SidebarNav
} from './developer-sidebar-components'

interface DeveloperSidebarProps {
    title?: string
    className?: string
}

/**
 * Developer Portal Sidebar - Responsive with mobile support
 * Provides navigation for the Developer Portal section
 */
export function DeveloperSidebar({ title = 'Developer', className = '' }: DeveloperSidebarProps) {
    const pathname = usePathname()
    const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set())
    const [isMobileOpen, setIsMobileOpen] = useState(false)
    const [isMounted, setIsMounted] = useState(false)

    // Get current page title based on pathname
    const getCurrentPageTitle = () => {
        const currentItem = navigationItems.find(item =>
            pathname === item.href ||
            (item.href !== '/developer' && pathname.startsWith(`${item.href}/`))
        )
        return currentItem?.label ?? 'Developer Portal'
    }

    // Ensure component is mounted before showing interactive elements
    useEffect(() => {
        setIsMounted(true)
    }, [])

    // Close sidebar on route change (mobile)
    useEffect(() => {
        setIsMobileOpen(false)
    }, [pathname])

    // Close sidebar on escape key
    useEffect(() => {
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsMobileOpen(false)
            }
        }
        window.addEventListener('keydown', handleEscape)
        return () => window.removeEventListener('keydown', handleEscape)
    }, [])

    // Prevent body scroll when mobile sidebar is open
    useEffect(() => {
        if (isMobileOpen) {
            document.body.style.overflow = 'hidden'
        } else {
            document.body.style.overflow = ''
        }
        return () => {
            document.body.style.overflow = ''
        }
    }, [isMobileOpen])

    const toggleExpanded = useCallback((itemId: string) => {
        setExpandedItems(prev => {
            const newExpanded = new Set(prev)
            if (newExpanded.has(itemId)) {
                newExpanded.delete(itemId)
            } else {
                newExpanded.add(itemId)
            }
            return newExpanded
        })
    }, [])

    const handleOpenSidebar = useCallback(() => {
        setIsMobileOpen(true)
    }, [])

    const handleCloseSidebar = useCallback(() => {
        setIsMobileOpen(false)
    }, [])

    // Don't render interactive mobile elements until mounted (prevents hydration mismatch)
    if (!isMounted) {
        return (
            <div className={`hidden lg:flex w-64 min-w-0 max-w-64 bg-white dark:bg-card border-r border-gray-200 dark:border-gray-700 h-screen flex-col ${className}`}>
                <div className="p-4 border-b border-gray-200 dark:border-gray-700">
                    <h1 className="text-xl font-bold text-gray-900 dark:text-white truncate">{title}</h1>
                </div>
                <SidebarNav
                    items={navigationItems}
                    pathname={pathname}
                    expandedItems={expandedItems}
                    onToggleExpand={toggleExpanded}
                />
                <SidebarFooter />
            </div>
        )
    }

    return (
        <>
            <MobileHeader
                title={getCurrentPageTitle()}
                onOpenSidebar={handleOpenSidebar}
            />

            <MobileOverlay
                isOpen={isMobileOpen}
                onClose={handleCloseSidebar}
            />

            {/* Sidebar - Desktop (always visible) */}
            <div className={`hidden lg:flex w-64 min-w-0 max-w-64 bg-white dark:bg-card border-r border-gray-200 dark:border-gray-700 h-screen flex-col ${className}`}>
                <div className="p-4 border-b border-gray-200 dark:border-gray-700">
                    <h1 className="text-xl font-bold text-gray-900 dark:text-white truncate">{title}</h1>
                </div>
                <SidebarNav
                    items={navigationItems}
                    pathname={pathname}
                    expandedItems={expandedItems}
                    onToggleExpand={toggleExpanded}
                />
                <SidebarFooter />
            </div>

            {/* Sidebar - Mobile */}
            <MobileSidebar
                isOpen={isMobileOpen}
                title={title}
                navigationItems={navigationItems}
                pathname={pathname}
                expandedItems={expandedItems}
                onToggleExpand={toggleExpanded}
                onClose={handleCloseSidebar}
            />
        </>
    )
}

/**
 * Mobile Header Component - Optional for pages that need header content
 */
export function DeveloperMobileHeader({ title }: { title?: string }) {
    return (
        <div className="lg:hidden flex items-center justify-center h-14 border-b border-gray-200 dark:border-gray-700 bg-white/80 dark:bg-card backdrop-blur-xl sticky top-0 z-30">
            <h1 className="text-lg font-semibold text-gray-900 dark:text-white">
                {title ?? 'Developer Portal'}
            </h1>
        </div>
    )
}
