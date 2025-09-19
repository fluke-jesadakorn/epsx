/**
 * Admin Permission Utilities - Updated to use shared permission system
 * Client-side utilities for validating admin permissions using structured permission system
 * Separated from server actions to avoid Next.js build errors
 */

import { 
  hasPermissionWithWildcard,
  isAdminPermission,
  hasGlobalAdminAccess
} from '@/shared/permissions/utils'

/**
 * Check if user has admin permissions using structured permission system
 */
export function validateAdminPermissions(permissions: string[]): boolean {
  if (!permissions || permissions.length === 0) {
    return false
  }
  
  // Check for admin permissions using shared utilities
  const hasAdminAccess = permissions.some(permission => 
    isAdminPermission(permission) || hasGlobalAdminAccess({ permissions: {} }, permission)
  )
  
  console.log('🔍 Admin permission check:', {
    permissions,
    hasAdminAccess,
    requiredFormat: 'admin:*:* or admin:{resource}:{action}'
  })
  
  return hasAdminAccess
}

/**
 * Check if user has specific admin permission
 */
export function hasAdminPermission(permissions: string[], resource: string, action: string): boolean {
  if (!permissions || permissions.length === 0) {
    return false
  }
  
  const specificPermission = `admin:${resource}:${action}`
  
  // Use shared wildcard permission checking
  return hasPermissionWithWildcard(permissions, specificPermission)
}

/**
 * Get user's admin permission level
 */
export function getAdminPermissionLevel(permissions: string[]): 'none' | 'partial' | 'full' {
  const permissionsArray = Array.isArray(permissions) ? permissions : [];
  
  if (!permissionsArray || permissionsArray.length === 0) {
    return 'none'
  }
  
  if (permissionsArray.includes('admin:*:*')) {
    return 'full'
  }
  
  if (permissionsArray.some(p => isAdminPermission(p))) {
    return 'partial'
  }
  
  return 'none'
}