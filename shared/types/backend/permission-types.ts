/**
 * Backend Permission Types
 * Standard request/response interfaces matching backend Rust structs
 * These types represent what the backend sends/receives - NO client-side validation
 */

// ============================================================================
// PERMISSION ERROR TYPES (from backend permission_errors.rs)
// ============================================================================

export interface BackendPermissionError {
  code: number
  error_type: 'PermissionDenied' | 'InsufficientGroup' | 'UsageLimitExceeded' | 'PermissionExpired' | 'SecurityRestriction' | 'AuthenticationRequired'
  message: string
  permission?: string
  reason?: string
  current_group?: string
  required_group?: string
  upgrade_url?: string
  benefits?: string[]
  suggested_actions?: string[]
  expired_at?: string
  renewal_url?: string
  current_usage?: number
  limit?: number
  reset_at?: string
  risk_level?: 'low' | 'medium' | 'high' | 'critical'
  contact_support?: boolean
}

// ============================================================================
// PERMISSION VALIDATION TYPES (from backend)
// ============================================================================

export interface PermissionCheckResult {
  granted: boolean
  permission: string
  reason?: string
  expires_at?: number
}

export interface BatchPermissionCheckResult {
  results: PermissionCheckResult[]
  user_id: string
}

// ============================================================================
// PERMISSION GROUP TYPES (from backend)
// ============================================================================

export interface PermissionGroup {
  id: string
  name: string
  description?: string
  permissions: string[]
  is_system_group: boolean
  priority: number
  created_at: string
  updated_at: string
}

export interface CreatePermissionGroupRequest {
  name: string
  description?: string
  permissions: string[]
  is_system_group?: boolean
  priority?: number
}

export interface UpdatePermissionGroupRequest {
  name?: string
  description?: string
  permissions?: string[]
  priority?: number
}

// ============================================================================
// USER PERMISSION TYPES (from backend)
// ============================================================================

export interface UserPermissions {
  user_id: string
  wallet_address: string
  permissions: string[]
  permission_groups: string[]
  expires_at?: number
}

export interface GrantPermissionRequest {
  user_id: string
  permission: string
  expires_at?: number
  reason?: string
}

export interface RevokePermissionRequest {
  user_id: string
  permission: string
  reason?: string
}

export interface BulkPermissionRequest {
  user_ids: string[]
  permissions: string[]
  action: 'grant' | 'revoke'
  expires_at?: number
  reason?: string
}

// ============================================================================
// PERMISSION AUDIT TYPES (from backend)
// ============================================================================

export interface PermissionAuditEntry {
  id: string
  user_id: string
  wallet_address: string
  permission: string
  action: 'grant' | 'revoke' | 'expire'
  performed_by: string
  performed_at: string
  reason?: string
  metadata?: Record<string, any>
}

export interface PermissionAuditResponse {
  entries: PermissionAuditEntry[]
  total: number
  page: number
  per_page: number
}

// ============================================================================
// GROUP MEMBERSHIP TYPES (from backend)
// ============================================================================

export interface GroupMembership {
  id: string
  user_id: string
  group_id: string
  wallet_address: string
  assigned_by: string
  assigned_at: string
  expires_at?: string
  is_active: boolean
}

export interface AssignGroupRequest {
  user_id: string
  group_id: string
  expires_at?: number
  reason?: string
}

export interface RemoveGroupRequest {
  membership_id: string
  reason?: string
}

// ============================================================================
// PERMISSION ANALYTICS TYPES (from backend)
// ============================================================================

export interface PermissionStats {
  total_users: number
  total_groups: number
  total_permissions: number
  active_permissions: number
  expired_permissions: number
  users_by_group: Record<string, number>
}

// ============================================================================
// WEB3 PERMISSION TYPES (from backend)
// ============================================================================

export interface Web3PermissionContext {
  wallet_address: string
  chain_id: number
  permissions: string[]
  groups: string[]
  session_expires_at: number
}

export interface Web3SessionToken {
  token: string
  wallet_address: string
  expires_at: number
  permissions: string[]
}

// ============================================================================
// TYPE GUARDS
// ============================================================================

export function isBackendPermissionError(error: unknown): error is BackendPermissionError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    'error_type' in error &&
    'message' in error
  )
}

export function isPermissionCheckResult(result: unknown): result is PermissionCheckResult {
  return (
    typeof result === 'object' &&
    result !== null &&
    'granted' in result &&
    'permission' in result &&
    typeof (result as any).granted === 'boolean'
  )
}
