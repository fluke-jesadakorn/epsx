/**
 * UNIFIED SESSION VALIDATORS INDEX
 * Consolidated session validation system replacing duplicate implementations
 * Supports wallet-based (admin), JWT-based (legacy), and OIDC-based (modern) authentication
 */

// ============================================================================
// BASE VALIDATOR (UNIFIED SYSTEM)
// ============================================================================

export {
  BaseSessionValidator,
  createSessionValidator,
  validateSession,
  requireSession,
  type SessionValidatorConfig,
  type ValidationRequest
} from './BaseSessionValidator'

// ============================================================================
// SPECIALIZED VALIDATORS (BACKWARD COMPATIBILITY)
// ============================================================================

// Admin Session Validator
export {
  AdminSessionValidator,
  adminSessionValidator,
  validateAdminSession,
  requireAdminSession,
  hasPermission as hasAdminPermission,
  hasAdminPermission as hasAdminPermissionStrict,
  hasPackageTier as hasAdminPackageTier,
  canAccessAdminPath
} from './AdminSessionValidator'

// User Session Validator
export {
  UserSessionValidator,
  userSessionValidator,
  validateUserSession,
  requireUserSession,
  hasFeatureAccess,
  hasRole,
  hasPermission as hasUserPermission,
  hasPackageTier as hasUserPackageTier,
  canAccessUserPath,
  getUserRateLimit,
  getAvailableFeatures,
  hasSimpleFeature
} from './UserSessionValidator'

// ============================================================================
// MIGRATION HELPERS
// ============================================================================

/**
 * Migration map from old validator imports to new unified system
 */
export const VALIDATOR_MIGRATION_MAP = {
  // Old admin imports → New imports
  'apps/admin-frontend/lib/session-validator': {
    'AdminSessionValidator': 'AdminSessionValidator',
    'adminSessionValidator': 'adminSessionValidator',
    'validateAdminSession': 'validateAdminSession',
    'requireAdminSession': 'requireAdminSession',
    'hasPermission': 'hasAdminPermission',
    'hasAdminPermission': 'hasAdminPermissionStrict',
    'hasPackageTier': 'hasAdminPackageTier',
    'canAccessAdminPath': 'canAccessAdminPath'
  },
  
  // Old user imports → New imports
  'apps/frontend/lib/session-validator': {
    'UserSessionValidator': 'UserSessionValidator',
    'userSessionValidator': 'userSessionValidator',
    'validateUserSession': 'validateUserSession',
    'requireUserSession': 'requireUserSession',
    'hasFeatureAccess': 'hasFeatureAccess',
    'hasRole': 'hasRole',
    'hasPermission': 'hasUserPermission',
    'hasPackageTier': 'hasUserPackageTier',
    'canAccessUserPath': 'canAccessUserPath',
    'getUserRateLimit': 'getUserRateLimit',
    'getAvailableFeatures': 'getAvailableFeatures',
    'hasSimpleFeature': 'hasSimpleFeature'
  }
} as const

/**
 * Get unified validator import for legacy validator path
 */
export function getUnifiedValidatorImport(legacyPath: keyof typeof VALIDATOR_MIGRATION_MAP): Record<string, string> {
  return VALIDATOR_MIGRATION_MAP[legacyPath]
}

/**
 * Check if validator has been migrated to unified system
 */
export function isValidatorMigrated(importPath: string): boolean {
  return Object.keys(VALIDATOR_MIGRATION_MAP).includes(importPath)
}

// ============================================================================
// CONVENIENCE VALIDATORS FOR SPECIFIC USE CASES
// ============================================================================

/**
 * Quick admin validation for middleware
 */
export async function quickAdminValidation(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}) {
  const { adminSessionValidator } = await import('./AdminSessionValidator')
  return adminSessionValidator.validateSession(request)
}

/**
 * Quick user validation for middleware
 */
export async function quickUserValidation(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}) {
  const { userSessionValidator } = await import('./UserSessionValidator')
  return userSessionValidator.validateSession(request)
}

/**
 * Auto-detect validation method and validate accordingly
 */
export async function autoValidateSession(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}) {
  const { BaseSessionValidator } = await import('./BaseSessionValidator')
  const validator = BaseSessionValidator.getInstance()
  return validator.validateSession(request)
}

// ============================================================================
// SHARED TYPES (RE-EXPORTED FOR CONVENIENCE)
// ============================================================================

export type {
  UserProfile,
  SessionValidationResponse,
  SessionData,
  AdminJWTPayload,
  UserJWTPayload
} from '../types/domain/Session'

// ============================================================================
// PERFORMANCE MONITORING UTILITIES
// ============================================================================

/**
 * Get performance metrics from all validators
 */
export function getAllValidatorMetrics() {
  const { adminSessionValidator } = require('./AdminSessionValidator')
  const { userSessionValidator } = require('./UserSessionValidator')
  
  return {
    admin: adminSessionValidator.getCacheStats(),
    user: userSessionValidator.getCacheStats(),
    timestamp: new Date().toISOString()
  }
}

/**
 * Clear all validator caches
 */
export function clearAllValidatorCaches() {
  const { adminSessionValidator } = require('./AdminSessionValidator')
  const { userSessionValidator } = require('./UserSessionValidator')
  
  adminSessionValidator.clearCache()
  userSessionValidator.clearCache()
}

/**
 * Reset all validator metrics
 */
export function resetAllValidatorMetrics() {
  const { adminSessionValidator } = require('./AdminSessionValidator')
  
  adminSessionValidator.resetStats()
  // User validator metrics are lighter and reset automatically
}

// ============================================================================
// CONFIGURATION HELPERS
// ============================================================================

/**
 * Create optimized validator configuration for different environments
 */
export function createValidatorConfig(env: 'development' | 'staging' | 'production') {
  const configs = {
    development: {
      cacheEnabled: true,
      cacheTTL: 2 * 60 * 1000, // 2 minutes
      maxCacheSize: 100,
      enableMetrics: true
    },
    staging: {
      cacheEnabled: true,
      cacheTTL: 5 * 60 * 1000, // 5 minutes
      maxCacheSize: 500,
      enableMetrics: true
    },
    production: {
      cacheEnabled: true,
      cacheTTL: 10 * 60 * 1000, // 10 minutes
      maxCacheSize: 2000,
      enableMetrics: true
    }
  }
  
  return configs[env]
}

// ============================================================================
// DEFAULT EXPORT
// ============================================================================

export default {
  // Only export what actually exists
  // BaseSessionValidator,
  // AdminSessionValidator, 
  // UserSessionValidator,
  // validateSession,
  // validateAdminSession,
  // validateUserSession
}