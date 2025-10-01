// Embedded timestamp permission types based on Rust backend structures

export interface EmbeddedPermissionRequest {
  embedded_permission: string;
  base_permission: string;
  platform: string;
  resource: string;
  action: string;
  expiry_timestamp: number;
  reason?: string;
  metadata?: Record<string, any>;
}

export interface EmbeddedPermissionData {
  base_permission: string;
  platform: string;
  resource: string;
  action: string;
  expiry_timestamp: number;
}

export interface BulkEmbeddedPermissionRequest {
  user_ids: string[];
  permissions: EmbeddedPermissionData[];
  reason?: string;
  metadata?: Record<string, any>;
}

export interface ExtendPermissionRequest {
  permission: string;
  new_expiry_timestamp: number;
  reason?: string;
}

export interface RevokePermissionRequest {
  permission: string;
  reason?: string;
}

export interface ValidatePermissionsRequest {
  permissions: string[];
}

export interface CleanupExpiredRequest {
  dry_run?: boolean;
  batch_size?: number;
}

// Response types
export interface EmbeddedPermissionResponse {
  permission: string;
  expires_at: number;
}

export interface UserPermissionResult {
  user_id: string;
  permissions: string[];
}

export interface UserPermissionError {
  user_id: string;
  error: string;
}

export interface BulkSummary {
  total: number;
  successful: number;
  failed: number;
}

export interface BulkPermissionResponse {
  successful: UserPermissionResult[];
  failed: UserPermissionError[];
  summary: BulkSummary;
}

export interface ExpiredPermission {
  permission: string;
  base_permission: string;
  expired_at: number;
  expired_for: number; // milliseconds
}

export interface ExpiringSoonPermission {
  permission: string;
  base_permission: string;
  expires_at: number;
  expires_in: number; // milliseconds
}

export interface ValidationSummary {
  total: number;
  valid_count: number;
  expired_count: number;
  expiring_soon_count: number;
}

export interface ValidationResult {
  valid: string[];
  expired: ExpiredPermission[];
  expiring_soon: ExpiringSoonPermission[];
  summary: ValidationSummary;
}

export interface PermissionExpiryInfo {
  permission: string;
  base_permission: string;
  expires_at?: number;
  is_expired: boolean;
  time_remaining?: number; // milliseconds
  expires_in?: string; // human readable
}

export interface ExpiryHealthInfo {
  has_expired: boolean;
  has_expiring_soon: boolean;
  next_expiry?: number;
  time_until_next_expiry?: number; // milliseconds
}

export interface ExpiryStatusResponse {
  user_id: string;
  permissions: PermissionExpiryInfo[];
  health: ExpiryHealthInfo;
}

export interface ExtendPermissionResponse {
  old_permission: string;
  new_permission: string;
  extension: number; // milliseconds
}

export interface CleanupDetail {
  user_id: string;
  permission: string;
  expired_at: number;
  status: string; // "cleaned" or "failed"
  error?: string;
}

export interface CleanupResponse {
  cleaned: number;
  failed: number;
  details: CleanupDetail[];
}

export interface PermissionApiErrorResponse {
  error: string;
  message: string;
  details?: string;
}

// Helper types for frontend
export interface PermissionWithHealth {
  permission: string;
  base_permission: string;
  platform: string;
  resource: string;
  action: string;
  expires_at?: number;
  is_expired: boolean;
  time_remaining?: number;
  expires_in?: string;
  health_status: 'healthy' | 'expiring' | 'expired';
}

export interface UserPermissionSummary {
  user_id: string;
  total_permissions: number;
  active_permissions: number;
  expired_permissions: number;
  expiring_soon_permissions: number;
  health_score: number; // 0-100
}

// Permission platforms supported
export type PermissionPlatform = 'epsx' | 'epsx-pay' | 'epsx-token' | 'admin';

// Permission actions
export type PermissionAction = 'view' | 'create' | 'edit' | 'delete' | 'manage' | '*';

// Permission resources
export type PermissionResource = 'users' | 'analytics' | 'payments' | 'tokens' | 'system' | '*';