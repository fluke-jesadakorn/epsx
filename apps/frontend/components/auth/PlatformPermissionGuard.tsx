'use client'

import { ReactNode } from 'react'
import { useAuth } from '@/lib/auth'
import { getPlatformDisplayName } from '@/lib/auth'

interface PlatformPermissionGuardProps {
  children: ReactNode
  permission?: string
  resource?: string
  action?: string
  platform?: string
  role?: string
  tier?: string
  requireAll?: boolean
  fallback?: ReactNode
  showUpgradePrompt?: boolean
}

export default function PlatformPermissionGuard({
  children,
  permission,
  resource,
  action,
  platform,
  role,
  tier,
  requireAll = false,
  fallback = null,
  showUpgradePrompt = true,
}: PlatformPermissionGuardProps) {
  const { 
    user, 
    isAuthenticated, 
    can, 
    hasTier, 
    getCurrentPlatform,
    canAccessPlatform 
  } = useAuth.getState()
  
  // Not authenticated
  if (!isAuthenticated || !user) {
    return fallback as JSX.Element
  }
  
  const currentPlatform = getCurrentPlatform()
  const targetPlatform = platform || currentPlatform
  
  // Check platform access
  if (platform && !canAccessPlatform(platform)) {
    return showUpgradePrompt ? (
      <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
        <div className="flex items-start">
          <div className="flex-shrink-0">
            <span className="text-lg" role="img" aria-hidden="true">
              🔒
            </span>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-yellow-800">
              Access Required
            </h3>
            <p className="mt-1 text-sm text-yellow-700">
              You need access to {getPlatformDisplayName(platform)} to view this content.
            </p>
            <div className="mt-2">
              <button className="text-sm font-medium text-yellow-800 underline hover:text-yellow-900">
                Request Access
              </button>
            </div>
          </div>
        </div>
      </div>
    ) : (fallback as JSX.Element)
  }
  
  // Collect all conditions
  const conditions: boolean[] = []
  
  // Permission check
  if (permission) {
    conditions.push(can(permission))
  }
  
  // Resource + action check
  if (resource && action) {
    const permissionString = `${targetPlatform}:${resource}:${action}`
    conditions.push(can(permissionString))
  }
  
  // Role check (converted to admin permission check)
  if (role) {
    // Convert legacy role check to permission-based check
    if (role.toLowerCase() === 'admin') {
      conditions.push(can('admin:*:*'))
    }
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
  
  // Show upgrade prompt if specified
  if (showUpgradePrompt && (tier || role)) {
    return (
      <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <div className="flex items-start">
          <div className="flex-shrink-0">
            <span className="text-lg" role="img" aria-hidden="true">
              ⭐
            </span>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-blue-800">
              Upgrade Required
            </h3>
            <p className="mt-1 text-sm text-blue-700">
              {tier && `This feature requires ${tier} tier or higher. `}
              {role && `This feature requires ${role} role or higher.`}
            </p>
            <div className="mt-2">
              <button className="text-sm font-medium text-blue-800 underline hover:text-blue-900">
                Upgrade Account
              </button>
            </div>
          </div>
        </div>
      </div>
    )
  }
  
  return fallback as JSX.Element
}

// Convenience components for common use cases
export function RequirePermission({ 
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
    <PlatformPermissionGuard
      permission={permission}
      platform={platform}
      fallback={fallback}
    >
      {children}
    </PlatformPermissionGuard>
  )
}

export function RequireRole({ 
  role, 
  children, 
  fallback = null 
}: {
  role: string
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <PlatformPermissionGuard
      role={role}
      fallback={fallback}
    >
      {children}
    </PlatformPermissionGuard>
  )
}

export function RequireTier({ 
  tier, 
  children, 
  fallback = null 
}: {
  tier: string
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <PlatformPermissionGuard
      tier={tier}
      fallback={fallback}
    >
      {children}
    </PlatformPermissionGuard>
  )
}

export function RequireAccess({ 
  resource, 
  action, 
  platform, 
  children, 
  fallback = null 
}: {
  resource: string
  action: string
  platform?: string
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <PlatformPermissionGuard
      resource={resource}
      action={action}
      platform={platform}
      fallback={fallback}
    >
      {children}
    </PlatformPermissionGuard>
  )
}