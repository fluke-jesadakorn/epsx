/**
 * PermissionGuard - Server Component for conditional rendering based on structured permissions
 * Clean implementation using "platform:resource:action" permission format
 */

import { ReactNode } from 'react'
import { getServerSession } from '@/lib/auth'

interface PermissionGuardProps {
  children: ReactNode
  fallback?: ReactNode
  
  // Single permission check
  permission?: string
  
  // Multiple permissions (AND logic - user needs all)
  permissions?: string[]
  
  // Multiple permissions (OR logic - user needs any)
  anyPermission?: string[]
  
  // Predefined capability checks (for common use cases)
  requireAdmin?: boolean
  requireUserManagement?: boolean
  requireAnalyticsAccess?: boolean  
  requireBillingAccess?: boolean
  requireSystemManagement?: boolean
  requireAuditAccess?: boolean
}

/**
 * Server Component that conditionally renders content based on structured permissions
 * Only renders children if user meets the specified requirements
 */
export default async function PermissionGuard({
  children,
  fallback = null,
  permission,
  permissions = [],
  anyPermission = [],
  requireAdmin = false,
  requireUserManagement = false,
  requireAnalyticsAccess = false,
  requireBillingAccess = false,
  requireSystemManagement = false,
  requireAuditAccess = false,
}: PermissionGuardProps) {
  
  let hasAccess = true
  
  try {
    const session = await getServerSession()
    
    // If no session, deny access
    if (!session?.user) {
      return <>{fallback}</>
    }
    
    const userPermissions = (session.user as any).permissions as string[] || []
    
    // Helper function to check structured permissions with wildcard support
    const checkPermissionAccess = (userPerms: string[], requiredPerm: string): boolean => {
      const required = parsePermission(requiredPerm);
      if (!required) return false;
      
      for (const permStr of userPerms) {
        const userPerm = parsePermission(permStr);
        if (!userPerm) continue;
        
        // Check for exact match
        if (userPerm.platform === required.platform && 
            userPerm.resource === required.resource && 
            userPerm.action === required.action) {
          return true;
        }
        
        // Check for wildcard matches
        if (userPerm.platform === required.platform) {
          // Platform-level wildcard: "epsx:*:*"
          if (userPerm.resource === '*' && userPerm.action === '*') {
            return true;
          }
          
          // Resource-level wildcard: "epsx:analytics:*"
          if (userPerm.resource === required.resource && userPerm.action === '*') {
            return true;
          }
        }
        
        // Global admin permission: "admin:*:*"
        if (userPerm.platform === 'admin' && userPerm.resource === '*' && userPerm.action === '*') {
          return true;
        }
      }
      
      return false;
    };

    const parsePermission = (permissionString: string): { platform: string; resource: string; action: string } | null => {
      const parts = permissionString.split(':');
      if (parts.length !== 3) return null;
      
      return {
        platform: parts[0],
        resource: parts[1],
        action: parts[2]
      };
    };
    
    // Permission checking helper
    const hasPermission = (perm: string) => checkPermissionAccess(userPermissions, perm)
    
    // Predefined capability helpers
    const isAdmin = () => hasPermission('admin:*:*')
    const canManageUsers = () => hasPermission('admin:users:manage') || hasPermission('epsx:users:manage')
    const canViewAnalytics = () => hasPermission('epsx:analytics:view') || hasPermission('epsx:analytics:*') || hasPermission('admin:*:*')
    const canManageBilling = () => hasPermission('epsx:billing:manage') || hasPermission('admin:*:*')
    const canManageSystem = () => hasPermission('admin:system:manage') || hasPermission('admin:*:*')
    const canViewAudit = () => hasPermission('admin:audit:read') || hasPermission('epsx:audit:read') || hasPermission('admin:*:*')
    
    // Single permission check
    if (permission) {
      hasAccess = hasAccess && hasPermission(permission)
    }
    
    // Predefined capability checks
    if (requireAdmin) {
      hasAccess = hasAccess && isAdmin()
    }
    
    if (requireUserManagement) {
      hasAccess = hasAccess && canManageUsers()
    }
    
    if (requireAnalyticsAccess) {
      hasAccess = hasAccess && canViewAnalytics()
    }
    
    if (requireBillingAccess) {
      hasAccess = hasAccess && canManageBilling()
    }

    if (requireSystemManagement) {
      hasAccess = hasAccess && canManageSystem()
    }

    if (requireAuditAccess) {
      hasAccess = hasAccess && canViewAudit()
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
    
  } catch (error) {
    // If auth check fails, deny access
    console.error('PermissionGuard auth check failed:', error)
    hasAccess = false
  }
  
  return hasAccess ? <>{children}</> : <>{fallback}</>
}

/**
 * Convenience components for common access patterns using structured permissions
 */

// Admin-only content (global admin permission)
export async function AdminOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <PermissionGuard 
      requireAdmin
      fallback={fallback}
    >
      {children}
    </PermissionGuard>
  )
}

// User management access
export async function UserManagementOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <PermissionGuard 
      requireUserManagement 
      fallback={fallback}
    >
      {children}
    </PermissionGuard>
  )
}

// Analytics access
export async function AnalyticsOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <PermissionGuard 
      requireAnalyticsAccess 
      fallback={fallback}
    >
      {children}
    </PermissionGuard>
  )
}

// Billing access
export async function BillingOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <PermissionGuard 
      requireBillingAccess 
      fallback={fallback}
    >
      {children}
    </PermissionGuard>
  )
}

// System management only
export async function SystemManagementOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <PermissionGuard 
      requireSystemManagement 
      fallback={fallback}
    >
      {children}
    </PermissionGuard>
  )
}

// Audit access only
export async function AuditOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <PermissionGuard 
      requireAuditAccess 
      fallback={fallback}
    >
      {children}
    </PermissionGuard>
  )
}

// Multi-platform admin (for cross-platform administration)
export async function CrossPlatformAdminOnly({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) {
  return (
    <PermissionGuard 
      anyPermission={['admin:*:*', 'epsx:*:*', 'epsx-pay:*:*', 'epsx-token:*:*']}
      fallback={fallback}
    >
      {children}
    </PermissionGuard>
  )
}

// Legacy exports for backward compatibility
export { PermissionGuard as ModuleGuard }
export { PermissionGuard as default }