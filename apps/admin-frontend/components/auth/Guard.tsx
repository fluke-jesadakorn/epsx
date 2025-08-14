'use client'

import { useEffect } from 'react'
import { redirect } from 'next/navigation'
import { useAuth } from '@/lib/auth'
import { Loader2 } from 'lucide-react'

export interface GuardProps {
  children: React.ReactNode
  fallback?: React.ReactNode
  redirectTo?: string
  require?: {
    authenticated?: boolean
    role?: string
    permission?: string
    module?: string
    tier?: string
  }
}

export default function Guard({ 
  children, 
  fallback,
  redirectTo = '/login',
  require = { authenticated: true }
}: GuardProps) {
  const { user, isLoading, isAuthenticated, can, hasRole, hasModule, hasTier } = useAuth()

  // Show loading while checking auth
  if (isLoading) {
    return fallback || (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4" />
          <p className="text-muted-foreground">Checking authentication...</p>
        </div>
      </div>
    )
  }

  // Check authentication requirement
  if (require.authenticated && !isAuthenticated) {
    redirect(redirectTo)
  }

  // Check role requirement
  if (require.role && !hasRole(require.role)) {
    redirect('/access-denied')
  }

  // Check permission requirement
  if (require.permission && !can(require.permission)) {
    redirect('/access-denied')
  }

  // Check module requirement
  if (require.module && !hasModule(require.module)) {
    redirect('/access-denied')
  }

  // Check tier requirement
  if (require.tier && !hasTier(require.tier)) {
    redirect('/access-denied')
  }

  return <>{children}</>
}

// Specific guard components for common use cases

export function AuthGuard({ 
  children, 
  fallback,
  redirectTo = '/login'
}: {
  children: React.ReactNode
  fallback?: React.ReactNode
  redirectTo?: string
}) {
  return (
    <Guard 
      require={{ authenticated: true }}
      fallback={fallback}
      redirectTo={redirectTo}
    >
      {children}
    </Guard>
  )
}

export function AdminGuard({ 
  children, 
  fallback,
  redirectTo = '/access-denied'
}: {
  children: React.ReactNode
  fallback?: React.ReactNode
  redirectTo?: string
}) {
  return (
    <Guard 
      require={{ authenticated: true, role: 'admin' }}
      fallback={fallback}
      redirectTo={redirectTo}
    >
      {children}
    </Guard>
  )
}

export function ModuleGuard({ 
  module,
  children, 
  fallback,
  redirectTo = '/access-denied'
}: {
  module: string
  children: React.ReactNode
  fallback?: React.ReactNode
  redirectTo?: string
}) {
  return (
    <Guard 
      require={{ authenticated: true, module }}
      fallback={fallback}
      redirectTo={redirectTo}
    >
      {children}
    </Guard>
  )
}

export function PermissionGuard({ 
  permission,
  children, 
  fallback,
  redirectTo = '/access-denied'
}: {
  permission: string
  children: React.ReactNode
  fallback?: React.ReactNode
  redirectTo?: string
}) {
  return (
    <Guard 
      require={{ authenticated: true, permission }}
      fallback={fallback}
      redirectTo={redirectTo}
    >
      {children}
    </Guard>
  )
}

export function TierGuard({ 
  tier,
  children, 
  fallback,
  redirectTo = '/access-denied'
}: {
  tier: string
  children: React.ReactNode
  fallback?: React.ReactNode
  redirectTo?: string
}) {
  return (
    <Guard 
      require={{ authenticated: true, tier }}
      fallback={fallback}
      redirectTo={redirectTo}
    >
      {children}
    </Guard>
  )
}

// Higher-order component for protecting pages
export function withAuth<P extends object>(
  Component: React.ComponentType<P>,
  requirements?: GuardProps['require']
) {
  return function AuthenticatedComponent(props: P) {
    return (
      <Guard require={requirements}>
        <Component {...props} />
      </Guard>
    )
  }
}

// Example usage:
// const AdminPage = withAuth(MyAdminPage, { role: 'admin' })
// const PremiumPage = withAuth(MyPremiumPage, { tier: 'PREMIUM' })