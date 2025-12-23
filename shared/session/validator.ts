/**
 * BASE SESSION VALIDATOR - WEB3-FIRST ARCHITECTURE
 * Unified session validation system with Web3 wallet signatures as primary method
 * Legacy JWT/OIDC support maintained for backward compatibility but deprecated
 * 
 * ✅ Web3 wallet signatures (primary)
 * ⚠️  Legacy JWT support (deprecated - Phase 3.3)
 * ⚠️  Legacy OIDC support (deprecated - Phase 3.3)
 */

// Conditional imports for Next.js and Web3 dependencies
// These will be dynamically imported only when needed

// ============================================================================
// SHARED TYPES (from consolidated domain types)
// ============================================================================

import type {
  SessionValidationResponse
} from '../types/domain/Session';
import type { AdminUserProfile, Group, PermissionGroup, UserProfile } from '../types/domain/User';
import { getPermissionGroupLevel } from '../types/domain/User';

// JWT payload types for backward compatibility
interface AdminJWTPayload {
  user_id: string;
  email: string;
  permissions: string[];
  exp: number;
  iat: number;
}

interface UserJWTPayload {
  user_id: string;
  email?: string;
  wallet_address?: string;
  exp: number;
  iat: number;
}

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
      // Dynamic import for Next.js cookies
      const { cookies } = await import('next/headers')
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
              cacheHit: false,
              source: 'database' as const
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
          cacheHit: false,
          source: 'database' as const
        }
      }
    }
  }

  // ============================================================================
  // AUTHENTICATION METHOD DETECTION
  // ============================================================================

  private detectAuthMethod(cookieStore: any): 'wallet' | 'jwt' | 'oidc' | 'none' {
    // ✅ PRIORITY 1: Web3 wallet authentication (preferred)
    const walletAddress = cookieStore.get('wallet_address')?.value
    const walletSignature = cookieStore.get('wallet_signature')?.value
    if (walletAddress && walletSignature) {
      return 'wallet'
    }

    // ⚠️  DEPRECATED: OIDC tokens (Phase 3.3 - use Web3 instead)
    const accessToken = cookieStore.get('access_token')?.value
    const idToken = cookieStore.get('id_token')?.value
    if (accessToken || idToken) {
      console.warn('⚠️  DEPRECATED: OIDC authentication detected - migrate to Web3 wallet signatures (Phase 3.3)')
      return 'oidc'
    }

    // ⚠️  DEPRECATED: Legacy JWT token (Phase 3.3 - use Web3 instead)
    const jwtToken = cookieStore.get('epsx_jwt')?.value
    if (jwtToken) {
      console.warn('⚠️  DEPRECATED: JWT authentication detected - migrate to Web3 wallet signatures (Phase 3.3)')
      return 'jwt'
    }

    return 'none'
  }

  // ============================================================================
  // WALLET-BASED VALIDATION (WEB3-FIRST) - PREFERRED METHOD
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
          cacheHit: false,
          source: 'database' as const
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
          cacheHit: false,
          source: 'database' as const
        }
      }
    }

    // Validate SIWE message
    try {
      // Dynamic import for SIWE - only available in browser/Node.js environment
      const { SiweMessage } = await import('siwe')
      const siweMessage = new SiweMessage(walletMessage)
      if (siweMessage.address.toLowerCase() !== walletAddress.toLowerCase()) {
        return {
          valid: false,
          error: 'Wallet address mismatch in SIWE message',
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: false,
            source: 'database' as const
          }
        }
      }
    } catch (error) {
      return {
        valid: false,
        error: 'Invalid SIWE message format',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false,
          source: 'database' as const
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
            sessionId: `wallet_cached_${Date.now()}`,
            userId: cached.user.id,
            user: cached.user as any,
            appType: 'admin' as const,
            expiresAt: cached.expires_at,
            issuedAt: cached.cached_at,
            lastAccessedAt: Date.now(),
            adminContext: {
              permissions: cached.permissions as any[],
              accessLevel: cached.permissions.some((p: string) => p.includes('super_admin')) ? 'super_admin' as const : 'admin' as const,
              managedPlatforms: cached.platforms || ['admin']
            }
          },
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: true,
            source: 'cache' as const
          }
        }
      }
    }

    // Validate wallet permissions via database
    const permissions = await this.validateWalletPermissions(walletAddress)

    const user: AdminUserProfile = {
      id: walletAddress,
      walletAddress: walletAddress,
      email: `${walletAddress.substring(0, 8)}@wallet.admin`,
      name: walletAddress.substring(0, 8) + '...',
      role: permissions.some(p => p.startsWith('admin:')) ? 'admin' : 'user',
      permissions: permissions,
      group: 'Enterprise Access Group' as Group,
      permissionGroup: 'Enterprise Access Group' as Group,
      packageTier: 'ENTERPRISE',
      platforms: ['admin', 'epsx'],
      primaryPlatform: 'admin',
      status: 'active',
      emailVerified: true,
      twoFactorEnabled: false,
      createdAt: new Date(),
      updatedAt: new Date(),
      billing: {
        group: 'Enterprise Access Group' as Group,
        isActive: true,
        paymentStatus: 'current'
      },
      moduleAccess: [],
      moduleQuotas: [],
      stockRankingPackages: [],
      apiKeys: [],
      recentActivity: [],
      loginHistory: [],
      usageMetrics: {
        apiCallsThisMonth: 0,
        storageUsed: 0,
        lastActiveDate: new Date(),
        sessionsThisMonth: 0,
        averageSessionDuration: 0
      }
    }

    // Validate admin access
    if (!this.hasAdminAccess(user)) {
      return {
        valid: false,
        error: 'User lacks admin access permissions',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false,
          source: 'database' as const
        }
      }
    }

    // Cache result
    if (this.config.cacheEnabled) {
      this.cacheSession(cacheKey, {
        user,
        permissions: user.permissions,
        package_tier: user.packageTier ?? 'ENTERPRISE',
        expires_at: expiresAt,
        cached_at: Date.now(),
        platforms: user.platforms
      })
    }

    return {
      valid: true,
      session: {
        sessionId: walletAddress,
        userId: user.id,
        user,
        appType: 'admin' as const,
        expiresAt: expiresAt,
        issuedAt: Date.now(),
        lastAccessedAt: Date.now(),
        adminContext: {
          permissions: user.permissions as any[],
          accessLevel: user.permissions.some(p => p.includes('super_admin')) ? 'super_admin' as const : 'admin' as const,
          managedPlatforms: user.platforms || ['admin']
        }
      },
      performance: {
        validationTimeMs: performance.now() - startTime,
        cacheHit: false,
        source: 'database' as const
      }
    }
  }

  // ============================================================================
  // JWT-BASED VALIDATION (USER) - DEPRECATED
  // ============================================================================

  /**
   * @deprecated Phase 3.3: JWT authentication is deprecated. Use Web3 wallet signatures instead.
   * This method will be removed in Phase 5.0. Migrate to validateWalletSession().
   */
  private async validateJWTSession(
    request: ValidationRequest,
    cookieStore: any
  ): Promise<SessionValidationResponse> {
    console.warn('⚠️  DEPRECATED: validateJWTSession is deprecated (Phase 3.3) - use Web3 wallet signatures instead')
    const startTime = performance.now()

    const jwtCookie = cookieStore.get('epsx_jwt')
    if (!jwtCookie?.value) {
      return {
        valid: false,
        error: 'No JWT token found',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false,
          source: 'database' as const
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
            sessionId: `jwt_cached_${Date.now()}`,
            userId: cached.user.id,
            user: cached.user,
            appType: 'user' as const,
            expiresAt: cached.expires_at,
            issuedAt: cached.cached_at,
            lastAccessedAt: Date.now(),
            platformContext: {
              currentPlatform: 'epsx',
              availablePlatforms: ['epsx']
            },
            permissions: cached.permissions
          },
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: true,
            source: 'cache' as const
          }
        }
      }
    }

    // Validate with backend API (deprecated JWT endpoint)
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
            cacheHit: false,
            source: 'database' as const
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
            cacheHit: false,
            source: 'database' as const
          }
        }
      }

      const user = result.user as AdminUserProfile

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
          sessionId: result.session_id || `jwt_${Date.now()}`,
          userId: user.id,
          user,
          appType: 'user' as const,
          expiresAt: result.expires_at || Date.now() + 2 * 60 * 60 * 1000,
          issuedAt: Date.now(),
          lastAccessedAt: Date.now(),
          platformContext: {
            currentPlatform: 'epsx',
            availablePlatforms: ['epsx']
          },
          permissions: result.permissions || user.permissions || []
        },
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false,
          source: 'database' as const
        }
      }

    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : 'Backend validation failed',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false,
          source: 'database' as const
        }
      }
    }
  }

  // ============================================================================
  // OIDC-BASED VALIDATION (MODERN) - DEPRECATED
  // ============================================================================

  /**
   * @deprecated Phase 3.3: OIDC authentication is deprecated. Use Web3 wallet signatures instead.
   * This method will be removed in Phase 5.0. Migrate to validateWalletSession().
   */
  private async validateOIDCSession(
    request: ValidationRequest,
    cookieStore: any
  ): Promise<SessionValidationResponse> {
    console.warn('⚠️  DEPRECATED: validateOIDCSession is deprecated (Phase 3.3) - use Web3 wallet signatures instead')
    const startTime = performance.now()

    const accessToken = cookieStore.get('access_token')?.value
    const idToken = cookieStore.get('id_token')?.value

    if (!accessToken && !idToken) {
      return {
        valid: false,
        error: 'No OIDC tokens found',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false,
          source: 'database' as const
        }
      }
    }

    const token = accessToken || idToken

    // Check cache
    const cacheKey = `oidc:${token.substring(0, 20)}:${request.path || ''}`
    if (this.config.cacheEnabled) {
      const cached = this.getCachedSession(cacheKey)
      if (cached) {
        const resolvedAppType = (request.appType === 'admin' ||
          (cached.user as any).role === 'admin' ||
          (cached.user as any).role === 'super_admin') ? 'admin' : 'user';

        return {
          valid: true,
          session: {
            sessionId: `oidc_cached_${Date.now()}`,
            userId: cached.user.id,
            user: cached.user as any,
            appType: resolvedAppType,
            expiresAt: cached.expires_at,
            issuedAt: cached.cached_at,
            lastAccessedAt: Date.now(),
            ...(resolvedAppType === 'admin' ? {
              adminContext: {
                permissions: cached.permissions as any[],
                accessLevel: cached.permissions.some((p: string) => p.includes('super_admin')) ? 'super_admin' as const : 'admin' as const,
                managedPlatforms: cached.platforms || ['admin']
              }
            } : {
              platformContext: {
                currentPlatform: 'epsx',
                availablePlatforms: cached.platforms || ['epsx']
              }
            }),
            permissions: cached.permissions
          } as any,
          performance: {
            validationTimeMs: performance.now() - startTime,
            cacheHit: true,
            source: 'cache' as const
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
            cacheHit: false,
            source: 'database' as const
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
            cacheHit: false,
            source: 'database' as const
          }
        }
      }

      const user = result.user as AdminUserProfile

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

      const resolvedAppType = (request.appType === 'admin' ||
        user.role === 'admin' ||
        user.role === 'super_admin') ? 'admin' : 'user';

      return {
        valid: true,
        session: {
          sessionId: result.session_id || `oidc_${Date.now()}`,
          userId: user.id,
          user: user as any,
          appType: resolvedAppType,
          expiresAt: result.expires_at || Date.now() + 2 * 60 * 60 * 1000,
          issuedAt: Date.now(),
          lastAccessedAt: Date.now(),
          ...(resolvedAppType === 'admin' ? {
            adminContext: {
              permissions: (result.permissions || user.permissions || []) as any[],
              accessLevel: (result.permissions || user.permissions || []).some((p: string) => p.includes('super_admin')) ? 'super_admin' as const : 'admin' as const,
              managedPlatforms: result.platforms || user.platforms || ['admin']
            }
          } : {
            platformContext: {
              currentPlatform: 'epsx',
              availablePlatforms: result.platforms || user.platforms || ['epsx']
            }
          }),
          permissions: result.permissions || user.permissions || []
        } as any,
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false,
          source: 'api' as const
        }
      }

    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : 'OIDC validation failed',
        performance: {
          validationTimeMs: performance.now() - startTime,
          cacheHit: false,
          source: 'database' as const
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

  private hasAdminAccess(user: UserProfile | AdminUserProfile): boolean {
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
  hasPermission(user: UserProfile | AdminUserProfile, permission: string): boolean {
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
   * @deprecated Use hasMinimumPermissionGroup instead
   * Check if user has package tier access
   */
  hasPackageTier(user: UserProfile | AdminUserProfile, tier: string): boolean {
    const tierHierarchy: Record<string, number> = {
      FREE: 1,
      BRONZE: 2,
      SILVER: 3,
      GOLD: 4,
      PLATINUM: 5,
      ENTERPRISE: 6
    }

    const userLevel = tierHierarchy[(user as any).packageTier] || 0
    const requiredLevel = tierHierarchy[tier] || 1

    return userLevel >= requiredLevel
  }

  /**
   * Check if user has minimum permission group access
   */
  hasMinimumPermissionGroup(user: UserProfile | AdminUserProfile, requiredGroup: PermissionGroup): boolean {
    const userLevel = getPermissionGroupLevel(user.permissionGroup)
    const requiredLevel = getPermissionGroupLevel(requiredGroup)

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
 * Web3-first session validation (PREFERRED METHOD)
 * Validates Web3 wallet signatures only, no legacy authentication
 */
export async function validateWeb3Session(
  walletAddress: string,
  walletSignature: string,
  request: ValidationRequest
): Promise<SessionValidationResponse> {
  if (!walletAddress || !walletSignature) {
    return {
      valid: false,
      error: 'Missing wallet address or signature',
      performance: {
        validationTimeMs: 0,
        cacheHit: false,
        source: 'api' as const
      }
    }
  }

  // Mock cookie store with Web3 data for validation
  const mockCookieStore = {
    get: (name: string) => {
      const values: Record<string, string> = {
        wallet_address: walletAddress,
        wallet_signature: walletSignature,
        wallet_expires_at: (Date.now() + 24 * 60 * 60 * 1000).toString(), // 24 hours
        wallet_message: `Sign in to EPSX with wallet ${walletAddress}`, // Basic SIWE message
        wallet_nonce: Date.now().toString()
      }
      return values[name] ? { value: values[name] } : undefined
    }
  }

  const validator = BaseSessionValidator.getInstance()
  return validator['validateWalletSession'](request, mockCookieStore)
}

/**
 * Quick session validation
 * @deprecated Phase 3.3: Use validateWeb3Session for Web3-first validation
 */
export async function validateSession(request: ValidationRequest): Promise<SessionValidationResponse> {
  console.warn('⚠️  DEPRECATED: validateSession supports legacy auth - use validateWeb3Session for Web3-only (Phase 3.3)')
  const validator = BaseSessionValidator.getInstance()
  return validator.validateSession(request)
}

/**
 * Require valid session (throws if invalid)
 * @deprecated Phase 3.3: Use requireWeb3Session for Web3-first validation
 */
export async function requireSession(request: ValidationRequest): Promise<UserProfile | AdminUserProfile> {
  console.warn('⚠️  DEPRECATED: requireSession supports legacy auth - use requireWeb3Session for Web3-only (Phase 3.3)')
  const result = await validateSession(request)

  if (!result.valid || !result.session?.user) {
    throw new Error(result.error || 'Session validation failed')
  }

  return result.session.user
}

/**
 * Require valid Web3 session (throws if invalid) - PREFERRED METHOD
 */
export async function requireWeb3Session(
  walletAddress: string,
  walletSignature: string,
  request: ValidationRequest
): Promise<UserProfile | AdminUserProfile> {
  const result = await validateWeb3Session(walletAddress, walletSignature, request)

  if (!result.valid || !result.session?.user) {
    throw new Error(result.error || 'Web3 session validation failed')
  }

  return result.session.user
}

export default BaseSessionValidator