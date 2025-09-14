'use client'

import { usePathname } from 'next/navigation'
import Link from 'next/link'

interface BreadcrumbItem {
  label: string
  href: string
  icon?: string
}

// Route configuration for breadcrumb generation
const routeConfig: Record<string, BreadcrumbItem> = {
  '/': { label: 'Dashboard', href: '/', icon: '🏠' },
  '/users': { label: 'Users', href: '/users', icon: '👥' },
  '/users/create': { label: 'Create User', href: '/users/create' },
  '/users/bulk': { label: 'Bulk Operations', href: '/users/bulk' },
  '/permissions': { label: 'Permissions', href: '/permissions', icon: '🔑' },
  '/permissions/grant': { label: 'Grant Permissions', href: '/permissions/grant' },
  '/permissions/request': { label: 'Request Permissions', href: '/permissions/request' },
  '/analytics': { label: 'Analytics', href: '/analytics', icon: '📊' },
  '/analytics/eps': { label: 'EPS Analytics', href: '/analytics/eps' },
  '/notifications': { label: 'Notifications', href: '/notifications', icon: '🔔' },
  '/notifications/create': { label: 'Create Notification', href: '/notifications/create' },
  '/settings': { label: 'Settings', href: '/settings', icon: '⚙️' },
  '/system': { label: 'System', href: '/system', icon: '🖥️' },
  '/bulk-permissions': { label: 'Bulk Permissions', href: '/bulk-permissions', icon: '⚡' },
  '/developer-portal': { label: 'Developer Portal', href: '/developer-portal', icon: '👨‍💻' },
  '/docs': { label: 'Documentation', href: '/docs', icon: '📚' },
  '/docs/api': { label: 'API Docs', href: '/docs/api' },
}

function generateBreadcrumbs(pathname: string): BreadcrumbItem[] {
  const segments = pathname.split('/').filter(Boolean)
  const breadcrumbs: BreadcrumbItem[] = []
  
  // Always start with dashboard
  breadcrumbs.push(routeConfig['/'])
  
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
    return [breadcrumbs[0]]
  }
  
  return breadcrumbs
}

export function DynamicBreadcrumb() {
  const pathname = usePathname()
  const breadcrumbs = generateBreadcrumbs(pathname)
  
  if (breadcrumbs.length <= 1) {
    return (
      <div className="flex items-center gap-2 text-sm">
        <span className="text-gray-600 dark:text-gray-300">
          {breadcrumbs[0]?.icon || '🏠'}
        </span>
        <span className="font-semibold bg-gradient-to-r from-yellow-600 to-orange-600 dark:from-purple-400 dark:to-pink-400 bg-clip-text text-transparent">
          {breadcrumbs[0]?.label || 'Dashboard'}
        </span>
      </div>
    )
  }
  
  return (
    <div className="flex items-center gap-2 text-sm">
      {breadcrumbs.map((item, index) => (
        <div key={item.href} className="flex items-center gap-2">
          {index === 0 && (
            <span className="text-gray-600 dark:text-gray-300">
              {item.icon || '🏠'}
            </span>
          )}
          
          {index < breadcrumbs.length - 1 ? (
            <>
              <Link 
                href={item.href}
                className="text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-gray-100 transition-colors"
              >
                {item.label}
              </Link>
              <span className="text-gray-400 dark:text-gray-500">/</span>
            </>
          ) : (
            <span className="font-semibold bg-gradient-to-r from-yellow-600 to-orange-600 dark:from-purple-400 dark:to-pink-400 bg-clip-text text-transparent">
              {item.label}
            </span>
          )}
        </div>
      ))}
    </div>
  )
}