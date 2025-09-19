// ============================================================================
// ADMIN FRONTEND PERMISSION GUARDS
// ============================================================================
// Admin-specific permission guards using shared base guard system

'use client'

import { ReactNode } from 'react'
import { useAuth } from '@/lib/auth/session'
import { useAdminGranularPermissions } from './hooks'
import { 
  createBasePermissionGuard,
  createRequirePermissionGuard,
  createRequireAnyPermissionGuard,
  createRequireAllPermissionsGuard,
  BaseGuardContext,
  BasePermissionGuardOptions,
  AccessDeniedProps
} from '@/shared/permissions/guards'
import { EnhancedUserClaims } from '@/shared/permissions/types'
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

// ============================================================================
// ADMIN GUARD CONTEXT
// ============================================================================

function useAdminGuardContext(): BaseGuardContext {
  const { user } = useAuth.getState()
  const { loading, error, refreshPermissions } = useAdminGranularPermissions()

  return {
    userClaims: user as EnhancedUserClaims | null,
    isAuthenticated: !!user,
    loading,
    error,
    refreshPermissions
  }
}

// ============================================================================
// ADMIN GUARD OPTIONS
// ============================================================================

const adminGuardOptions: BasePermissionGuardOptions = {
  defaultPlatform: 'admin',
  enableExpiryWarnings: true,
  enableStrictMode: true,
  customAccessDeniedComponent: AdminAccessDenied,
  customLoadingComponent: AdminLoadingComponent
}

// ============================================================================
// ADMIN-SPECIFIC COMPONENTS
// ============================================================================

function AdminLoadingComponent() {
  return (
    <Alert>
      <RefreshCw className="h-4 w-4 animate-spin" />
      <AlertDescription>
        Validating admin permissions...
      </AlertDescription>
    </Alert>
  )
}

function AdminAccessDenied({ 
  type, 
  message, 
  requiredPermissions = [],
  onRefresh,
  className = ''
}: AccessDeniedProps) {
  const getIcon = () => {
    switch (type) {
      case 'authentication':
        return <UserX className="w-6 h-6 text-red-500" />
      case 'expired':
        return <Clock className="w-6 h-6 text-red-600" />
      case 'permission':
        return <ShieldAlert className="w-6 h-6 text-yellow-500" />
      default:
        return <ShieldX className="w-6 h-6 text-red-600" />
    }
  }
  
  const getBgColor = () => {
    switch (type) {
      case 'authentication':
        return 'bg-red-50 border-red-200'
      case 'expired':
        return 'bg-red-50 border-red-300'
      case 'permission':
        return 'bg-yellow-50 border-yellow-200'
      default:
        return 'bg-red-50 border-red-300'
    }
  }
  
  const getTextColor = () => {
    switch (type) {
      case 'authentication':
        return 'text-red-800'
      case 'expired':
        return 'text-red-900'
      case 'permission':
        return 'text-yellow-800'
      default:
        return 'text-red-900'
    }
  }

  const getTitle = () => {
    switch (type) {
      case 'authentication':
        return 'Admin Authentication Required'
      case 'expired':
        return 'Admin Permissions Expired'
      case 'permission':
        return 'Insufficient Admin Permissions'
      default:
        return 'Admin Access Denied'
    }
  }
  
  return (
    <Alert className={`p-6 ${getBgColor()} ${className}`}>
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
          
          {type === 'expired' && (
            <Alert className="mt-3 bg-red-100 border-red-300">
              <Shield className="w-4 h-4 text-red-600" />
              <AlertDescription className="text-red-800 text-sm">
                <strong>Security Notice:</strong> Your admin permissions have expired for security reasons.
              </AlertDescription>
            </Alert>
          )}
          
          {requiredPermissions.length > 0 && (
            <div className="mt-4">
              <p className={`text-sm font-medium ${getTextColor()}`}>
                Required Admin Permissions:
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
            {onRefresh && (
              <Button 
                variant="outline" 
                size="sm"
                onClick={onRefresh}
              >
                <RefreshCw className="w-3 h-3" />
                Refresh Permissions
              </Button>
            )}
            
            <Button 
              variant="outline" 
              size="sm"
              onClick={() => window.location.href = '/admin/request-access'}
            >
              <Settings className="w-3 h-3" />
              Request Admin Access
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

// ============================================================================
// ADMIN GUARD IMPLEMENTATIONS
// ============================================================================

// Create admin guard instances using shared base
export const AdminPermissionGuard = createBasePermissionGuard(
  useAdminGuardContext(),
  adminGuardOptions
)

export const RequireAdminPermission = createRequirePermissionGuard(
  useAdminGuardContext(),
  adminGuardOptions
)

export const RequireAnyAdminPermission = createRequireAnyPermissionGuard(
  useAdminGuardContext(),
  adminGuardOptions
)

export const RequireAllAdminPermissions = createRequireAllPermissionsGuard(
  useAdminGuardContext(),
  adminGuardOptions
)

// ============================================================================
// CONVENIENCE ADMIN GUARD COMPONENTS
// ============================================================================

export function RequireUserManagementAccess({ 
  children, 
  fallback = null,
  action = 'manage'
}: {
  children: ReactNode
  fallback?: ReactNode
  action?: string
}) {
  return (
    <AdminPermissionGuard
      resource="users"
      action={action}
      platform="admin"
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
  )
}

export function RequireSystemManagementAccess({ 
  children, 
  fallback = null 
}: {
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminPermissionGuard
      resource="system"
      action="manage"
      platform="admin"
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
  )
}

export function RequirePermissionManagementAccess({ 
  children, 
  fallback = null
}: {
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminPermissionGuard
      resource="permissions"
      action="manage"
      platform="admin"
      requireValidFor={1} // Must be valid for at least 1 hour
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
  )
}

export function RequireAnalyticsAccess({ 
  children, 
  fallback = null 
}: {
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminPermissionGuard
      resource="analytics"
      action="read"
      platform="admin"
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
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
    <AdminPermissionGuard
      permission="admin:*:*"
      requireValidFor={2} // Must be valid for at least 2 hours for full admin access
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
  )
}

export function RequireAuditLogAccess({ 
  children, 
  fallback = null 
}: {
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <RequireAnyAdminPermission
      permissions={['admin:audit:read', 'admin:*:*']}
      fallback={fallback}
    >
      {children}
    </RequireAnyAdminPermission>
  )
}

// ============================================================================
// STRICT MODE GUARDS (FOR SENSITIVE OPERATIONS)
// ============================================================================

export function RequireStrictAdminAccess({ 
  children, 
  fallback = null,
  operation = 'sensitive operation'
}: {
  children: ReactNode
  fallback?: ReactNode
  operation?: string
}) {
  return (
    <AdminPermissionGuard
      permission="admin:*:*"
      requireValidFor={4} // Must be valid for at least 4 hours
      showAccessDenied={true}
      fallback={fallback}
      onPermissionDenied={(reason, perms) => {
        console.warn(`Strict admin access denied for ${operation}:`, { reason, perms })
      }}
    >
      {children}
    </AdminPermissionGuard>
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
    <AdminPermissionGuard
      permission={permission}
      requireValidFor={hours}
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
  )
}

// Export default guard for backward compatibility
export default AdminPermissionGuard