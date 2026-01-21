/**
 * CANONICAL SESSION DOMAIN TYPES
 * Single source of truth for all session-related interfaces across EPSX
 * Consolidates AdminSessionValidator and UserSessionValidator types
 */

import type { UserProfile, AdminUserProfile, PackageTier } from './User'
import type { EPSXPermission } from './Permission'

// ============================================================================
// CORE SESSION TYPES
// ============================================================================

/**
 * Application context for session validation
 */
export type SessionAppType = 'admin' | 'user'

/**
 * Session validation request context
 */
export interface SessionValidationRequest {
  app_type: SessionAppType
  user_agent?: string
  ip_address?: string
  path?: string
  method?: string
  additional_context?: Record<string, any>
}

/**
 * Base session data shared across all session types
 */
export interface BaseSessionData {
  sessionId: string
  userId: string
  expiresAt: number
  issuedAt: number
  lastAccessedAt: number
  ipAddress?: string
  userAgent?: string
  
  // Backward compatibility with legacy validators
  permissions?: string[]
  packageTier?: string
  platforms?: string[]
}

/**
 * User session data for frontend application
 */
export interface UserSessionData extends BaseSessionData {
  user: UserProfile
  appType: 'user'
  platformContext: {
    currentPlatform: string
    availablePlatforms: string[]
  }
}

/**
 * Admin session data for admin frontend application
 */
export interface AdminSessionData extends BaseSessionData {
  user: AdminUserProfile
  appType: 'admin'
  adminContext: {
    permissions: EPSXPermission[]
    accessLevel: 'admin' | 'super_admin'
    managedPlatforms: string[]
  }
}

/**
 * Wallet-based session data for Web3 authentication
 */
export interface WalletSessionData extends BaseSessionData {
  walletAddress: string
  chainId?: number
  signatureMessage: string
  signatureNonce: string
  signatureExpiresAt: number
  providerType: 'metamask' | 'walletconnect' | 'coinbase' | 'other'
}

/**
 * Combined session data type
 */
export type SessionData = UserSessionData | AdminSessionData

// ============================================================================
// SESSION VALIDATION TYPES
// ============================================================================

/**
 * Session validation response with performance metrics
 */
export interface SessionValidationResponse {
  valid: boolean
  session?: SessionData
  user?: UserProfile | AdminUserProfile
  permissions?: EPSXPermission[]
  packageTier?: PackageTier
  platforms?: string[]
  expiresAt?: number
  sessionId?: string
  error?: string
  performance: {
    validationTimeMs: number
    cacheHit: boolean
    source: 'cache' | 'database' | 'api'
  }
}

/**
 * Session cache entry for performance optimization
 */
export interface SessionCacheEntry {
  sessionId: string
  userId: string
  user: UserProfile | AdminUserProfile
  permissions: EPSXPermission[]
  packageTier: PackageTier
  platforms: string[]
  expiresAt: number
  cachedAt: number
  appType: SessionAppType
}

/**
 * Session refresh request
 */
export interface SessionRefreshRequest {
  sessionId: string
  refreshToken?: string
  extendBy?: number // Milliseconds to extend session
}

/**
 * Session refresh response
 */
export interface SessionRefreshResponse {
  success: boolean
  session?: SessionData
  newToken?: string
  expiresAt?: number
  error?: string
}

// ============================================================================
// SESSION MANAGEMENT TYPES
// ============================================================================

/**
 * Session creation request
 */
export interface SessionCreateRequest {
  userId: string
  appType: SessionAppType
  duration?: number // Session duration in milliseconds
  ipAddress?: string
  userAgent?: string
  walletData?: {
    address: string
    signature: string
    message: string
    nonce: string
  }
}

/**
 * Session creation response
 */
export interface SessionCreateResponse {
  success: boolean
  session?: SessionData
  token?: string
  error?: string
}

/**
 * Session termination request
 */
export interface SessionTerminateRequest {
  sessionId: string
  userId?: string
  reason?: 'logout' | 'expired' | 'revoked' | 'security'
}

/**
 * Session termination response
 */
export interface SessionTerminateResponse {
  success: boolean
  error?: string
}

/**
 * Active sessions list for a user
 */
export interface UserActiveSessions {
  userId: string
  sessions: {
    sessionId: string
    appType: SessionAppType
    deviceInfo: {
      userAgent: string
      ipAddress: string
      location?: string
    }
    createdAt: number
    lastAccessedAt: number
    expiresAt: number
    isCurrent: boolean
  }[]
  totalCount: number
}

// ============================================================================
// SESSION SECURITY TYPES
// ============================================================================

/**
 * Session security event types
 */
export type SessionSecurityEventType = 
  | 'suspicious_login'
  | 'multiple_devices'
  | 'location_change'
  | 'unusual_activity'
  | 'session_hijack'
  | 'expired_session'

/**
 * Session security event
 */
export interface SessionSecurityEvent {
  id: string
  sessionId: string
  userId: string
  eventType: SessionSecurityEventType
  severity: 'low' | 'medium' | 'high' | 'critical'
  details: Record<string, any>
  detectedAt: number
  resolved: boolean
  resolvedAt?: number
  actions: string[]
}

/**
 * Session security policy
 */
export interface SessionSecurityPolicy {
  maxConcurrentSessions: number
  sessionTimeout: number
  idleTimeout: number
  requireNewLocationConfirmation: boolean
  allowMultipleDevices: boolean
  forceLogoutOnSuspicion: boolean
  notifyOnNewDevice: boolean
}

// ============================================================================
// WALLET SESSION TYPES
// ============================================================================

/**
 * Wallet authentication challenge
 */
export interface WalletAuthChallenge {
  nonce: string
  message: string
  expiresAt: number
  chainId?: number
}

/**
 * Wallet authentication verification
 */
export interface WalletAuthVerification {
  walletAddress: string
  signature: string
  message: string
  nonce: string
  chainId?: number
}

/**
 * Wallet session status
 */
export interface WalletSessionStatus {
  isConnected: boolean
  walletAddress?: string
  chainId?: number
  provider?: string
  permissions: EPSXPermission[]
  sessionExpiresAt?: number
}

// ============================================================================
// SESSION ANALYTICS TYPES
// ============================================================================

/**
 * Session analytics metrics
 */
export interface SessionAnalytics {
  totalSessions: number
  activeSessions: number
  sessionsToday: number
  averageSessionDuration: number
  uniqueUsers: number
  peakConcurrentSessions: number
  sessionsByAppType: Record<SessionAppType, number>
  sessionsByDevice: Record<string, number>
  sessionsByLocation: Record<string, number>
  computedAt: number
}

/**
 * User session patterns
 */
export interface UserSessionPattern {
  userId: string
  averageSessionDuration: number
  sessionsPerDay: number
  preferredDevices: string[]
  commonLocations: string[]
  peakUsageHours: number[]
  lastAnalyzed: number
}

// ============================================================================
// SESSION VALIDATOR CONFIGURATION
// ============================================================================

/**
 * Session validator configuration
 */
export interface SessionValidatorConfig {
  cache: {
    enabled: boolean
    ttl: number // Time to live in milliseconds
    maxSize: number
    cleanupInterval: number
  }
  validation: {
    strictMode: boolean
    allowExpiredGracePeriod: number
    validateUserAgent: boolean
    validateIpAddress: boolean
  }
  performance: {
    enableMetrics: boolean
    slowQueryThreshold: number
    enableTracing: boolean
  }
}

/**
 * Session validator statistics
 */
export interface SessionValidatorStats {
  totalValidations: number
  cacheHits: number
  cacheMisses: number
  averageValidationTime: number
  errorRate: number
  slowQueries: number
  uptime: number
  lastReset: number
}

// ============================================================================
// TYPE GUARDS & HELPERS
// ============================================================================

/**
 * Check if session data is for user application
 */
export function isUserSession(session: SessionData): session is UserSessionData {
  return session.appType === 'user'
}

/**
 * Check if session data is for admin application
 */
export function isAdminSession(session: SessionData): session is AdminSessionData {
  return session.appType === 'admin'
}

/**
 * Check if session is expired
 */
export function isSessionExpired(session: BaseSessionData): boolean {
  return Date.now() > session.expiresAt
}

/**
 * Check if session is about to expire (within 5 minutes)
 */
export function isSessionExpiringSoon(session: BaseSessionData, thresholdMs = 5 * 60 * 1000): boolean {
  return Date.now() > (session.expiresAt - thresholdMs)
}

/**
 * Get session time remaining in milliseconds
 */
export function getSessionTimeRemaining(session: BaseSessionData): number {
  return Math.max(0, session.expiresAt - Date.now())
}

/**
 * Calculate session duration
 */
export function getSessionDuration(session: BaseSessionData): number {
  return session.lastAccessedAt - session.issuedAt
}

/**
 * Validate session request context
 */
export function validateSessionRequest(request: SessionValidationRequest): boolean {
  return !!(request.app_type && ['admin', 'user'].includes(request.app_type))
}

// ============================================================================
// LEGACY COMPATIBILITY ALIASES
// ============================================================================

/** @deprecated Use UserSessionData instead */
export type UserSession = UserSessionData

/** @deprecated Use AdminSessionData instead */
export type AdminSession = AdminSessionData

/** @deprecated Use SessionValidationResponse instead */
export type SessionResult = SessionValidationResponse

/** @deprecated Use SessionCacheEntry instead */
export type SessionCache = SessionCacheEntry

// Re-export domain types for validator use
export type { UserProfile, AdminUserProfile, PackageTier } from './User'
export type { EPSXPermission } from './Permission'

// JWT payload types for backward compatibility
export interface AdminJWTPayload {
  user_id: string;
  email: string;
  permissions: string[];
  exp: number;
  iat: number;
}

export interface UserJWTPayload {
  user_id: string;
  email?: string;
  wallet_address?: string;
  exp: number;
  iat: number;
}