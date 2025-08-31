'use client'

import { ReactNode } from 'react'
import { useAuth } from '@/lib/auth'
import { getPlatformDisplayName } from '@/lib/auth'
import { ShieldExclamationIcon, ExclamationTriangleIcon, LockClosedIcon } from '@heroicons/react/24/outline'

interface AdminPermissionGuardProps {
  children: ReactNode
  permission?: string
  resource?: string
  action?: string
  platform?: string
  role?: string
  tier?: string
  requireAll?: boolean
  fallback?: ReactNode
  showAccessDenied?: boolean
  adminAction?: 'manage' | 'read' | 'write' | 'delete'
}

export default function AdminPermissionGuard({
  children,
  permission,
  resource,
  action,
  platform,
  role,
  tier,
  requireAll = false,
  fallback = null,
  showAccessDenied = true,
  adminAction,
}: AdminPermissionGuardProps) {
  const { 
    user, 
    isAuthenticated, 
    can, 
    hasRole, 
    hasTier, 
    getCurrentPlatform,
    canAccessPlatform,
    canManageUsers,
    canManageSystem,
    canViewAnalytics,
    canManagePlatforms,
  } = useAuth.getState()
  
  // Not authenticated
  if (!isAuthenticated || !user) {
    return showAccessDenied ? (
      <AdminAccessDenied 
        type="authentication" 
        message="Authentication required to access admin features"
      />
    ) : (fallback as JSX.Element)
  }
  
  const currentPlatform = getCurrentPlatform()
  const targetPlatform = platform || currentPlatform
  
  // Check platform access
  if (platform && !canAccessPlatform(platform)) {
    return showAccessDenied ? (
      <AdminAccessDenied 
        type="platform" 
        message={`Admin access required for ${getPlatformDisplayName(platform)} platform`}
        platform={platform}
      />
    ) : (fallback as JSX.Element)
  }
  
  // Collect all conditions
  const conditions: boolean[] = []
  
  // Admin action shortcuts
  if (adminAction) {
    switch (adminAction) {
      case 'manage':
        if (resource === 'users') conditions.push(canManageUsers())
        else if (resource === 'system') conditions.push(canManageSystem())
        else if (resource === 'platforms') conditions.push(canManagePlatforms())
        else conditions.push(can(`${resource}:manage`))
        break
      case 'read':
        if (resource === 'analytics') conditions.push(canViewAnalytics())
        else conditions.push(can(`${resource}:read`))
        break
      case 'write':
        conditions.push(can(`${resource}:write`))
        break
      case 'delete':
        conditions.push(can(`${resource}:delete`))
        break
    }
  }
  
  // Permission check
  if (permission) {
    conditions.push(can(permission))
  }
  
  // Resource + action check
  if (resource && action) {
    const permissionString = `${targetPlatform}:${resource}:${action}`
    conditions.push(can(permissionString))
  }
  
  // Role check
  if (role) {
    conditions.push(hasRole(role))
  }
  
  // Tier check
  if (tier) {
    conditions.push(hasTier(tier))
  }
  
  // If no conditions specified, just show content
  if (conditions.length === 0) {
    return children as JSX.Element
  }
  
  // Check conditions based on requireAll flag
  const hasAccess = requireAll 
    ? conditions.every(condition => condition)
    : conditions.some(condition => condition)
  
  if (hasAccess) {
    return children as JSX.Element
  }
  
  // Show access denied message
  if (showAccessDenied) {
    return (
      <AdminAccessDenied 
        type="permission"
        message="Insufficient admin permissions to access this feature"
        requiredRole={role}
        requiredTier={tier}
        requiredPermission={permission}
        platform={targetPlatform}
      />
    )
  }
  
  return fallback as JSX.Element
}

// Access denied component for admin interface
function AdminAccessDenied({ 
  type, 
  message, 
  platform, 
  requiredRole, 
  requiredTier, 
  requiredPermission 
}: {
  type: 'authentication' | 'platform' | 'permission'
  message: string
  platform?: string
  requiredRole?: string
  requiredTier?: string
  requiredPermission?: string
}) {
  const getIcon = () => {
    switch (type) {
      case 'authentication':
        return <LockClosedIcon className="w-8 h-8 text-red-500" />
      case 'platform':
        return <ShieldExclamationIcon className="w-8 h-8 text-orange-500" />
      case 'permission':
        return <ExclamationTriangleIcon className="w-8 h-8 text-yellow-500" />
    }
  }
  
  const getBgColor = () => {
    switch (type) {
      case 'authentication':
        return 'bg-red-50 border-red-200'
      case 'platform':
        return 'bg-orange-50 border-orange-200'
      case 'permission':
        return 'bg-yellow-50 border-yellow-200'
    }
  }
  
  const getTextColor = () => {
    switch (type) {
      case 'authentication':
        return 'text-red-800'
      case 'platform':
        return 'text-orange-800'
      case 'permission':
        return 'text-yellow-800'
    }
  }
  
  return (
    <div className={`p-6 border rounded-lg ${getBgColor()}`}>
      <div className="flex items-start">
        <div className="flex-shrink-0">
          {getIcon()}
        </div>
        <div className="ml-4">
          <h3 className={`text-lg font-medium ${getTextColor()}`}>
            Access Denied
          </h3>
          <p className={`mt-2 text-sm ${getTextColor()}`}>
            {message}
          </p>
          
          {(requiredRole || requiredTier || requiredPermission || platform) && (
            <div className="mt-4 space-y-2">
              <p className={`text-xs font-medium ${getTextColor()}`}>
                Requirements:
              </p>
              <ul className={`text-xs ${getTextColor()}`}>
                {requiredRole && <li>• Role: {requiredRole}</li>}
                {requiredTier && <li>• Tier: {requiredTier}</li>}
                {requiredPermission && <li>• Permission: {requiredPermission}</li>}
                {platform && <li>• Platform Access: {getPlatformDisplayName(platform)}</li>}
              </ul>
            </div>
          )}
          
          <div className="mt-4">
            <button 
              onClick={() => window.location.href = '/admin/access-request'}
              className={`text-sm font-medium underline ${getTextColor()} hover:opacity-75`}
            >
              Request Access
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

// Convenience components for admin use cases
export function RequireAdminPermission({ 
  permission, 
  platform, 
  children, 
  fallback = null 
}: {
  permission: string
  platform?: string
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminPermissionGuard
      permission={permission}
      platform={platform}
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
  )
}

export function RequireAdminRole({ 
  role, 
  children, 
  fallback = null 
}: {
  role: string
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminPermissionGuard
      role={role}
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
  )
}

export function RequireUserManagement({ 
  children, 
  fallback = null 
}: {
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminPermissionGuard
      adminAction="manage"
      resource="users"
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
  )
}

export function RequireSystemManagement({ 
  children, 
  fallback = null 
}: {
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminPermissionGuard
      adminAction="manage"
      resource="system"
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
      adminAction="read"
      resource="analytics"
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
  )
}

export function RequirePlatformManagement({ 
  children, 
  fallback = null 
}: {
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <AdminPermissionGuard
      adminAction="manage"
      resource="platforms"
      fallback={fallback}
    >
      {children}
    </AdminPermissionGuard>
  )
}