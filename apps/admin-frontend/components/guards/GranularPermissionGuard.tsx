'use client'

import { ReactNode, useState } from 'react'
import { useAdminGranularPermissions } from '@/hooks/useGranularPermissions'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { 
  Shield, 
  ShieldAlert, 
  ShieldX,
  Clock, 
  AlertTriangle, 
  Lock,
  RefreshCw,
  UserX,
  Settings
} from 'lucide-react'
import { PermissionError } from '@/types/granular-permissions'
import { useAuth } from '@/lib/auth/session'

interface AdminGranularPermissionGuardProps {
  children: ReactNode
  permission?: string
  permissions?: string[] // For multiple permissions
  resource?: string
  action?: string
  platform?: string
  requireAll?: boolean // For multiple permissions
  requireValidFor?: number // Hours the permission must be valid for
  fallback?: ReactNode
  showExpiryWarning?: boolean
  showAccessDenied?: boolean
  adminAction?: 'manage' | 'read' | 'write' | 'delete'
  strictMode?: boolean // Extra validation for sensitive operations
}

export default function AdminGranularPermissionGuard({
  children,
  permission,
  permissions,
  resource,
  action,
  platform = 'admin',
  requireAll = false,
  requireValidFor = 0,
  fallback = null,
  showExpiryWarning = true,
  showAccessDenied = true,
  adminAction,
  strictMode = false,
}: AdminGranularPermissionGuardProps) {
  const { user } = useAuth.getState()
  const { 
    loading, 
    error 
  } = useAdminGranularPermissions()

  const [isRefreshing, setIsRefreshing] = useState(false)

  // Cast user to enhanced user claims for granular permissions
  const enhancedUser = user as any // In real implementation, this would be properly typed

  // Build permission string from components
  const buildPermission = (): string => {
    if (permission) return permission
    if (resource && action) return `${platform}:${resource}:${action}`
    if (adminAction && resource) return `${platform}:${resource}:${adminAction}`
    return ''
  }

  // Get all permissions to check
  const getPermissionsToCheck = (): string[] => {
    if (permissions) return permissions
    const builtPermission = buildPermission()
    return builtPermission ? [builtPermission] : []
  }

  // Check if user has permission with granular validation
  const hasGranularPermission = (perm: string): boolean => {
    if (!enhancedUser?.permissions) return false

    // Check exact match first
    const claim = enhancedUser.permissions[perm]
    if (claim) {
      // Check if not expired
      if (claim.expires_at && claim.expires_at * 1000 <= Date.now()) {
        return false
      }
      
      // Check validity duration if required
      if (requireValidFor > 0 && claim.expires_at) {
        const validUntil = claim.expires_at * 1000
        const requiredValidUntil = Date.now() + (requireValidFor * 60 * 60 * 1000)
        if (validUntil < requiredValidUntil) {
          return false
        }
      }
      
      return true
    }

    // Check wildcard matches
    const parts = perm.split(':')
    if (parts.length !== 3) return false

    const [reqPlatform, reqResource, reqAction] = parts

    for (const [userPerm, claim] of Object.entries(enhancedUser.permissions)) {
      // Check if claim is expired
      if (claim.expires_at && claim.expires_at * 1000 <= Date.now()) {
        continue
      }

      const userParts = userPerm.split(':')
      if (userParts.length !== 3) continue

      const [userPlatform, userResource, userAction] = userParts

      // Check for matches
      if (userPlatform === reqPlatform || userPlatform === 'admin') {
        if ((userResource === '*' && userAction === '*') ||
            (userResource === reqResource && userAction === '*') ||
            (userResource === reqResource && userAction === reqAction)) {
          
          // Additional validity duration check for wildcard matches
          if (requireValidFor > 0 && claim.expires_at) {
            const validUntil = claim.expires_at * 1000
            const requiredValidUntil = Date.now() + (requireValidFor * 60 * 60 * 1000)
            if (validUntil < requiredValidUntil) {
              continue
            }
          }
          
          return true
        }
      }
    }

    return false
  }

  const permissionsToCheck = getPermissionsToCheck()
  
  // Check if user has required permissions
  const checkPermissions = (): boolean => {
    if (permissionsToCheck.length === 0) return true
    
    if (permissionsToCheck.length === 1) {
      return hasGranularPermission(permissionsToCheck[0])
    }
    
    return requireAll 
      ? permissionsToCheck.every(p => hasGranularPermission(p))
      : permissionsToCheck.some(p => hasGranularPermission(p))
  }

  // Check authentication
  if (!user || !enhancedUser) {
    return showAccessDenied ? (
      <AdminAccessDenied 
        type="authentication" 
        message="Admin authentication required to access this feature"
      />
    ) : (fallback as JSX.Element)
  }

  const hasAccess = checkPermissions()

  // Additional strict mode validation for sensitive operations
  if (strictMode && hasAccess) {
    const hasAdminWildcard = hasGranularPermission('admin:*:*')
    if (!hasAdminWildcard) {
      return showAccessDenied ? (
        <AdminAccessDenied 
          type="strict_mode" 
          message="Full admin privileges required for this sensitive operation"
          requiredPermissions={['admin:*:*']}
        />
      ) : (fallback as JSX.Element)
    }
  }

  // Loading state
  if (loading) {
    return (
      <Alert>
        <RefreshCw className="h-4 w-4 animate-spin" />
        <AlertDescription>
          Validating admin permissions...
        </AlertDescription>
      </Alert>
    )
  }

  // Error state
  if (error) {
    return showAccessDenied ? (
      <Alert variant="destructive">
        <AlertTriangle className="h-4 w-4" />
        <AlertDescription>
          Admin permission check failed: {error.message}
        </AlertDescription>
      </Alert>
    ) : (fallback as JSX.Element)
  }

  // Has access - show content
  if (hasAccess) {
    return children as JSX.Element
  }

  // No access - show appropriate message
  if (!showAccessDenied) {
    return fallback as JSX.Element
  }

  return (
    <AdminAccessDenied 
      type="permission"
      message="Insufficient admin permissions to access this feature"
      requiredPermissions={permissionsToCheck}
      strictMode={strictMode}
    />
  )
}

// Access denied component for admin interface
function AdminAccessDenied({ 
  type, 
  message, 
  requiredPermissions = [],
  strictMode = false 
}: {
  type: 'authentication' | 'permission' | 'strict_mode'
  message: string
  requiredPermissions?: string[]
  strictMode?: boolean
}) {
  const getIcon = () => {
    switch (type) {
      case 'authentication':
        return <UserX className="w-6 h-6 text-red-500" />
      case 'strict_mode':
        return <ShieldX className="w-6 h-6 text-red-600" />
      case 'permission':
        return <ShieldAlert className="w-6 h-6 text-yellow-500" />
    }
  }
  
  const getBgColor = () => {
    switch (type) {
      case 'authentication':
        return 'bg-red-50 border-red-200'
      case 'strict_mode':
        return 'bg-red-50 border-red-300'
      case 'permission':
        return 'bg-yellow-50 border-yellow-200'
    }
  }
  
  const getTextColor = () => {
    switch (type) {
      case 'authentication':
        return 'text-red-800'
      case 'strict_mode':
        return 'text-red-900'
      case 'permission':
        return 'text-yellow-800'
    }
  }

  const getTitle = () => {
    switch (type) {
      case 'authentication':
        return 'Authentication Required'
      case 'strict_mode':
        return 'Elevated Privileges Required'
      case 'permission':
        return 'Access Denied'
    }
  }
  
  return (
    <Alert className={`p-6 ${getBgColor()}`}>
      <div className="flex items-start">
        <div className="flex-shrink-0">
          {getIcon()}
        </div>
        <div className="ml-4 flex-1">
          <h3 className={`text-lg font-medium ${getTextColor()}`}>
            {getTitle()}
          </h3>
          <p className={`mt-2 text-sm ${getTextColor()}`}>
            {message}
          </p>
          
          {strictMode && (
            <Alert className="mt-3 bg-red-100 border-red-300">
              <Shield className="w-4 h-4 text-red-600" />
              <AlertDescription className="text-red-800 text-sm">
                <strong>Strict Mode:</strong> This operation requires the highest level of admin privileges for security reasons.
              </AlertDescription>
            </Alert>
          )}
          
          {requiredPermissions.length > 0 && (
            <div className="mt-4">
              <p className={`text-sm font-medium ${getTextColor()}`}>
                Required Permissions:
              </p>
              <div className="mt-2 flex flex-wrap gap-1">
                {requiredPermissions.map(perm => (
                  <Badge key={perm} variant="outline" className="text-xs">
                    {perm}
                  </Badge>
                ))}
              </div>
            </div>
          )}
          
          <div className="mt-4 flex gap-2">
            <Button 
              variant="outline" 
              size="sm"
              onClick={() => window.location.href = '/admin/access-request'}
            >
              <Settings className="w-3 h-3" />
              Request Access
            </Button>
            {type === 'authentication' && (
              <Button 
                variant="outline" 
                size="sm"
                onClick={() => window.location.href = '/admin/login'}
              >
                <Lock className="w-3 h-3" />
                Admin Login
              </Button>
            )}
          </div>
        </div>
      </div>
    </Alert>
  )
}

// Convenience components for admin use cases
export function RequireAdminGranularPermission({ 
  permission, 
  platform = 'admin', 
  children, 
  fallback = null 
}: {
  permission: string
  platform?: string
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminGranularPermissionGuard
      permission={permission}
      platform={platform}
      fallback={fallback}
    >
      {children}
    </AdminGranularPermissionGuard>
  )
}

export function RequireUserManagementGranular({ 
  children, 
  fallback = null,
  action = 'manage'
}: {
  children: ReactNode
  fallback?: ReactNode
  action?: string
}) {
  return (
    <AdminGranularPermissionGuard
      resource="users"
      action={action}
      platform="admin"
      fallback={fallback}
    >
      {children}
    </AdminGranularPermissionGuard>
  )
}

export function RequireSystemManagementGranular({ 
  children, 
  fallback = null 
}: {
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminGranularPermissionGuard
      resource="system"
      action="manage"
      platform="admin"
      fallback={fallback}
    >
      {children}
    </AdminGranularPermissionGuard>
  )
}

export function RequirePermissionManagementGranular({ 
  children, 
  fallback = null,
  strictMode = true
}: {
  children: ReactNode
  fallback?: ReactNode
  strictMode?: boolean
}) {
  return (
    <AdminGranularPermissionGuard
      resource="permissions"
      action="manage"
      platform="admin"
      strictMode={strictMode}
      fallback={fallback}
    >
      {children}
    </AdminGranularPermissionGuard>
  )
}

export function RequireAnalyticsAccessGranular({ 
  children, 
  fallback = null 
}: {
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminGranularPermissionGuard
      resource="analytics"
      action="read"
      platform="admin"
      fallback={fallback}
    >
      {children}
    </AdminGranularPermissionGuard>
  )
}

export function RequireFullAdminAccess({ 
  children, 
  fallback = null 
}: {
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminGranularPermissionGuard
      permission="admin:*:*"
      strictMode={true}
      fallback={fallback}
    >
      {children}
    </AdminGranularPermissionGuard>
  )
}

export function RequireValidAdminPermissionFor({ 
  permission,
  hours, 
  children, 
  fallback = null 
}: {
  permission: string
  hours: number
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminGranularPermissionGuard
      permission={permission}
      requireValidFor={hours}
      fallback={fallback}
    >
      {children}
    </AdminGranularPermissionGuard>
  )
}