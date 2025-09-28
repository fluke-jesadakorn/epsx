/**
 * ADMIN SESSION VALIDATOR
 * Specialized wrapper for admin authentication using the unified BaseSessionValidator
 * Maintains backward compatibility with existing admin middleware
 */

import { BaseSessionValidator, type ValidationRequest, type SessionValidatorConfig } from './BaseSessionValidator'
import type { UserProfile, SessionValidationResponse } from '../types/domain/Session'

// ============================================================================
// ADMIN-SPECIFIC CONFIGURATION
// ============================================================================

const ADMIN_CONFIG: SessionValidatorConfig = {
  cacheEnabled: true,
  cacheTTL: 5 * 60 * 1000, // 5 minutes
  maxCacheSize: 1000,
  enableMetrics: true,
  dbConnectionString: process.env.DATABASE_URL,
  backendUrl: process.env.BACKEND_URL || 'http://localhost:8080'
}

// ============================================================================
// ADMIN SESSION VALIDATOR CLASS
// ============================================================================

export class AdminSessionValidator {
  private static instance: AdminSessionValidator
  public baseValidator: BaseSessionValidator

  private constructor() {
    this.baseValidator = BaseSessionValidator.getInstance(ADMIN_CONFIG)
  }

  static getInstance(): AdminSessionValidator {
    if (!AdminSessionValidator.instance) {
      AdminSessionValidator.instance = new AdminSessionValidator()
    }
    return AdminSessionValidator.instance
  }

  /**
   * Validate admin session (wallet-based or OIDC)
   */
  async validateSession(request: {
    userAgent?: string
    ipAddress?: string
    path?: string
    method?: string
  }): Promise<SessionValidationResponse> {
    const adminRequest: ValidationRequest = {
      ...request,
      appType: 'admin'
    }

    const result = await this.baseValidator.validateSession(adminRequest)

    // Additional admin-specific validation
    if (result.valid && result.session?.user) {
      if (!this.hasAdminAccess(result.session.user)) {
        return {
          valid: false,
          error: 'User lacks admin access permissions',
          performance: result.performance
        }
      }
    }

    return result
  }

  /**
   * Check if user has admin access
   */
  private hasAdminAccess(user: UserProfile): boolean {
    const permissions = Array.isArray(user.permissions) ? user.permissions : []
    
    // Check for admin wildcard permission
    if (permissions.includes('admin:*:*')) {
      return true
    }
    
    // Check if user has any admin-scoped permissions
    return permissions.some(p => p.startsWith('admin:'))
  }

  /**
   * Check admin permission
   */
  hasAdminPermission(user: UserProfile, permission: string): boolean {
    const permissions = Array.isArray(user.permissions) ? user.permissions : []
    
    // Check for admin wildcard permission
    if (permissions.includes('admin:*:*')) {
      return true
    }
    
    // Check if user has the specific permission
    if (permissions.includes(permission)) {
      return true
    }
    
    // Check for broader permissions (e.g., admin:users:* covers admin:users:manage)
    if (permission.includes(':')) {
      const [platform, resource] = permission.split(':')
      if (permissions.some(p => 
        p === `${platform}:${resource}:*` || 
        p === `${platform}:*:*`
      )) {
        return true
      }
    }
    
    return false
  }

  /**
   * Check if user can access admin path
   */
  canAccessAdminPath(user: UserProfile, path: string): boolean {
    const permissions = Array.isArray(user.permissions) ? user.permissions : []
    
    // Check for admin wildcard permission
    if (permissions.includes('admin:*:*')) {
      return true
    }
    
    // Map paths to required permissions
    const pathPermissions: Record<string, string> = {
      '/users': 'admin:users:manage',
      '/analytics': 'admin:analytics:view',
      '/permissions': 'admin:permissions:manage',
      '/notifications': 'admin:notifications:manage',
      '/settings': 'admin:system:configure',
      '/developer-portal': 'admin:developer:access',
      '/stock-ranking-packages': 'admin:packages:manage',
    }
    
    // Check specific path permissions
    for (const [pathPrefix, permission] of Object.entries(pathPermissions)) {
      if (path.includes(pathPrefix)) {
        return this.hasAdminPermission(user, permission)
      }
    }
    
    // Default: allow if user has any admin-scoped permissions
    return permissions.some(p => p.startsWith('admin:'))
  }

  /**
   * Cache management methods (delegate to base validator)
   */
  clearCache(): void {
    this.baseValidator.clearCache()
  }

  resetStats(): void {
    this.baseValidator.resetMetrics()
  }

  getCacheStats() {
    return this.baseValidator.getMetrics()
  }
}

// ============================================================================
// CONVENIENCE FUNCTIONS (BACKWARD COMPATIBILITY)
// ============================================================================

/**
 * Singleton instance for middleware use
 */
export const adminSessionValidator = AdminSessionValidator.getInstance()

/**
 * Validate admin session
 */
export async function validateAdminSession(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}): Promise<SessionValidationResponse> {
  return adminSessionValidator.validateSession(request)
}

/**
 * Require admin session (throws if invalid)
 */
export async function requireAdminSession(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}): Promise<UserProfile> {
  const result = await validateAdminSession(request)
  
  if (!result.valid || !result.session?.user) {
    throw new Error(result.error || 'Admin session validation failed')
  }
  
  return result.session.user
}

/**
 * Check if user has specific permission
 */
export function hasPermission(user: UserProfile, permission: string): boolean {
  return adminSessionValidator.baseValidator.hasPermission(user, permission)
}

/**
 * Check if user has admin permission
 */
export function hasAdminPermission(user: UserProfile, permission: string): boolean {
  return adminSessionValidator.hasAdminPermission(user, permission)
}

/**
 * Check if user has package tier
 */
export function hasPackageTier(user: UserProfile, tier: string): boolean {
  return adminSessionValidator.baseValidator.hasPackageTier(user, tier)
}

/**
 * Check if user can access admin path
 */
export function canAccessAdminPath(user: UserProfile, path: string): boolean {
  return adminSessionValidator.canAccessAdminPath(user, path)
}

export default AdminSessionValidator