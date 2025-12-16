/**
 * USER SESSION VALIDATOR
 * Specialized wrapper for user authentication using the unified BaseSessionValidator
 * Maintains backward compatibility with existing user middleware
 */

import type { SessionValidationResponse } from '../types/domain/Session'
import type { PermissionGroup, UserProfile } from '../types/domain/User'
import { getPermissionGroupLevel } from '../types/domain/User'
import { BaseSessionValidator, type SessionValidatorConfig, type ValidationRequest } from './validator'

// ============================================================================
// USER-SPECIFIC CONFIGURATION
// ============================================================================

const USER_CONFIG: SessionValidatorConfig = {
  cacheEnabled: true,
  cacheTTL: 5 * 60 * 1000, // 5 minutes
  maxCacheSize: 1000,
  enableMetrics: false, // Lighter metrics for user sessions
  dbConnectionString: process.env.DATABASE_URL,
  backendUrl: process.env.BACKEND_URL || 'http://localhost:8080'
}

// ============================================================================
// USER SESSION VALIDATOR CLASS
// ============================================================================

export class UserSessionValidator {
  private static instance: UserSessionValidator
  public baseValidator: BaseSessionValidator

  private constructor() {
    this.baseValidator = BaseSessionValidator.getInstance(USER_CONFIG)
  }

  static getInstance(): UserSessionValidator {
    if (!UserSessionValidator.instance) {
      UserSessionValidator.instance = new UserSessionValidator()
    }
    return UserSessionValidator.instance
  }

  /**
   * Validate user session (JWT or OIDC)
   */
  async validateSession(request: {
    userAgent?: string
    ipAddress?: string
    path?: string
    method?: string
  }): Promise<SessionValidationResponse> {
    const userRequest: ValidationRequest = {
      ...request,
      appType: 'user'
    }

    return this.baseValidator.validateSession(userRequest)
  }

  /**
   * Check feature access based on user profile
   */
  hasFeatureAccess(user: UserProfile, feature: string): boolean {
    try {
      const role = user.role.toLowerCase()

      // Admin users have access to all features
      if (role === 'admin') {
        return true
      }

      // Map features to permission requirements
      switch (feature) {
        case 'view_eps':
          return user.permissions?.some(p => p.startsWith('epsx:rankings:')) || true
        case 'export_data':
          return user.permissions?.some(p => p.includes('export') || p.includes('advanced')) || user.permissionGroup !== 'Basic Access Group'
        case 'realtime':
          return user.permissions?.some(p => p.includes('realtime')) || user.permissionGroup !== 'Basic Access Group'
        case 'profile':
        case 'notifications':
        case 'billing':
          return true // Basic features available to all users
        case 'advanced_filters':
          return user.permissions?.some(p => p.includes('advanced')) || ['Standard Access Group', 'Premium Access Group', 'Professional Access Group', 'Enterprise Access Group'].includes(user.permissionGroup)
        default:
          return user.permissions?.includes(feature) || false
      }
    } catch (error) {
      console.error('Failed to check feature access:', error)
      return false
    }
  }

  /**
   * Check user role access
   */
  hasRole(user: UserProfile, role: string): boolean {
    try {
      const userRole = user.role.toLowerCase()
      const requiredRole = role.toLowerCase()

      // Admin role has access to everything
      if (userRole === 'admin') {
        return true
      }

      // Exact role match
      if (userRole === requiredRole) {
        return true
      }

      // Role hierarchy: admin > premium > user > guest
      const roleHierarchy = {
        'guest': 0,
        'user': 1,
        'premium': 2,
        'admin': 3
      }

      const userLevel = roleHierarchy[userRole as keyof typeof roleHierarchy] || 0
      const requiredLevel = roleHierarchy[requiredRole as keyof typeof roleHierarchy] || 0

      return userLevel >= requiredLevel
    } catch (error) {
      console.error('Failed to check role access:', error)
      return false
    }
  }

  /**
   * Check path access based on package tier
   */
  canAccessUserPath(user: UserProfile, path: string): boolean {
    // All users can access public routes
    if (path.startsWith('/public') || path === '/' || path.startsWith('/login') || path.startsWith('/register')) {
      return true
    }

    // Premium features require at least BRONZE tier
    if (path.includes('/premium') || path.includes('/advanced-analytics')) {
      return getPermissionGroupLevel(user.permissionGroup) >= getPermissionGroupLevel('Standard Access Group')
    }

    // Professional features require SILVER tier
    if (path.includes('/professional') || path.includes('/alerts')) {
      return getPermissionGroupLevel(user.permissionGroup) >= getPermissionGroupLevel('Standard Access Group')
    }

    // VIP features require GOLD tier
    if (path.includes('/vip') || path.includes('/priority-support')) {
      return getPermissionGroupLevel(user.permissionGroup) >= getPermissionGroupLevel('Premium Access Group')
    }

    // Elite features require PLATINUM tier
    if (path.includes('/elite') || path.includes('/custom-dashboards')) {
      return getPermissionGroupLevel(user.permissionGroup) >= getPermissionGroupLevel('Professional Access Group')
    }

    // Enterprise features require ENTERPRISE tier
    if (path.includes('/enterprise') || path.includes('/api-access')) {
      return getPermissionGroupLevel(user.permissionGroup) >= getPermissionGroupLevel('Enterprise Access Group')
    }

    // Default: allow access for authenticated users
    return true
  }

  /**
   * Get user rate limits based on permission group
   */
  getUserRateLimit(user: UserProfile): { perMinute: number; perHour: number } {
    const rateLimits: Record<PermissionGroup, { perMinute: number; perHour: number }> = {
      'Basic Access Group': { perMinute: 10, perHour: 100 },
      'Standard Access Group': { perMinute: 30, perHour: 500 },
      'Premium Access Group': { perMinute: 120, perHour: 5000 },
      'Professional Access Group': { perMinute: 300, perHour: 15000 },
      'Enterprise Access Group': { perMinute: 1000, perHour: 50000 }
    }

    return rateLimits[user.permissionGroup] || rateLimits['Basic Access Group']
  }

  /**
   * Get available features for user
   */
  getAvailableFeatures(user: UserProfile): string[] {
    try {
      // Admin users get all features
      if (user.role.toLowerCase() === 'admin') {
        return ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing', 'advanced_filters']
      }

      // Feature mapping by package tier
      const featuresByTier: Record<string, string[]> = {
        FREE: ['view_eps', 'profile'],
        BRONZE: ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing'],
        SILVER: ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing', 'advanced_filters'],
        GOLD: ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing', 'advanced_filters'],
        PLATINUM: ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing', 'advanced_filters'],
        ENTERPRISE: ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing', 'advanced_filters']
      }

      return featuresByTier[user.packageTier ?? 'FREE'] ?? featuresByTier['FREE']
    } catch (error) {
      console.error('Failed to get available features:', error)
      return ['view_eps', 'profile'] // Safe fallback
    }
  }

  /**
   * Legacy permission check (backward compatibility)
   */
  hasPermission(user: UserProfile, permission: string): boolean {
    // Map legacy permissions to features
    switch (permission) {
      case 'users.view':
      case 'dashboard.view':
      case 'analytics.view':
        return this.hasFeatureAccess(user, 'view_eps')

      case 'analytics.export':
        return this.hasFeatureAccess(user, 'export_data')

      case 'admin':
      case 'admin.users':
        return user.role.toLowerCase() === 'admin'

      default:
        // Try checking as a feature first
        return this.hasFeatureAccess(user, permission) ||
          // Fallback to permission system
          this.baseValidator.hasPermission(user, permission)
    }
  }

  /**
   * Cache management methods (delegate to base validator)
   */
  clearCache(): void {
    this.baseValidator.clearCache()
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
export const userSessionValidator = UserSessionValidator.getInstance()

/**
 * Validate user session
 */
export async function validateUserSession(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}): Promise<SessionValidationResponse> {
  return userSessionValidator.validateSession(request)
}

/**
 * Require user session (throws if invalid)
 */
export async function requireUserSession(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}): Promise<UserProfile> {
  const result = await validateUserSession(request)

  if (!result.valid || !result.session?.user) {
    throw new Error(result.error || 'User session validation failed')
  }

  return result.session.user
}

/**
 * Check feature access
 */
export function hasFeatureAccess(user: UserProfile, feature: string): boolean {
  return userSessionValidator.hasFeatureAccess(user, feature)
}

/**
 * Check role access
 */
export function hasRole(user: UserProfile, role: string): boolean {
  return userSessionValidator.hasRole(user, role)
}

/**
 * Legacy permission check
 */
export function hasPermission(user: UserProfile, permission: string): boolean {
  return userSessionValidator.hasPermission(user, permission)
}

/**
 * Check package tier access
 */
export function hasPackageTier(user: UserProfile, tier: string): boolean {
  return userSessionValidator.baseValidator.hasPackageTier(user, tier)
}

/**
 * Check path access
 */
export function canAccessUserPath(user: UserProfile, path: string): boolean {
  return userSessionValidator.canAccessUserPath(user, path)
}

/**
 * Get user rate limits
 */
export function getUserRateLimit(user: UserProfile): { perMinute: number; perHour: number } {
  return userSessionValidator.getUserRateLimit(user)
}

/**
 * Get available features
 */
export function getAvailableFeatures(user: UserProfile): string[] {
  return userSessionValidator.getAvailableFeatures(user)
}

/**
 * Simple feature check
 */
export function hasSimpleFeature(user: UserProfile, feature: string): boolean {
  const availableFeatures = getAvailableFeatures(user)
  return availableFeatures.includes(feature)
}

export default UserSessionValidator