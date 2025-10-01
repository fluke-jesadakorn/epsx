'use client'

import { usePathname, useRouter } from 'next/navigation'
import { PancakeAdminLayout } from './PancakeAdminLayout'
import { ReactNode, useEffect, useState } from 'react'
import { useSharedAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider'

interface ConditionalAdminLayoutProps {
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
  '/request-access'
]

// Pages that don't require authentication
const PUBLIC_PATHS = [
  '/login',
  '/auth',
  '/unauthorized',
  '/access-denied',
  '/request-access'
]

export function ConditionalAdminLayout({ children, user: serverUser }: ConditionalAdminLayoutProps) {
  const pathname = usePathname()
  const router = useRouter()
  const [mounted, setMounted] = useState(false)
  const [authChecked, setAuthChecked] = useState(false)
  const [redirecting, setRedirecting] = useState(false) // Prevent redirect loops
  const [authTimeout, setAuthTimeout] = useState(false) // Handle auth timeout
  const { user: authUser, isAuthenticated, hasPermissionForDisplay, isLoading } = useSharedAuth()
  
  useEffect(() => {
    setMounted(true)
    
    // Set a timeout to prevent infinite loading state
    const timeoutId = setTimeout(() => {
      if (isLoading && !isAuthenticated && !authChecked) {
        console.warn('🔄 ConditionalAdminLayout: Auth timeout - forcing redirect to auth page');
        setAuthTimeout(true);
        setAuthChecked(true);
      }
    }, 5000); // 5 second timeout
    
    return () => clearTimeout(timeoutId);
  }, [])
  
  // Handle authentication redirects
  useEffect(() => {
    if (!mounted || (isLoading && !authTimeout) || redirecting) return
    
    const isPublicPath = PUBLIC_PATHS.some(path => pathname === path || pathname.startsWith(path))
    
    // Skip auth check for public paths
    if (isPublicPath) {
      setAuthChecked(true)
      setRedirecting(false) // Reset redirecting flag for public paths
      return
    }
    
    // Auth check for protected routes
    const checkAuth = async () => {
      // Handle timeout case or clear non-authenticated state
      if ((!isAuthenticated && mounted && (!isLoading || authTimeout) && authChecked === false)) {
        console.log('🔄 ConditionalAdminLayout: Redirecting to auth - no session' + (authTimeout ? ' (timeout)' : ''));
        setRedirecting(true);
        const authUrl = new URL('/auth', window.location.origin)
        authUrl.searchParams.set('return_url', pathname)
        authUrl.searchParams.set('reason', 'no-session')
        router.push(authUrl.toString())
        return
      }
      
      // If authenticated but no admin permissions, redirect to auth with different reason
      if (isAuthenticated && !hasPermissionForDisplay('admin:*:*')) {
        console.log('🔄 ConditionalAdminLayout: Redirecting to auth - no admin permissions');
        setRedirecting(true);
        const authUrl = new URL('/auth', window.location.origin)
        authUrl.searchParams.set('return_url', pathname)
        authUrl.searchParams.set('reason', 'no-admin-permissions')
        router.push(authUrl.toString())
        return
      }
      
      // If we reach here, auth is valid
      setAuthChecked(true)
      setRedirecting(false)
    }
    
    // Add a small delay to prevent race conditions with authentication state
    const timeoutId = setTimeout(checkAuth, 100)
    return () => clearTimeout(timeoutId)
  }, [mounted, isLoading, isAuthenticated, hasPermissionForDisplay, pathname, router, redirecting, authChecked, authTimeout])
  
  // Wait for client-side hydration and auth check
  if (!mounted || (!authChecked && !PUBLIC_PATHS.some(path => pathname === path || pathname.startsWith(path)))) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-gray-600 dark:text-gray-400">
          {!mounted ? "Loading..." : "Checking authentication..."}
        </div>
      </div>
    )
  }
  
  // Special pages that never get layout
  const isNoLayoutPage = NO_LAYOUT_PATHS.some(path => pathname === path)
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
    <PancakeAdminLayout user={layoutUser}>
      {children}
    </PancakeAdminLayout>
  )
}