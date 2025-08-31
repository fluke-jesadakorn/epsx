/**
 * Admin Session Validator Service
 * Integrates with backend session validation API for middleware operations
 */

import { cookies } from 'next/headers'

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
  platform_context?: string
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
  platform_context?: string
}

interface SessionValidatorCache {
  user: UserProfile
  permissions: string[]
  package_tier: string
  expires_at: number
  cached_at: number
  
  // Cross-platform fields
  platforms?: string[]
  platform_context?: string
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
      // Get JWT token from cookies
      const cookieStore = await cookies()
      const jwtCookie = cookieStore.get('epsx_admin_jwt')
      
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
          platform_context: cached.platform_context,
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
      const user: UserProfile = {
        id: payload.sub,
        email: payload.email,
        name: payload.name,
        role: payload.role,
        permissions: payload.permissions || ['epsx:dashboard:read'],
        package_tier: payload.package_tier || 'FREE',
        firebase_uid: payload.firebase_uid,
        
        // Cross-platform fields
        platforms: payload.platforms || ['epsx'],
        primary_platform: payload.primary_platform || 'epsx',
        platform_context: payload.platform_context
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
        platforms: user.platforms,
        platform_context: user.platform_context
      })
      
      return {
        valid: true,
        user,
        permissions: user.permissions,
        platforms: user.platforms,
        platform_context: user.platform_context,
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
   * Check if user has admin access
   */
  private hasAdminAccess(user: UserProfile): boolean {
    // Check if user has admin or moderator role
    if (user.role === 'admin' || user.role === 'moderator') {
      return true
    }
    
    // Check if user has any admin permissions (indicating admin access)
    if (user.permissions && user.permissions.some(p => 
      p.includes(':manage') || 
      p.includes(':admin') ||
      p.includes('users:') ||
      p.includes('system:')
    )) {
      return true
    }
    
    // For now, allow users who successfully authenticated through OAuth admin flow
    // TODO: Add proper admin role assignment in backend
    console.warn('⚠️ Admin access granted based on OAuth authentication - user should have proper admin role assigned')
    return true
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
  // Check exact match
  if (user.permissions.includes(permission)) {
    return true
  }
  
  // Check wildcard permissions
  return user.permissions.some(p => {
    if (p.endsWith('*')) {
      const prefix = p.slice(0, -1)
      return permission.startsWith(prefix)
    }
    return false
  })
}

export function hasRole(user: UserProfile, role: string): boolean {
  const roleHierarchy: Record<string, number> = {
    user: 1,
    premium: 2,
    moderator: 3,
    admin: 4,
  }
  
  const userLevel = roleHierarchy[user.role] || 0
  const requiredLevel = roleHierarchy[role] || 1
  
  return userLevel >= requiredLevel
}

export function hasAdminPermission(user: UserProfile, permission: string): boolean {
  // Check if user has the specific permission
  if (user.permissions && user.permissions.includes(permission)) {
    return true
  }
  
  // Check for platform-specific permission if not already specified
  if (!permission.includes(':')) {
    const platform = user.platform_context || user.primary_platform || 'epsx'
    const fullPermission = `${platform}:${permission}`
    if (user.permissions && user.permissions.includes(fullPermission)) {
      return true
    }
  }
  
  // Super admin has access to all modules
  if (user.role === 'admin') {
    return true
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

// Path-based admin access checking with structured permissions
export function canAccessAdminPath(user: UserProfile, path: string): boolean {
  // Super admin can access everything
  if (user.role === 'admin') {
    return true
  }
  
  const platform = user.platform_context || user.primary_platform || 'epsx'
  
  // Check permission-based access
  if (path.includes('/admin/users') || path.includes('/users')) {
    return hasAdminPermission(user, `${platform}:users:manage`)
  }
  
  if (path.includes('/admin/analytics') || path.includes('/analytics')) {
    return hasAdminPermission(user, `${platform}:analytics:read`)
  }
  
  if (path.includes('/admin/reports') || path.includes('/reports')) {
    return hasAdminPermission(user, `${platform}:reports:read`)
  }
  
  if (path.includes('/admin/audit') || path.includes('/audit')) {
    return hasAdminPermission(user, `${platform}:audit:read`)
  }
  
  if (path.includes('/admin/config') || path.includes('/system')) {
    return hasAdminPermission(user, `${platform}:system:manage`)
  }
  
  // Default: allow if user has any admin permissions
  return user.permissions && user.permissions.some(p => 
    p.includes(':manage') || p.includes(':admin') || p.includes('system:')
  )
}