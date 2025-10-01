// ============================================================================
// SHARED PERMISSION API TYPES
// ============================================================================
// API request/response types for permission-related operations

import { 
  GranularPermissionClaim, 
  PermissionHealthInfo, 
  PermissionSource,
  TokenValidationResult,
  HashValidationResult
} from './core'
import { PermissionAuditEntry, PermissionTemplate } from './audit'

// ============================================================================
// API REQUEST TYPES
// ============================================================================

export interface GrantPermissionRequest {
  user_id: string
  permission: string
  expires_at?: number // Unix timestamp for temporary permission
  source: PermissionSource
  reason?: string
}

export interface RevokePermissionRequest {
  user_id: string
  permission: string
  reason?: string
}

export interface BulkPermissionRequest {
  user_ids: string[]
  permission: string
  expires_at?: number
  source: PermissionSource
  reason?: string
}

export interface ExtendPermissionRequest {
  user_id: string
  permission: string
  new_expires_at: number
  reason?: string
}

export interface PermissionSearchFilters {
  user_email?: string
  permission_pattern?: string
  source?: PermissionSource
  expires_within_hours?: number
  is_expired?: boolean
  has_expiring_soon?: boolean
  limit?: number
  offset?: number
}

export interface PermissionValidationRequest {
  permission: string
  user_id?: string
  check_expiry?: boolean
  min_valid_hours?: number
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

export interface PermissionStatusResponse {
  user_id: string
  permissions: Record<string, GranularPermissionClaim>
  permission_hash: string
  permission_version: number
  health: PermissionHealthInfo
}

export interface UserPermissionOverview {
  user_id: string
  email: string
  display_name?: string
  permissions: Record<string, GranularPermissionClaim>
  permission_hash: string
  permission_version: number
  health: PermissionHealthInfo
  last_activity?: number
  created_at: number
}

export interface BulkOperationResult {
  operation: string
  total_requested: number
  successful: number
  failed: number
  details: Array<{
    user_id: string
    success: boolean
    error?: string
  }>
  started_at: number
  completed_at: number
}

export interface PermissionValidationResponse {
  is_valid: boolean
  permission: string
  user_id?: string
  expires_at?: number
  expires_in_ms?: number
  is_expired: boolean
  reason?: string
}

export interface SystemHealthResponse {
  health_score: number
  total_users: number
  total_permissions: number
  expired_permissions: number
  expiring_soon: number
  issues: string[]
  last_check: number
}

// ============================================================================
// API CLIENT INTERFACE TYPES
// ============================================================================

export interface PermissionApiClient {
  // User permission status
  getUserPermissions: (userId?: string) => Promise<PermissionStatusResponse>
  refreshUserToken: () => Promise<TokenValidationResult>
  validatePermissionHash: (hash: string) => Promise<HashValidationResult>
  
  // Permission validation
  validatePermission: (request: PermissionValidationRequest) => Promise<PermissionValidationResponse>
  
  // Basic permission operations (require appropriate permissions)
  grantPermission: (request: GrantPermissionRequest) => Promise<void>
  revokePermission: (request: RevokePermissionRequest) => Promise<void>
  extendPermission: (request: ExtendPermissionRequest) => Promise<void>
}

export interface AdminPermissionApiClient extends PermissionApiClient {
  // Advanced user management
  getAllUsersWithPermissions: (filters?: PermissionSearchFilters) => Promise<UserPermissionOverview[]>
  searchUsers: (query: string) => Promise<Array<{ user_id: string; email: string; display_name?: string }>>
  
  // Bulk operations  
  bulkGrantPermissions: (request: BulkPermissionRequest) => Promise<BulkOperationResult>
  bulkRevokePermissions: (request: Omit<BulkPermissionRequest, 'expires_at' | 'source'>) => Promise<BulkOperationResult>
  bulkCleanupExpired: (userIds?: string[]) => Promise<BulkOperationResult>
  
  // Templates
  getPermissionTemplates: () => Promise<PermissionTemplate[]>
  createPermissionTemplate: (template: Omit<PermissionTemplate, 'id' | 'created_at' | 'created_by'>) => Promise<PermissionTemplate>
  deletePermissionTemplate: (templateId: string) => Promise<void>
  applyPermissionTemplate: (templateId: string, userIds: string[]) => Promise<BulkOperationResult>
  
  // Monitoring and audit
  getDashboard: () => Promise<AdminPermissionDashboard>
  getPermissionAudit: (userId?: string, limit?: number) => Promise<PermissionAuditEntry[]>
  getSystemHealth: () => Promise<SystemHealthResponse>
  
  // Cache management
  invalidateUserPermissionCache: (userId: string) => Promise<void>
  refreshPermissionCache: () => Promise<void>
}

// ============================================================================
// PAGINATION TYPES
// ============================================================================

export interface PaginatedResponse<T> {
  data: T[]
  pagination: {
    total: number
    page: number
    per_page: number
    total_pages: number
    has_next: boolean
    has_prev: boolean
  }
}

export type PaginatedUsersResponse = PaginatedResponse<UserPermissionOverview>
export type PaginatedAuditResponse = PaginatedResponse<PermissionAuditEntry>

// ============================================================================
// WEBHOOK AND EVENT TYPES
// ============================================================================

export type NotificationEventType = 
  | 'PermissionGranted'
  | 'PermissionRevoked'
  | 'PermissionExpiring'
  | 'PermissionExpired'
  | 'PermissionExtended'
  | 'BulkOperationCompleted'

export interface PermissionNotificationData {
  user_id: string
  permission: string
  base_permission: string
  expires_at?: number
  granted_by?: string
  reason?: string
}

export interface BulkOperationNotificationData {
  operation: string
  user_count: number
  permission: string
  successful: number
  failed: number
  performed_by: string
}

export interface PermissionNotificationEvent {
  event_type: NotificationEventType
  data: PermissionNotificationData | BulkOperationNotificationData
  timestamp: number
}

// Forward declarations from other files
// PermissionTemplate imported from './audit' - removed duplicate definition

export interface AdminPermissionDashboard {
  total_users_with_permissions: number
  total_permissions_granted: number
  expiring_permissions_24h: number
  expired_permissions: number
  recent_grants: PermissionAuditEntry[]
  recent_revocations: PermissionAuditEntry[]
  system_health_score: number
}

// PermissionAuditEntry imported from './audit' - removed duplicate definition