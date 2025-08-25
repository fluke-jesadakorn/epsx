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
  permissions: string[]
  admin_modules: string[]
  package_tier: string
  firebase_uid?: string
}

interface SessionValidationResponse {
  valid: boolean
  user?: UserProfile
  permissions?: string[]
  admin_modules?: string[]
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
  admin_modules: string[]
  package_tier: string
  expires_at: number
  cached_at: number
}

export class AdminSessionValidator {
  private static instance: AdminSessionValidator
  private cache = new Map<string, SessionValidatorCache>()
  private readonly CACHE_TTL = 5 * 60 * 1000 // 5 minutes
  private readonly MAX_CACHE_SIZE = 1000
  
  private constructor() {}
  
  static getInstance(): AdminSessionValidator {
    if (!AdminSessionValidator.instance) {
      AdminSessionValidator.instance = new AdminSessionValidator()
    }
    return AdminSessionValidator.instance
  }
  
  /**
   * Validate admin session with backend API
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
        return {
          valid: true,
          user: cached.user,
          permissions: cached.permissions,
          admin_modules: cached.admin_modules,
          package_tier: cached.package_tier,
          expires_at: cached.expires_at,
          performance: {
            validation_time_ms: performance.now() - startTime,
            cache_hit: true
          }
        }
      }
      
      // Validate with backend API
      const backendUrl = process.env.NODE_ENV === 'production'
        ? (process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io')
        : 'http://localhost:8080'
      
      const validationRequest: SessionValidationRequest = {
        app_type: 'admin',
        user_agent: request.userAgent,
        ip_address: request.ipAddress,
        path: request.path,
        method: request.method || 'GET'
      }
      
      const response = await fetch(`${backendUrl}/api/v1/auth/sessions/validate`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'User-Agent': request.userAgent || 'AdminMiddleware/1.0',
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
      
      // Validate admin permissions
      const user = result.user as UserProfile
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
      
      // Cache the successful validation
      this.cacheSession(cacheKey, {
        user,
        permissions: result.permissions || user.permissions || [],
        admin_modules: result.admin_modules || user.admin_modules || [],
        package_tier: result.package_tier || user.package_tier || 'FREE',
        expires_at: result.expires_at || (Date.now() + 2 * 60 * 60 * 1000), // 2 hours default
        cached_at: Date.now()
      })
      
      return {
        valid: true,
        user,
        permissions: result.permissions || user.permissions || [],
        admin_modules: result.admin_modules || user.admin_modules || [],
        package_tier: result.package_tier || user.package_tier || 'FREE',
        expires_at: result.expires_at,
        session_id: result.session_id,
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
    // Must have moderator role or higher
    const roles = ['moderator', 'admin', 'super_admin']
    if (!roles.includes(user.role)) {
      return false
    }
    
    // Must have at least one admin module
    if (!user.admin_modules || user.admin_modules.length === 0) {
      return false
    }
    
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
    super_admin: 5
  }
  
  const userLevel = roleHierarchy[user.role] || 0
  const requiredLevel = roleHierarchy[role] || 1
  
  return userLevel >= requiredLevel
}

export function hasAdminModule(user: UserProfile, module: string): boolean {
  // Check if user has the specific admin module
  if (user.admin_modules && user.admin_modules.includes(module)) {
    return true
  }
  
  // Super admin has access to all modules
  if (user.role === 'super_admin') {
    return true
  }
  
  // Check for admin-full-004 profile
  if (user.admin_modules && user.admin_modules.includes('admin-full-004')) {
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

// Path-based admin access checking
export function canAccessAdminPath(user: UserProfile, path: string): boolean {
  // Super admin can access everything
  if (user.role === 'super_admin') {
    return true
  }
  
  // Check module-specific access
  if (path.includes('/admin/users') || path.includes('/users')) {
    return hasAdminModule(user, 'user_management')
  }
  
  if (path.includes('/admin/analytics') || path.includes('/analytics')) {
    return hasAdminModule(user, 'analytics')
  }
  
  if (path.includes('/admin/reports') || path.includes('/reports')) {
    return hasAdminModule(user, 'reporting')
  }
  
  if (path.includes('/admin/audit') || path.includes('/audit')) {
    return hasAdminModule(user, 'audit_logs')
  }
  
  if (path.includes('/admin/config') || path.includes('/system')) {
    return hasAdminModule(user, 'system_config')
  }
  
  // Default: allow if user has any admin module
  return user.admin_modules && user.admin_modules.length > 0
}