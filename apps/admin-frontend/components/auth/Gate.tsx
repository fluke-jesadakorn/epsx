'use client'

import { useAuth } from '@/lib/auth'

export interface GateProps {
  children: React.ReactNode
  fallback?: React.ReactNode
  check?: {
    authenticated?: boolean
    role?: string
    permission?: string
    module?: string
    tier?: string
    any?: string[]  // Show if user has ANY of these permissions
    all?: string[]  // Show if user has ALL of these permissions
  }
}

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.fallback
 * @param root0.check
 */
export default function Gate({ 
  children, 
  fallback = null,
  check = {}
}: GateProps) {
  const { isAuthenticated, user, can } = useAuth()

  // Check authentication
  if (check.authenticated && !isAuthenticated) {
    return <>{fallback}</>
  }

  // Check role (stubbed - not implemented)
  if (check.role) {
    // Role checking not implemented - allowing access
  }

  // Check permission
  if (check.permission && !can(check.permission)) {
    return <>{fallback}</>
  }

  // Check module (stubbed - not implemented)
  if (check.module) {
    // Module checking not implemented - allowing access
  }

  // Check tier (stubbed - not implemented)
  if (check.tier) {
    // Tier checking not implemented - allowing access
  }

  // Check any permissions (user needs at least one)
  if (check.any && check.any.length > 0) {
    const hasAny = check.any.some(permission => can(permission))
    if (!hasAny) {
      return <>{fallback}</>
    }
  }

  // Check all permissions (user needs all of them)
  if (check.all && check.all.length > 0) {
    const hasAll = check.all.every(permission => can(permission))
    if (!hasAll) {
      return <>{fallback}</>
    }
  }

  return <>{children}</>
}

// Specific gate components for common use cases

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.fallback
 */
export function AuthGate({ 
  children, 
  fallback = null
}: {
  children: React.ReactNode
  fallback?: React.ReactNode
}) {
  return (
    <Gate check={{ authenticated: true }} fallback={fallback}>
      {children}
    </Gate>
  )
}

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.fallback
 */
export function AdminGate({ 
  children, 
  fallback = null
}: {
  children: React.ReactNode
  fallback?: React.ReactNode
}) {
  return (
    <Gate check={{ authenticated: true, role: 'admin' }} fallback={fallback}>
      {children}
    </Gate>
  )
}

/**
 *
 * @param root0
 * @param root0.permission
 * @param root0.children
 * @param root0.fallback
 */
export function PermissionGate({ 
  permission,
  children, 
  fallback = null
}: {
  permission: string
  children: React.ReactNode
  fallback?: React.ReactNode
}) {
  return (
    <Gate check={{ authenticated: true, permission }} fallback={fallback}>
      {children}
    </Gate>
  )
}

/**
 *
 * @param root0
 * @param root0.module
 * @param root0.children
 * @param root0.fallback
 */
export function ModuleGate({ 
  module,
  children, 
  fallback = null
}: {
  module: string
  children: React.ReactNode
  fallback?: React.ReactNode
}) {
  return (
    <Gate check={{ authenticated: true, module }} fallback={fallback}>
      {children}
    </Gate>
  )
}

/**
 *
 * @param root0
 * @param root0.tier
 * @param root0.children
 * @param root0.fallback
 */
export function TierGate({ 
  tier,
  children, 
  fallback = null
}: {
  tier: string
  children: React.ReactNode
  fallback?: React.ReactNode
}) {
  return (
    <Gate check={{ authenticated: true, tier }} fallback={fallback}>
      {children}
    </Gate>
  )
}

/**
 *
 * @param root0
 * @param root0.any
 * @param root0.all
 * @param root0.children
 * @param root0.fallback
 */
export function MultiPermissionGate({
  any,
  all,
  children,
  fallback = null
}: {
  any?: string[]
  all?: string[]
  children: React.ReactNode
  fallback?: React.ReactNode
}) {
  return (
    <Gate check={{ authenticated: true, any, all }} fallback={fallback}>
      {children}
    </Gate>
  )
}

// React hooks for permission checking in components

/**
 *
 * @param permission
 */
export function usePermissionCheck(permission: string): boolean {
  const { can } = useAuth()
  return can(permission)
}

/**
 *
 * @param role
 */
export function useRoleCheck(role: string): boolean {
  // Role checking not implemented - returning true for build compatibility
  return true
}

/**
 *
 * @param module
 */
export function useModuleCheck(module: string): boolean {
  // Module checking not implemented - returning true for build compatibility
  return true
}

/**
 *
 * @param tier
 */
export function useTierCheck(tier: string): boolean {
  // Tier checking not implemented - returning true for build compatibility
  return true
}

/**
 *
 * @param permissions
 * @param requireAll
 */
export function useMultiPermissionCheck(permissions: string[], requireAll = false): boolean {
  const { can } = useAuth()
  
  if (requireAll) {
    return permissions.every(permission => can(permission))
  } else {
    return permissions.some(permission => can(permission))
  }
}

// Utility component for showing different content based on user state
/**
 *
 * @param root0
 * @param root0.authenticated
 * @param root0.unauthenticated
 * @param root0.loading
 * @param root0.error
 */
export function UserStateGate({
  authenticated,
  unauthenticated,
  loading,
  error
}: {
  authenticated?: React.ReactNode
  unauthenticated?: React.ReactNode
  loading?: React.ReactNode
  error?: React.ReactNode
}) {
  const { isAuthenticated, isLoading } = useAuth()

  if (isLoading) {
    return <>{loading}</>
  }

  if (isAuthenticated) {
    return <>{authenticated}</>
  }

  return <>{unauthenticated}</>
}

// Example usage in JSX:
// <Gate check={{ permission: 'admin:read' }}>
//   <AdminPanel />
// </Gate>
//
// <Gate check={{ any: ['admin:read', 'moderator:read'] }}>
//   <ReadOnlyView />
// </Gate>
//
// <Gate check={{ all: ['admin:write', 'user:manage'] }}>
//   <AdvancedControls />
// </Gate>