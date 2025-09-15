/**
 * User Session Validator Service
 * Integrates with backend session validation API for middleware operations
 */

import { cookies } from 'next/headers'
import { logger, safeError } from '@/lib/logger'

// Types matching backend API
interface SessionValidationRequest {
  app_type: 'admin' | 'user'
  user_agent?: string
  ip_address?: string
  path?: string
  method?: string
}

interface UserProfile {
  id: string
  email: string
  name?: string
  role: string
  permissions: string[]
  package_tier: string
  firebase_uid?: string
}

interface SessionValidationResponse {
  valid: boolean
  user?: UserProfile
  permissions?: string[]
  package_tier?: string
  expires_at?: number
  session_id?: string
  error?: string
  performance?: {
    validation_time_ms: number
    cache_hit: boolean
  }
}

interface SessionValidatorCache {
  user: UserProfile
  permissions: string[]
  package_tier: string
  expires_at: number
  cached_at: number
}

export class UserSessionValidator {
  private static instance: UserSessionValidator
  private cache = new Map<string, SessionValidatorCache>()
  private readonly CACHE_TTL = 5 * 60 * 1000 // 5 minutes
  private readonly MAX_CACHE_SIZE = 1000
  
  private constructor() {}
  
  static getInstance(): UserSessionValidator {
    if (!UserSessionValidator.instance) {
      UserSessionValidator.instance = new UserSessionValidator()
    }
    return UserSessionValidator.instance
  }
  
  /**
   * Validate user session with backend API
   */
  async validateSession(request: {
    userAgent?: string
    ipAddress?: string
    path?: string
    method?: string
  }): Promise<SessionValidationResponse> {
    const startTime = performance.now()
    
    try {
      // Get JWT token from cookies
      const cookieStore = await cookies()
      const jwtCookie = cookieStore.get('epsx_jwt')
      
      if (!jwtCookie?.value) {
        return {
          valid: false,
          error: 'No session token found',
          performance: {
            validation_time_ms: performance.now() - startTime,
            cache_hit: false
          }
        }
      }
      
      const token = jwtCookie.value
      
      // Check cache first
      const cacheKey = `user:${token.substring(0, 20)}:${request.path || ''}`
      const cached = this.getCachedSession(cacheKey)
      
      if (cached) {
        return {
          valid: true,
          user: cached.user,
          permissions: cached.permissions,
          package_tier: cached.package_tier,
          expires_at: cached.expires_at,
          performance: {
            validation_time_ms: performance.now() - startTime,
            cache_hit: true
          }
        }
      }
      
      // Validate with backend API
      const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || (process.env.NODE_ENV === 'development' ? 'http://localhost:8080' : (() => { throw new Error('NEXT_PUBLIC_BACKEND_URL is required') })())
      
      const validationRequest: SessionValidationRequest = {
        app_type: 'user',
        user_agent: request.userAgent,
        ip_address: request.ipAddress,
        path: request.path,
        method: request.method || 'GET'
      }
      
      const response = await fetch(`${backendUrl}/api/auth/validate-session`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'User-Agent': request.userAgent || 'UserMiddleware/1.0',
          'X-Forwarded-For': request.ipAddress || '',
        },
        body: JSON.stringify(validationRequest)
      })
      
      if (!response.ok) {
        const errorText = await response.text()
        return {
          valid: false,
          error: `Backend validation failed: ${response.status} - ${errorText}`,
          performance: {
            validation_time_ms: performance.now() - startTime,
            cache_hit: false
          }
        }
      }
      
      const result = await response.json()
      
      if (!result.valid || !result.user) {
        return {
          valid: false,
          error: result.error || 'Session validation failed',
          performance: {
            validation_time_ms: performance.now() - startTime,
            cache_hit: false
          }
        }
      }
      
      const user = result.user as UserProfile
      
      // Cache the successful validation
      this.cacheSession(cacheKey, {
        user,
        permissions: result.permissions || user.permissions || [],
        package_tier: result.package_tier || user.package_tier || 'FREE',
        expires_at: result.expires_at || (Date.now() + 2 * 60 * 60 * 1000), // 2 hours default
        cached_at: Date.now()
      })
      
      return {
        valid: true,
        user,
        permissions: result.permissions || user.permissions || [],
        package_tier: result.package_tier || user.package_tier || 'FREE',
        expires_at: result.expires_at,
        session_id: result.session_id,
        performance: {
          validation_time_ms: performance.now() - startTime,
          cache_hit: false
        }
      }
      
    } catch (error) {
      logger.error('User session validation failed', error)
      return {
        valid: false,
        error: error instanceof Error ? error.message : 'Unknown validation error',
        performance: {
          validation_time_ms: performance.now() - startTime,
          cache_hit: false
        }
      }
    }
  }
  
  /**
   * Get cached session if valid
   */
  private getCachedSession(key: string): SessionValidatorCache | null {
    const cached = this.cache.get(key)
    
    if (!cached) {
      return null
    }
    
    // Check if expired
    if (Date.now() > cached.expires_at || 
        Date.now() > cached.cached_at + this.CACHE_TTL) {
      this.cache.delete(key)
      return null
    }
    
    return cached
  }
  
  /**
   * Cache session validation result
   */
  private cacheSession(key: string, data: SessionValidatorCache): void {
    // Implement LRU eviction if cache is full
    if (this.cache.size >= this.MAX_CACHE_SIZE) {
      const firstKey = this.cache.keys().next().value
      if (firstKey) {
        this.cache.delete(firstKey)
      }
    }
    
    this.cache.set(key, data)
  }
  
  /**
   * Clear cache (useful for testing or forced refresh)
   */
  clearCache(): void {
    this.cache.clear()
  }
  
  /**
   * Get cache statistics
   */
  getCacheStats(): { size: number; maxSize: number; hitRatio: number } {
    return {
      size: this.cache.size,
      maxSize: this.MAX_CACHE_SIZE,
      hitRatio: 0 // TODO: Track hit/miss ratio
    }
  }
}

// Singleton instance for middleware use
export const userSessionValidator = UserSessionValidator.getInstance()

// Utility functions for middleware
export async function validateUserSession(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}): Promise<SessionValidationResponse> {
  return userSessionValidator.validateSession(request)
}

export async function requireUserSession(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}): Promise<UserProfile> {
  const result = await validateUserSession(request)
  
  if (!result.valid || !result.user) {
    throw new Error(result.error || 'User session validation failed')
  }
  
  return result.user
}

// Permission checking utilities using simple role system
export function hasFeatureAccess(user: UserProfile, feature: string): boolean {
  try {
    // Import the simple role system (using require for non-async context)
    const { checkFeatureAccess, roleFromString } = require('../types/permissions');
    
    // Convert user role to simple role enum
    const role = roleFromString(user.role.toLowerCase());
    
    return checkFeatureAccess(role, feature);
  } catch (error) {
    logger.error('Failed to check feature access', error);
    return false;
  }
}

export function hasRole(user: UserProfile, role: string): boolean {
  try {
    // Import the simple role system
    const { checkRoleAccess, roleFromString } = require('../types/permissions');
    
    // Convert both roles to simple role enums
    const userRole = roleFromString(user.role.toLowerCase());
    const requiredRoleEnum = roleFromString(role.toLowerCase());
    
    return checkRoleAccess(userRole, requiredRoleEnum);
  } catch (error) {
    logger.error('Failed to check role access', error);
    return false;
  }
}

// Legacy permission check - maps old permission strings to features
export function hasPermission(user: UserProfile, permission: string): boolean {
  // Map legacy permissions to features
  switch (permission) {
    case 'users.view':
    case 'dashboard.view':
    case 'analytics.view':
      return hasFeatureAccess(user, 'view_eps');
    
    case 'analytics.export':
      return hasFeatureAccess(user, 'export_data');
    
    case 'admin':
    case 'admin.users':
      return user.role.toLowerCase() === 'admin';
    
    default:
      // Try checking as a feature first
      return hasFeatureAccess(user, permission) ||
             // Fallback to old permission system for backward compatibility
             (user.permissions?.includes(permission) || 
              user.permissions?.some(p => p.endsWith('*') && permission.startsWith(p.slice(0, -1)))) || false;
  }
}

export function hasPackageTier(user: UserProfile, tier: string): boolean {
  const tierHierarchy: Record<string, number> = {
    FREE: 1,
    BRONZE: 2,
    SILVER: 3,
    GOLD: 4,
    PLATINUM: 5,
    ENTERPRISE: 6
  }
  
  const userLevel = tierHierarchy[user.package_tier] || 0
  const requiredLevel = tierHierarchy[tier] || 1
  
  return userLevel >= requiredLevel
}

// Path-based user access checking for premium features
export function canAccessUserPath(user: UserProfile, path: string): boolean {
  // All users can access public routes
  if (path.startsWith('/public') || path === '/' || path.startsWith('/login') || path.startsWith('/register')) {
    return true
  }
  
  // Premium features require at least BRONZE tier
  if (path.includes('/premium') || path.includes('/advanced-analytics')) {
    return hasPackageTier(user, 'BRONZE')
  }
  
  // Professional features require SILVER tier
  if (path.includes('/professional') || path.includes('/alerts')) {
    return hasPackageTier(user, 'SILVER')
  }
  
  // VIP features require GOLD tier
  if (path.includes('/vip') || path.includes('/priority-support')) {
    return hasPackageTier(user, 'GOLD')
  }
  
  // Elite features require PLATINUM tier
  if (path.includes('/elite') || path.includes('/custom-dashboards')) {
    return hasPackageTier(user, 'PLATINUM')
  }
  
  // Enterprise features require ENTERPRISE tier
  if (path.includes('/enterprise') || path.includes('/api-access')) {
    return hasPackageTier(user, 'ENTERPRISE')
  }
  
  // Default: allow access for authenticated users
  return true
}

// Rate limiting helpers for different user tiers
export function getUserRateLimit(user: UserProfile): { perMinute: number; perHour: number } {
  const rateLimits: Record<string, { perMinute: number; perHour: number }> = {
    FREE: { perMinute: 10, perHour: 100 },
    BRONZE: { perMinute: 30, perHour: 500 },
    SILVER: { perMinute: 60, perHour: 1500 },
    GOLD: { perMinute: 120, perHour: 5000 },
    PLATINUM: { perMinute: 300, perHour: 15000 },
    ENTERPRISE: { perMinute: 1000, perHour: 50000 }
  }
  
  return rateLimits[user.package_tier] || rateLimits.FREE
}

// Feature access helpers using simple role system
export function getAvailableFeatures(user: UserProfile): string[] {
  try {
    // Import the simple role system
    const { getUserFeatures, roleFromString } = require('../types/permissions');
    
    // Convert user role to simple role enum
    const role = roleFromString(user.role.toLowerCase());
    
    // Get the 7 core features based on role
    return getUserFeatures(role);
  } catch (error) {
    logger.error('Failed to get available features', error);
    // Fallback to legacy system
    const featuresByTier: Record<string, string[]> = {
      FREE: ['view_eps'],
      BRONZE: ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing', 'advanced_filters'],
      SILVER: ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing', 'advanced_filters'],
      GOLD: ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing', 'advanced_filters'],
      PLATINUM: ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing', 'advanced_filters'],
      ENTERPRISE: ['view_eps', 'export_data', 'realtime', 'profile', 'notifications', 'billing', 'advanced_filters']
    }
    
    return featuresByTier[user.package_tier] || featuresByTier.FREE
  }
}

// Simple feature check that maps to the new system
export function hasSimpleFeature(user: UserProfile, feature: string): boolean {
  const availableFeatures = getAvailableFeatures(user);
  return availableFeatures.includes(feature);
}