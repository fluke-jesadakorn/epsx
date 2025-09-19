// ============================================================================
// SHARED PERMISSION VALIDATION UTILITIES
// ============================================================================
// Permission validation, requirements checking, and security utilities

import { 
  EnhancedUserClaims, 
  GranularPermissionClaim,
  PermissionSource 
} from '../types/core'
import { 
  createValidationError, 
  createInsufficientPermissionError,
  createExpiredError,
  PermissionErrorCode
} from '../types/errors'
import { isValidPermissionFormat, parsePermission } from './parsing'
import { hasPermissionGranular } from './checking'
import { isClaimValid } from './expiry'
import { PLATFORMS, PERMISSION_SOURCES } from '../constants'

// ============================================================================
// BASIC VALIDATION
// ============================================================================

/**
 * Validate permission string format and content
 */
export const validatePermissionString = (permission: string): {
  isValid: boolean
  errors: string[]
} => {
  const errors: string[] = []
  
  if (!permission || typeof permission !== 'string') {
    errors.push('Permission must be a non-empty string')
    return { isValid: false, errors }
  }
  
  if (!isValidPermissionFormat(permission)) {
    errors.push('Permission must follow format: platform:resource:action')
    return { isValid: false, errors }
  }
  
  const parsed = parsePermission(permission)
  if (!parsed) {
    errors.push('Failed to parse permission string')
    return { isValid: false, errors }
  }
  
  // Validate platform
  const validPlatforms = Object.values(PLATFORMS)
  if (!validPlatforms.includes(parsed.platform as any)) {
    errors.push(`Invalid platform '${parsed.platform}'. Valid platforms: ${validPlatforms.join(', ')}`)
  }
  
  // Validate resource and action aren't empty
  if (!parsed.resource || parsed.resource.trim().length === 0) {
    errors.push('Resource cannot be empty')
  }
  
  if (!parsed.action || parsed.action.trim().length === 0) {
    errors.push('Action cannot be empty')
  }
  
  // Check for invalid characters
  const invalidChars = /[^a-zA-Z0-9\-_*]/
  if (invalidChars.test(parsed.resource)) {
    errors.push('Resource contains invalid characters. Only alphanumeric, hyphens, underscores, and wildcards allowed')
  }
  
  if (invalidChars.test(parsed.action)) {
    errors.push('Action contains invalid characters. Only alphanumeric, hyphens, underscores, and wildcards allowed')
  }
  
  return { isValid: errors.length === 0, errors }
}

/**
 * Validate array of permissions
 */
export const validatePermissions = (permissions: string[]): {
  isValid: boolean
  validPermissions: string[]
  invalidPermissions: Array<{ permission: string; errors: string[] }>
} => {
  const validPermissions: string[] = []
  const invalidPermissions: Array<{ permission: string; errors: string[] }> = []
  
  for (const permission of permissions) {
    const validation = validatePermissionString(permission)
    if (validation.isValid) {
      validPermissions.push(permission)
    } else {
      invalidPermissions.push({ permission, errors: validation.errors })
    }
  }
  
  return {
    isValid: invalidPermissions.length === 0,
    validPermissions,
    invalidPermissions
  }
}

/**
 * Validate permission source
 */
export const validatePermissionSource = (source: string): {
  isValid: boolean
  error?: string
} => {
  const validSources = Object.values(PERMISSION_SOURCES)
  if (!validSources.includes(source as any)) {
    return {
      isValid: false,
      error: `Invalid permission source '${source}'. Valid sources: ${validSources.join(', ')}`
    }
  }
  
  return { isValid: true }
}

// ============================================================================
// REQUIREMENT VALIDATION
// ============================================================================

/**
 * Require user to have specific permission (throws on failure)
 */
export const requirePermission = (
  userClaims: EnhancedUserClaims | null, 
  permission: string
): EnhancedUserClaims => {
  if (!userClaims) {
    throw createInsufficientPermissionError(permission, undefined, [])
  }
  
  // Validate permission format
  const validation = validatePermissionString(permission)
  if (!validation.isValid) {
    throw createValidationError(
      `Invalid permission format: ${validation.errors.join(', ')}`,
      'permission',
      'format',
      'platform:resource:action'
    )
  }
  
  if (!hasPermissionGranular(userClaims.permissions, permission)) {
    const userPermissions = Object.keys(userClaims.permissions)
    throw createInsufficientPermissionError(permission, userClaims.sub, userPermissions)
  }
  
  return userClaims
}

/**
 * Require user to have any of the specified permissions
 */
export const requireAnyPermission = (
  userClaims: EnhancedUserClaims | null, 
  permissions: string[]
): EnhancedUserClaims => {
  if (!userClaims) {
    throw createInsufficientPermissionError(permissions.join(' OR '), undefined, [])
  }
  
  // Validate all permissions
  const validation = validatePermissions(permissions)
  if (!validation.isValid) {
    const errors = validation.invalidPermissions.map(ip => 
      `${ip.permission}: ${ip.errors.join(', ')}`
    ).join('; ')
    throw createValidationError(
      `Invalid permission formats: ${errors}`,
      'permissions',
      'format'
    )
  }
  
  const hasAny = permissions.some(permission => 
    hasPermissionGranular(userClaims.permissions, permission)
  )
  
  if (!hasAny) {
    const userPermissions = Object.keys(userClaims.permissions)
    throw createInsufficientPermissionError(
      permissions.join(' OR '), 
      userClaims.sub, 
      userPermissions
    )
  }
  
  return userClaims
}

/**
 * Require user to have all specified permissions
 */
export const requireAllPermissions = (
  userClaims: EnhancedUserClaims | null, 
  permissions: string[]
): EnhancedUserClaims => {
  if (!userClaims) {
    throw createInsufficientPermissionError(permissions.join(' AND '), undefined, [])
  }
  
  // Validate all permissions
  const validation = validatePermissions(permissions)
  if (!validation.isValid) {
    const errors = validation.invalidPermissions.map(ip => 
      `${ip.permission}: ${ip.errors.join(', ')}`
    ).join('; ')
    throw createValidationError(
      `Invalid permission formats: ${errors}`,
      'permissions',
      'format'
    )
  }
  
  const missingPermissions = permissions.filter(permission => 
    !hasPermissionGranular(userClaims.permissions, permission)
  )
  
  if (missingPermissions.length > 0) {
    const userPermissions = Object.keys(userClaims.permissions)
    throw createInsufficientPermissionError(
      missingPermissions.join(' AND '), 
      userClaims.sub, 
      userPermissions
    )
  }
  
  return userClaims
}

// ============================================================================
// PERMISSION CLAIM VALIDATION
// ============================================================================

/**
 * Validate granular permission claim
 */
export const validatePermissionClaim = (claim: GranularPermissionClaim): {
  isValid: boolean
  errors: string[]
} => {
  const errors: string[] = []
  
  // Validate source
  const sourceValidation = validatePermissionSource(claim.source)
  if (!sourceValidation.isValid) {
    errors.push(sourceValidation.error!)
  }
  
  // Validate timestamps
  if (claim.granted_at && claim.granted_at <= 0) {
    errors.push('granted_at must be a positive timestamp')
  }
  
  if (claim.expires_at) {
    if (claim.expires_at <= 0) {
      errors.push('expires_at must be a positive timestamp')
    }
    
    if (claim.granted_at && claim.expires_at <= claim.granted_at) {
      errors.push('expires_at must be after granted_at')
    }
    
    // Check if already expired
    const now = Math.floor(Date.now() / 1000)
    if (claim.expires_at <= now) {
      errors.push('Permission has already expired')
    }
  }
  
  // Validate granted_by if present
  if (claim.granted_by && (typeof claim.granted_by !== 'string' || claim.granted_by.trim().length === 0)) {
    errors.push('granted_by must be a non-empty string')
  }
  
  return { isValid: errors.length === 0, errors }
}

/**
 * Validate permission claims object
 */
export const validatePermissionClaims = (
  permissions: Record<string, GranularPermissionClaim>
): {
  isValid: boolean
  validCount: number
  invalidCount: number
  errors: Array<{ permission: string; errors: string[] }>
} => {
  const errors: Array<{ permission: string; errors: string[] }> = []
  let validCount = 0
  
  for (const [permission, claim] of Object.entries(permissions)) {
    // Validate permission string
    const permissionValidation = validatePermissionString(permission)
    
    // Validate claim
    const claimValidation = validatePermissionClaim(claim)
    
    const allErrors = [...permissionValidation.errors, ...claimValidation.errors]
    
    if (allErrors.length > 0) {
      errors.push({ permission, errors: allErrors })
    } else {
      validCount++
    }
  }
  
  return {
    isValid: errors.length === 0,
    validCount,
    invalidCount: errors.length,
    errors
  }
}

// ============================================================================
// SECURITY VALIDATION
// ============================================================================

/**
 * Validate permission doesn't grant excessive access
 */
export const validatePermissionSecurity = (permission: string): {
  isSecure: boolean
  warnings: string[]
  riskLevel: 'low' | 'medium' | 'high' | 'critical'
} => {
  const warnings: string[] = []
  let riskLevel: 'low' | 'medium' | 'high' | 'critical' = 'low'
  
  const parsed = parsePermission(permission)
  if (!parsed) {
    return { isSecure: false, warnings: ['Invalid permission format'], riskLevel: 'critical' }
  }
  
  // Check for wildcards
  if (parsed.resource === '*' && parsed.action === '*') {
    warnings.push('Permission grants full platform access')
    riskLevel = 'critical'
  } else if (parsed.action === '*') {
    warnings.push('Permission grants all actions on resource')
    riskLevel = 'high'
  }
  
  // Check for admin permissions
  if (parsed.platform === 'admin') {
    warnings.push('Admin permission detected')
    if (riskLevel === 'low') riskLevel = 'medium'
    
    if (parsed.resource === '*' || parsed.action === '*') {
      warnings.push('Admin permission with wildcards')
      riskLevel = 'critical'
    }
  }
  
  // Check for system-level permissions
  const systemActions = ['delete', 'destroy', 'admin', 'system', 'sudo']
  if (systemActions.some(action => parsed.action.toLowerCase().includes(action))) {
    warnings.push('Permission includes potentially destructive action')
    if (riskLevel === 'low') riskLevel = 'medium'
  }
  
  return {
    isSecure: riskLevel !== 'critical',
    warnings,
    riskLevel
  }
}

/**
 * Validate bulk permission operation
 */
export const validateBulkPermissionOperation = (
  permissions: string[],
  userIds: string[],
  maxBatchSize: number = 100
): {
  isValid: boolean
  errors: string[]
  warnings: string[]
} => {
  const errors: string[] = []
  const warnings: string[] = []
  
  // Check batch size
  if (userIds.length > maxBatchSize) {
    errors.push(`Batch size ${userIds.length} exceeds maximum of ${maxBatchSize}`)
  }
  
  // Validate user IDs
  if (userIds.length === 0) {
    errors.push('At least one user ID is required')
  }
  
  const invalidUserIds = userIds.filter(id => !id || typeof id !== 'string' || id.trim().length === 0)
  if (invalidUserIds.length > 0) {
    errors.push(`Invalid user IDs: ${invalidUserIds.length} empty or invalid`)
  }
  
  // Validate permissions
  const permissionValidation = validatePermissions(permissions)
  if (!permissionValidation.isValid) {
    errors.push(`Invalid permissions: ${permissionValidation.invalidPermissions.length}`)
  }
  
  // Security validation for each permission
  let highRiskCount = 0
  let criticalRiskCount = 0
  
  for (const permission of permissionValidation.validPermissions) {
    const security = validatePermissionSecurity(permission)
    if (security.riskLevel === 'high') highRiskCount++
    if (security.riskLevel === 'critical') criticalRiskCount++
  }
  
  if (criticalRiskCount > 0) {
    warnings.push(`${criticalRiskCount} critical risk permission(s) in bulk operation`)
  }
  
  if (highRiskCount > 0) {
    warnings.push(`${highRiskCount} high risk permission(s) in bulk operation`)
  }
  
  // Check for potential over-provisioning
  if (permissions.length > 10) {
    warnings.push('Large number of permissions being granted - review for over-provisioning')
  }
  
  return {
    isValid: errors.length === 0,
    errors,
    warnings
  }
}

// ============================================================================
// EXPIRY VALIDATION
// ============================================================================

/**
 * Validate permission expiry time
 */
export const validatePermissionExpiry = (
  expiresAt: number,
  maxDurationDays: number = 365
): {
  isValid: boolean
  error?: string
  warning?: string
} => {
  const now = Math.floor(Date.now() / 1000)
  
  // Check if already expired
  if (expiresAt <= now) {
    return { isValid: false, error: 'Expiry time is in the past' }
  }
  
  // Check maximum duration
  const maxExpiry = now + (maxDurationDays * 24 * 60 * 60)
  if (expiresAt > maxExpiry) {
    return { 
      isValid: false, 
      error: `Expiry time exceeds maximum duration of ${maxDurationDays} days` 
    }
  }
  
  // Warning for very long durations
  const longDurationDays = 180 // 6 months
  const longDurationExpiry = now + (longDurationDays * 24 * 60 * 60)
  if (expiresAt > longDurationExpiry) {
    return { 
      isValid: true, 
      warning: `Permission expires in more than ${longDurationDays} days - consider shorter duration` 
    }
  }
  
  return { isValid: true }
}

// ============================================================================
// COMPREHENSIVE VALIDATION
// ============================================================================

/**
 * Comprehensive validation for granting a permission
 */
export const validatePermissionGrant = (
  permission: string,
  userId: string,
  source: PermissionSource,
  expiresAt?: number,
  grantedBy?: string
): {
  isValid: boolean
  errors: string[]
  warnings: string[]
  securityRisk: 'low' | 'medium' | 'high' | 'critical'
} => {
  const errors: string[] = []
  const warnings: string[] = []
  let securityRisk: 'low' | 'medium' | 'high' | 'critical' = 'low'
  
  // Validate permission string
  const permissionValidation = validatePermissionString(permission)
  if (!permissionValidation.isValid) {
    errors.push(...permissionValidation.errors)
  }
  
  // Validate user ID
  if (!userId || typeof userId !== 'string' || userId.trim().length === 0) {
    errors.push('User ID is required and must be non-empty')
  }
  
  // Validate source
  const sourceValidation = validatePermissionSource(source)
  if (!sourceValidation.isValid) {
    errors.push(sourceValidation.error!)
  }
  
  // Validate expiry if provided
  if (expiresAt) {
    const expiryValidation = validatePermissionExpiry(expiresAt)
    if (!expiryValidation.isValid) {
      errors.push(expiryValidation.error!)
    }
    if (expiryValidation.warning) {
      warnings.push(expiryValidation.warning)
    }
  }
  
  // Validate granted by if provided
  if (grantedBy && (typeof grantedBy !== 'string' || grantedBy.trim().length === 0)) {
    errors.push('granted_by must be a non-empty string if provided')
  }
  
  // Security validation
  if (permissionValidation.isValid) {
    const security = validatePermissionSecurity(permission)
    warnings.push(...security.warnings)
    securityRisk = security.riskLevel
    
    if (!security.isSecure) {
      warnings.push('Permission may grant excessive access')
    }
  }
  
  return {
    isValid: errors.length === 0,
    errors,
    warnings,
    securityRisk
  }
}