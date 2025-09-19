// ============================================================================
// SHARED PERMISSION EXPIRY UTILITIES
// ============================================================================
// Permission expiry, timestamp validation, and temporal logic

import { 
  GranularPermissionClaim, 
  TimestampedPermission, 
  PermissionExpiryDetails,
  PermissionExpiryInfo
} from '../types/core'
import { UserClaims, EnhancedUserClaims } from '../types/claims'
import { parsePermissionWithTimestamp, createTimestampedPermission } from './parsing'
import { EXPIRY_THRESHOLDS } from '../constants'

// ============================================================================
// BASIC EXPIRY CHECKING
// ============================================================================

/**
 * Check if permission claim is still valid (not expired)
 */
export const isClaimValid = (claim: GranularPermissionClaim): boolean => {
  if (!claim.expires_at) return true // Permanent permission
  const now = Math.floor(Date.now() / 1000)
  return claim.expires_at > now
}

/**
 * Check if a permission with timestamp is still valid
 */
export const isPermissionValidWithTime = (permission: string): boolean => {
  const { timestamp } = parsePermissionWithTimestamp(permission)
  if (!timestamp) return true // No timestamp means permanent permission
  
  const now = Math.floor(Date.now() / 1000) // Current Unix timestamp
  return timestamp > now
}

/**
 * Check if permission is expiring soon
 */
export const isPermissionExpiringSoon = (
  permission: string, 
  withinHours: number = EXPIRY_THRESHOLDS.EXPIRING_SOON_HOURS
): boolean => {
  const { timestamp } = parsePermissionWithTimestamp(permission)
  if (!timestamp) return false // Permanent permission
  
  const now = Math.floor(Date.now() / 1000)
  const withinSeconds = withinHours * 60 * 60
  const threshold = now + withinSeconds
  
  return timestamp <= threshold && timestamp > now
}

/**
 * Check if claim is expiring soon
 */
export const isClaimExpiringSoon = (
  claim: GranularPermissionClaim,
  withinHours: number = EXPIRY_THRESHOLDS.EXPIRING_SOON_HOURS
): boolean => {
  if (!claim.expires_at) return false // Permanent permission
  
  const now = Math.floor(Date.now() / 1000)
  const withinSeconds = withinHours * 60 * 60
  const threshold = now + withinSeconds
  
  return claim.expires_at <= threshold && claim.expires_at > now
}

// ============================================================================
// PERMISSION FILTERING
// ============================================================================

/**
 * Filter out expired permissions from a permission array
 */
export const filterValidPermissions = (permissions: string[]): string[] => {
  return permissions.filter(isPermissionValidWithTime)
}

/**
 * Filter valid permissions from granular permission claims
 */
export const filterValidGranularPermissions = (
  permissions: Record<string, GranularPermissionClaim>
): Record<string, GranularPermissionClaim> => {
  const validPermissions: Record<string, GranularPermissionClaim> = {}
  
  for (const [permission, claim] of Object.entries(permissions)) {
    if (isClaimValid(claim)) {
      validPermissions[permission] = claim
    }
  }
  
  return validPermissions
}

/**
 * Get expired permissions from granular permission claims
 */
export const getExpiredGranularPermissions = (
  permissions: Record<string, GranularPermissionClaim>
): Record<string, GranularPermissionClaim> => {
  const expiredPermissions: Record<string, GranularPermissionClaim> = {}
  
  for (const [permission, claim] of Object.entries(permissions)) {
    if (!isClaimValid(claim)) {
      expiredPermissions[permission] = claim
    }
  }
  
  return expiredPermissions
}

// ============================================================================
// EXPIRY DETAILS AND INFO
// ============================================================================

/**
 * Get permission expiry details for granular permissions
 */
export const getPermissionExpiryDetails = (
  permissions: Record<string, GranularPermissionClaim>,
  permission: string
): PermissionExpiryDetails | null => {
  const claim = permissions[permission]
  if (!claim) return null

  const now = Date.now()
  const isExpired = claim.expires_at ? (claim.expires_at * 1000) <= now : false
  const expiresInMs = claim.expires_at ? (claim.expires_at * 1000) - now : undefined
  
  let expiresInHuman: string | undefined
  if (expiresInMs && expiresInMs > 0) {
    expiresInHuman = formatTimeRemaining(expiresInMs)
  }

  return {
    permission,
    base_permission: permission,
    claim,
    is_expired: isExpired,
    expires_in_ms: expiresInMs,
    expires_in_human: expiresInHuman,
    is_permanent: !claim.expires_at
  }
}

/**
 * Get expiry information for user's permissions (granular system)
 */
export const getPermissionExpiryInfoGranular = (
  permissions: Record<string, GranularPermissionClaim>
): PermissionExpiryInfo => {
  const timestampedPermissions = Object.keys(permissions).map(perm => {
    const claim = permissions[perm]
    const timestamped = createTimestampedPermission(perm)
    
    // Override with claim data for consistency
    return {
      ...timestamped,
      expiresAt: claim.expires_at,
      isExpired: !isClaimValid(claim)
    }
  })
  
  const now = Date.now()
  const twentyFourHoursFromNow = now + (EXPIRY_THRESHOLDS.EXPIRING_SOON_HOURS * 60 * 60 * 1000)
  
  const expired = timestampedPermissions.filter(p => p.isExpired)
  const expiringSoon = timestampedPermissions.filter(p => 
    !p.isExpired && 
    p.expiresAt && 
    (p.expiresAt * 1000) <= twentyFourHoursFromNow
  )
  
  const nextExpiry = timestampedPermissions
    .filter(p => !p.isExpired && p.expiresAt)
    .sort((a, b) => (a.expiresAt || 0) - (b.expiresAt || 0))[0]
  
  return {
    hasExpiringPermissions: expiringSoon.length > 0,
    expiringSoon,
    expired,
    nextExpiry
  }
}

/**
 * Get expiry information for user's permissions (legacy system)
 */
export const getPermissionExpiryInfo = (userClaims: UserClaims | null): PermissionExpiryInfo => {
  if (!userClaims) {
    return {
      hasExpiringPermissions: false,
      expiringSoon: [],
      expired: []
    }
  }
  
  const timestampedPermissions = userClaims.permissions.map(createTimestampedPermission)
  const now = Date.now()
  const twentyFourHoursFromNow = now + (EXPIRY_THRESHOLDS.EXPIRING_SOON_HOURS * 60 * 60 * 1000)
  
  const expired = timestampedPermissions.filter(p => p.isExpired)
  const expiringSoon = timestampedPermissions.filter(p => 
    !p.isExpired && 
    p.expiresAt && 
    (p.expiresAt * 1000) <= twentyFourHoursFromNow
  )
  
  const nextExpiry = timestampedPermissions
    .filter(p => !p.isExpired && p.expiresAt)
    .sort((a, b) => (a.expiresAt || 0) - (b.expiresAt || 0))[0]
  
  return {
    hasExpiringPermissions: expiringSoon.length > 0,
    expiringSoon,
    expired,
    nextExpiry
  }
}

// ============================================================================
// TIME CALCULATIONS
// ============================================================================

/**
 * Get time until next permission expiry
 */
export const getTimeUntilNextExpiry = (userClaims: UserClaims | null): number | null => {
  const expiryInfo = getPermissionExpiryInfo(userClaims)
  return expiryInfo.nextExpiry?.timeRemaining || null
}

/**
 * Get time until next expiry for granular permissions
 */
export const getTimeUntilNextExpiryGranular = (
  permissions: Record<string, GranularPermissionClaim>
): number | null => {
  const expiryInfo = getPermissionExpiryInfoGranular(permissions)
  return expiryInfo.nextExpiry?.timeRemaining || null
}

/**
 * Check if permissions will change soon due to expiry
 */
export const willPermissionsChangeSoon = (
  permissions: Record<string, GranularPermissionClaim>, 
  hoursAhead: number = EXPIRY_THRESHOLDS.EXPIRING_SOON_HOURS
): boolean => {
  const checkTime = Date.now() + (hoursAhead * 60 * 60 * 1000)
  
  return Object.values(permissions).some(claim => 
    claim.expires_at && (claim.expires_at * 1000) <= checkTime
  )
}

/**
 * Get effective permissions at a specific time
 */
export const getEffectivePermissionsAtTime = (
  permissions: Record<string, GranularPermissionClaim>, 
  atTime: Date
): Record<string, GranularPermissionClaim> => {
  const targetTimestamp = Math.floor(atTime.getTime() / 1000)
  const effectivePermissions: Record<string, GranularPermissionClaim> = {}
  
  for (const [permission, claim] of Object.entries(permissions)) {
    if (!claim.expires_at || claim.expires_at > targetTimestamp) {
      effectivePermissions[permission] = claim
    }
  }
  
  return effectivePermissions
}

// ============================================================================
// EXPIRY FORMATTING
// ============================================================================

/**
 * Format expiry time as human-readable string
 */
export const formatExpiryTime = (expiresAt: number): string => {
  const expiryDate = new Date(expiresAt * 1000)
  const now = new Date()
  
  if (expiryDate <= now) {
    return 'Expired'
  }
  
  const diffMs = expiryDate.getTime() - now.getTime()
  return formatTimeRemaining(diffMs)
}

/**
 * Format time remaining in human readable format
 */
export const formatTimeRemaining = (timeRemaining: number): string => {
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
 * Format detailed expiry information
 */
export const formatDetailedExpiryInfo = (expiresAt: number): {
  absolute: string
  relative: string
  isExpired: boolean
  timeRemaining: number
} => {
  const now = Date.now()
  const expiryTime = expiresAt * 1000
  const isExpired = expiryTime <= now
  const timeRemaining = Math.max(0, expiryTime - now)
  
  const absoluteDate = new Date(expiryTime)
  const absolute = absoluteDate.toLocaleString()
  
  const relative = isExpired ? 'Expired' : formatTimeRemaining(timeRemaining)
  
  return {
    absolute,
    relative,
    isExpired,
    timeRemaining
  }
}

// ============================================================================
// BATCH OPERATIONS
// ============================================================================

/**
 * Get all permissions with expiry information for UI display
 */
export const getAllPermissionsWithExpiry = (permissions: string[]): TimestampedPermission[] => {
  return permissions.map(createTimestampedPermission)
}

/**
 * Get permissions expiring within a specific time frame
 */
export const getPermissionsExpiringWithin = (
  permissions: Record<string, GranularPermissionClaim>,
  hours: number
): Array<{ permission: string; claim: GranularPermissionClaim; expiresIn: number }> => {
  const now = Math.floor(Date.now() / 1000)
  const threshold = now + (hours * 60 * 60)
  
  const expiring: Array<{ permission: string; claim: GranularPermissionClaim; expiresIn: number }> = []
  
  for (const [permission, claim] of Object.entries(permissions)) {
    if (claim.expires_at && claim.expires_at <= threshold && claim.expires_at > now) {
      expiring.push({
        permission,
        claim,
        expiresIn: claim.expires_at - now
      })
    }
  }
  
  // Sort by expiry time (soonest first)
  return expiring.sort((a, b) => a.expiresIn - b.expiresIn)
}

/**
 * Count permissions by expiry status
 */
export const countPermissionsByExpiryStatus = (
  permissions: Record<string, GranularPermissionClaim>
): {
  total: number
  permanent: number
  temporary: number
  expired: number
  expiringSoon: number
} => {
  let permanent = 0
  let temporary = 0
  let expired = 0
  let expiringSoon = 0
  
  const now = Math.floor(Date.now() / 1000)
  const soonThreshold = now + (EXPIRY_THRESHOLDS.EXPIRING_SOON_HOURS * 60 * 60)
  
  for (const claim of Object.values(permissions)) {
    if (!claim.expires_at) {
      permanent++
    } else if (claim.expires_at <= now) {
      expired++
    } else if (claim.expires_at <= soonThreshold) {
      expiringSoon++
      temporary++
    } else {
      temporary++
    }
  }
  
  return {
    total: Object.keys(permissions).length,
    permanent,
    temporary,
    expired,
    expiringSoon
  }
}