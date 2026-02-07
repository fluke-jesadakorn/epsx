/**
 * Permission Domain Types
 * Business domain permission types (NOT validation logic)
 * For backend permission error types, use shared/types/backend/permission-types.ts
 */

// ============================================================================
// BASIC PERMISSION TYPES
// ============================================================================

export type Permission = string // Format: "platform:resource:action"
export type EPSXPermission = string // Alias for clarity

export interface ParsedPermission {
  platform: string
  resource: string
  action: string
  full: string
}

export type PermissionSource = 'direct' | 'role' | 'template' | 'system' | 'group'
export type Platform = 'epsx' | 'admin' | 'epsx-pay' | 'epsx-token'
export type PermissionScope = 'read' | 'write' | 'admin' | 'owner'

// ============================================================================
// TIMESTAMPED PERMISSIONS
// ============================================================================

export interface TimestampedPermission {
  permission: string
  expires_at: number
  granted_at: number
  granted_by?: string
}

export interface PermissionExpiryDetails {
  permission: string
  expires_at: number
  is_expired: boolean
  time_until_expiry_seconds: number
}

export interface PermissionExpiryInfo {
  total_permissions: number
  expired_count: number
  expiring_soon_count: number
  expired_permissions: string[]
  expiring_permissions: PermissionExpiryDetails[]
}

// ============================================================================
// PERMISSION CATEGORIES
// ============================================================================

export type PermissionCategory =
  | 'analytics'
  | 'market'
  | 'admin'
  | 'billing'
  | 'api'
  | 'web3'
  | 'notifications'
  | 'developer'

// ============================================================================
// PERMISSION TEMPLATES
// ============================================================================

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

// ============================================================================
// PERMISSION VALIDATION
// ============================================================================

export interface PermissionCheck {
  permission: EPSXPermission
  platform?: string
  scope?: PermissionScope
  context?: Record<string, any>
  requiresSubscription?: boolean
}

export interface PermissionValidation {
  hasPermission: boolean
  permission: EPSXPermission
  reason: 'granted' | 'denied' | 'expired' | 'insufficient_tier' | 'not_found'
  expiresAt?: number
  grantedBy?: string
  grantedAt?: number
}

// ============================================================================
// PERMISSION ROLES
// ============================================================================

export interface PermissionRole {
  id: string
  name: string
  description: string
  permissions: EPSXPermission[]
  inherits?: string[]
  isSystemRole: boolean
  platforms: string[]
  createdAt: Date
  updatedAt: Date
}

// ============================================================================
// EFFECTIVE PERMISSIONS
// ============================================================================

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

// ============================================================================
// PERMISSION ASSIGNMENT
// ============================================================================

export interface PermissionAssignment {
  id: string
  userId: string
  permission: EPSXPermission
  source: PermissionSource
  sourceId?: string
  grantedBy: string
  grantedAt: number
  expiresAt?: number
  isActive: boolean
  metadata?: Record<string, any>
}

// ============================================================================
// BULK OPERATIONS
// ============================================================================

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

// ============================================================================
// ANALYTICS
// ============================================================================

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
// INHERITANCE & CONTEXT
// ============================================================================

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

export interface PlatformPermissionContext {
  platform: string
  permissions: EPSXPermission[]
  roles: string[]
  isPrimary: boolean
  accessLevel: PermissionScope
  restrictions?: Record<string, any>
}

// ============================================================================
// PERMISSION MANAGEMENT
// ============================================================================

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
// HELPER FUNCTIONS
// ============================================================================

export function parsePermission(permission: EPSXPermission): ParsedPermission {
  const [platform, resource, action] = permission.split(':')
  return {
    platform: platform || '',
    resource: resource || '',
    action: action || '',
    full: permission
  }
}

export function isWildcardPermission(permission: EPSXPermission): boolean {
  return permission.includes('*')
}

export function matchesPermissionPattern(permission: EPSXPermission, pattern: EPSXPermission): boolean {
  if (pattern === permission) {return true}
  if (!pattern.includes('*')) {return false}

  const permParts = permission.split(':')
  const patternParts = pattern.split(':')

  return patternParts.every((part, index) =>
    part === '*' || part === permParts[index]
  )
}

export function resolveUserPermissions(
  directPermissions: EPSXPermission[],
  roles: PermissionRole[],
  templates: PermissionTemplate[]
): EffectivePermissions {
  const allPermissions = new Set<EPSXPermission>()
  const inheritedPermissions: EPSXPermission[] = []
  const roleIds: string[] = []

  directPermissions.forEach(p => allPermissions.add(p))

  roles.forEach(role => {
    roleIds.push(role.id)
    role.permissions.forEach(p => {
      allPermissions.add(p)
      inheritedPermissions.push(p)
    })
  })

  templates.forEach(template => {
    template.permissions.forEach(p => {
      allPermissions.add(p)
      inheritedPermissions.push(p)
    })
  })

  return {
    userId: '',
    permissions: Array.from(allPermissions),
    roles: roleIds,
    directPermissions,
    inheritedPermissions,
    temporaryPermissions: [],
    deniedPermissions: [],
    computedAt: Date.now()
  }
}

export function hasEffectivePermission(
  effective: EffectivePermissions,
  requiredPermission: EPSXPermission
): boolean {
  return effective.permissions.some(permission =>
    permission === requiredPermission ||
    matchesPermissionPattern(requiredPermission, permission)
  )
}
