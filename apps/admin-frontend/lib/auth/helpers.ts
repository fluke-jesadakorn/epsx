/**
 * Admin Authentication Helpers using Separated Types
 * Enhanced security functions for admin context
 */

'use client';

import {
  AdminSessionData,
  AdminUserProfile,
  SecurityContext,
  isAdminSession,
  isAdminUser,
} from '@/types/auth-separation';

// ============================================================================
// Permission Validation (Admin-Specific)
// ============================================================================

/**
 * Check if admin user has specific permission with security context
 * @param user
 * @param permission
 * @param securityContext
 */
export function hasAdminPermission(
  user: AdminUserProfile | null | undefined,
  permission: string,
  securityContext?: SecurityContext
): boolean {
  if (!user?.permissions) { return false; }

  const permissions = Array.isArray(user.permissions) ? user.permissions : [];

  // Check for admin wildcard permission
  if (permissions.includes('admin:*:*')) {
    return true;
  }

  // Check for exact permission match
  if (permissions.includes(permission)) {
    return true;
  }

  // Check for broader permissions (e.g., admin:users:* covers admin:users:view)
  if (permission.includes(':')) {
    const [platform, resource] = permission.split(':');
    const hasMatchingPattern = permissions.some(p =>
      p === `${platform}:${resource}:*` ||
      p === `${platform}:*:*`
    );

    if (hasMatchingPattern) {
      // Additional security checks for elevated permissions
      if (securityContext && isElevatedPermission(permission)) {
        return validateElevatedAccess(user, securityContext);
      }
      return true;
    }
  }

  return false;
}

/**
 * Check if admin has system-level access
 * @param user
 * @param requiredLevel
 */
export function hasSystemAccess(
  user: AdminUserProfile | null | undefined,
  requiredLevel: 'standard' | 'elevated' | 'critical' = 'standard'
): boolean {
  if (!user) { return false; }

  // Check security level hierarchy
  const levelHierarchy = {
    'standard': 1,
    'elevated': 2,
    'critical': 3
  };

  const userLevel = levelHierarchy[user.securityLevel] || 0;
  const requiredLevelValue = levelHierarchy[requiredLevel] || 1;

  return userLevel >= requiredLevelValue;
}

/**
 * Check if admin can perform bulk operations
 * @param user
 * @param _operation
 */
export function canPerformBulkOperations(
  user: AdminUserProfile | null | undefined,
  _operation: string
): boolean {
  if (!user) { return false; }

  // Bulk operations require elevated security level
  if (user.securityLevel === 'standard') { return false; }

  // Check specific bulk operation permissions
  const bulkPermissions = [
    'admin:users:bulk_manage',
    'admin:permissions:bulk_assign',
    'admin:system:bulk_operations',
    'admin:*:*'
  ];

  return user.permissions.some(p => bulkPermissions.includes(p));
}

// ============================================================================
// Security Context Validation
// ============================================================================

/**
 * Validate elevated access based on security context
 * @param user
 * @param context
 */
function validateElevatedAccess(
  user: AdminUserProfile,
  context: SecurityContext
): boolean {
  // MFA must be verified for elevated operations
  if (!context.mfaVerified) { return false; }

  // Device must be trusted for critical operations
  if (user.securityLevel === 'critical' && !context.deviceTrusted) { return false; }

  return true;
}

/**
 * Check if permission requires elevated security
 * @param permission
 */
function isElevatedPermission(permission: string): boolean {
  const elevatedPatterns = [
    'admin:users:delete',
    'admin:system:manage',
    'admin:security:',
    'admin:permissions:bulk_',
    'admin:*:*'
  ];

  return elevatedPatterns.some(pattern => permission.includes(pattern));
}

// ============================================================================
// Role and Department Validation
// ============================================================================

/**
 * Check if admin user has specific role
 * @param user
 * @param role
 */
export function hasAdminRole(
  user: AdminUserProfile | null | undefined,
  role: string
): boolean {
  if (!user) { return false; }
  return user.role === role || user.role === 'super_admin';
}

/**
 * Check if admin user is in specific department
 * @param user
 * @param department
 */
export function isInDepartment(
  user: AdminUserProfile | null | undefined,
  department: string
): boolean {
  if (!user?.department) { return false; }
  return user.department === department;
}

/**
 * Check if admin has minimum clearance level
 * @param user
 * @param requiredLevel
 */
export function hasClearanceLevel(
  user: AdminUserProfile | null | undefined,
  requiredLevel: number
): boolean {
  if (!user) { return false; }
  return user.clearanceLevel >= requiredLevel;
}

// ============================================================================
// Session Validation
// ============================================================================

/**
 * Validate admin session with security checks
 * @param session
 */
export function validateAdminSession(
  session: AdminSessionData | null | undefined
): boolean {
  if (!session || !isAdminSession(session)) { return false; }

  // Check session expiry
  if (Date.now() > session.expiresAt) { return false; }

  // Validate user profile
  if (!isAdminUser(session.user)) { return false; }

  // Check security context
  const context = session.securityContext;
  if (!context.sessionId || !context.deviceTrusted) { return false; }

  return true;
}

/**
 * Check if session requires security upgrade
 * @param session
 * @param targetOperation
 */
export function requiresSecurityUpgrade(
  session: AdminSessionData,
  targetOperation: string
): boolean {
  const context = session.securityContext;

  // Critical operations require MFA
  if (targetOperation.includes('critical') && !context.mfaVerified) {
    return true;
  }

  // System operations require elevated security
  if (targetOperation.includes('system') && context.securityLevel === 'standard') {
    return true;
  }

  return false;
}

// ============================================================================
// Audit and Logging Helpers
// ============================================================================

/**
 * Create audit context for admin operations
 * @param user
 * @param session
 * @param operation
 */
export function createAuditContext(
  user: AdminUserProfile,
  session: AdminSessionData,
  operation: string
): Record<string, unknown> {
  return {
    adminId: user.id,
    adminEmail: user.email,
    operation,
    securityLevel: user.securityLevel,
    sessionId: session.securityContext.sessionId,
    mfaVerified: session.securityContext.mfaVerified,
    deviceTrusted: session.securityContext.deviceTrusted,
    timestamp: new Date().toISOString(),
    department: user.department,
    clearanceLevel: user.clearanceLevel
  };
}

/**
 * Check if operation should be audited
 * @param operation
 */
export function shouldAuditOperation(operation: string): boolean {
  const auditableOperations = [
    'user_delete',
    'permission_bulk_assign',
    'system_config_change',
    'security_setting_change',
    'data_export',
    'bulk_operation'
  ];

  return auditableOperations.some(op => operation.includes(op));
}

// ============================================================================
// UI Helper Functions
// ============================================================================

/**
 * Get admin user display name with role indicator
 * @param user
 */
export function getAdminDisplayName(user: AdminUserProfile): string {
  const baseName = user.name || user.email.split('@')[0];
  const roleIndicator = user.role === 'super_admin' ? ' (Super Admin)' : '';
  return `${baseName}${roleIndicator}`;
}

/**
 * Get security level badge color
 * @param level
 */
export function getSecurityLevelColor(level: string): string {
  switch (level) {
    case 'critical': return 'bg-red-100 text-red-800';
    case 'elevated': return 'bg-orange-100 text-orange-800';
    case 'standard': return 'bg-green-100 text-green-800';
    default: return 'bg-gray-100 text-gray-800';
  }
}

/**
 * Format admin permissions for display
 * @param permissions
 */
export function formatAdminPermissions(permissions: string[]): string[] {
  return permissions.map(permission => {
    // Handle admin wildcard
    if (permission === 'admin:*:*') { return 'Full Admin Access'; }

    // Format structured permissions
    const [platform, resource, action] = permission.split(':');
    if (platform === 'admin' && resource && action) {
      const resourceLabel = resource.charAt(0).toUpperCase() + resource.slice(1);
      const actionLabel = action === '*' ? 'All Actions' :
        action.charAt(0).toUpperCase() + action.slice(1);
      return `${resourceLabel} - ${actionLabel}`;
    }

    return permission;
  });
}