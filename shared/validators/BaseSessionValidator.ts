/**
 * BASE SESSION VALIDATOR
 * Unified session validation system consolidating AdminSessionValidator and UserSessionValidator
 * Supports both wallet-based (admin) and JWT-based (user) authentication methods
 */

import { cookies } from 'next/headers'
import { SiweMessage } from 'siwe'

// ============================================================================
// SHARED TYPES (from consolidated domain types)
// ============================================================================

import type {
  UserProfile,
  SessionValidationResponse,
  SessionData,
  AdminJWTPayload,
  UserJWTPayload
} from '../types/domain/Session'

// ============================================================================
// VALIDATOR CONFIGURATION
// ============================================================================

export interface SessionValidatorConfig {
  cacheEnabled?: boolean
  cacheTTL?: number
  maxCacheSize?: number
  enableMetrics?: boolean
  dbConnectionString?: string
  backendUrl?: string
}

export interface ValidationRequest {
  userAgent?: string
  ipAddress?: string
  path?: string
  method?: string
  appType?: 'admin' | 'user'
}

interface SessionValidatorCache {
  user: UserProfile
  permissions: string[]
  package_tier: string
  expires_at: number
  cached_at: number
  platforms?: string[]
}

interface CacheMetrics {
  hitCount: number
  missCount: number
  totalRequests: number
  hitRatio: number
}

// ============================================================================
// BASE SESSION VALIDATOR CLASS
// ============================================================================

export class BaseSessionValidator {
  private static instance: BaseSessionValidator
  private cache = new Map<string, SessionValidatorCache>()
  private readonly config: Required<SessionValidatorConfig>
  private hitCount = 0
  private missCount = 0

  private constructor(config: SessionValidatorConfig = {}) {
    this.config = {
      cacheEnabled: config.cacheEnabled ?? true,
      cacheTTL: config.cacheTTL ?? 5 * 60 * 1000, // 5 minutes
      maxCacheSize: config.maxCacheSize ?? 1000,
      enableMetrics: config.enableMetrics ?? true,
      dbConnectionString: config.dbConnectionString ?? process.env.DATABASE_URL ?? '',
      backendUrl: config.backendUrl ?? process.env.BACKEND_URL ?? 'http://localhost:8080'
    }
  }

  static getInstance(config?: SessionValidatorConfig): BaseSessionValidator {
    if (!BaseSessionValidator.instance) {
      BaseSessionValidator.instance = new BaseSessionValidator(config)
    }
    return BaseSessionValidator.instance
  }

  // ============================================================================
  // MAIN VALIDATION METHOD
  // ============================================================================

  /**
   * Unified session validation - detects authentication method and validates accordingly
   */
  async validateSession(request: ValidationRequest): Promise<SessionValidationResponse> {
    const startTime = performance.now()

    try {
      const cookieStore = await cookies()
      
      // Detect authentication method
      const authMethod = this.detectAuthMethod(cookieStore)
      
      let result: SessionValidationResponse
      
      switch (authMethod) {
        case 'wallet':
          result = await this.validateWalletSession(request, cookieStore)
          break
        case 'jwt':
          result = await this.validateJWTSession(request, cookieStore)
          break
        case 'oidc':
          result = await this.validateOIDCSession(request, cookieStore)
          break
        default:
          result = {
            valid: false,
            error: 'No valid authentication method found',
            performance: {
              validationTimeMs: performance.now() - startTime,
              cacheHit: false
            }
          }
      }

      // Update metrics
      if (this.config.enableMetrics) {
        if (result.performance?.cacheHit) {
          this.hitCount++
        } else {
          this.missCount++
        }
      }

      return result

    } catch (error) {
      if (this.config.enableMetrics) {
        this.missCount++
      }

      return {
        valid: false,
        error: error instanceof Error ? error.message : 'Unknown validation error',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }
    }
  }

  // ============================================================================
  // AUTHENTICATION METHOD DETECTION
  // ============================================================================

  private detectAuthMethod(cookieStore: any): 'wallet' | 'jwt' | 'oidc' | 'none' {
    // Check for wallet authentication cookies
    const walletAddress = cookieStore.get('wallet_address')?.value
    const walletSignature = cookieStore.get('wallet_signature')?.value
    if (walletAddress && walletSignature) {
      return 'wallet'
    }

    // Check for OIDC tokens
    const accessToken = cookieStore.get('access_token')?.value
    const idToken = cookieStore.get('id_token')?.value
    if (accessToken || idToken) {
      return 'oidc'
    }

    // Check for legacy JWT token
    const jwtToken = cookieStore.get('epsx_jwt')?.value
    if (jwtToken) {
      return 'jwt'
    }

    return 'none'
  }

  // ============================================================================
  // WALLET-BASED VALIDATION (ADMIN)
  // ============================================================================

  private async validateWalletSession(
    request: ValidationRequest,
    cookieStore: any
  ): Promise<SessionValidationResponse> {
    const startTime = performance.now()

    const walletAddress = cookieStore.get('wallet_address')?.value
    const walletNonce = cookieStore.get('wallet_nonce')?.value
    const walletSignature = cookieStore.get('wallet_signature')?.value
    const walletMessage = cookieStore.get('wallet_message')?.value
    const walletExpiresAt = cookieStore.get('wallet_expires_at')?.value

    if (!walletAddress || !walletSignature || !walletMessage || !walletExpiresAt) {
      return {
        valid: false,
        error: 'Incomplete wallet session data',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }
    }

    // Check expiration
    const expiresAt = parseInt(walletExpiresAt, 10)
    if (Date.now() > expiresAt) {
      return {
        valid: false,
        error: 'Wallet session expired',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }
    }

    // Validate SIWE message
    try {
      const siweMessage = new SiweMessage(walletMessage)
      if (siweMessage.address.toLowerCase() !== walletAddress.toLowerCase()) {
        return {
          valid: false,
          error: 'Wallet address mismatch in SIWE message',
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: false
          }
        }
      }
    } catch (error) {
      return {
        valid: false,
        error: 'Invalid SIWE message format',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }
    }

    // Check cache
    const cacheKey = `wallet:${walletAddress.substring(0, 20)}:${request.path || ''}`
    if (this.config.cacheEnabled) {
      const cached = this.getCachedSession(cacheKey)
      if (cached) {
        return {
          valid: true,
          session: {
            user: cached.user,
            permissions: cached.permissions,
            packageTier: cached.package_tier,
            expiresAt: cached.expires_at,
            platforms: cached.platforms
          },
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: true
          }
        }
      }
    }

    // Validate wallet permissions via database
    const permissions = await this.validateWalletPermissions(walletAddress)
    
    const user: UserProfile = {
      id: walletAddress,
      walletAddress: walletAddress,
      email: null,
      name: walletAddress.substring(0, 8) + '...',
      role: permissions.some(p => p.startsWith('admin:')) ? 'admin' : 'user',
      permissions: permissions,
      packageTier: 'ENTERPRISE',
      platforms: ['admin', 'epsx'],
      primaryPlatform: 'admin'
    }

    // Validate admin access
    if (!this.hasAdminAccess(user)) {
      return {
        valid: false,
        error: 'User lacks admin access permissions',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }
    }

    // Cache result
    if (this.config.cacheEnabled) {
      this.cacheSession(cacheKey, {
        user,
        permissions: user.permissions,
        package_tier: user.packageTier,
        expires_at: expiresAt,
        cached_at: Date.now(),
        platforms: user.platforms
      })
    }

    return {
      valid: true,
      session: {
        user,
        permissions: user.permissions,
        packageTier: user.packageTier,
        expiresAt: expiresAt,
        platforms: user.platforms
      },
      performance: {
        validationTimeMs: performance.now() - startTime,
        cacheHit: false
      }
    }
  }

  // ============================================================================
  // JWT-BASED VALIDATION (USER)
  // ============================================================================

  private async validateJWTSession(
    request: ValidationRequest,
    cookieStore: any
  ): Promise<SessionValidationResponse> {
    const startTime = performance.now()

    const jwtCookie = cookieStore.get('epsx_jwt')
    if (!jwtCookie?.value) {
      return {
        valid: false,
        error: 'No JWT token found',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }
    }

    const token = jwtCookie.value

    // Check cache
    const cacheKey = `jwt:${token.substring(0, 20)}:${request.path || ''}`
    if (this.config.cacheEnabled) {
      const cached = this.getCachedSession(cacheKey)
      if (cached) {
        return {
          valid: true,
          session: {
            user: cached.user,
            permissions: cached.permissions,
            packageTier: cached.package_tier,
            expiresAt: cached.expires_at
          },
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: true
          }
        }
      }
    }

    // Validate with backend API
    try {
      const response = await fetch(`${this.config.backendUrl}/api/auth/validate-session`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'User-Agent': request.userAgent || 'SessionValidator/1.0',
          'X-Forwarded-For': request.ipAddress || '',
        },
        body: JSON.stringify({
          app_type: 'user',
          user_agent: request.userAgent,
          ip_address: request.ipAddress,
          path: request.path,
          method: request.method || 'GET'
        })
      })

      if (!response.ok) {
        const errorText = await response.text()
        return {
          valid: false,
          error: `Backend validation failed: ${response.status} - ${errorText}`,
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: false
          }
        }
      }

      const result = await response.json()

      if (!result.valid || !result.user) {
        return {
          valid: false,
          error: result.error || 'Session validation failed',
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: false
          }
        }
      }

      const user = result.user as UserProfile

      // Cache result
      if (this.config.cacheEnabled) {
        this.cacheSession(cacheKey, {
          user,
          permissions: result.permissions || user.permissions || [],
          package_tier: result.package_tier || user.packageTier || 'FREE',
          expires_at: result.expires_at || (Date.now() + 2 * 60 * 60 * 1000),
          cached_at: Date.now()
        })
      }

      return {
        valid: true,
        session: {
          user,
          permissions: result.permissions || user.permissions || [],
          packageTier: result.package_tier || user.packageTier || 'FREE',
          expiresAt: result.expires_at,
          sessionId: result.session_id
        },
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }

    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : 'Backend validation failed',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }
    }
  }

  // ============================================================================
  // OIDC-BASED VALIDATION (MODERN)
  // ============================================================================

  private async validateOIDCSession(
    request: ValidationRequest,
    cookieStore: any
  ): Promise<SessionValidationResponse> {
    const startTime = performance.now()

    const accessToken = cookieStore.get('access_token')?.value
    const idToken = cookieStore.get('id_token')?.value

    if (!accessToken && !idToken) {
      return {
        valid: false,
        error: 'No OIDC tokens found',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }
    }

    const token = accessToken || idToken

    // Check cache
    const cacheKey = `oidc:${token.substring(0, 20)}:${request.path || ''}`
    if (this.config.cacheEnabled) {
      const cached = this.getCachedSession(cacheKey)
      if (cached) {
        return {
          valid: true,
          session: {
            user: cached.user,
            permissions: cached.permissions,
            packageTier: cached.package_tier,
            expiresAt: cached.expires_at,
            platforms: cached.platforms
          },
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: true
          }
        }
      }
    }

    // Validate with backend API using Bearer token
    try {
      const response = await fetch(`${this.config.backendUrl}/api/auth/validate-session`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'User-Agent': request.userAgent || 'SessionValidator/1.0',
          'X-Forwarded-For': request.ipAddress || '',
        },
        body: JSON.stringify({
          app_type: request.appType || 'user',
          user_agent: request.userAgent,
          ip_address: request.ipAddress,
          path: request.path,
          method: request.method || 'GET'
        })
      })

      if (!response.ok) {
        const errorText = await response.text()
        return {
          valid: false,
          error: `OIDC validation failed: ${response.status} - ${errorText}`,
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: false
          }
        }
      }

      const result = await response.json()

      if (!result.valid || !result.user) {
        return {
          valid: false,
          error: result.error || 'OIDC session validation failed',
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: false
          }
        }
      }

      const user = result.user as UserProfile

      // Cache result
      if (this.config.cacheEnabled) {
        this.cacheSession(cacheKey, {
          user,
          permissions: result.permissions || user.permissions || [],
          package_tier: result.package_tier || user.packageTier || 'FREE',
          expires_at: result.expires_at || (Date.now() + 2 * 60 * 60 * 1000),
          cached_at: Date.now(),
          platforms: result.platforms || user.platforms
        })
      }

      return {
        valid: true,
        session: {
          user,
          permissions: result.permissions || user.permissions || [],
          packageTier: result.package_tier || user.packageTier || 'FREE',
          expiresAt: result.expires_at,
          sessionId: result.session_id,
          platforms: result.platforms || user.platforms
        },
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }

    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : 'OIDC validation failed',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false
        }
      }
    }
  }

  // ============================================================================
  // WALLET PERMISSION VALIDATION
  // ============================================================================

  private async validateWalletPermissions(walletAddress: string): Promise<string[]> {
    try {
      const { Pool } = require('pg')
      const pool = new Pool({
        connectionString: this.config.dbConnectionString,
        max: 5,
        idleTimeoutMillis: 30000,
        connectionTimeoutMillis: 2000,
        native: false
      })

      const client = await pool.connect()
      let permissions: string[] = []

      try {
        const result = await client.query(`
          SELECT permission, permission_type, is_active, expires_at, granted_at
          FROM wallet_permissions 
          WHERE wallet_address = $1 
            AND is_active = true 
            AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
          ORDER BY granted_at DESC
        `, [walletAddress.toLowerCase()])

        permissions = result.rows.map((row: any) => row.permission)
      } finally {
        client.release()
        await pool.end()
      }

      return permissions

    } catch (error) {
      console.error('Failed to validate wallet permissions:', error)
      return []
    }
  }

  // ============================================================================
  // PERMISSION CHECKING UTILITIES
  // ============================================================================

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
   * Check if user has specific permission
   */
  hasPermission(user: UserProfile, permission: string): boolean {
    const permissions = Array.isArray(user.permissions) ? user.permissions : []
    
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

  /**
   * Check if user has package tier access
   */
  hasPackageTier(user: UserProfile, tier: string): boolean {
    const tierHierarchy: Record<string, number> = {
      FREE: 1,
      BRONZE: 2,
      SILVER: 3,
      GOLD: 4,
      PLATINUM: 5,
      ENTERPRISE: 6
    }
    
    const userLevel = tierHierarchy[user.packageTier] || 0
    const requiredLevel = tierHierarchy[tier] || 1
    
    return userLevel >= requiredLevel
  }

  // ============================================================================
  // CACHING UTILITIES
  // ============================================================================

  private getCachedSession(key: string): SessionValidatorCache | null {
    if (!this.config.cacheEnabled) return null

    const cached = this.cache.get(key)
    
    if (!cached) {
      return null
    }
    
    // Check if expired
    if (Date.now() > cached.expires_at || 
        Date.now() > cached.cached_at + this.config.cacheTTL) {
      this.cache.delete(key)
      return null
    }
    
    return cached
  }

  private cacheSession(key: string, data: SessionValidatorCache): void {
    if (!this.config.cacheEnabled) return

    // Implement LRU eviction if cache is full
    if (this.cache.size >= this.config.maxCacheSize) {
      const firstKey = this.cache.keys().next().value
      if (firstKey) {
        this.cache.delete(firstKey)
      }
    }
    
    this.cache.set(key, data)
  }

  // ============================================================================
  // CACHE MANAGEMENT
  // ============================================================================

  /**
   * Clear all cached sessions
   */
  clearCache(): void {
    this.cache.clear()
    if (this.config.enableMetrics) {
      this.hitCount = 0
      this.missCount = 0
    }
  }

  /**
   * Reset metrics without clearing cache
   */
  resetMetrics(): void {
    if (this.config.enableMetrics) {
      this.hitCount = 0
      this.missCount = 0
    }
  }

  /**
   * Get cache and performance metrics
   */
  getMetrics(): CacheMetrics & { cacheSize: number; maxCacheSize: number } {
    const totalRequests = this.hitCount + this.missCount
    const hitRatio = totalRequests > 0 ? this.hitCount / totalRequests : 0
    
    return {
      hitCount: this.hitCount,
      missCount: this.missCount,
      totalRequests,
      hitRatio: Math.round(hitRatio * 100) / 100,
      cacheSize: this.cache.size,
      maxCacheSize: this.config.maxCacheSize
    }
  }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/**
 * Create session validator instance with configuration
 */
export function createSessionValidator(config?: SessionValidatorConfig): BaseSessionValidator {
  return BaseSessionValidator.getInstance(config)
}

/**
 * Quick session validation
 */
export async function validateSession(request: ValidationRequest): Promise<SessionValidationResponse> {
  const validator = BaseSessionValidator.getInstance()
  return validator.validateSession(request)
}

/**
 * Require valid session (throws if invalid)
 */
export async function requireSession(request: ValidationRequest): Promise<UserProfile> {
  const result = await validateSession(request)
  
  if (!result.valid || !result.session?.user) {
    throw new Error(result.error || 'Session validation failed')
  }
  
  return result.session.user
}

export default BaseSessionValidator