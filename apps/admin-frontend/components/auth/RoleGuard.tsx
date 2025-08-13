/**
 * NextAuth.js ModuleGuard - Server Component for conditional rendering based on admin modules
 * Modern replacement for legacy role-based authentication
 */

import { ReactNode } from 'react'
import { auth } from '@/lib/auth'

interface ModuleGuardProps {
  children: ReactNode
  fallback?: ReactNode
  
  // Permission-based access
  permission?: string
  
  // Admin module-based access
  adminModule?: string
  
  // Predefined capability checks
  requireUserManagement?: boolean
  requireAnalyticsAccess?: boolean  
  requireBillingAccess?: boolean
  
  // Multiple requirements (AND logic)
  permissions?: string[]
  adminModules?: string[]
  
  // OR logic - user needs any of these
  anyPermission?: string[]
  anyAdminModule?: string[]
}

/**
 * NextAuth.js Server Component that conditionally renders content based on admin modules
 * Only renders children if user meets the specified requirements
 */
export default async function ModuleGuard({
  children,
  fallback = null,
  permission,
  adminModule,
  requireUserManagement = false,
  requireAnalyticsAccess = false,
  requireBillingAccess = false,
  permissions = [],
  adminModules = [],
  anyPermission = [],
  anyAdminModule = []
}: ModuleGuardProps) {
  
  let hasAccess = true
  
  try {
    const session = await auth()
    
    // If no session, deny access
    if (!session?.user) {
      return <>{fallback}</>
    }
    
    const userPermissions = (session.user as any).permissions as string[] || []
    const userAdminModules = (session.user as any).admin_modules as string[] || []
    
    // Helper functions using NextAuth session
    const hasPermission = (perm: string) => userPermissions.includes(perm)
    const hasAdminModule = (module: string) => userAdminModules.includes(module)
    const canManageUsers = () => hasAdminModule('user_operations')
    const canViewAnalytics = () => hasAdminModule('analytics_specialist')
    const canManageBilling = () => hasAdminModule('billing_admin')
    
    // Single permission check
    if (permission) {
      hasAccess = hasAccess && hasPermission(permission)
    }
    
    // Single admin module check
    if (adminModule) {
      hasAccess = hasAccess && hasAdminModule(adminModule)
    }
    
    // Predefined capability checks
    if (requireUserManagement) {
      hasAccess = hasAccess && canManageUsers()
    }
    
    if (requireAnalyticsAccess) {
      hasAccess = hasAccess && canViewAnalytics()
    }
    
    if (requireBillingAccess) {
      hasAccess = hasAccess && canManageBilling()
    }
    
    // Multiple permissions check (AND logic)
    if (permissions.length > 0) {
      for (const perm of permissions) {
        if (!hasPermission(perm)) {
          hasAccess = false
          break
        }
      }
    }
    
    // Multiple admin modules check (AND logic)
    if (adminModules.length > 0) {
      for (const module of adminModules) {
        if (!hasAdminModule(module)) {
          hasAccess = false
          break
        }
      }
    }
    
    // Any permission check (OR logic)
    if (anyPermission.length > 0) {
      let hasAnyPermission = false
      for (const perm of anyPermission) {
        if (hasPermission(perm)) {
          hasAnyPermission = true
          break
        }
      }
      hasAccess = hasAccess && hasAnyPermission
    }
    
    // Any admin module check (OR logic)
    if (anyAdminModule.length > 0) {
      let hasAnyModule = false
      for (const module of anyAdminModule) {
        if (hasAdminModule(module)) {
          hasAnyModule = true
          break
        }
      }
      hasAccess = hasAccess && hasAnyModule
    }
    
  } catch (error) {
    // If auth check fails, deny access
    console.error('ModuleGuard auth check failed:', error)
    hasAccess = false
  }
  
  return hasAccess ? <>{children}</> : <>{fallback}</>
}

/**
 * Convenience components for common access patterns using admin modules
 */

// Admin-only content (any admin module)
export async function AdminOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <ModuleGuard anyAdminModule={['user_operations', 'system_admin', 'permission_admin']} fallback={fallback}>
      {children}
    </ModuleGuard>
  )
}

// User management access
export async function UserManagementOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <ModuleGuard requireUserManagement fallback={fallback}>
      {children}
    </ModuleGuard>
  )
}

// Analytics access
export async function AnalyticsOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <ModuleGuard requireAnalyticsAccess fallback={fallback}>
      {children}
    </ModuleGuard>
  )
}

// Billing access
export async function BillingOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <ModuleGuard requireBillingAccess fallback={fallback}>
      {children}
    </ModuleGuard>
  )
}

// System admin only (highest level access)
export async function SystemAdminOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <ModuleGuard adminModule="system_admin" fallback={fallback}>
      {children}
    </ModuleGuard>
  )
}