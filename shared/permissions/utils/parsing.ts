// ============================================================================
// SHARED PERMISSION PARSING UTILITIES
// ============================================================================
// Permission string parsing and format validation utilities

import { ParsedPermission, Permission, TimestampedPermission } from '../types/core'
import { createValidationError } from '../types/errors'
import { PLATFORMS, WILDCARDS } from '../constants'

// ============================================================================
// PERMISSION PARSING
// ============================================================================

/**
 * Parse permission string into components
 * Format: "platform:resource:action" or "platform:resource:action:timestamp"
 */
export const parsePermission = (permission: string): ParsedPermission | null => {
  if (!permission || typeof permission !== 'string') {
    return null
  }

  const parts = permission.split(':')
  if (parts.length < 3) {
    return null
  }

  return {
    platform: parts[0],
    resource: parts[1], 
    action: parts[2],
    full: permission
  }
}

/**
 * Parse permission with embedded timestamp support
 * Format: "platform:resource:action:unix_timestamp"
 * Returns: { basePermission, timestamp }
 */
export const parsePermissionWithTimestamp = (permission: string): { 
  basePermission: string
  timestamp?: number 
} => {
  if (!permission || typeof permission !== 'string') {
    return { basePermission: permission }
  }

  const parts = permission.split(':')
  
  // Check if last part is a timestamp (numeric)
  if (parts.length >= 4) {
    const lastPart = parts[parts.length - 1]
    const timestamp = parseInt(lastPart, 10)
    
    if (!isNaN(timestamp) && timestamp > 0) {
      const basePermission = parts.slice(0, -1).join(':')
      return { basePermission, timestamp }
    }
  }
  
  return { basePermission: permission }
}

/**
 * Build permission string from components
 */
export const buildPermission = (platform: string, resource: string, action: string): string => {
  if (!platform || !resource || !action) {
    throw createValidationError(
      'Invalid permission components',
      'permission',
      'required',
      'platform:resource:action'
    )
  }
  
  return `${platform}:${resource}:${action}`
}

/**
 * Build permission with timestamp
 */
export const buildPermissionWithTimestamp = (
  platform: string, 
  resource: string, 
  action: string, 
  timestamp: number
): string => {
  const basePermission = buildPermission(platform, resource, action)
  return `${basePermission}:${timestamp}`
}

// ============================================================================
// PERMISSION VALIDATION
// ============================================================================

/**
 * Validate if a permission string is properly formatted
 */
export const isValidPermissionFormat = (permission: string): boolean => {
  if (!permission || typeof permission !== 'string') {
    return false
  }

  const parts = permission.split(':')
  
  // Basic format validation (3 parts minimum, 4 maximum with timestamp)
  if (parts.length < 3 || parts.length > 4) {
    return false
  }
  
  // Check for empty parts
  if (parts.some(part => !part || part.trim().length === 0)) {
    return false
  }
  
  // Validate platform
  const validPlatforms = Object.values(PLATFORMS)
  if (!validPlatforms.includes(parts[0] as any)) {
    return false
  }
  
  // If there's a 4th part, it should be a valid timestamp
  if (parts.length === 4) {
    const timestamp = parseInt(parts[3], 10)
    if (isNaN(timestamp) || timestamp <= 0) {
      return false
    }
  }
  
  return true
}

/**
 * Validate permission with embedded timestamp support
 */
export const isValidPermission = (permission: string): boolean => {
  const { basePermission } = parsePermissionWithTimestamp(permission)
  return isValidPermissionFormat(basePermission)
}

/**
 * Check if permission contains wildcards
 */
export const hasWildcards = (permission: string): boolean => {
  return permission.includes(WILDCARDS.ALL_RESOURCES) || permission.includes(WILDCARDS.ALL_ACTIONS)
}

/**
 * Check if permission is a wildcard permission
 */
export const isWildcardPermission = (permission: string): boolean => {
  const parsed = parsePermission(permission)
  if (!parsed) return false
  
  return parsed.resource === WILDCARDS.ALL_RESOURCES || 
         parsed.action === WILDCARDS.ALL_ACTIONS
}

/**
 * Check if permission is full platform access
 */
export const isFullPlatformAccess = (permission: string): boolean => {
  const parsed = parsePermission(permission)
  if (!parsed) return false
  
  return parsed.resource === WILDCARDS.ALL_RESOURCES && 
         parsed.action === WILDCARDS.ALL_ACTIONS
}

// ============================================================================
// PERMISSION TRANSFORMATION
// ============================================================================

/**
 * Convert permission to timestamped permission object
 */
export const createTimestampedPermission = (permission: string): TimestampedPermission => {
  const { basePermission, timestamp } = parsePermissionWithTimestamp(permission)
  const now = Date.now()
  const expiresAt = timestamp
  
  let isExpired = false
  let expiresIn: string | undefined
  let timeRemaining: number | undefined
  
  if (expiresAt) {
    const expiryTime = expiresAt * 1000 // Convert to milliseconds
    isExpired = expiryTime <= now
    timeRemaining = Math.max(0, expiryTime - now)
    
    if (!isExpired && timeRemaining) {
      expiresIn = formatTimeRemaining(timeRemaining)
    }
  }
  
  return {
    permission,
    basePermission,
    expiresAt,
    isExpired,
    expiresIn,
    timeRemaining
  }
}

/**
 * Add timestamp to permission string
 */
export const addTimestampToPermission = (permission: string, expiresAt: number): string => {
  const { basePermission } = parsePermissionWithTimestamp(permission)
  return `${basePermission}:${expiresAt}`
}

/**
 * Remove timestamp from permission string
 */
export const removeTimestampFromPermission = (permission: string): string => {
  const { basePermission } = parsePermissionWithTimestamp(permission)
  return basePermission
}

/**
 * Create permission string with relative expiry
 */
export const createPermissionWithRelativeExpiry = (
  permission: string, 
  duration: number, 
  unit: 'minutes' | 'hours' | 'days' | 'weeks' = 'hours'
): string => {
  const now = Math.floor(Date.now() / 1000)
  let seconds: number
  
  switch (unit) {
    case 'minutes':
      seconds = duration * 60
      break
    case 'hours':
      seconds = duration * 60 * 60
      break
    case 'days':
      seconds = duration * 24 * 60 * 60
      break
    case 'weeks':
      seconds = duration * 7 * 24 * 60 * 60
      break
  }
  
  const expiresAt = now + seconds
  return addTimestampToPermission(permission, expiresAt)
}

// ============================================================================
// PERMISSION NORMALIZATION
// ============================================================================

/**
 * Normalize permission format (remove extra spaces, lowercase platform)
 */
export const normalizePermission = (permission: string): string => {
  if (!permission) return permission
  
  const parts = permission.split(':').map(part => part.trim())
  
  if (parts.length >= 3) {
    // Platform should be lowercase
    parts[0] = parts[0].toLowerCase()
    
    // Resource and action case should be preserved
    // Timestamp (if present) should remain as-is
  }
  
  return parts.join(':')
}

/**
 * Normalize array of permissions
 */
export const normalizePermissions = (permissions: string[]): string[] => {
  return permissions
    .filter(perm => perm && typeof perm === 'string')
    .map(normalizePermission)
    .filter(perm => isValidPermission(perm))
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Format time remaining in human readable format
 */
const formatTimeRemaining = (timeRemaining: number): string => {
  const seconds = Math.floor(timeRemaining / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) {
    return `${days} day${days > 1 ? 's' : ''}`
  } else if (hours > 0) {
    return `${hours} hour${hours > 1 ? 's' : ''}`
  } else if (minutes > 0) {
    return `${minutes} minute${minutes > 1 ? 's' : ''}`
  } else {
    return `${seconds} second${seconds > 1 ? 's' : ''}`
  }
}

/**
 * Extract platform from permission string
 */
export const extractPlatform = (permission: string): string | null => {
  const parsed = parsePermission(permission)
  return parsed?.platform || null
}

/**
 * Extract resource from permission string
 */
export const extractResource = (permission: string): string | null => {
  const parsed = parsePermission(permission)
  return parsed?.resource || null
}

/**
 * Extract action from permission string
 */
export const extractAction = (permission: string): string | null => {
  const parsed = parsePermission(permission)
  return parsed?.action || null
}

/**
 * Get all permissions for a specific platform
 */
export const filterPermissionsByPlatform = (permissions: string[], platform: string): string[] => {
  return permissions.filter(perm => {
    const parsed = parsePermission(perm)
    return parsed?.platform === platform
  })
}

/**
 * Get all permissions for a specific resource
 */
export const filterPermissionsByResource = (permissions: string[], resource: string): string[] => {
  return permissions.filter(perm => {
    const parsed = parsePermission(perm)
    return parsed?.resource === resource
  })
}

/**
 * Get all permissions for a specific action
 */
export const filterPermissionsByAction = (permissions: string[], action: string): string[] => {
  return permissions.filter(perm => {
    const parsed = parsePermission(perm)
    return parsed?.action === action
  })
}