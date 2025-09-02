// ============================================================================
// GRANULAR PERMISSION SYSTEM - ADMIN FRONTEND TYPES (MATCHES RUST BACKEND)
// ============================================================================
// Updated to match the new backend granular permission system with hash-based caching
// and instant revocation capabilities.

// ============================================================================
// GRANULAR PERMISSION CORE TYPES
// ============================================================================

export type PermissionSource = 'Subscription' | 'Admin' | 'Trial' | 'Legacy' | 'System';

export interface GranularPermissionClaim {
  expires_at?: number; // Unix timestamp
  source: PermissionSource;
  granted_at: number; // Unix timestamp
  granted_by?: string; // Admin user ID who granted the permission
}

export interface GranularPermissionSet {
  permissions: Record<string, GranularPermissionClaim>;
  hash: string; // SHA-256 hash for instant revocation validation
  version: number; // Version for cache invalidation
  created_at: number; // Unix timestamp
}

// ============================================================================
// ENHANCED USER CLAIMS WITH GRANULAR PERMISSIONS
// ============================================================================

export interface EnhancedUserClaims {
  sub: string; // User ID
  email?: string;
  name?: string;
  role?: string;
  firebase_uid?: string;
  
  // Granular permission system
  permissions: Record<string, GranularPermissionClaim>;
  permission_hash: string; // For instant revocation validation
  permission_version: number; // For cache synchronization
  
  // Standard JWT claims
  iat: number;
  exp: number;
  aud: string;
  iss: string;
  
  // Legacy compatibility
  package_tier?: string;
  is_active?: boolean;
  platforms?: string[];
  primary_platform?: string;
}

// ============================================================================
// TOKEN VALIDATION AND CACHING
// ============================================================================

export interface TokenValidationResult {
  claims: EnhancedUserClaims;
  valid_permissions: string[]; // Filtered non-expired permissions
  updated_token?: string; // New token if permissions were cleaned
}

export type HashValidationResult = 
  | { status: 'Valid' }
  | { status: 'Revoked' }
  | { status: 'Updated'; new_hash: string }
  | { status: 'NotFound' };

export interface PermissionCacheEntry {
  user_id: string;
  permission_hash: string;
  permission_version: number;
  permissions: Record<string, GranularPermissionClaim>;
  cached_at: number;
  expires_at?: number;
  is_revoked: boolean;
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

export interface PermissionStatusResponse {
  user_id: string;
  permissions: Record<string, GranularPermissionClaim>;
  permission_hash: string;
  permission_version: number;
  health: PermissionHealthInfo;
}

export interface PermissionHealthInfo {
  total_permissions: number;
  active_permissions: number;
  expired_permissions: number;
  expiring_soon_permissions: number; // Within 24 hours
  next_expiry?: number;
  time_until_next_expiry?: number; // Milliseconds
  health_score: number; // 0-100 based on expiry status
}

// ============================================================================
// ADMIN PERMISSION MANAGEMENT TYPES
// ============================================================================

export interface GrantPermissionRequest {
  user_id: string;
  permission: string;
  expires_at?: number; // Unix timestamp for temporary permission
  source: PermissionSource;
  reason?: string;
}

export interface RevokePermissionRequest {
  user_id: string;
  permission: string;
  reason?: string;
}

export interface BulkPermissionRequest {
  user_ids: string[];
  permission: string;
  expires_at?: number;
  source: PermissionSource;
  reason?: string;
}

export interface ExtendPermissionRequest {
  user_id: string;
  permission: string;
  new_expires_at: number;
  reason?: string;
}

export interface PermissionAuditEntry {
  id: string;
  user_id: string;
  permission: string;
  action: 'grant' | 'revoke' | 'extend' | 'cleanup';
  performed_by: string;
  performed_at: number;
  expires_at?: number;
  reason?: string;
  metadata?: Record<string, any>;
}

// ============================================================================
// ADMIN-SPECIFIC MANAGEMENT TYPES
// ============================================================================

export interface AdminPermissionDashboard {
  total_users_with_permissions: number;
  total_permissions_granted: number;
  expiring_permissions_24h: number;
  expired_permissions: number;
  recent_grants: PermissionAuditEntry[];
  recent_revocations: PermissionAuditEntry[];
  system_health_score: number;
}

export interface UserPermissionOverview {
  user_id: string;
  email: string;
  display_name?: string;
  permissions: Record<string, GranularPermissionClaim>;
  permission_hash: string;
  permission_version: number;
  health: PermissionHealthInfo;
  last_activity?: number;
  created_at: number;
}

export interface PermissionTemplate {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  default_expiry_hours?: number;
  source: PermissionSource;
  created_by: string;
  created_at: number;
}

export interface BulkOperationResult {
  operation: string;
  total_requested: number;
  successful: number;
  failed: number;
  details: {
    user_id: string;
    success: boolean;
    error?: string;
  }[];
}

// ============================================================================
// NOTIFICATION AND EVENT TYPES
// ============================================================================

export type NotificationEventType = 
  | 'PermissionGranted'
  | 'PermissionRevoked'
  | 'PermissionExpiring'
  | 'PermissionExpired'
  | 'PermissionExtended'
  | 'BulkOperationCompleted';

export interface PermissionNotificationData {
  user_id: string;
  permission: string;
  base_permission: string;
  expires_at?: number;
  granted_by?: string;
  reason?: string;
}

export interface BulkOperationNotificationData {
  operation: string;
  user_count: number;
  permission: string;
  successful: number;
  failed: number;
  performed_by: string;
}

export interface PermissionNotificationEvent {
  event_type: NotificationEventType;
  data: PermissionNotificationData | BulkOperationNotificationData;
  timestamp: number;
}

// ============================================================================
// UTILITY AND HELPER TYPES
// ============================================================================

export interface ParsedPermission {
  platform: string;
  resource: string;
  action: string;
  full: string; // Original permission string
}

export interface PermissionExpiryDetails {
  permission: string;
  base_permission: string;
  claim: GranularPermissionClaim;
  is_expired: boolean;
  expires_in_ms?: number;
  expires_in_human?: string;
  is_permanent: boolean;
}

export interface UserPermissionSummary {
  user_id: string;
  total_permissions: number;
  permanent_permissions: number;
  temporary_permissions: number;
  expired_permissions: number;
  expiring_soon_permissions: number;
  permission_hash: string;
  permission_version: number;
  last_updated: number;
}

// ============================================================================
// ADMIN UI COMPONENT TYPES
// ============================================================================

export interface PermissionSearchFilters {
  user_email?: string;
  permission_pattern?: string;
  source?: PermissionSource;
  expires_within_hours?: number;
  is_expired?: boolean;
  has_expiring_soon?: boolean;
}

export interface PermissionTableColumn {
  key: string;
  label: string;
  sortable?: boolean;
  filterable?: boolean;
  width?: string;
}

export interface AdminPermissionRowData {
  user_id: string;
  email: string;
  display_name?: string;
  permission: string;
  base_permission: string;
  source: PermissionSource;
  granted_at: number;
  granted_by?: string;
  expires_at?: number;
  is_expired: boolean;
  expires_in_human?: string;
  health_status: 'healthy' | 'expiring' | 'expired';
}

// ============================================================================
// ERROR TYPES
// ============================================================================

export interface PermissionError {
  code: 'INSUFFICIENT_PERMISSION' | 'PERMISSION_EXPIRED' | 'HASH_MISMATCH' | 'TOKEN_INVALID' | 'USER_NOT_FOUND' | 'ADMIN_REQUIRED';
  message: string;
  details?: string;
  permission?: string;
}

export class GranularPermissionError extends Error {
  constructor(
    message: string,
    public readonly code: PermissionError['code'],
    public readonly permission?: string,
    public readonly details?: string
  ) {
    super(message);
    this.name = 'GranularPermissionError';
  }
}

// ============================================================================
// ADMIN-SPECIFIC HOOKS AND CONTEXT TYPES
// ============================================================================

export interface AdminPermissionHookResult {
  // User permission queries
  getUserPermissions: (userId: string) => Promise<PermissionStatusResponse>;
  getAllUsersWithPermissions: (filters?: PermissionSearchFilters) => Promise<UserPermissionOverview[]>;
  
  // Permission management
  grantPermission: (request: GrantPermissionRequest) => Promise<void>;
  revokePermission: (request: RevokePermissionRequest) => Promise<void>;
  bulkGrantPermissions: (request: BulkPermissionRequest) => Promise<BulkOperationResult>;
  bulkRevokePermissions: (request: Omit<BulkPermissionRequest, 'expires_at' | 'source'>) => Promise<BulkOperationResult>;
  extendPermission: (request: ExtendPermissionRequest) => Promise<void>;
  
  // Templates
  createPermissionTemplate: (template: Omit<PermissionTemplate, 'id' | 'created_at' | 'created_by'>) => Promise<PermissionTemplate>;
  applyPermissionTemplate: (templateId: string, userIds: string[]) => Promise<BulkOperationResult>;
  
  // Monitoring
  getDashboard: () => Promise<AdminPermissionDashboard>;
  getPermissionAudit: (userId?: string, limit?: number) => Promise<PermissionAuditEntry[]>;
  
  // State
  loading: boolean;
  error: PermissionError | null;
}

export interface AdminPermissionContextValue {
  userClaims: EnhancedUserClaims | null;
  isAdmin: boolean;
  canManagePermissions: boolean;
  permissionApi: AdminPermissionHookResult;
  refreshAdminSession: () => Promise<void>;
}

// ============================================================================
// ADMIN API CLIENT TYPES
// ============================================================================

export interface AdminPermissionApiClient {
  // User management
  getUserPermissions: (userId: string) => Promise<PermissionStatusResponse>;
  getAllUsersWithPermissions: (filters?: PermissionSearchFilters) => Promise<UserPermissionOverview[]>;
  searchUsers: (query: string) => Promise<{ user_id: string; email: string; display_name?: string }[]>;
  
  // Permission management
  grantPermission: (request: GrantPermissionRequest) => Promise<void>;
  revokePermission: (request: RevokePermissionRequest) => Promise<void>;
  extendPermission: (request: ExtendPermissionRequest) => Promise<void>;
  
  // Bulk operations
  bulkGrantPermissions: (request: BulkPermissionRequest) => Promise<BulkOperationResult>;
  bulkRevokePermissions: (request: Omit<BulkPermissionRequest, 'expires_at' | 'source'>) => Promise<BulkOperationResult>;
  bulkCleanupExpired: (userIds?: string[]) => Promise<BulkOperationResult>;
  
  // Templates
  getPermissionTemplates: () => Promise<PermissionTemplate[]>;
  createPermissionTemplate: (template: Omit<PermissionTemplate, 'id' | 'created_at' | 'created_by'>) => Promise<PermissionTemplate>;
  deletePermissionTemplate: (templateId: string) => Promise<void>;
  applyPermissionTemplate: (templateId: string, userIds: string[]) => Promise<BulkOperationResult>;
  
  // Monitoring and audit
  getDashboard: () => Promise<AdminPermissionDashboard>;
  getPermissionAudit: (userId?: string, limit?: number) => Promise<PermissionAuditEntry[]>;
  getSystemHealth: () => Promise<{ health_score: number; issues: string[] }>;
  
  // Cache management
  invalidateUserPermissionCache: (userId: string) => Promise<void>;
  refreshPermissionCache: () => Promise<void>;
}

// ============================================================================
// COMPONENT PROP TYPES
// ============================================================================

export interface AdminPermissionDashboardProps {
  refreshInterval?: number; // milliseconds
  showRecentActivity?: boolean;
  className?: string;
}

export interface AdminUserPermissionTableProps {
  filters?: PermissionSearchFilters;
  onUserSelect?: (userId: string) => void;
  onBulkAction?: (action: string, userIds: string[]) => void;
  pageSize?: number;
  className?: string;
}

export interface AdminPermissionManagerProps {
  userId: string;
  onPermissionChange?: (permissions: Record<string, GranularPermissionClaim>) => void;
  readOnly?: boolean;
  showAuditLog?: boolean;
  className?: string;
}

export interface AdminBulkPermissionManagerProps {
  userIds: string[];
  onBulkOperation?: (result: BulkOperationResult) => void;
  availableTemplates?: PermissionTemplate[];
  className?: string;
}

export interface AdminPermissionAuditProps {
  userId?: string;
  limit?: number;
  showFilters?: boolean;
  className?: string;
}

// ============================================================================
// LEGACY COMPATIBILITY TYPES
// ============================================================================

// For gradual migration from old permission system
export interface LegacyPermissionMapping {
  legacy_permission: string;
  granular_permission: string;
  default_source: PermissionSource;
  default_expiry_hours?: number;
}

export interface MigrationStatus {
  total_users: number;
  migrated_users: number;
  failed_migrations: number;
  legacy_permissions_remaining: number;
}

export interface MigrationPlan {
  phase: 'analysis' | 'migration' | 'validation' | 'cleanup';
  total_steps: number;
  completed_steps: number;
  current_step_description: string;
  estimated_completion?: number; // Unix timestamp
}

// Export commonly used types for easy importing
export {
  type PermissionSource,
  type GranularPermissionClaim,
  type GranularPermissionSet,
  type EnhancedUserClaims,
  type PermissionStatusResponse,
  type PermissionHealthInfo,
  type PermissionError
};