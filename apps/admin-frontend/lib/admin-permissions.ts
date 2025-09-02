/**
 * Admin Permission Utilities
 * Client-side utilities for validating admin permissions using structured permission system
 * Separated from server actions to avoid Next.js build errors
 */

/**
 * Check if user has admin permissions using structured permission system
 */
export function validateAdminPermissions(permissions: string[]): boolean {
  if (!permissions || permissions.length === 0) {
    return false
  }
  
  // Check for admin permissions using structured format
  const hasAdminAccess = permissions.some(permission => 
    permission === 'admin:*:*' ||           // Full admin access
    permission.startsWith('admin:')         // Any admin-scoped permission
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
  const wildcardResource = `admin:${resource}:*`
  const fullAdmin = 'admin:*:*'
  
  return permissions.some(permission => 
    permission === specificPermission ||
    permission === wildcardResource ||
    permission === fullAdmin
  )
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
  
  if (permissionsArray.some(p => p.startsWith('admin:'))) {
    return 'partial'
  }
  
  return 'none'
}