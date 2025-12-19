'use client'

import { usePathname, useRouter } from 'next/navigation'
import { ReactNode, useEffect, useState } from 'react'

import { MainLayout } from './MainLayout'

import { useSharedAuth } from '@/shared/components/auth/Provider'

interface AuthLayoutProps {
  children: ReactNode
  user?: {
    id: string
    email: string
    name?: string
    role: string
  }
}

// Pages that should NEVER have the admin layout
const NO_LAYOUT_PATHS = [
  '/unauthorized',
  '/access-denied',
  '/request-access',
  '/permissions/policies'
]

// Pages that don't require authentication
const PUBLIC_PATHS = [
  '/login',
  '/auth',
  '/unauthorized',
  '/access-denied',
  '/request-access'
]

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.user
 */
export function AuthLayout({ children, user: serverUser }: AuthLayoutProps) {
  const pathname = usePathname()
  const router = useRouter()
  const [mounted, setMounted] = useState(false)
  const [authChecked, setAuthChecked] = useState(false)
  const [redirecting, setRedirecting] = useState(false) // Prevent redirect loops
  const { user: authUser, isAuthenticated, hasPermissionForDisplay, isLoading } = useSharedAuth()

  useEffect(() => {
    setMounted(true)
  }, [])

  // Handle authentication redirects
  useEffect(() => {
    console.log('🔍 AuthLayout: Auth check triggered', {
      mounted,
      isLoading,
      isAuthenticated,
      redirecting,
      authChecked,
      pathname,
      hasUser: !!authUser
    });

    if (!mounted || isLoading || redirecting) {
      console.log('⏸️ AuthLayout: Skipping auth check (waiting for provider)');
      return;
    }

    const isPublicPath = PUBLIC_PATHS.some(path => pathname === path || pathname.startsWith(path))

    // Skip auth check for public paths
    if (isPublicPath) {
      console.log('✅ AuthLayout: Public path, no auth required', { pathname });
      setAuthChecked(true)
      setRedirecting(false) // Reset redirecting flag for public paths
      return
    }

    // Auth check for protected routes
    const checkAuth = async () => {
      // Redirect to auth if not authenticated
      if (!isAuthenticated && mounted && !isLoading && !authChecked) {
        console.warn('❌ AuthLayout: Not authenticated - redirecting to auth page', {
          pathname,
          isAuthenticated,
          hasUser: !!authUser,
          authChecked
        });
        setRedirecting(true);
        const authUrl = new URL('/auth', window.location.origin)
        authUrl.searchParams.set('return_url', pathname)
        authUrl.searchParams.set('reason', 'no-session')
        router.push(authUrl.toString())
        return
      }

      // Backend validates all permissions - frontend just checks if authenticated
      // No need to check permissions here - let backend return 403 if no access

      // If we reach here, auth is valid
      console.log('✅ AuthLayout: Auth valid, allowing access', { pathname });
      setAuthChecked(true)
      setRedirecting(false)
    }

    // No delay needed - SharedOpenIDWeb3Provider now ensures permissions are loaded
    // before setting isAuthenticated=true
    checkAuth()
  }, [mounted, isLoading, isAuthenticated, hasPermissionForDisplay, pathname, router, redirecting, authChecked, authUser])

  // Wait for client-side hydration and auth check
  if (!mounted || (!authChecked && !PUBLIC_PATHS.some(path => pathname === path || pathname.startsWith(path)))) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="flex flex-col items-center gap-4">
          <div className="relative">
            <div className="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-500 rounded-full blur opacity-30 animate-pulse" />
            <svg className="h-8 w-8 animate-spin text-blue-600 dark:text-blue-400 relative" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          </div>
          <p className="text-gray-600 dark:text-gray-400 text-sm">
            {!mounted ? "Initializing..." : "Checking authentication..."}
          </p>
        </div>
      </div>
    )
  }

  // Special pages that never get layout
  const isNoLayoutPage = NO_LAYOUT_PATHS.some(path => pathname === path || pathname.startsWith(path))
  if (isNoLayoutPage) {
    return <>{children}</>
  }

  // Use client-side auth user if available, otherwise use server user
  const layoutUser = authUser ? {
    id: authUser.wallet_address || authUser.sub,
    email: authUser.email || `${authUser.wallet_address}@web3.epsx.io`,
    name: authUser.wallet_address ? `Admin (${authUser.wallet_address.slice(0, 6)}...${authUser.wallet_address.slice(-4)})` : 'Admin',
    role: 'admin'
  } : serverUser

  // Always show layout for all pages except the excluded ones
  // This ensures consistent navigation and branding regardless of auth status
  return (
    <MainLayout user={layoutUser}>
      {children}
    </MainLayout>
  )
}