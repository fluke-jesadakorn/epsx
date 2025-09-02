/**
 * Admin Session Validator Service
 * Integrates with backend session validation API for middleware operations
 */

import { cookies } from 'next/headers'

/**
 * Extract permissions array from JWT payload (handles both admin and user tokens)
 */
function extractPermissionsFromPayload(payload: any): string[] {
  // Handle admin token structure (AdminJWTClaims)
  if (payload.token_type === 'admin_access' && payload.permissions?.system_access?.capabilities) {
    return payload.permissions.system_access.capabilities;
  }
  
  // Handle user token structure (UserJWTClaims)
  if (payload.token_type === 'user_access' && payload.permissions?.permissions) {
    return payload.permissions.permissions;
  }
  
  // Handle legacy or simple structure
  if (Array.isArray(payload.permissions)) {
    return payload.permissions;
  }
  
  // Fallback for admin users
  if (payload.role === 'admin' || payload.email?.includes('admin')) {
    return [
      'admin:*:*',
      'admin:users:manage',
      'admin:analytics:view',
      'admin:system:manage'
    ];
  }
  
  // Default fallback
  return ['epsx:dashboard:read'];
}

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
  permissions: string[]  // Structured permissions: "platform:resource:action"
  package_tier: string
  firebase_uid?: string
  
  // Cross-platform fields
  platforms?: string[]
  primary_platform?: string
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
  
  // Cross-platform fields
  platforms?: string[]
}

interface SessionValidatorCache {
  user: UserProfile
  permissions: string[]
  package_tier: string
  expires_at: number
  cached_at: number
  
  // Cross-platform fields
  platforms?: string[]
}

export class AdminSessionValidator {
  private static instance: AdminSessionValidator
  private cache = new Map<string, SessionValidatorCache>()
  private readonly CACHE_TTL = 5 * 60 * 1000 // 5 minutes
  private readonly MAX_CACHE_SIZE = 1000
  private hitCount = 0
  private missCount = 0
  
  private constructor() {}
  
  static getInstance(): AdminSessionValidator {
    if (!AdminSessionValidator.instance) {
      AdminSessionValidator.instance = new AdminSessionValidator()
    }
    return AdminSessionValidator.instance
  }
  
  /**
   * Validate admin session using local JWT verification
   */
  async validateSession(request: {
    userAgent?: string
    ipAddress?: string
    path?: string
    method?: string
  }): Promise<SessionValidationResponse> {
    const startTime = performance.now()
    
    try {
      // OIDC Migration: Get access token from OIDC cookies
      const cookieStore = await cookies()
      const jwtCookie = cookieStore.get('access_token')
      
      if (!jwtCookie?.value) {
        this.missCount++
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
      const cacheKey = `admin:${token.substring(0, 20)}:${request.path || ''}`
      const cached = this.getCachedSession(cacheKey)
      
      if (cached) {
        this.hitCount++
        return {
          valid: true,
          user: cached.user,
          permissions: cached.permissions,
          platforms: cached.platforms,
          package_tier: cached.package_tier,
          expires_at: cached.expires_at,
          performance: {
            validation_time_ms: performance.now() - startTime,
            cache_hit: true
          }
        }
      }
      
      // Cache miss - increment miss counter
      this.missCount++
      
      // Validate JWT token locally using jose library
      const { verifyJWT } = await import('@/lib/auth-utils')
      const payload = await verifyJWT(token)
      
      if (!payload) {
        return {
          valid: false,
          error: 'Invalid or expired JWT token',
          performance: {
            validation_time_ms: performance.now() - startTime,
            cache_hit: false
          }
        }
      }
      
      // Convert JWT payload to UserProfile format
      // Handle both admin and user JWT structures
      const permissions = extractPermissionsFromPayload(payload);
      
      const user: UserProfile = {
        id: payload.sub,
        email: payload.email,
        name: payload.name,
        role: payload.role || 'admin',
        permissions: permissions,
        package_tier: payload.package_tier || 'ENTERPRISE',
        firebase_uid: payload.firebase_uid,
        
        // Cross-platform fields
        platforms: payload.platforms || ['admin', 'epsx'],
        primary_platform: payload.primary_platform || 'admin'
      }
      
      // Validate admin permissions
      if (!this.hasAdminAccess(user)) {
        return {
          valid: false,
          error: 'User lacks admin access permissions',
          performance: {
            validation_time_ms: performance.now() - startTime,
            cache_hit: false
          }
        }
      }
      
      // Calculate expiration from JWT payload
      const expires_at = payload.exp * 1000 // Convert to milliseconds
      
      // Cache the successful validation
      this.cacheSession(cacheKey, {
        user,
        permissions: user.permissions,
        package_tier: user.package_tier,
        expires_at,
        cached_at: Date.now(),
        platforms: user.platforms
      })
      
      return {
        valid: true,
        user,
        permissions: user.permissions,
        platforms: user.platforms,
        package_tier: user.package_tier,
        expires_at,
        performance: {
          validation_time_ms: performance.now() - startTime,
          cache_hit: false
        }
      }
      
    } catch (error) {
      console.error('❌ Admin session validation failed:', error)
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
   * Check if user has admin access using structured permissions only
   */
  private hasAdminAccess(user: UserProfile): boolean {
    const permissions = Array.isArray(user.permissions) ? user.permissions : [];
    
    // Check for admin wildcard permission
    if (permissions.includes('admin:*:*')) {
      return true
    }
    
    // Check if user has any admin-scoped permissions
    if (permissions.some(p => p.startsWith('admin:'))) {
      return true
    }
    
    // No valid admin permissions found
    return false
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
    this.hitCount = 0
    this.missCount = 0
  }
  
  /**
   * Reset cache statistics without clearing cache
   */
  resetStats(): void {
    this.hitCount = 0
    this.missCount = 0
  }
  
  /**
   * Get cache statistics
   */
  getCacheStats(): { 
    size: number; 
    maxSize: number; 
    hitRatio: number;
    hitCount: number;
    missCount: number;
    totalRequests: number;
  } {
    const totalRequests = this.hitCount + this.missCount
    const hitRatio = totalRequests > 0 ? this.hitCount / totalRequests : 0
    
    return {
      size: this.cache.size,
      maxSize: this.MAX_CACHE_SIZE,
      hitRatio: Math.round(hitRatio * 100) / 100, // Round to 2 decimal places
      hitCount: this.hitCount,
      missCount: this.missCount,
      totalRequests
    }
  }
}

// Singleton instance for middleware use
export const adminSessionValidator = AdminSessionValidator.getInstance()

// Utility functions for middleware
export async function validateAdminSession(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}): Promise<SessionValidationResponse> {
  return adminSessionValidator.validateSession(request)
}

export async function requireAdminSession(request: {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
}): Promise<UserProfile> {
  const result = await validateAdminSession(request)
  
  if (!result.valid || !result.user) {
    throw new Error(result.error || 'Admin session validation failed')
  }
  
  return result.user
}

// Permission checking utilities
export function hasPermission(user: UserProfile, permission: string): boolean {
  const permissions = Array.isArray(user.permissions) ? user.permissions : [];
  
  // Check exact match
  if (permissions.includes(permission)) {
    return true
  }
  
  // Check wildcard permissions
  return permissions.some(p => {
    if (p.endsWith('*')) {
      const prefix = p.slice(0, -1)
      return permission.startsWith(prefix)
    }
    return false
  })
}


export function hasAdminPermission(user: UserProfile, permission: string): boolean {
  const permissions = Array.isArray(user.permissions) ? user.permissions : [];
  
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

// Path-based admin access checking with structured permissions only
export function canAccessAdminPath(user: UserProfile, path: string): boolean {
  const permissions = Array.isArray(user.permissions) ? user.permissions : [];
  
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
      return hasAdminPermission(user, permission)
    }
  }
  
  // Default: allow if user has any admin-scoped permissions
  return user.permissions && user.permissions.some(p => p.startsWith('admin:'))
}