// ============================================================================
// GRANULAR PERMISSION SYSTEM - FRONTEND TYPES (MATCHES RUST BACKEND)
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
// NOTIFICATION AND EVENT TYPES
// ============================================================================

export type NotificationEventType = 
  | 'PermissionGranted'
  | 'PermissionRevoked'
  | 'PermissionExpiring'
  | 'PermissionExpired'
  | 'PermissionExtended';

export interface PermissionNotificationData {
  user_id: string;
  permission: string;
  base_permission: string;
  expires_at?: number;
  granted_by?: string;
  reason?: string;
}

export interface PermissionNotificationEvent {
  event_type: NotificationEventType;
  data: PermissionNotificationData;
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
// ERROR TYPES
// ============================================================================

export interface PermissionError {
  code: 'INSUFFICIENT_PERMISSION' | 'PERMISSION_EXPIRED' | 'HASH_MISMATCH' | 'TOKEN_INVALID' | 'USER_NOT_FOUND';
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
// FRONTEND-SPECIFIC TYPES
// ============================================================================

export interface PermissionGuardProps {
  permission: string;
  children: React.ReactNode;
  fallback?: React.ReactNode;
  requireAll?: boolean; // For multiple permissions
}

export interface UsePermissionHookResult {
  hasPermission: (permission: string) => boolean;
  hasAnyPermission: (permissions: string[]) => boolean;
  hasAllPermissions: (permissions: string[]) => boolean;
  getPermissionExpiry: (permission: string) => PermissionExpiryDetails | null;
  getPermissionHealth: () => PermissionHealthInfo | null;
  isPermissionExpiring: (permission: string, withinHours?: number) => boolean;
  refreshPermissions: () => Promise<void>;
  loading: boolean;
  error: PermissionError | null;
}

export interface PermissionContextValue {
  userClaims: EnhancedUserClaims | null;
  permissionSet: GranularPermissionSet | null;
  loading: boolean;
  error: PermissionError | null;
  refreshPermissions: () => Promise<void>;
  validatePermissionHash: () => Promise<HashValidationResult>;
}

// ============================================================================
// API CLIENT TYPES
// ============================================================================

export interface PermissionApiClient {
  // User permission status
  getUserPermissions: (userId?: string) => Promise<PermissionStatusResponse>;
  refreshUserToken: () => Promise<TokenValidationResult>;
  validatePermissionHash: (hash: string) => Promise<HashValidationResult>;
  
  // Admin operations (require admin permissions)
  grantPermission: (request: GrantPermissionRequest) => Promise<void>;
  revokePermission: (request: RevokePermissionRequest) => Promise<void>;
  bulkGrantPermissions: (request: BulkPermissionRequest) => Promise<void>;
  extendPermission: (request: ExtendPermissionRequest) => Promise<void>;
  
  // Audit and monitoring
  getPermissionAudit: (userId: string, limit?: number) => Promise<PermissionAuditEntry[]>;
  getPermissionHealth: (userId: string) => Promise<PermissionHealthInfo>;
}

// ============================================================================
// COMPONENT PROP TYPES
// ============================================================================

export interface PermissionStatusCardProps {
  userId?: string;
  showDetails?: boolean;
  showExpiry?: boolean;
  className?: string;
}

export interface PermissionExpiryIndicatorProps {
  permission: string;
  className?: string;
  showCountdown?: boolean;
}

export interface AdminPermissionManagerProps {
  userId: string;
  onPermissionChange?: (permissions: Record<string, GranularPermissionClaim>) => void;
  readOnly?: boolean;
}

export interface BulkPermissionManagerProps {
  userIds: string[];
  onBulkOperation?: (operation: string, success: number, failed: number) => void;
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

// Export all types for easy importing
export * from './permissions'; // Re-export legacy types for compatibility