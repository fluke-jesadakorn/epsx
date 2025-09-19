// ============================================================================
// SHARED PERMISSION CORE TYPES
// ============================================================================
// Core permission interfaces and types used across EPSX applications

import { PERMISSION_SOURCES, PLATFORMS } from '../constants'

// Re-export EnhancedUserClaims from claims to fix import issues
export type { EnhancedUserClaims } from './claims'

// ============================================================================
// BASIC PERMISSION TYPES
// ============================================================================

export interface Permission {
  platform: string
  resource: string
  action: string
}

export interface ParsedPermission {
  platform: string
  resource: string
  action: string
  full: string // Original permission string
}

export type PermissionSource = typeof PERMISSION_SOURCES[keyof typeof PERMISSION_SOURCES]
export type Platform = typeof PLATFORMS[keyof typeof PLATFORMS]

// ============================================================================
// GRANULAR PERMISSION TYPES
// ============================================================================

export interface GranularPermissionClaim {
  expires_at?: number // Unix timestamp
  source: PermissionSource
  granted_at: number // Unix timestamp
  granted_by?: string // Admin user ID who granted the permission
}

export interface GranularPermissionSet {
  permissions: Record<string, GranularPermissionClaim>
  hash: string // SHA-256 hash for instant revocation validation
  version: number // Version for cache invalidation
  created_at: number // Unix timestamp
}

// ============================================================================
// EMBEDDED TIMESTAMP PERMISSION TYPES
// ============================================================================

export interface TimestampedPermission {
  permission: string
  basePermission: string
  expiresAt?: number // Unix timestamp
  isExpired: boolean
  expiresIn?: string // Human readable (e.g., "2 hours")
  timeRemaining?: number // Milliseconds remaining
}

export interface PermissionExpiryDetails {
  permission: string
  base_permission: string
  claim: GranularPermissionClaim
  is_expired: boolean
  expires_in_ms?: number
  expires_in_human?: string
  is_permanent: boolean
}

export interface PermissionExpiryInfo {
  hasExpiringPermissions: boolean
  expiringSoon: TimestampedPermission[] // Expiring within 24 hours
  expired: TimestampedPermission[]
  nextExpiry?: TimestampedPermission
}

// ============================================================================
// PERMISSION HEALTH TYPES
// ============================================================================

export interface PermissionHealthInfo {
  total_permissions: number
  active_permissions: number
  expired_permissions: number
  expiring_soon_permissions: number // Within 24 hours
  next_expiry?: number
  time_until_next_expiry?: number // Milliseconds
  health_score: number // 0-100 based on expiry status
}

export interface UserPermissionSummary {
  user_id: string
  total_permissions: number
  permanent_permissions: number
  temporary_permissions: number
  expired_permissions: number
  expiring_soon_permissions: number
  permission_hash: string
  permission_version: number
  last_updated: number
}

// ============================================================================
// CACHE AND VALIDATION TYPES
// ============================================================================

export interface PermissionCacheEntry {
  user_id: string
  permission_hash: string
  permission_version: number
  permissions: Record<string, GranularPermissionClaim>
  cached_at: number
  expires_at?: number
  is_revoked: boolean
}

export type HashValidationResult = 
  | { status: 'Valid' }
  | { status: 'Revoked' }
  | { status: 'Updated'; new_hash: string }
  | { status: 'NotFound' }

// TokenValidationResult with EnhancedUserClaims import 
export interface TokenValidationResult {
  claims: any // EnhancedUserClaims - imported from claims.ts to avoid circular dependency
  valid_permissions: string[] // Filtered non-expired permissions
  updated_token?: string // New token if permissions were cleaned
}

// ============================================================================
// LEGACY COMPATIBILITY TYPES
// ============================================================================

export interface LegacyPermissionMapping {
  legacy_permission: string
  granular_permission: string
  default_source: PermissionSource
  default_expiry_hours?: number
}

export interface MigrationStatus {
  total_users: number
  migrated_users: number
  failed_migrations: number
  legacy_permissions_remaining: number
}

// EnhancedUserClaims is defined in claims.ts