// ============================================================================
// SHARED PERMISSION CHECKING UTILITIES
// ============================================================================
// Core permission validation and checking logic

import { GranularPermissionClaim } from '../types/core'
import { UserClaims, EnhancedUserClaims } from '../types/claims'
import { parsePermission, parsePermissionWithTimestamp } from './parsing'
import { isPermissionValidWithTime, isClaimValid } from './expiry'
import { WILDCARDS } from '../constants'

// ============================================================================
// GRANULAR PERMISSION CHECKING
// ============================================================================

/**
 * Check if user has a specific permission using granular permission system
 */
export const hasPermissionGranular = (
  permissions: Record<string, GranularPermissionClaim>,
  requiredPermission: string
): boolean => {
  if (!permissions || !requiredPermission) {
    return false
  }

  // Check exact match first
  if (permissions[requiredPermission] && isClaimValid(permissions[requiredPermission])) {
    return true
  }

  const required = parsePermission(requiredPermission)
  if (!required) return false

  // Check wildcard permissions
  for (const [perm, claim] of Object.entries(permissions)) {
    if (!isClaimValid(claim)) continue

    const userPerm = parsePermission(perm)
    if (!userPerm) continue

    // Check for exact match
    if (userPerm.platform === required.platform && 
        userPerm.resource === required.resource && 
        userPerm.action === required.action) {
      return true
    }

    // Check for wildcard matches
    if (userPerm.platform === required.platform) {
      // Platform-level wildcard: "epsx:*:*"
      if (userPerm.resource === WILDCARDS.ALL_RESOURCES && userPerm.action === WILDCARDS.ALL_ACTIONS) {
        return true
      }
      
      // Resource-level wildcard: "epsx:analytics:*"
      if (userPerm.resource === required.resource && userPerm.action === WILDCARDS.ALL_ACTIONS) {
        return true
      }
    }

    // Global admin permission: "admin:*:*"
    if (userPerm.platform === 'admin' && 
        userPerm.resource === WILDCARDS.ALL_RESOURCES && 
        userPerm.action === WILDCARDS.ALL_ACTIONS) {
      return true
    }
  }

  return false
}

/**
 * Check if user has any of the specified permissions
 */
export const hasAnyPermissionGranular = (
  permissions: Record<string, GranularPermissionClaim>,
  requiredPermissions: string[]
): boolean => {
  return requiredPermissions.some(permission => 
    hasPermissionGranular(permissions, permission)
  )
}

/**
 * Check if user has all of the specified permissions
 */
export const hasAllPermissionsGranular = (
  permissions: Record<string, GranularPermissionClaim>,
  requiredPermissions: string[]
): boolean => {
  return requiredPermissions.every(permission => 
    hasPermissionGranular(permissions, permission)
  )
}

// ============================================================================
// LEGACY PERMISSION CHECKING (WITH EMBEDDED TIMESTAMPS)
// ============================================================================

/**
 * Check permission access with embedded timestamp validation
 */
export const checkPermissionAccess = (userPermissions: string[], requiredPermission: string): boolean => {
  const required = parsePermission(requiredPermission)
  if (!required) return false
  
  for (const permStr of userPermissions) {
    // Skip expired permissions
    if (!isPermissionValidWithTime(permStr)) continue
    
    const { basePermission } = parsePermissionWithTimestamp(permStr)
    const userPerm = parsePermission(basePermission)
    if (!userPerm) continue
    
    // Check for exact match
    if (userPerm.platform === required.platform && 
        userPerm.resource === required.resource && 
        userPerm.action === required.action) {
      return true
    }
    
    // Check for wildcard matches
    if (userPerm.platform === required.platform) {
      // Platform-level wildcard: "epsx:*:*"
      if (userPerm.resource === WILDCARDS.ALL_RESOURCES && userPerm.action === WILDCARDS.ALL_ACTIONS) {
        return true
      }
      
      // Resource-level wildcard: "epsx:analytics:*"
      if (userPerm.resource === required.resource && userPerm.action === WILDCARDS.ALL_ACTIONS) {
        return true
      }
    }
    
    // Global admin permission: "admin:*:*"
    if (userPerm.platform === 'admin' && 
        userPerm.resource === WILDCARDS.ALL_RESOURCES && 
        userPerm.action === WILDCARDS.ALL_ACTIONS) {
      return true
    }
  }
  
  return false
}

/**
 * Enhanced permission checking with timestamp validation
 */
export const checkPermissionAccessWithTime = (userPermissions: string[], requiredPermission: string): boolean => {
  // Filter out expired permissions first
  const validPermissions = userPermissions.filter(isPermissionValidWithTime)
  
  // Then use the existing permission checking logic
  return checkPermissionAccess(validPermissions, requiredPermission)
}

// ============================================================================
// USER CLAIMS CHECKING
// ============================================================================

/**
 * Check if enhanced user has permission
 */
export const hasPermission = (userClaims: EnhancedUserClaims | null, permission: string): boolean => {
  if (!userClaims) return false
  return hasPermissionGranular(userClaims.permissions, permission)
}

/**
 * Check if enhanced user has any of the permissions
 */
export const hasAnyPermission = (userClaims: EnhancedUserClaims | null, permissions: string[]): boolean => {
  if (!userClaims) return false
  return hasAnyPermissionGranular(userClaims.permissions, permissions)
}

/**
 * Check if enhanced user has all of the permissions
 */
export const hasAllPermissions = (userClaims: EnhancedUserClaims | null, permissions: string[]): boolean => {
  if (!userClaims) return false
  return hasAllPermissionsGranular(userClaims.permissions, permissions)
}

/**
 * Check if legacy user claims has permission (with timestamp support)
 */
export const hasPermissionWithTime = (userClaims: UserClaims | null, permission: string): boolean => {
  if (!userClaims) return false
  return checkPermissionAccessWithTime(userClaims.permissions, permission)
}

/**
 * Check if legacy user claims has any permissions (with timestamp support)
 */
export const hasAnyPermissionWithTime = (userClaims: UserClaims | null, permissions: string[]): boolean => {
  if (!userClaims) return false
  return permissions.some(permission => checkPermissionAccessWithTime(userClaims.permissions, permission))
}

/**
 * Check if legacy user claims has all permissions (with timestamp support)
 */
export const hasAllPermissionsWithTime = (userClaims: UserClaims | null, permissions: string[]): boolean => {
  if (!userClaims) return false
  return permissions.every(permission => checkPermissionAccessWithTime(userClaims.permissions, permission))
}

// ============================================================================
// PLATFORM PERMISSION HELPERS
// ============================================================================

/**
 * Check if user has permission on specific platform
 */
export const hasPlatformPermission = (
  userClaims: EnhancedUserClaims | null, 
  platform: string, 
  resource: string, 
  action: string
): boolean => {
  const permission = `${platform}:${resource}:${action}`
  return hasPermission(userClaims, permission)
}

/**
 * Check if user can access platform
 */
export const canAccessPlatform = (userClaims: EnhancedUserClaims | null, platform: string): boolean => {
  if (!userClaims) return false
  
  // Check if platform is in user's accessible platforms (legacy)
  if (userClaims.platforms?.includes(platform)) {
    return true
  }
  
  // Check if user has any permissions for this platform
  return Object.keys(userClaims.permissions).some(perm => perm.startsWith(`${platform}:`))
}

/**
 * Get all permissions for a platform
 */
export const getPlatformPermissions = (userClaims: EnhancedUserClaims | null, platform: string): string[] => {
  if (!userClaims) return []
  return Object.keys(userClaims.permissions).filter(perm => perm.startsWith(`${platform}:`))
}

// ============================================================================
// ADMIN PERMISSION HELPERS
// ============================================================================

/**
 * Check if user is admin
 */
export const isAdmin = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasPermission(userClaims, 'admin:*:*')
}

/**
 * Check if user can manage users
 */
export const canManageUsers = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasAnyPermission(userClaims, ['admin:users:manage', 'epsx:users:manage'])
}

/**
 * Check if user can view users
 */
export const canViewUsers = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasAnyPermission(userClaims, [
    'admin:users:read', 
    'admin:users:manage', 
    'epsx:users:read', 
    'epsx:users:manage'
  ])
}

/**
 * Check if user can manage permissions
 */
export const canManagePermissions = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasAnyPermission(userClaims, ['admin:permissions:manage', 'admin:*:*'])
}

/**
 * Check if user can view audit logs
 */
export const canViewAuditLogs = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasAnyPermission(userClaims, ['admin:audit:read', 'epsx:audit:read', 'admin:*:*'])
}

/**
 * Check if user can manage system
 */
export const canManageSystem = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasAnyPermission(userClaims, ['admin:system:manage', 'admin:*:*'])
}

// ============================================================================
// FEATURE ACCESS HELPERS
// ============================================================================

/**
 * Check if user can view analytics
 */
export const canViewAnalytics = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasPermission(userClaims, 'epsx:analytics:view')
}

/**
 * Check if user can export data
 */
export const canExportData = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasPermission(userClaims, 'epsx:analytics:export')
}

/**
 * Check if user can access realtime features
 */
export const canAccessRealtime = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasPermission(userClaims, 'epsx:realtime:access')
}

/**
 * Check if user can manage profile
 */
export const canManageProfile = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasPermission(userClaims, 'epsx:profile:manage')
}

/**
 * Check if user can receive notifications
 */
export const canReceiveNotifications = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasPermission(userClaims, 'epsx:notifications:receive')
}

/**
 * Check if user can manage billing
 */
export const canManageBilling = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasPermission(userClaims, 'epsx:billing:manage')
}

/**
 * Check if user can use advanced filters
 */
export const canUseAdvancedFilters = (userClaims: EnhancedUserClaims | null): boolean => {
  return hasPermission(userClaims, 'epsx:analytics:advanced')
}

// ============================================================================
// PERMISSION DURATION VALIDATION
// ============================================================================

/**
 * Check if user has permission that will be valid for at least the specified duration
 */
export const hasPermissionForDuration = (
  userClaims: EnhancedUserClaims | null, 
  permission: string, 
  durationMinutes: number = 0
): boolean => {
  if (!userClaims) return false
  
  const claim = userClaims.permissions[permission]
  if (!claim) return false
  
  // If no timestamp, it's permanent
  if (!claim.expires_at) return true
  
  const now = Date.now()
  const requiredValidUntil = now + (durationMinutes * 60 * 1000)
  const expiryTime = claim.expires_at * 1000
  
  return expiryTime >= requiredValidUntil
}

/**
 * Check if user has permission that's valid for a specific amount of time using wildcard matching
 */
export const hasPermissionForDurationWithWildcards = (
  userClaims: EnhancedUserClaims | null, 
  permission: string, 
  durationMinutes: number = 0
): boolean => {
  if (!userClaims) return false
  
  const now = Date.now()
  const requiredValidUntil = now + (durationMinutes * 60 * 1000)
  
  // Check all user permissions to see if any grant the required permission
  for (const [userPerm, claim] of Object.entries(userClaims.permissions)) {
    // Skip expired claims
    if (!isClaimValid(claim)) continue
    
    // Check if this permission grants access to the required permission
    if (hasPermissionGranular({ [userPerm]: claim }, permission)) {
      // If no timestamp, it's permanent
      if (!claim.expires_at) return true
      
      // Check if it's valid for the required duration
      const expiryTime = claim.expires_at * 1000
      if (expiryTime >= requiredValidUntil) return true
    }
  }
  
  return false
}