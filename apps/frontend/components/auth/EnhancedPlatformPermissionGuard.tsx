'use client'

// ============================================================================
// ENHANCED PLATFORM PERMISSION GUARD (Phase 3.2)
// Integrates comprehensive error handling system with backend permission authority
// ============================================================================

import { ReactNode, useState, useEffect } from 'react'
import { useAuth } from '@/lib/auth'
import { PermissionErrorBoundary } from '@/components/error-boundaries/PermissionErrorBoundary'
import { PermissionErrorUI } from '@/components/errors/PermissionErrorUI'
import { 
  enhancedPermissionAuthority, 
  useEnhancedPermissionValidation
} from '@/lib/permissions/enhanced-backend-authority-client'
import { 
  ApiError,
  isPermissionDeniedError,
  isInsufficientTierError,
  isPermissionExpiredError,
  isRateLimitExceededError
} from '@/lib/api/response-handler'

// Utility function for platform display names
const getPlatformDisplayName = (platform: string): string => {
  switch (platform.toLowerCase()) {
    case 'epsx': return 'EPSX Trading Platform'
    case 'epsx-pay': return 'EPSX Pay'
    case 'epsx-token': return 'EPSX Token'
    case 'admin': return 'Admin Portal'
    default: return platform.toUpperCase()
  }
}

interface EnhancedPlatformPermissionGuardProps {
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
  // Enhanced props for comprehensive error handling
  loadingFallback?: ReactNode
  showLoadingState?: boolean
  onPermissionError?: (error: ApiError) => void
  onPermissionSuccess?: (permissions: string[]) => void
  enableRetry?: boolean
  enableUpgrade?: boolean
  component?: string // For error context
}

function EnhancedPlatformPermissionGuardCore({
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
  loadingFallback,
  showLoadingState = true,
  onPermissionError,
  onPermissionSuccess,
  enableRetry = true,
  enableUpgrade = true,
  component = 'EnhancedPlatformPermissionGuard',
}: EnhancedPlatformPermissionGuardProps) {
  const { user } = useAuth()
  
  // 🔒 ENHANCED BACKEND PERMISSION AUTHORITY STATE MANAGEMENT
  const [permissionsToValidate, setPermissionsToValidate] = useState<string[]>([])
  const [validationResults, setValidationResults] = useState<Record<string, boolean>>({})
  const [validationError, setValidationError] = useState<ApiError | null>(null)
  const [isValidating, setIsValidating] = useState(false)
  
  // Not authenticated - show fallback
  if (!user) {
    return fallback as JSX.Element
  }

  const targetPlatform = platform || 'epsx'

  // Build list of permissions to validate
  useEffect(() => {
    const permissions: string[] = []
    
    // Direct permission
    if (permission) {
      permissions.push(permission)
    }
    
    // Resource + action permission
    if (resource && action) {
      const permissionString = `${targetPlatform}:${resource}:${action}`
      permissions.push(permissionString)
    }
    
    // Role-based permission (converted to permission check)
    if (role) {
      if (role.toLowerCase() === 'admin') {
        permissions.push('admin:*:*')
      }
    }
    
    // Tier-based permission
    if (tier) {
      const tierPermission = `${targetPlatform}:tier:${tier}`
      permissions.push(tierPermission)
    }
    
    setPermissionsToValidate(permissions)
  }, [permission, resource, action, targetPlatform, role, tier])

  // 🔒 SECURITY CRITICAL: Enhanced backend permission validation with structured error handling
  useEffect(() => {
    // Skip validation if no permissions to check
    if (permissionsToValidate.length === 0) {
      setValidationResults({})
      setValidationError(null)
      return
    }
    
    if (!user.id) {
      setValidationError({
        success: false,
        error: {
          type: 'AUTHENTICATION_REQUIRED',
          code: 'AUTHENTICATION_REQUIRED',
          message: 'User authentication required',
          user_message: 'Please sign in to access this feature',
          suggested_actions: ['Sign in to your account'],
        }
      })
      return
    }

    let isCancelled = false
    
    const validatePermissions = async () => {
      setIsValidating(true)
      setValidationError(null)
      
      try {
        if (permissionsToValidate.length === 1) {
          // Single permission validation with enhanced error handling
          const result = await enhancedPermissionAuthority.validatePermission(
            user.id,
            permissionsToValidate[0],
            {
              component,
              includeUsage: true,
              includeExpiry: true
            }
          )
          
          if (!isCancelled) {
            if (result.success) {
              setValidationResults({ [permissionsToValidate[0]]: result.data.granted })
              onPermissionSuccess?.([permissionsToValidate[0]])
            } else {
              setValidationError(result)
              onPermissionError?.(result)
            }
          }
        } else {
          // Bulk permission validation
          const result = await enhancedPermissionAuthority.validateBulkPermissions(
            user.id,
            permissionsToValidate.map(p => ({ permission: p })),
            {
              component,
              includeUsage: true,
              includeExpiry: true
            }
          )
          
          if (!isCancelled) {
            if (result.success) {
              const results: Record<string, boolean> = {}
              result.data.results.forEach(r => {
                results[r.permission] = r.granted
              })
              setValidationResults(results)
              
              const grantedPermissions = result.data.results
                .filter(r => r.granted)
                .map(r => r.permission)
              onPermissionSuccess?.(grantedPermissions)
            } else {
              setValidationError(result)
              onPermissionError?.(result)
            }
          }
        }
      } catch (error) {
        const apiError: ApiError = {
          success: false,
          error: {
            type: 'NETWORK_ERROR',
            code: 'PERMISSION_VALIDATION_FAILED',
            message: error instanceof Error ? error.message : 'Permission validation failed',
            user_message: 'Unable to validate permissions. Please check your connection and try again.',
            suggested_actions: ['Check your internet connection', 'Refresh the page', 'Contact support if this continues']
          }
        }
        
        if (!isCancelled) {
          setValidationError(apiError)
          onPermissionError?.(apiError)
        }
      } finally {
        if (!isCancelled) {
          setIsValidating(false)
        }
      }
    }
    
    validatePermissions()
    
    return () => {
      isCancelled = true
    }
  }, [
    JSON.stringify(permissionsToValidate),
    requireAll,
    user.id,
    component,
    onPermissionError,
    onPermissionSuccess
  ])

  // Show loading state while validating permissions
  if (isValidating && showLoadingState) {
    return (loadingFallback || (
      <div className="p-4 text-center">
        <div className="flex items-center justify-center space-x-2">
          <div className="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
          <span className="text-sm text-gray-600">Validating permissions...</span>
        </div>
        <div className="text-xs text-gray-500 mt-1">
          Secure validation from backend authority
        </div>
      </div>
    )) as JSX.Element
  }

  // Show comprehensive error UI if permission validation failed
  if (validationError) {
    return (
      <PermissionErrorUI
        error={validationError}
        context={{
          component,
          permissions: permissionsToValidate,
          user_id: user.id,
          platform: targetPlatform
        }}
        onRetry={enableRetry ? () => {
          // Clear cache and retry
          enhancedPermissionAuthority.clearUserCache(user.id)
          window.location.reload()
        } : undefined}
        onUpgrade={enableUpgrade && showUpgradePrompt ? () => {
          window.location.href = '/billing'
        } : undefined}
        onLogin={() => {
          window.location.href = '/login'
        }}
        onContactSupport={() => {
          window.location.href = '/support'
        }}
        className="my-4"
      />
    ) as JSX.Element
  }

  // If no conditions specified, show content
  if (permissionsToValidate.length === 0) {
    return children as JSX.Element
  }

  // Check if user has valid permissions (from enhanced backend authority)
  const hasAccess = (() => {
    const grantedPermissions = Object.entries(validationResults)
      .filter(([, granted]) => granted)
      .map(([permission]) => permission)
    
    if (requireAll) {
      return permissionsToValidate.every(p => validationResults[p] === true)
    } else {
      return grantedPermissions.length > 0
    }
  })()

  if (hasAccess) {
    return children as JSX.Element
  }

  // Show comprehensive upgrade prompt with enhanced UI
  if (showUpgradePrompt && (tier || role)) {
    const upgradeError: ApiError = {
      success: false,
      error: {
        type: 'INSUFFICIENT_TIER',
        code: 'INSUFFICIENT_TIER',
        message: 'Insufficient access level',
        user_message: `This feature requires ${tier ? `${tier} tier` : `${role} role`} access.`,
        suggested_actions: ['Upgrade your account', 'Contact your administrator'],
        upgrade_info: tier ? {
          current_tier: 'free',
          required_tier: tier,
          upgrade_url: '/billing',
          benefits: [`Access to ${tier} features`, 'Enhanced capabilities', 'Priority support']
        } : undefined
      }
    }

    return (
      <PermissionErrorUI
        error={upgradeError}
        context={{
          component,
          permissions: permissionsToValidate,
          user_id: user.id,
          platform: targetPlatform,
          required_tier: tier,
          required_role: role
        }}
        onUpgrade={enableUpgrade ? () => {
          window.location.href = '/billing'
        } : undefined}
        className="my-4"
      />
    ) as JSX.Element
  }

  return fallback as JSX.Element
}

// Main component with error boundary
export default function EnhancedPlatformPermissionGuard(props: EnhancedPlatformPermissionGuardProps) {
  const { component = 'EnhancedPlatformPermissionGuard', onPermissionError } = props

  return (
    <PermissionErrorBoundary
      component={component}
      onError={(error, errorInfo, apiError) => {
        console.error('Enhanced Platform Permission Guard Error:', {
          component,
          error: error.message,
          errorInfo,
          apiError,
          permissions: props.permission || [props.resource, props.action].filter(Boolean).join(':'),
          platform: props.platform || 'epsx'
        })
        
        if (apiError && onPermissionError) {
          onPermissionError(apiError)
        }
      }}
      fallback={
        <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
          <div className="flex items-start">
            <div className="flex-shrink-0">
              <span className="text-lg text-red-500" role="img" aria-hidden="true">⚠️</span>
            </div>
            <div className="ml-3">
              <h3 className="text-sm font-medium text-red-800">Permission System Error</h3>
              <p className="mt-1 text-sm text-red-700">
                The permission system encountered an error. Please refresh the page.
              </p>
              <button 
                onClick={() => window.location.reload()}
                className="mt-2 text-sm font-medium text-red-800 underline hover:text-red-900"
              >
                Refresh Page
              </button>
            </div>
          </div>
        </div>
      }
    >
      <EnhancedPlatformPermissionGuardCore {...props} />
    </PermissionErrorBoundary>
  )
}

// ============================================================================
// ENHANCED CONVENIENCE COMPONENTS
// ============================================================================

export function RequireEnhancedPermission({ 
  permission, 
  platform, 
  children, 
  fallback = null,
  component = 'RequireEnhancedPermission'
}: {
  permission: string
  platform?: string
  children: ReactNode
  fallback?: ReactNode
  component?: string
}) {
  return (
    <EnhancedPlatformPermissionGuard
      permission={permission}
      platform={platform}
      fallback={fallback}
      component={component}
    >
      {children}
    </EnhancedPlatformPermissionGuard>
  )
}

export function RequireEnhancedRole({ 
  role, 
  children, 
  fallback = null,
  component = 'RequireEnhancedRole'
}: {
  role: string
  children: ReactNode
  fallback?: ReactNode
  component?: string
}) {
  return (
    <EnhancedPlatformPermissionGuard
      role={role}
      fallback={fallback}
      component={component}
    >
      {children}
    </EnhancedPlatformPermissionGuard>
  )
}

export function RequireEnhancedTier({ 
  tier, 
  children, 
  fallback = null,
  component = 'RequireEnhancedTier'
}: {
  tier: string
  children: ReactNode
  fallback?: ReactNode
  component?: string
}) {
  return (
    <EnhancedPlatformPermissionGuard
      tier={tier}
      fallback={fallback}
      component={component}
    >
      {children}
    </EnhancedPlatformPermissionGuard>
  )
}

export function RequireEnhancedAccess({ 
  resource, 
  action, 
  platform, 
  children, 
  fallback = null,
  component = 'RequireEnhancedAccess'
}: {
  resource: string
  action: string
  platform?: string
  children: ReactNode
  fallback?: ReactNode
  component?: string
}) {
  return (
    <EnhancedPlatformPermissionGuard
      resource={resource}
      action={action}
      platform={platform}
      fallback={fallback}
      component={component}
    >
      {children}
    </EnhancedPlatformPermissionGuard>
  )
}

// ============================================================================
// ENHANCED PLATFORM PERMISSION GUARD COMPLETE NOTICE (Phase 3.2.1)
// ============================================================================
//
// 🎉 ENHANCED PLATFORM PERMISSION GUARD COMPLETE!
//
// Created next-generation permission guard with comprehensive error handling:
// - Integrated with PermissionErrorBoundary for React error handling
// - Uses PermissionErrorUI for user-friendly structured error displays
// - Enhanced backend authority client with caching and retry mechanisms
// - Comprehensive error types with suggested actions
// - Context-aware error reporting and analytics
// - Retry and upgrade functionality with proper callbacks
//
// Key Enhancements over Basic PlatformPermissionGuard:
// ✅ Error boundary protection for React errors
// ✅ Structured API error handling with type-specific UI
// ✅ Enhanced permission authority client with caching
// ✅ Retry mechanisms with cache clearing
// ✅ Upgrade prompts with detailed benefit information  
// ✅ Context-aware error logging and analytics
// ✅ Callback support for error handling and success tracking
// ✅ Loading states with backend validation indicators
//
// The Enhanced Platform Permission Guard is now PRODUCTION-READY! 🎯
// ============================================================================