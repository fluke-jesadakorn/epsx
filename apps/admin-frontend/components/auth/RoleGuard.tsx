/**
 * RoleGuard - Server Component for conditional rendering based on auth
 * Replaces client-side permission checks with server-side rendering
 */

import { ReactNode } from 'react'
import { hasPermission, hasRole, canManageUsers, canViewAnalytics, canManageBilling } from '@/lib/auth/server-auth-enhanced'

interface RoleGuardProps {
  children: ReactNode
  fallback?: ReactNode
  
  // Permission-based access
  permission?: string
  
  // Role-based access
  role?: string
  
  // Predefined capability checks
  requireUserManagement?: boolean
  requireAnalyticsAccess?: boolean  
  requireBillingAccess?: boolean
  
  // Multiple requirements (AND logic)
  permissions?: string[]
  roles?: string[]
  
  // OR logic - user needs any of these
  anyPermission?: string[]
  anyRole?: string[]
}

/**
 * Server Component that conditionally renders content based on user permissions
 * Only renders children if user meets the specified requirements
 */
export default async function RoleGuard({
  children,
  fallback = null,
  permission,
  role,
  requireUserManagement = false,
  requireAnalyticsAccess = false,
  requireBillingAccess = false,
  permissions = [],
  roles = [],
  anyPermission = [],
  anyRole = []
}: RoleGuardProps) {
  
  let hasAccess = true
  
  try {
    // Single permission check
    if (permission) {
      hasAccess = hasAccess && await hasPermission(permission)
    }
    
    // Single role check
    if (role) {
      hasAccess = hasAccess && await hasRole(role)
    }
    
    // Predefined capability checks
    if (requireUserManagement) {
      hasAccess = hasAccess && await canManageUsers()
    }
    
    if (requireAnalyticsAccess) {
      hasAccess = hasAccess && await canViewAnalytics()
    }
    
    if (requireBillingAccess) {
      hasAccess = hasAccess && await canManageBilling()
    }
    
    // Multiple permissions check (AND logic)
    if (permissions.length > 0) {
      for (const perm of permissions) {
        if (!await hasPermission(perm)) {
          hasAccess = false
          break
        }
      }
    }
    
    // Multiple roles check (AND logic)
    if (roles.length > 0) {
      for (const r of roles) {
        if (!await hasRole(r)) {
          hasAccess = false
          break
        }
      }
    }
    
    // Any permission check (OR logic)
    if (anyPermission.length > 0) {
      let hasAnyPermission = false
      for (const perm of anyPermission) {
        if (await hasPermission(perm)) {
          hasAnyPermission = true
          break
        }
      }
      hasAccess = hasAccess && hasAnyPermission
    }
    
    // Any role check (OR logic)  
    if (anyRole.length > 0) {
      let hasAnyRole = false
      for (const r of anyRole) {
        if (await hasRole(r)) {
          hasAnyRole = true
          break
        }
      }
      hasAccess = hasAccess && hasAnyRole
    }
    
  } catch (error) {
    // If auth check fails, deny access
    console.error('RoleGuard auth check failed:', error)
    hasAccess = false
  }
  
  return hasAccess ? <>{children}</> : <>{fallback}</>
}

/**
 * Convenience components for common access patterns
 */

// Admin-only content
export async function AdminOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <RoleGuard anyRole={['admin', 'system_administrator', 'super_admin']} fallback={fallback}>
      {children}
    </RoleGuard>
  )
}

// User management access
export async function UserManagementOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <RoleGuard requireUserManagement fallback={fallback}>
      {children}
    </RoleGuard>
  )
}

// Analytics access
export async function AnalyticsOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <RoleGuard requireAnalyticsAccess fallback={fallback}>
      {children}
    </RoleGuard>
  )
}

// Billing access
export async function BillingOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <RoleGuard requireBillingAccess fallback={fallback}>
      {children}
    </RoleGuard>
  )
}

// Super admin only
export async function SuperAdminOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <RoleGuard role="super_admin" fallback={fallback}>
      {children}
    </RoleGuard>
  )
}