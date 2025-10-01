/**
 * CANONICAL PERMISSION DOMAIN TYPES
 * Single source of truth for all permission-related interfaces across EPSX
 * Consolidates shared permission system with domain-specific business logic
 */

// Re-export core permission types from shared system
export type {
  Permission,
  ParsedPermission,
  PermissionSource,
  Platform,
  GranularPermissionClaim,
  GranularPermissionSet,
  TimestampedPermission,
  PermissionExpiryDetails,
  PermissionExpiryInfo,
  PermissionHealthInfo,
  UserPermissionSummary,
  PermissionCacheEntry,
  HashValidationResult,
  TokenValidationResult,
  LegacyPermissionMapping,
  MigrationStatus
} from '../../permissions/types/core'

export type {
  EnhancedUserClaims,
  PermissionClaims,
  AdminPermissionClaims,
  CrossPlatformClaims,
  EmbeddedPermissionClaims
} from '../../permissions/types/claims'

export {
  PermissionAPIRequest,
  PermissionAPIResponse,
  PermissionValidationRequest,
  PermissionValidationResponse,
  PermissionUpdateRequest,
  PermissionUpdateResponse,
  PermissionListRequest,
  PermissionListResponse,
  PermissionSyncRequest,
  PermissionSyncResponse,
  PermissionRevokeRequest,
  PermissionRevokeResponse,
  BulkPermissionRequest,
  BulkPermissionResponse,
  PermissionImportRequest,
  PermissionImportResponse,
  PermissionExportRequest,
  PermissionExportResponse
} from '../../permissions/types/api'

export {
  PermissionError,
  ValidationError,
  ExpiryError,
  CacheError,
  SyncError,
  ImportError,
  ExportError,
  PermissionErrorContext
} from '../../permissions/types/errors'

export {
  PermissionAuditLog,
  PermissionChangeEvent,
  PermissionRevocationEvent,
  PermissionExpiryEvent,
  PermissionBulkEvent,
  PermissionImportEvent,
  PermissionSystemEvent
} from '../../permissions/types/audit'

// ============================================================================
// BUSINESS DOMAIN PERMISSION TYPES
// ============================================================================

/**
 * Standard EPSX permission format: "platform:resource:action"
 * Examples: "epsx:rankings:view", "admin:users:manage", "epsx-pay:transactions:create"
 */
export type EPSXPermission = string

/**
 * Permission templates for quick assignment
 */
export interface PermissionTemplate {
  id: string
  name: string
  description: string
  permissions: EPSXPermission[]
  category: PermissionCategory
  isSystemDefault: boolean
  createdAt: Date
  updatedAt: Date
}

/**
 * Permission categories for organization
 */
export type PermissionCategory = 
  | 'analytics'
  | 'trading'
  | 'admin'
  | 'billing'
  | 'api'
  | 'web3'
  | 'notifications'
  | 'developer'

/**
 * Permission scope levels
 */
export type PermissionScope = 'read' | 'write' | 'admin' | 'owner'

/**
 * Context-aware permission check
 */
export interface PermissionCheck {
  permission: EPSXPermission
  platform?: string
  scope?: PermissionScope
  context?: Record<string, any>
  requiresSubscription?: boolean
  minimumTier?: import('./User').PackageTier
}

/**
 * Permission validation result with business context
 */
export interface PermissionValidation {
  hasPermission: boolean
  permission: EPSXPermission
  reason: 'granted' | 'denied' | 'expired' | 'insufficient_tier' | 'not_found'
  expiresAt?: number
  requiredTier?: import('./User').PackageTier
  upgradeUrl?: string
  grantedBy?: string
  grantedAt?: number
}

/**
 * Role-based permission set
 */
export interface PermissionRole {
  id: string
  name: string
  description: string
  permissions: EPSXPermission[]
  inherits?: string[] // Other role IDs this role inherits from
  isSystemRole: boolean
  platforms: string[]
  createdAt: Date
  updatedAt: Date
}

/**
 * User's effective permissions after role resolution
 */
export interface EffectivePermissions {
  userId: string
  permissions: EPSXPermission[]
  roles: string[]
  directPermissions: EPSXPermission[]
  inheritedPermissions: EPSXPermission[]
  temporaryPermissions: TimestampedPermission[]
  deniedPermissions: EPSXPermission[]
  computedAt: number
  expiresAt?: number
}

/**
 * Permission assignment with metadata
 */
export interface PermissionAssignment {
  id: string
  userId: string
  permission: EPSXPermission
  source: 'direct' | 'role' | 'template' | 'system'
  sourceId?: string // Role ID, template ID, etc.
  grantedBy: string
  grantedAt: number
  expiresAt?: number
  isActive: boolean
  metadata?: Record<string, any>
}

/**
 * Bulk permission operation
 */
export interface BulkPermissionOperation {
  operation: 'grant' | 'revoke' | 'update'
  userIds: string[]
  permissions?: EPSXPermission[]
  roleIds?: string[]
  templateIds?: string[]
  expiresAt?: number
  reason?: string
  performedBy: string
  performedAt: number
}

/**
 * Permission analytics and insights
 */
export interface PermissionAnalytics {
  totalUsers: number
  totalPermissions: number
  activePermissions: number
  expiredPermissions: number
  expiringPermissions: number
  permissionsByPlatform: Record<string, number>
  permissionsByCategory: Record<PermissionCategory, number>
  roleUsage: Record<string, number>
  templateUsage: Record<string, number>
  computedAt: number
}

// ============================================================================
// FEATURE-SPECIFIC PERMISSION TYPES
// ============================================================================

/**
 * Analytics-specific permissions
 */
export interface AnalyticsPermissions {
  canViewRankings: boolean
  canExportData: boolean
  canViewHistoricalData: boolean
  canAccessRealtime: boolean
  maxStocksTracked: number
  canCreateCustomDashboards: boolean
}

/**
 * Trading-specific permissions
 */
export interface TradingPermissions {
  paperTrading: boolean
  liveTrading: boolean
  advancedOrders: boolean
  marginTrading: boolean
  optionsTrading: boolean
  cryptoTrading: boolean
}

/**
 * Admin-specific permissions
 */
export interface AdminPermissions {
  canManageUsers: boolean
  canManageRoles: boolean
  canManagePermissions: boolean
  canViewAuditLogs: boolean
  canManageBilling: boolean
  canManageSystem: boolean
  canAccessDeveloperTools: boolean
}

/**
 * API-specific permissions
 */
export interface APIPermissions {
  canCreateApiKeys: boolean
  canManageApiKeys: boolean
  canAccessWebhooks: boolean
  rateLimit: {
    requestsPerMinute: number
    requestsPerHour: number
    requestsPerDay: number
  }
}

// ============================================================================
// PERMISSION CONTEXT & INHERITANCE
// ============================================================================

/**
 * Permission inheritance chain
 */
export interface PermissionInheritance {
  userId: string
  permission: EPSXPermission
  inheritanceChain: {
    source: 'user' | 'role' | 'template' | 'system'
    sourceId: string
    sourceName: string
    grantedAt: number
  }[]
  isOverridden: boolean
  overriddenBy?: string
  finalValue: boolean
}

/**
 * Platform-specific permission context
 */
export interface PlatformPermissionContext {
  platform: string
  permissions: EPSXPermission[]
  roles: string[]
  isPrimary: boolean
  accessLevel: PermissionScope
  restrictions?: Record<string, any>
}

/**
 * Time-based permission constraints
 */
export interface PermissionConstraint {
  id: string
  permission: EPSXPermission
  constraint: {
    type: 'time_window' | 'usage_limit' | 'geographic' | 'conditional'
    config: Record<string, any>
  }
  isActive: boolean
  createdAt: number
}

// ============================================================================
// PERMISSION MANAGEMENT TYPES
// ============================================================================

/**
 * Permission request/approval workflow
 */
export interface PermissionRequest {
  id: string
  requestedBy: string
  requestedFor: string
  permissions: EPSXPermission[]
  reason: string
  status: 'pending' | 'approved' | 'rejected' | 'expired'
  reviewedBy?: string
  reviewedAt?: number
  reviewNotes?: string
  expiresAt?: number
  createdAt: number
}

/**
 * Permission policy definition
 */
export interface PermissionPolicy {
  id: string
  name: string
  description: string
  conditions: {
    userAttributes?: Record<string, any>
    timeConstraints?: Record<string, any>
    platformRestrictions?: string[]
  }
  actions: {
    grant?: EPSXPermission[]
    deny?: EPSXPermission[]
    require?: EPSXPermission[]
  }
  priority: number
  isActive: boolean
  createdAt: Date
  updatedAt: Date
}

// ============================================================================
// HELPER FUNCTIONS & TYPE GUARDS
// ============================================================================

/**
 * Parse permission string into components
 */
export function parsePermission(permission: EPSXPermission): ParsedPermission {
  const [platform, resource, action] = permission.split(':')
  return {
    platform: platform || '',
    resource: resource || '',
    action: action || '',
    full: permission
  }
}

/**
 * Check if permission is wildcard
 */
export function isWildcardPermission(permission: EPSXPermission): boolean {
  return permission.includes('*')
}

/**
 * Check if permission matches pattern
 */
export function matchesPermissionPattern(permission: EPSXPermission, pattern: EPSXPermission): boolean {
  if (pattern === permission) return true
  if (!pattern.includes('*')) return false
  
  const permParts = permission.split(':')
  const patternParts = pattern.split(':')
  
  return patternParts.every((part, index) => 
    part === '*' || part === permParts[index]
  )
}

/**
 * Get all permissions for a user including inheritance
 */
export function resolveUserPermissions(
  directPermissions: EPSXPermission[],
  roles: PermissionRole[],
  templates: PermissionTemplate[]
): EffectivePermissions {
  const allPermissions = new Set<EPSXPermission>()
  const inheritedPermissions: EPSXPermission[] = []
  const roleIds: string[] = []
  
  // Add direct permissions
  directPermissions.forEach(p => allPermissions.add(p))
  
  // Add role permissions with inheritance
  roles.forEach(role => {
    roleIds.push(role.id)
    role.permissions.forEach(p => {
      allPermissions.add(p)
      inheritedPermissions.push(p)
    })
  })
  
  // Add template permissions
  templates.forEach(template => {
    template.permissions.forEach(p => {
      allPermissions.add(p)
      inheritedPermissions.push(p)
    })
  })
  
  return {
    userId: '', // Will be set by caller
    permissions: Array.from(allPermissions),
    roles: roleIds,
    directPermissions,
    inheritedPermissions,
    temporaryPermissions: [],
    deniedPermissions: [],
    computedAt: Date.now()
  }
}

/**
 * Check if user has permission considering all sources
 */
export function hasEffectivePermission(
  effective: EffectivePermissions,
  requiredPermission: EPSXPermission
): boolean {
  return effective.permissions.some(permission =>
    permission === requiredPermission ||
    matchesPermissionPattern(requiredPermission, permission)
  )
}

// ============================================================================
// LEGACY COMPATIBILITY ALIASES
// ============================================================================

/** @deprecated Use PermissionTemplate instead */
export type PermissionProfile = PermissionTemplate

/** @deprecated Use PermissionValidation instead */
export type PermissionResult = PermissionValidation

/** @deprecated Use EPSXPermission instead */
export type PermissionString = EPSXPermission