// ============================================================================
// SHARED BASE PERMISSION GUARD
// ============================================================================
// Base permission guard component that can be extended by specific applications

'use client'

import { ReactNode, useState } from 'react'
import { 
  EnhancedUserClaims, 
  GranularPermissionClaim, 
  PermissionExpiryDetails 
} from '../types/core'
import { GranularPermissionError } from '../types/errors'
import { 
  hasPermissionGranular, 
  hasAnyPermissionGranular, 
  hasAllPermissionsGranular 
} from '../utils/checking'
import { 
  getPermissionExpiryDetails, 
  isClaimExpiringSoon 
} from '../utils/expiry'
import { validatePermissionString } from '../utils/validation'

// ============================================================================
// BASE GUARD INTERFACES
// ============================================================================

export interface BasePermissionGuardProps {
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
  onPermissionDenied?: (reason: string, requiredPermissions: string[]) => void
  onPermissionExpiring?: (permission: string, expiresIn: number) => void
  className?: string
}

export interface PermissionCheckResult {
  hasAccess: boolean
  reason?: string
  expiringPermissions: Array<{ permission: string; expiresIn: number }>
  expiredPermissions: string[]
}

export interface BaseGuardContext {
  userClaims: EnhancedUserClaims | null
  isAuthenticated: boolean
  loading: boolean
  error: any
  refreshPermissions?: () => Promise<void>
}

// ============================================================================
// BASE PERMISSION GUARD COMPONENT
// ============================================================================

export interface BasePermissionGuardOptions {
  defaultPlatform?: string
  enableExpiryWarnings?: boolean
  enableStrictMode?: boolean
  customErrorHandler?: (error: GranularPermissionError) => ReactNode
  customLoadingComponent?: ReactNode
  customAccessDeniedComponent?: (props: AccessDeniedProps) => ReactNode
}

export interface AccessDeniedProps {
  type: 'authentication' | 'permission' | 'expired' | 'expiring' | 'validation'
  message: string
  requiredPermissions: string[]
  expiringPermissions?: Array<{ permission: string; expiresIn: number }>
  expiredPermissions?: string[]
  onRetry?: () => void
  onRefresh?: () => void
  className?: string
}

/**
 * Base permission guard that provides core permission checking logic
 * This can be extended by application-specific guards
 */
export const createBasePermissionGuard = (
  context: BaseGuardContext,
  options: BasePermissionGuardOptions = {}
) => {
  return function BasePermissionGuard({
    children,
    permission,
    permissions,
    resource,
    action,
    platform = options.defaultPlatform || 'epsx',
    requireAll = false,
    requireValidFor = 0,
    fallback = null,
    showExpiryWarning = options.enableExpiryWarnings ?? true,
    showAccessDenied = true,
    onPermissionDenied,
    onPermissionExpiring,
    className
  }: BasePermissionGuardProps) {
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

    // Validate permissions format
    const validatePermissions = (perms: string[]): { valid: string[]; errors: string[] } => {
      const valid: string[] = []
      const errors: string[] = []

      for (const perm of perms) {
        const validation = validatePermissionString(perm)
        if (validation.isValid) {
          valid.push(perm)
        } else {
          errors.push(`${perm}: ${validation.errors.join(', ')}`)
        }
      }

      return { valid, errors }
    }

    // Check permission with duration validation
    const checkPermissionWithDuration = (perm: string): boolean => {
      if (!context.userClaims?.permissions) return false

      const claim = context.userClaims.permissions[perm]
      if (!claim) {
        // Check wildcard matches
        return hasPermissionGranular(context.userClaims.permissions, perm)
      }

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

    // Perform permission check
    const performPermissionCheck = (): PermissionCheckResult => {
      const permissionsToCheck = getPermissionsToCheck()
      
      if (permissionsToCheck.length === 0) {
        return { hasAccess: true, expiringPermissions: [], expiredPermissions: [] }
      }

      // Validate permission format
      const { valid: validPermissions, errors } = validatePermissions(permissionsToCheck)
      if (errors.length > 0) {
        return {
          hasAccess: false,
          reason: `Invalid permission format: ${errors.join('; ')}`,
          expiringPermissions: [],
          expiredPermissions: []
        }
      }

      let hasAccess: boolean
      if (validPermissions.length === 1) {
        hasAccess = checkPermissionWithDuration(validPermissions[0])
      } else {
        hasAccess = requireAll 
          ? validPermissions.every(p => checkPermissionWithDuration(p))
          : validPermissions.some(p => checkPermissionWithDuration(p))
      }

      // Get expiry information
      const expiringPermissions: Array<{ permission: string; expiresIn: number }> = []
      const expiredPermissions: string[] = []

      if (context.userClaims?.permissions) {
        for (const perm of validPermissions) {
          const claim = context.userClaims.permissions[perm]
          if (claim) {
            if (claim.expires_at && claim.expires_at * 1000 <= Date.now()) {
              expiredPermissions.push(perm)
            } else if (isClaimExpiringSoon(claim, 24)) {
              const expiresIn = claim.expires_at ? (claim.expires_at * 1000) - Date.now() : 0
              expiringPermissions.push({ permission: perm, expiresIn })
            }
          }
        }
      }

      return {
        hasAccess,
        expiringPermissions,
        expiredPermissions
      }
    }

    // Handle refresh permissions
    const handleRefresh = async () => {
      if (!context.refreshPermissions) return

      setIsRefreshing(true)
      try {
        await context.refreshPermissions()
      } catch (err) {
        console.error('Failed to refresh permissions:', err)
      } finally {
        setIsRefreshing(false)
      }
    }

    // Check authentication
    if (!context.isAuthenticated || !context.userClaims) {
      if (!showAccessDenied) {
        return fallback as JSX.Element
      }

      const accessDeniedProps: AccessDeniedProps = {
        type: 'authentication',
        message: 'Authentication required to access this feature',
        requiredPermissions: [],
        onRefresh: handleRefresh,
        className
      }

      if (options.customAccessDeniedComponent) {
        return options.customAccessDeniedComponent(accessDeniedProps)
      }

      return <DefaultAccessDenied {...accessDeniedProps} />
    }

    // Loading state
    if (context.loading) {
      if (options.customLoadingComponent) {
        return options.customLoadingComponent as JSX.Element
      }
      return <DefaultLoadingComponent />
    }

    // Error state
    if (context.error) {
      if (options.customErrorHandler && context.error instanceof GranularPermissionError) {
        return options.customErrorHandler(context.error)
      }

      if (!showAccessDenied) {
        return fallback as JSX.Element
      }

      const accessDeniedProps: AccessDeniedProps = {
        type: 'validation',
        message: `Permission check failed: ${context.error.message}`,
        requiredPermissions: [],
        onRetry: handleRefresh,
        className
      }

      if (options.customAccessDeniedComponent) {
        return options.customAccessDeniedComponent(accessDeniedProps)
      }

      return <DefaultAccessDenied {...accessDeniedProps} />
    }

    // Perform permission check
    const checkResult = performPermissionCheck()

    // Trigger callbacks
    if (!checkResult.hasAccess && onPermissionDenied) {
      onPermissionDenied(checkResult.reason || 'Insufficient permissions', getPermissionsToCheck())
    }

    if (checkResult.expiringPermissions.length > 0 && onPermissionExpiring) {
      checkResult.expiringPermissions.forEach(({ permission: perm, expiresIn }) => {
        onPermissionExpiring(perm, expiresIn)
      })
    }

    // Has access - show content with optional warnings
    if (checkResult.hasAccess) {
      return (
        <div className={className}>
          {showExpiryWarning && checkResult.expiringPermissions.length > 0 && (
            <ExpiryWarning
              expiringPermissions={checkResult.expiringPermissions}
              onRefresh={handleRefresh}
              isRefreshing={isRefreshing}
            />
          )}
          {showExpiryWarning && checkResult.expiredPermissions.length > 0 && (
            <ExpiredWarning
              expiredPermissions={checkResult.expiredPermissions}
              onRefresh={handleRefresh}
              isRefreshing={isRefreshing}
            />
          )}
          {children}
        </div>
      )
    }

    // No access - show appropriate message
    if (!showAccessDenied) {
      return fallback as JSX.Element
    }

    // Determine access denial type
    let accessDeniedType: AccessDeniedProps['type'] = 'permission'
    let message = 'You do not have the required permissions to access this feature'

    if (checkResult.expiredPermissions.length > 0) {
      accessDeniedType = 'expired'
      message = 'Your permissions for this feature have expired'
    } else if (requireValidFor > 0) {
      accessDeniedType = 'expiring'
      message = `This feature requires permissions valid for at least ${requireValidFor} hours`
    }

    const accessDeniedProps: AccessDeniedProps = {
      type: accessDeniedType,
      message,
      requiredPermissions: getPermissionsToCheck(),
      expiringPermissions: checkResult.expiringPermissions,
      expiredPermissions: checkResult.expiredPermissions,
      onRefresh: handleRefresh,
      className
    }

    if (options.customAccessDeniedComponent) {
      return options.customAccessDeniedComponent(accessDeniedProps)
    }

    return <DefaultAccessDenied {...accessDeniedProps} />
  }
}

// ============================================================================
// DEFAULT COMPONENTS
// ============================================================================

const DefaultLoadingComponent = () => (
  <div className="flex items-center space-x-2 p-4">
    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-gray-600"></div>
    <span className="text-sm text-gray-600">Checking permissions...</span>
  </div>
)

const DefaultAccessDenied = ({
  type,
  message,
  requiredPermissions,
  onRefresh,
  className = ''
}: AccessDeniedProps) => (
  <div className={`p-4 border rounded-lg bg-red-50 border-red-200 ${className}`}>
    <div className="flex items-start">
      <div className="flex-shrink-0">
        <svg className="w-5 h-5 text-red-500" fill="currentColor" viewBox="0 0 20 20">
          <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
        </svg>
      </div>
      <div className="ml-3 flex-1">
        <h3 className="text-sm font-medium text-red-800">
          {type === 'authentication' ? 'Authentication Required' : 'Access Denied'}
        </h3>
        <p className="mt-1 text-sm text-red-700">{message}</p>
        
        {requiredPermissions.length > 0 && (
          <div className="mt-2">
            <p className="text-xs font-medium text-red-800">Required Permissions:</p>
            <div className="mt-1 flex flex-wrap gap-1">
              {requiredPermissions.map(perm => (
                <span key={perm} className="inline-block px-2 py-1 text-xs bg-red-100 text-red-800 rounded">
                  {perm}
                </span>
              ))}
            </div>
          </div>
        )}
        
        {onRefresh && (
          <div className="mt-3">
            <button 
              onClick={onRefresh}
              className="text-sm bg-red-100 hover:bg-red-200 text-red-800 px-3 py-1 rounded"
            >
              Retry
            </button>
          </div>
        )}
      </div>
    </div>
  </div>
)

const ExpiryWarning = ({ 
  expiringPermissions, 
  onRefresh, 
  isRefreshing 
}: {
  expiringPermissions: Array<{ permission: string; expiresIn: number }>
  onRefresh: () => void
  isRefreshing: boolean
}) => (
  <div className="mb-4 p-3 border rounded-lg bg-yellow-50 border-yellow-200">
    <div className="flex items-center justify-between">
      <div className="flex items-start">
        <svg className="w-4 h-4 text-yellow-500 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
          <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
        </svg>
        <div className="ml-2">
          <p className="text-sm font-medium text-yellow-800">Permissions Expiring Soon</p>
          <div className="mt-1 text-xs text-yellow-700">
            {expiringPermissions.map(({ permission, expiresIn }) => (
              <div key={permission}>
                {permission} - expires in {Math.floor(expiresIn / (1000 * 60 * 60))} hours
              </div>
            ))}
          </div>
        </div>
      </div>
      <button 
        onClick={onRefresh}
        disabled={isRefreshing}
        className="text-xs bg-yellow-100 hover:bg-yellow-200 text-yellow-800 px-2 py-1 rounded disabled:opacity-50"
      >
        {isRefreshing ? 'Refreshing...' : 'Refresh'}
      </button>
    </div>
  </div>
)

const ExpiredWarning = ({ 
  expiredPermissions, 
  onRefresh, 
  isRefreshing 
}: {
  expiredPermissions: string[]
  onRefresh: () => void
  isRefreshing: boolean
}) => (
  <div className="mb-4 p-3 border rounded-lg bg-red-50 border-red-200">
    <div className="flex items-center justify-between">
      <div className="flex items-start">
        <svg className="w-4 h-4 text-red-500 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
          <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
        </svg>
        <div className="ml-2">
          <p className="text-sm font-medium text-red-800">Expired Permissions</p>
          <div className="mt-1 text-xs text-red-700">
            {expiredPermissions.map(permission => (
              <div key={permission}>{permission} - Expired</div>
            ))}
          </div>
        </div>
      </div>
      <button 
        onClick={onRefresh}
        disabled={isRefreshing}
        className="text-xs bg-red-100 hover:bg-red-200 text-red-800 px-2 py-1 rounded disabled:opacity-50"
      >
        {isRefreshing ? 'Refreshing...' : 'Refresh'}
      </button>
    </div>
  </div>
)

// ============================================================================
// CONVENIENCE GUARD CREATORS
// ============================================================================

/**
 * Create a permission guard that requires a specific permission
 */
export const createRequirePermissionGuard = (
  context: BaseGuardContext,
  options: BasePermissionGuardOptions = {}
) => {
  const BaseGuard = createBasePermissionGuard(context, options)
  
  return function RequirePermission({ 
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
      <BaseGuard
        permission={permission}
        platform={platform}
        fallback={fallback}
      >
        {children}
      </BaseGuard>
    )
  }
}

/**
 * Create a permission guard that requires any of multiple permissions
 */
export const createRequireAnyPermissionGuard = (
  context: BaseGuardContext,
  options: BasePermissionGuardOptions = {}
) => {
  const BaseGuard = createBasePermissionGuard(context, options)
  
  return function RequireAnyPermission({ 
    permissions, 
    children, 
    fallback = null 
  }: {
    permissions: string[]
    children: ReactNode
    fallback?: ReactNode
  }) {
    return (
      <BaseGuard
        permissions={permissions}
        requireAll={false}
        fallback={fallback}
      >
        {children}
      </BaseGuard>
    )
  }
}

/**
 * Create a permission guard that requires all specified permissions
 */
export const createRequireAllPermissionsGuard = (
  context: BaseGuardContext,
  options: BasePermissionGuardOptions = {}
) => {
  const BaseGuard = createBasePermissionGuard(context, options)
  
  return function RequireAllPermissions({ 
    permissions, 
    children, 
    fallback = null 
  }: {
    permissions: string[]
    children: ReactNode
    fallback?: ReactNode
  }) {
    return (
      <BaseGuard
        permissions={permissions}
        requireAll={true}
        fallback={fallback}
      >
        {children}
      </BaseGuard>
    )
  }
}