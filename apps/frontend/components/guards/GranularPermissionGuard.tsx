'use client'

import { ReactNode, useState, useEffect } from 'react'
import { useGranularPermissions } from '@/hooks/useGranularPermissions'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { 
  Shield, 
  ShieldAlert, 
  Clock, 
  AlertTriangle, 
  Upgrade,
  Lock,
  RefreshCw
} from 'lucide-react'
import { PermissionError, PermissionExpiryDetails } from '@/types/granular-permissions'

interface GranularPermissionGuardProps {
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
  showUpgradePrompt?: boolean
  showAccessDenied?: boolean
}

export default function GranularPermissionGuard({
  children,
  permission,
  permissions,
  resource,
  action,
  platform = 'epsx',
  requireAll = false,
  requireValidFor = 0,
  fallback = null,
  showExpiryWarning = true,
  showUpgradePrompt = true,
  showAccessDenied = true,
}: GranularPermissionGuardProps) {
  const { 
    hasPermission, 
    hasAnyPermission, 
    hasAllPermissions,
    getPermissionExpiry, 
    isPermissionExpiring,
    refreshPermissions,
    loading, 
    error 
  } = useGranularPermissions()

  const [isRefreshing, setIsRefreshing] = useState(false)

  // Build permission string from components
  const buildPermission = (): string => {
    if (permission) return permission
    if (resource && action) return `${platform}:${resource}:${action}`
    return ''
  }

  // Get all permissions to check
  const getPermissionsToCheck = (): string[] => {
    if (permissions) return permissions
    const builtPermission = buildPermission()
    return builtPermission ? [builtPermission] : []
  }

  const permissionsToCheck = getPermissionsToCheck()
  
  // Check if user has required permissions
  const checkPermissions = (): boolean => {
    if (permissionsToCheck.length === 0) return true
    
    if (permissionsToCheck.length === 1) {
      const perm = permissionsToCheck[0]
      
      // Check if permission needs to be valid for a specific duration
      if (requireValidFor > 0) {
        const expiry = getPermissionExpiry(perm)
        if (!expiry || expiry.is_expired) return false
        if (expiry.expires_in_ms && expiry.expires_in_ms < (requireValidFor * 60 * 60 * 1000)) {
          return false
        }
      }
      
      return hasPermission(perm)
    }
    
    return requireAll 
      ? hasAllPermissions(permissionsToCheck)
      : hasAnyPermission(permissionsToCheck)
  }

  // Get expiry information for warnings
  const getExpiryInfo = (): { 
    hasExpiring: boolean
    expiringPermissions: Array<{ permission: string; expiry: PermissionExpiryDetails }>
    hasExpired: boolean
    expiredPermissions: string[]
  } => {
    const expiringPermissions: Array<{ permission: string; expiry: PermissionExpiryDetails }> = []
    const expiredPermissions: string[] = []

    for (const perm of permissionsToCheck) {
      const expiry = getPermissionExpiry(perm)
      if (expiry) {
        if (expiry.is_expired) {
          expiredPermissions.push(perm)
        } else if (isPermissionExpiring(perm, 24)) {
          expiringPermissions.push({ permission: perm, expiry })
        }
      }
    }

    return {
      hasExpiring: expiringPermissions.length > 0,
      expiringPermissions,
      hasExpired: expiredPermissions.length > 0,
      expiredPermissions
    }
  }

  const hasAccess = checkPermissions()
  const expiryInfo = getExpiryInfo()

  // Handle refresh permissions
  const handleRefresh = async () => {
    setIsRefreshing(true)
    try {
      await refreshPermissions()
    } catch (err) {
      console.error('Failed to refresh permissions:', err)
    } finally {
      setIsRefreshing(false)
    }
  }

  // Loading state
  if (loading && !hasAccess) {
    return (
      <Alert>
        <RefreshCw className="h-4 w-4 animate-spin" />
        <AlertDescription>
          Checking permissions...
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
          Permission check failed: {error.message}
          <Button 
            variant="outline" 
            size="sm" 
            className="ml-2"
            onClick={handleRefresh}
            disabled={isRefreshing}
          >
            <RefreshCw className={`h-3 w-3 ${isRefreshing ? 'animate-spin' : ''}`} />
            Retry
          </Button>
        </AlertDescription>
      </Alert>
    ) : (fallback as JSX.Element)
  }

  // Has access - show content with optional expiry warnings
  if (hasAccess) {
    return (
      <>
        {showExpiryWarning && expiryInfo.hasExpiring && (
          <Alert className="mb-4">
            <Clock className="h-4 w-4" />
            <AlertDescription>
              <div className="flex items-center justify-between">
                <div>
                  <strong>Permissions Expiring Soon:</strong>
                  <div className="mt-1">
                    {expiryInfo.expiringPermissions.map(({ permission: perm, expiry }) => (
                      <Badge key={perm} variant="destructive" className="mr-1 mb-1">
                        {perm} - {expiry.expires_in_human}
                      </Badge>
                    ))}
                  </div>
                </div>
                <Button 
                  variant="outline" 
                  size="sm"
                  onClick={handleRefresh}
                  disabled={isRefreshing}
                >
                  <RefreshCw className={`h-3 w-3 ${isRefreshing ? 'animate-spin' : ''}`} />
                  Refresh
                </Button>
              </div>
            </AlertDescription>
          </Alert>
        )}
        {showExpiryWarning && expiryInfo.hasExpired && (
          <Alert variant="destructive" className="mb-4">
            <ShieldAlert className="h-4 w-4" />
            <AlertDescription>
              <div className="flex items-center justify-between">
                <div>
                  <strong>Expired Permissions:</strong>
                  <div className="mt-1">
                    {expiryInfo.expiredPermissions.map(perm => (
                      <Badge key={perm} variant="destructive" className="mr-1 mb-1">
                        {perm} - Expired
                      </Badge>
                    ))}
                  </div>
                </div>
                <Button 
                  variant="outline" 
                  size="sm"
                  onClick={handleRefresh}
                  disabled={isRefreshing}
                >
                  <RefreshCw className={`h-3 w-3 ${isRefreshing ? 'animate-spin' : ''}`} />
                  Refresh
                </Button>
              </div>
            </AlertDescription>
          </Alert>
        )}
        {children as JSX.Element}
      </>
    )
  }

  // No access - show appropriate message
  if (!showAccessDenied) {
    return fallback as JSX.Element
  }

  // Determine the type of access denial
  const getAccessDenialInfo = () => {
    if (expiryInfo.hasExpired) {
      return {
        type: 'expired',
        title: 'Permissions Expired',
        message: 'Your permissions for this feature have expired.',
        icon: <ShieldAlert className="h-5 w-5 text-red-500" />,
        bgColor: 'bg-red-50 border-red-200',
        textColor: 'text-red-800',
        actionText: 'Refresh Permissions'
      }
    }

    if (requireValidFor > 0) {
      return {
        type: 'insufficient_duration',
        title: 'Insufficient Permission Duration',
        message: `This feature requires permissions valid for at least ${requireValidFor} hours.`,
        icon: <Clock className="h-5 w-5 text-orange-500" />,
        bgColor: 'bg-orange-50 border-orange-200',
        textColor: 'text-orange-800',
        actionText: 'Contact Support'
      }
    }

    return {
      type: 'insufficient_permissions',
      title: 'Access Denied',
      message: 'You do not have the required permissions to access this feature.',
      icon: <Lock className="h-5 w-5 text-yellow-500" />,
      bgColor: 'bg-yellow-50 border-yellow-200',
      textColor: 'text-yellow-800',
      actionText: 'Request Access'
    }
  }

  const denialInfo = getAccessDenialInfo()

  return (
    <Alert className={denialInfo.bgColor}>
      <div className="flex items-start">
        <div className="flex-shrink-0">
          {denialInfo.icon}
        </div>
        <div className="ml-3 flex-1">
          <h3 className={`text-sm font-medium ${denialInfo.textColor}`}>
            {denialInfo.title}
          </h3>
          <p className={`mt-1 text-sm ${denialInfo.textColor}`}>
            {denialInfo.message}
          </p>
          
          {/* Show required permissions */}
          {permissionsToCheck.length > 0 && (
            <div className="mt-2">
              <p className={`text-xs font-medium ${denialInfo.textColor}`}>
                Required Permissions:
              </p>
              <div className="mt-1">
                {permissionsToCheck.map(perm => (
                  <Badge key={perm} variant="outline" className="mr-1 mb-1 text-xs">
                    {perm}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {/* Show validation duration if relevant */}
          {requireValidFor > 0 && (
            <div className="mt-2">
              <p className={`text-xs ${denialInfo.textColor}`}>
                Must be valid for at least {requireValidFor} hours
              </p>
            </div>
          )}

          <div className="mt-3 flex gap-2">
            {denialInfo.type === 'expired' ? (
              <Button 
                variant="outline" 
                size="sm"
                onClick={handleRefresh}
                disabled={isRefreshing}
              >
                <RefreshCw className={`h-3 w-3 ${isRefreshing ? 'animate-spin' : ''}`} />
                {denialInfo.actionText}
              </Button>
            ) : showUpgradePrompt ? (
              <Button 
                variant="outline" 
                size="sm"
                onClick={() => window.open('/upgrade', '_blank')}
              >
                <Upgrade className="h-3 w-3" />
                {denialInfo.actionText}
              </Button>
            ) : null}
          </div>
        </div>
      </div>
    </Alert>
  )
}

// Convenience components for common use cases
export function RequireGranularPermission({ 
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
    <GranularPermissionGuard
      permission={permission}
      platform={platform}
      fallback={fallback}
    >
      {children}
    </GranularPermissionGuard>
  )
}

export function RequireAnyGranularPermission({ 
  permissions, 
  children, 
  fallback = null 
}: {
  permissions: string[]
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <GranularPermissionGuard
      permissions={permissions}
      requireAll={false}
      fallback={fallback}
    >
      {children}
    </GranularPermissionGuard>
  )
}

export function RequireAllGranularPermissions({ 
  permissions, 
  children, 
  fallback = null 
}: {
  permissions: string[]
  children: ReactNode
  fallback?: ReactNode
}) {
  return (
    <GranularPermissionGuard
      permissions={permissions}
      requireAll={true}
      fallback={fallback}
    >
      {children}
    </GranularPermissionGuard>
  )
}

export function RequireGranularAccess({ 
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
    <GranularPermissionGuard
      resource={resource}
      action={action}
      platform={platform}
      fallback={fallback}
    >
      {children}
    </GranularPermissionGuard>
  )
}

export function RequireValidForDuration({ 
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
    <GranularPermissionGuard
      permission={permission}
      requireValidFor={hours}
      fallback={fallback}
    >
      {children}
    </GranularPermissionGuard>
  )
}