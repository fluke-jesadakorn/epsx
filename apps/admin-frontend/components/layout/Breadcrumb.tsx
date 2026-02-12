'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'

interface BreadcrumbItem {
  label: string
  href: string
  icon?: string
}

// Route configuration for breadcrumb generation
const routeConfig: Record<string, BreadcrumbItem> = {
  '/': { label: 'Dashboard', href: '/', icon: '🏠' },
  '/users': { label: 'Users', href: '/users', icon: '👥' },
  '/users/create': { label: 'Create user', href: '/users/create' },
  '/users/bulk': { label: 'Bulk Operations', href: '/users/bulk' },
  '/analytics': { label: 'Analytics', href: '/analytics', icon: '📊' },
  '/analytics/eps': { label: 'EPS Analytics', href: '/analytics/eps' },
  '/notifications': { label: 'Notifications', href: '/notifications', icon: '🔔' },
  '/notifications/manage': { label: 'Overview', href: '/notifications/manage' },
  '/notifications/create': { label: 'Create Notification', href: '/notifications/create' },
  '/settings': { label: 'Settings', href: '/settings', icon: '⚙️' },
  '/audit-log': { label: 'Audit Log', href: '/audit-log', icon: '📜' },
  '/bulk-permissions': { label: 'Bulk Permissions', href: '/bulk-permissions', icon: '⚡' },
  '/developer-portal': { label: 'Developer Portal', href: '/developer-portal', icon: '👨‍💻' },
  '/docs': { label: 'Documentation', href: '/docs', icon: '📚' },
  '/docs/api': { label: 'API Docs', href: '/docs/api' },
  '/wallet-management': { label: 'Wallet Management', href: '/wallet-management', icon: '👛' },
  '/wallet-management/wallets': { label: 'Wallets', href: '/wallet-management/wallets' },
  '/wallet-management/access': { label: 'Access Control', href: '/wallet-management/access' },
  '/wallet-management/access/permissions': { label: 'Permissions', href: '/wallet-management/access/permissions' },
  '/wallet-management/access/plans': { label: 'Plans', href: '/wallet-management/access/plans' },
  '/wallet-management/activity': { label: 'Activity Logs', href: '/wallet-management/activity' },
  // Dynamic wallet disable route is handled by generateBreadcrumbs fallback
  '/payments': { label: 'Payments', href: '/payments', icon: '💰' },

  // Access Management Routes (unified single page)
  '/subscriptions': { label: 'Access Management', href: '/subscriptions', icon: '🛡️' },
  '/subscriptions/plans': { label: 'Plans', href: '/subscriptions/plans' },
  '/subscriptions/plans/new': { label: 'New Plan', href: '/subscriptions/plans/new' },
  '/subscriptions/manual-access': { label: 'Manual Access', href: '/subscriptions/manual-access' },
  '/subscriptions/manual-access/assign': { label: 'Assign Wallet', href: '/subscriptions/manual-access/assign' },
  '/subscriptions/manual-access/expiring': { label: 'Expiring', href: '/subscriptions/manual-access/expiring' },
  '/subscriptions/manual-access/create-group': { label: 'Create Group', href: '/subscriptions/manual-access/create-group' },
  '/subscriptions/manual-access/groups': { label: 'Groups', href: '/subscriptions/manual-access/groups' },
}

function generateBreadcrumbs(pathname: string): BreadcrumbItem[] {
  const segments = pathname.split('/').filter(Boolean)
  const breadcrumbs: BreadcrumbItem[] = []

  // Always start with dashboard
  const dashboardItem = routeConfig['/'] ?? { label: 'Dashboard', href: '/', icon: '🏠' }
  breadcrumbs.push(dashboardItem)

  // Build path progressively
  let currentPath = ''
  for (const segment of segments) {
    currentPath += `/${segment}`
    const config = routeConfig[currentPath]

    if (config) {
      breadcrumbs.push(config)
    } else {
      // Generate breadcrumb for dynamic routes
      const label = segment.replace(/-/g, ' ')
        .split(' ')
        .map(word => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ')

      breadcrumbs.push({
        label,
        href: currentPath
      })
    }
  }

  // Remove duplicate dashboard if current page is dashboard
  if (pathname === '/' && breadcrumbs.length > 1) {
    const first = breadcrumbs[0]
    return first ? [first] : []
  }

  return breadcrumbs
}

/**
 *
 */
export function Breadcrumb() {
  const pathname = usePathname()
  const breadcrumbs = generateBreadcrumbs(pathname)

  if (breadcrumbs.length <= 1) {
    return (
      <div className="flex items-center gap-1.5 sm:gap-2 text-xs sm:text-sm min-w-0">
        <span className="text-gray-600 dark:text-gray-300 flex-shrink-0">
          {breadcrumbs[0]?.icon ?? '🏠'}
        </span>
        <span className="font-semibold bg-gradient-to-r from-yellow-600 to-orange-600 dark:from-purple-400 dark:to-pink-400 bg-clip-text text-transparent truncate">
          {breadcrumbs[0]?.label ?? 'Dashboard'}
        </span>
      </div>
    )
  }

  return (
    <div className="flex items-center gap-1 sm:gap-1.5 lg:gap-2 text-xs sm:text-sm min-w-0 overflow-hidden">
      {breadcrumbs.map((item, index) => {
        if (!item) { return null }
        const isLast = index === breadcrumbs.length - 1
        const isFirst = index === 0

        return (
          <div key={item.href} className="flex items-center gap-1 sm:gap-1.5 lg:gap-2 flex-shrink-0">
            {isFirst && (
              <span className="text-gray-600 dark:text-gray-300 flex-shrink-0">
                {item.icon ?? '🏠'}
              </span>
            )}

            {!isLast ? (
              <>
                <Link
                  href={item.href}
                  className="text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-gray-100 truncate max-w-[100px] sm:max-w-[150px] lg:max-w-none"
                  title={item.label}
                >
                  {item.label}
                </Link>
                <span className="text-gray-400 dark:text-gray-500 flex-shrink-0">/</span>
              </>
            ) : (
              <span className="font-semibold bg-gradient-to-r from-yellow-600 to-orange-600 dark:from-purple-400 dark:to-pink-400 bg-clip-text text-transparent truncate max-w-[150px] sm:max-w-[200px] lg:max-w-none" title={item.label}>
                {item.label}
              </span>
            )}
          </div>
        )
      })}
    </div>
  )
}