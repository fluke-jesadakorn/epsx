// ============================================================================
// SHARED PERMISSION CLAIMS TYPES
// ============================================================================
// User claims and JWT-related permission types

import { GranularPermissionClaim, PermissionSource } from './core'

// ============================================================================
// USER CLAIMS TYPES
// ============================================================================

export interface EnhancedUserClaims {
  sub: string // User ID
  email?: string
  name?: string
  role?: string
  firebase_uid?: string
  
  // Granular permission system
  permissions: Record<string, GranularPermissionClaim>
  permission_hash: string // For instant revocation validation
  permission_version: number // For cache synchronization
  
  // Standard JWT claims
  iat: number
  exp: number
  aud: string
  iss: string
  
  // Legacy compatibility
  package_tier?: string
  is_active?: boolean
  platforms?: string[]
  primary_platform?: string
}

export interface UserClaims {
  firebase_uid: string
  email: string
  permissions: string[]  // Structured permissions: ["platform:resource:action", ...]
  package_tier: string
  display_name?: string
  name?: string
  avatar_url?: string
  is_active: boolean
  last_login_at?: string
  platforms?: string[]
  primary_platform?: string
  platform_context?: string
}

// ============================================================================
// TOKEN TYPES
// ============================================================================

export interface RefreshTokenClaims {
  sub: string
  type: 'refresh'
  iat: number
  exp: number
  jti: string // Token ID for revocation
}

export interface AccessTokenClaims extends EnhancedUserClaims {
  type: 'access'
  scope?: string[]
}

export interface IDTokenClaims {
  sub: string
  email?: string
  email_verified?: boolean
  name?: string
  picture?: string
  aud: string
  iss: string
  iat: number
  exp: number
  auth_time?: number
  nonce?: string
}

// ============================================================================
// PERMISSION CONTEXT TYPES
// ============================================================================

export interface PermissionContext {
  userClaims: EnhancedUserClaims | null
  permissionSet: Record<string, GranularPermissionClaim> | null
  loading: boolean
  error: any
  lastRefresh?: number
}

export interface AdminPermissionContext extends PermissionContext {
  isAdmin: boolean
  canManagePermissions: boolean
  canManageUsers: boolean
  canViewAuditLogs: boolean
}

// ============================================================================
// HOOK RESULT TYPES
// ============================================================================

export interface UsePermissionHookResult {
  hasPermission: (permission: string) => boolean
  hasAnyPermission: (permissions: string[]) => boolean
  hasAllPermissions: (permissions: string[]) => boolean
  getPermissionExpiry: (permission: string) => PermissionExpiryDetails | null
  getPermissionHealth: () => PermissionHealthInfo | null
  isPermissionExpiring: (permission: string, withinHours?: number) => boolean
  refreshPermissions: () => Promise<void>
  loading: boolean
  error: any
}

export interface AdminPermissionHookResult extends UsePermissionHookResult {
  // User permission queries
  getUserPermissions: (userId: string) => Promise<any> // PermissionStatusResponse from api.ts
  getAllUsersWithPermissions: (filters?: any) => Promise<any[]> // UserPermissionOverview from api.ts
  
  // Permission management
  grantPermission: (request: any) => Promise<void> // GrantPermissionRequest from api.ts
  revokePermission: (request: any) => Promise<void> // RevokePermissionRequest from api.ts
  bulkGrantPermissions: (request: any) => Promise<any> // BulkPermissionRequest/Result from api.ts
  bulkRevokePermissions: (request: any) => Promise<any> // BulkOperationResult from api.ts
  extendPermission: (request: any) => Promise<void> // ExtendPermissionRequest from api.ts
  
  // Templates and monitoring
  createPermissionTemplate: (template: any) => Promise<any> // PermissionTemplate from audit.ts
  applyPermissionTemplate: (templateId: string, userIds: string[]) => Promise<any> // BulkOperationResult from api.ts
  getDashboard: () => Promise<any> // AdminPermissionDashboard defined in api.ts
  getPermissionAudit: (userId?: string, limit?: number) => Promise<any[]> // PermissionAuditEntry defined in audit.ts
}

// Note: All permission operation types (GrantPermissionRequest, BulkOperationResult, etc.)
// are defined in api.ts and audit.ts to avoid circular dependencies and duplicates

// Re-import from core for consistency
import { PermissionHealthInfo, PermissionExpiryDetails } from './core'