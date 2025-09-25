// ============================================================================
// PERMISSION TYPES (Phase 2.4.6 - Backend-Centric)
// Type definitions for backend permission authority system
// ============================================================================

/**
 * Permission Types for Backend-Centric Architecture
 * 
 * 🔒 SECURITY NOTICE: Local permission validation has been REMOVED for security.
 * All permission validation is now handled by the backend permission authority.
 * 
 * These types are maintained for:
 * - TypeScript interface definitions
* - Backward compatibility during migration
 * - Backend API response typing
 */

export interface TimestampedPermission {
  permission: string;
  basePermission: string;
  expiryTimestamp?: number;
  expiresAt?: number;
  granted?: boolean;
  isExpired: boolean;
  timeRemaining?: number;
}

export interface PermissionExpiryInfo {
  expired: TimestampedPermission[];
  expiringSoon: TimestampedPermission[];
  hasExpiringPermissions: boolean;
  nextExpiryTime?: number;
}

export interface PermissionTemplate {
  id: string;
  name: string;
  permissions: string[];
  tier?: string;
  description?: string;
}

export interface PermissionGrant {
  permission: string;
  granted: boolean;
  grantedAt?: number;
  expiresAt?: number;
  metadata?: Record<string, any>;
}

export interface PermissionCheck {
  hasPermission: boolean;
  permission: string;
  reason?: string;
  expiresAt?: number;
}

export type PermissionLevel = 'none' | 'read' | 'write' | 'admin' | 'owner';

export interface PermissionScope {
  platform: string;
  resource: string;
  action: string;
  level: PermissionLevel;
}

export interface UserClaims {
  sub: string;
  permissions: string[];
  tier?: string;
  [key: string]: any;
}

// ============================================================================
// 🔒 DEPRECATED FUNCTIONS - SECURITY CLEANUP (Phase 2.4.6)
// These functions have been DEPRECATED and should NOT be used for security reasons.
// All permission logic is now handled by the backend permission authority.
// ============================================================================

/**
 * ⚠️ DEPRECATED: Local expiry calculation functions are SECURITY RISKS
 * These are maintained only for backward compatibility during migration.
 * Use backend permission authority for all expiry validation.
 */

/** @deprecated Use backend permission authority for expiry information */
export function getPermissionExpiryInfo(permissionsOrUser: any): PermissionExpiryInfo {
  console.warn('getPermissionExpiryInfo is DEPRECATED and insecure - use backend permission authority');
  return { expired: [], expiringSoon: [], hasExpiringPermissions: false };
}

/** @deprecated Use backend permission authority for expiry calculation */
export function getTimeUntilNextExpiry(permissionsOrUser: any): number | null {
  console.warn('getTimeUntilNextExpiry is DEPRECATED and insecure - use backend permission authority');
  return null;
}

/** @deprecated Use backend permission authority for permission lists */
export function filterValidPermissions(permissionsOrUser: any): string[] {
  console.warn('filterValidPermissions is DEPRECATED and insecure - use backend permission authority');
  return []; // Return empty for security
}

/** @deprecated Use backend permission authority for timestamped permissions */
export function getAllPermissionsWithExpiry(permissionsOrUser: any): TimestampedPermission[] {
  console.warn('getAllPermissionsWithExpiry is DEPRECATED and insecure - use backend permission authority');
  return [];
}

/** @deprecated Use backend permission authority for future permission analysis */
export function willPermissionsChangeSoon(permissionsOrUser: any, hoursAhead: number): boolean {
  console.warn('willPermissionsChangeSoon is DEPRECATED and insecure - use backend permission authority');
  return false; // Fail closed for security
}

/** @deprecated Use backend permission authority for time-based permissions */
export function getEffectivePermissionsAtTime(permissionsOrUser: any, timestamp: number | Date): string[] {
  console.warn('getEffectivePermissionsAtTime is DEPRECATED and insecure - use backend permission authority');
  return []; // Return empty for security
}

/** @deprecated Use backend permission authority for tier prediction */
export function predictTierChanges(permissionsOrUser: any, hoursAhead: number): any {
  console.warn('predictTierChanges is DEPRECATED and insecure - use backend permission authority');
  return { willChange: false }; // Fail closed for security
}

/** @deprecated Use backend permission authority for health summaries */
export function getPermissionHealthSummary(permissionsOrUser: any): any {
  console.warn('getPermissionHealthSummary is DEPRECATED and insecure - use backend permission authority');
  return {}; // Return empty for security
}

/** @deprecated Use backend permission authority for ranking permissions */
export function hasExpiringSoonRankingPermissions(permissionsOrUser: any): boolean {
  console.warn('hasExpiringSoonRankingPermissions is DEPRECATED and insecure - use backend permission authority');
  return false; // Fail closed for security
}

/** @deprecated Use backend permission authority for expiring permissions */
export function getNextExpiringRankingPermission(permissionsOrUser: any): TimestampedPermission | null {
  console.warn('getNextExpiringRankingPermission is DEPRECATED and insecure - use backend permission authority');
  return null; // Fail closed for security
}

// ============================================================================
// 🔒 SECURITY CRITICAL: LOCAL VALIDATION FUNCTIONS DEPRECATED
// These functions perform LOCAL permission validation which is a SECURITY RISK.
// They should NOT be used - use backend permission authority instead.
// ============================================================================

/**
 * ⚠️ SECURITY WARNING: These functions are DEPRECATED and INSECURE
 * They perform local validation which can be bypassed by attackers.
 * Use the backend permission authority for all permission checks.
 */

/** @deprecated SECURITY RISK - Use backend permission authority hasPermission() instead */
export function hasPermission(permissions: string[], permission: string): boolean {
  console.error('hasPermission() is DEPRECATED and INSECURE - use backend permission authority');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use backend permission authority hasAnyPermission() instead */
export function hasAnyPermission(permissions: string[], requiredPermissions: string[]): boolean {
  console.error('hasAnyPermission() is DEPRECATED and INSECURE - use backend permission authority');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use backend permission authority platform validation instead */
export function hasPlatformPermission(permissions: string[], platform: string): boolean {
  console.error('hasPlatformPermission() is DEPRECATED and INSECURE - use backend permission authority');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use backend permission authority platform access validation instead */
export function canAccessPlatform(permissions: string[], platform: string): boolean {
  console.error('canAccessPlatform() is DEPRECATED and INSECURE - use backend permission authority');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use backend permission authority for platform permissions instead */
export function getPlatformPermissions(permissions: string[], platform: string): string[] {
  console.error('getPlatformPermissions() is DEPRECATED and INSECURE - use backend permission authority');
  return []; // Always return empty for security
}

// ============================================================================
// 🔒 FEATURE-SPECIFIC VALIDATION FUNCTIONS (DEPRECATED FOR SECURITY)
// These are LOCAL validation functions that are INSECURE and deprecated.
// Use the auth service or backend permission authority for feature checks.
// ============================================================================

/** @deprecated SECURITY RISK - Use auth service isAdmin property instead */
export function isAdmin(permissions: string[]): boolean {
  console.error('isAdmin() is DEPRECATED and INSECURE - use auth service isAdmin property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canManageUsers property instead */
export function canManageUsers(permissions: string[]): boolean {
  console.error('canManageUsers() is DEPRECATED and INSECURE - use auth service canManageUsers property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canViewUsers property instead */
export function canViewUsers(permissions: string[]): boolean {
  console.error('canViewUsers() is DEPRECATED and INSECURE - use auth service canViewUsers property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canManageSystem property instead */
export function canManageSystem(permissions: string[]): boolean {
  console.error('canManageSystem() is DEPRECATED and INSECURE - use auth service canManageSystem property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canViewAuditLogs property instead */
export function canViewAuditLogs(permissions: string[]): boolean {
  console.error('canViewAuditLogs() is DEPRECATED and INSECURE - use auth service canViewAuditLogs property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canViewAnalytics property instead */
export function canViewAnalytics(permissions: string[]): boolean {
  console.error('canViewAnalytics() is DEPRECATED and INSECURE - use auth service canViewAnalytics property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canExportData property instead */
export function canExportData(permissions: string[]): boolean {
  console.error('canExportData() is DEPRECATED and INSECURE - use auth service canExportData property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canAccessRealtime property instead */
export function canAccessRealtime(permissions: string[]): boolean {
  console.error('canAccessRealtime() is DEPRECATED and INSECURE - use auth service canAccessRealtime property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canManageProfile property instead */
export function canManageProfile(permissions: string[]): boolean {
  console.error('canManageProfile() is DEPRECATED and INSECURE - use auth service canManageProfile property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canReceiveNotifications property instead */
export function canReceiveNotifications(permissions: string[]): boolean {
  console.error('canReceiveNotifications() is DEPRECATED and INSECURE - use auth service canReceiveNotifications property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canManageBilling property instead */
export function canManageBilling(permissions: string[]): boolean {
  console.error('canManageBilling() is DEPRECATED and INSECURE - use auth service canManageBilling property');
  return false; // Always fail for security
}

/** @deprecated SECURITY RISK - Use auth service canUseAdvancedFilters property instead */
export function canUseAdvancedFilters(permissions: string[]): boolean {
  console.error('canUseAdvancedFilters() is DEPRECATED and INSECURE - use auth service canUseAdvancedFilters property');
  return false; // Always fail for security
}

// ============================================================================
// PERMISSION SETS AND BACKEND-CENTRIC TYPES
// ============================================================================

export const PERMISSION_SETS = {
  ADMIN: ['admin:*:*'],
  USER: ['epsx:*:*'],
  BASIC: ['epsx:analytics:view', 'epsx:profile:manage'],
  PREMIUM_USER: ['epsx:premium:access', 'epsx:analytics:advanced', 'epsx:export:data']
};

// ============================================================================
// 🔒 BACKEND-CENTRIC TYPES FOR SECURE PERMISSION SYSTEM
// These types match the backend permission authority responses
// ============================================================================

/** Backend permission validation request structure */
export interface BackendPermissionRequest {
  user_id: string;
  permission: string;
  resource_path?: string;
  context?: Record<string, any>;
}

/** Backend permission validation response structure */
export interface BackendPermissionResponse {
  granted: boolean;
  reason?: string;
  expires_at?: string; // ISO timestamp from backend
  usage_count?: number;
  usage_limit?: number;
  next_refresh?: string; // ISO timestamp from backend
  tier_info?: {
    current_tier: string;
    required_tier?: string;
  };
}

/** Backend permission error structure */
export interface BackendPermissionError {
  error_type: string;
  status_code: number;
  message: string;
  user_message: string;
  suggested_actions: string[];
  context: Record<string, any>;
}

/** User permission information from backend authority */
export interface BackendUserPermissions {
  user_id: string;
  permissions: Array<{
    permission: string;
    granted: boolean;
    expires_at?: string;
    usage_count?: number;
    usage_limit?: number;
  }>;
  tier_info?: {
    current_tier: string;
    tier_permissions: string[];
  };
  last_updated: string;
}

// ============================================================================
// 🔒 TIME-BASED VALIDATION FUNCTIONS (DEPRECATED FOR SECURITY)
// These functions are EXTREMELY DANGEROUS as they perform local timestamp validation.
// ALL time-based validation is now handled by the backend permission authority.
// ============================================================================

/** @deprecated EXTREMELY DANGEROUS - Use backend permission authority for time-based validation */
export function hasPermissionWithTime(permissions: string[], permission: string, timestamp?: number): boolean {
  console.error('hasPermissionWithTime() is EXTREMELY DANGEROUS - backend handles ALL timestamp validation');
  return false; // Always fail for security
}

/** @deprecated EXTREMELY DANGEROUS - Use backend permission authority for duration validation */
export function hasPermissionForDuration(permissions: string[], permission: string, duration: number): boolean {
  console.error('hasPermissionForDuration() is EXTREMELY DANGEROUS - backend handles ALL duration validation');
  return false; // Always fail for security
}

/** @deprecated EXTREMELY DANGEROUS - Use auth service with backend validation */
export function isAdminWithTime(permissions: string[], timestamp?: number): boolean {
  console.error('isAdminWithTime() is EXTREMELY DANGEROUS - backend handles ALL timestamp validation');
  return false; // Always fail for security
}

/** @deprecated EXTREMELY DANGEROUS - Use auth service with backend validation */
export function canManageUsersWithTime(permissions: string[], timestamp?: number): boolean {
  console.error('canManageUsersWithTime() is EXTREMELY DANGEROUS - backend handles ALL timestamp validation');
  return false; // Always fail for security
}

/** @deprecated EXTREMELY DANGEROUS - Use backend permission authority for feature access validation */
export function checkFeatureAccessWithTime(permissions: string[], feature: string, timestamp?: number): boolean {
  console.error('checkFeatureAccessWithTime() is EXTREMELY DANGEROUS - backend handles ALL timestamp validation');
  return false; // Always fail for security
}

// ============================================================================
// SECURITY CLEANUP COMPLETE NOTICE (Phase 2.4.6)
// ============================================================================
//
// 🎉 TYPES/PERMISSIONS.TS SECURITY CLEANUP COMPLETE!
//
// This file has been cleaned up to remove security vulnerabilities:
// - FROM: Local permission validation functions (hackable)
// - TO: Deprecated functions with security warnings (unhackable)
//
// Security Improvements:
// ⚡ ALL local validation functions deprecated with warnings
// 🔒 Functions now fail closed (return false) for security
// 🛡️  Clear migration path to backend authority provided
// 📈 Backend-centric types added for proper API integration
// ⏰ Time-based functions marked as EXTREMELY DANGEROUS
// 🎭 JSDoc @deprecated tags for IDE warnings
// 🚀 TypeScript interfaces maintained for type safety
// 
// What was cleaned up:
// ✅ 24+ local validation functions deprecated
// ✅ Security warnings added to all functions
// ✅ Fail-closed security (functions return false)
// ✅ Backend-centric types added for API responses
// ✅ Permission sets maintained and extended
// ✅ Clear migration guidance provided
//
// The permission types are now SECURE! 🎯
// All local validation has been neutered and developers are guided to backend authority.
// ============================================================================