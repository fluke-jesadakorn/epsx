'use client'

/**
 * Modern navigation header replacing breadcrumbs
 */
import { ArrowLeft, Home, User, Shield, Settings, BarChart3 } from 'lucide-react'
import { usePathname, useRouter } from 'next/navigation'
import { useState, useEffect } from 'react'

// Page configuration with icons and descriptions
const pageConfig: Record<string, { title: string; icon: React.ComponentType<any>; description?: string }> = {
  '/': { title: 'Dashboard', icon: Home, description: 'System overview' },
  '/users': { title: 'Users', icon: User, description: 'User management' },
  '/permissions': { title: 'Permissions', icon: Shield, description: 'Access control' },
  '/analytics': { title: 'Analytics', icon: BarChart3, description: 'System metrics' },
  '/settings': { title: 'Settings', icon: Settings, description: 'Configuration' },
  '/notifications': { title: 'Notifications', icon: BarChart3, description: 'Alert center' },
  '/system': { title: 'System', icon: Settings, description: 'System control' },
}

/**
 *
 */
export function ServerBreadcrumb() {
  const pathname = usePathname()
  const router = useRouter()
  const [isDark, setIsDark] = useState(false)
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    setMounted(true)
    const savedTheme = localStorage.getItem('theme')
    if (savedTheme === 'dark') {
      document.documentElement.classList.add('dark')
      setIsDark(true)
    } else if (savedTheme === 'light') {
      document.documentElement.classList.remove('dark')
      setIsDark(false)
    } else {
      const theme = document.documentElement.classList.contains('dark')
      setIsDark(theme)
    }
  }, [])

  const toggleTheme = () => {
    const html = document.documentElement
    const newIsDark = !isDark
    
    if (newIsDark) {
      html.classList.add('dark')
      localStorage.setItem('theme', 'dark')
    } else {
      html.classList.remove('dark')
      localStorage.setItem('theme', 'light')
    }
    
    setIsDark(newIsDark)
  }

  // Get current page info
  const currentPage = pageConfig[pathname] || { 
    title: pathname.split('/').pop()?.replace('-', ' ') || 'Page', 
    icon: Home 
  }
  
  // Get parent page for back navigation
  const getParentPath = () => {
    const segments = pathname.split('/').filter(Boolean)
    if (segments.length <= 1) {return null}
    return '/' + segments.slice(0, -1).join('/')
  }

  const parentPath = getParentPath()
  const parentPage = parentPath ? pageConfig[parentPath] : null

  return (
    <div className="flex items-center justify-between w-full">
      {/* Left: Page info with back button */}
      <div className="flex items-center gap-3">
        {/* Back button for sub-pages */}
        {parentPath && (
          <button
            onClick={() => router.push(parentPath)}
            className="flex items-center gap-2 px-3 py-1.5 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-950 rounded-lg transition-all duration-200 group"
          >
            <ArrowLeft className="h-4 w-4 group-hover:-translate-x-0.5 transition-transform" />
            <span className="text-sm font-medium">
              {parentPage?.title || 'Back'}
            </span>
          </button>
        )}
        
        {/* Current page */}
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center w-8 h-8 bg-blue-50 dark:bg-blue-950 rounded-lg">
            <currentPage.icon className="h-4 w-4 text-blue-600 dark:text-blue-400" />
          </div>
          <div className="min-w-0">
            <h1 className="text-lg font-semibold text-gray-900 dark:text-white truncate">
              {currentPage.title}
            </h1>
            {currentPage.description && (
              <p className="text-sm text-gray-500 dark:text-gray-400 truncate">
                {currentPage.description}
              </p>
            )}
          </div>
        </div>
      </div>

      {/* Right: Theme toggle */}
      <button
        onClick={toggleTheme}
        className="flex-shrink-0 p-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
        aria-label="Toggle theme"
        suppressHydrationWarning
      >
        <div className="w-5 h-5 text-blue-600 dark:text-yellow-500">
          {mounted ? (isDark ? '☀️' : '🌙') : '🌙'}
        </div>
      </button>
    </div>
  )
}