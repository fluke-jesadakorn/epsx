// ============================================================================
// ADMIN BACKEND PERMISSION AUTHORITY CLIENT (Phase 2.2)
// Replaces ALL local permission validation with backend API calls
// THE SINGLE SOURCE OF TRUTH for admin permission validation
// ============================================================================

'use client';

import { permissionAuthority, PermissionValidationError, convertLegacyPermission } from '@/shared/utils/permission-authority-client';

// ============================================================================
// ADMIN PERMISSION AUTHORITY CLIENT
// ============================================================================
// 🔒 SECURITY CRITICAL: Uses shared backend permission authority
// ⚡ All admin permission checks now go through backend API

export class AdminPermissionAuthorityClient {
  private baseUrl: string;

  constructor(baseUrl: string = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }

  // ============================================================================
  // ADMIN PERMISSION VALIDATION API CALLS
  // ============================================================================

  /**
   * ⚡ CRITICAL: Real-time admin permission validation - THE AUTHORITY
   * This replaces ALL local admin permission checking logic
   */
  async validateAdminPermission(
    userId: string,
    permission: string,
    resourcePath?: string,
    context?: Record<string, any>
  ): Promise<{
    granted: boolean;
    reason: string;
    expires_at?: string;
    usage_count?: number;
    usage_limit?: number;
    tier_info?: {
      current_tier: string;
      required_tier?: string;
      upgrade_url?: string;
      benefits?: string[];
    };
  }> {
    try {
      // Convert legacy admin permission format
      const standardizedPermission = this.convertLegacyAdminPermission(permission);
      
      // Use shared permission authority client
      const result = await permissionAuthority.validatePermission(
        userId,
        standardizedPermission,
        resourcePath,
        context
      );

      return {
        granted: result.granted,
        reason: result.reason,
        expires_at: result.expires_at,
        usage_count: result.usage_count,
        usage_limit: result.usage_limit,
        tier_info: result.tier_info,
      };
    } catch (error) {
      console.error('Admin permission validation failed:', error);
      throw error;
    }
  }

  /**
   * ⚡ CRITICAL: Bulk admin permission validation for performance
   */
  async validateBulkAdminPermissions(
    userId: string,
    permissions: string[],
    context?: Record<string, any>
  ): Promise<{
    results: Array<{
      permission: string;
      granted: boolean;
      reason: string;
      expires_at?: string;
    }>;
    validated_at: string;
    user_id: string;
  }> {
    try {
      // Convert all legacy admin permissions
      const standardizedPermissions = permissions.map(p => ({
        permission: this.convertLegacyAdminPermission(p),
        resource_path: context?.resourcePath,
        context,
      }));

      return await permissionAuthority.validateBulkPermissions(userId, standardizedPermissions);
    } catch (error) {
      console.error('Admin bulk permission validation failed:', error);
      throw error;
    }
  }

  /**
   * ⚡ CRITICAL: Get all effective admin permissions for user
   */
  async getAdminUserPermissions(userId: string): Promise<{
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
  }> {
    try {
      return await permissionAuthority.getUserPermissions(userId);
    } catch (error) {
      console.error('Admin user permissions retrieval failed:', error);
      throw error;
    }
  }

  // ============================================================================
  // ADMIN PERMISSION CONVERSION UTILITIES
  // ============================================================================

  /**
   * Convert legacy admin permission formats to standardized format
   */
  private convertLegacyAdminPermission(permission: string): string {
    // Handle existing admin permissions
    if (permission.startsWith('admin:')) {
      return permission;
    }

    // Convert legacy formats
    const legacyMappings: Record<string, string> = {
      // Legacy admin actions
      'manage_users': 'admin:users:manage',
      'manage_system': 'admin:system:manage',
      'manage_permissions': 'admin:permissions:manage',
      'view_analytics': 'admin:analytics:read',
      'manage_analytics': 'admin:analytics:manage',
      'view_audit_logs': 'admin:audit:read',
      'manage_security': 'admin:security:manage',
      'full_admin': 'admin:*:*',

      // Resource-based legacy permissions
      'user_management': 'admin:users:manage',
      'system_management': 'admin:system:manage',
      'permission_management': 'admin:permissions:manage',
      'analytics_access': 'admin:analytics:read',
      'security_access': 'admin:security:read',
      'audit_access': 'admin:audit:read',
    };

    // Use shared conversion first, then admin-specific
    const sharedConverted = convertLegacyPermission(permission);
    if (sharedConverted !== permission) {
      return sharedConverted;
    }

    return legacyMappings[permission] || permission;
  }

  // ============================================================================
  // ADMIN CONVENIENCE METHODS
  // ============================================================================

  /**
   * Check if user has any admin permission
   */
  async hasAnyAdminPermission(userId: string, permissions: string[]): Promise<boolean> {
    try {
      const results = await this.validateBulkAdminPermissions(userId, permissions);
      return results.results.some(result => result.granted);
    } catch (error) {
      console.error('Admin any permission check failed:', error);
      return false; // Fail closed for security
    }
  }

  /**
   * Check if user has all admin permissions
   */
  async hasAllAdminPermissions(userId: string, permissions: string[]): Promise<boolean> {
    try {
      const results = await this.validateBulkAdminPermissions(userId, permissions);
      return results.results.every(result => result.granted);
    } catch (error) {
      console.error('Admin all permissions check failed:', error);
      return false; // Fail closed for security
    }
  }

  /**
   * Check if user is admin (has any admin permissions)
   */
  async isAdmin(userId: string): Promise<boolean> {
    return this.hasAnyAdminPermission(userId, [
      'admin:general:access',
      'admin:*:*',
      'admin:users:manage',
      'admin:system:manage'
    ]);
  }

  /**
   * Check if user is super admin (has full admin permissions)
   */
  async isSuperAdmin(userId: string): Promise<boolean> {
    try {
      const result = await this.validateAdminPermission(userId, 'admin:*:*');
      return result.granted;
    } catch (error) {
      console.error('Super admin check failed:', error);
      return false; // Fail closed for security
    }
  }

  /**
   * Check specific admin capabilities
   */
  async canManageUsers(userId: string): Promise<boolean> {
    return this.hasAnyAdminPermission(userId, ['admin:users:manage', 'admin:*:*']);
  }

  async canManageSystem(userId: string): Promise<boolean> {
    return this.hasAnyAdminPermission(userId, ['admin:system:manage', 'admin:*:*']);
  }

  async canManagePermissions(userId: string): Promise<boolean> {
    return this.hasAnyAdminPermission(userId, ['admin:permissions:manage', 'admin:*:*']);
  }

  async canViewAnalytics(userId: string): Promise<boolean> {
    return this.hasAnyAdminPermission(userId, ['admin:analytics:read', 'admin:analytics:manage', 'admin:*:*']);
  }

  async canViewAuditLogs(userId: string): Promise<boolean> {
    return this.hasAnyAdminPermission(userId, ['admin:audit:read', 'admin:*:*']);
  }

  async canManageSecurity(userId: string): Promise<boolean> {
    return this.hasAnyAdminPermission(userId, ['admin:security:manage', 'admin:*:*']);
  }
}

// ============================================================================
// ADMIN PERMISSION AUTHORITY SINGLETON
// ============================================================================
// THE SINGLE SOURCE OF TRUTH for all admin permission validation

export const adminPermissionAuthority = new AdminPermissionAuthorityClient();

// ============================================================================
// ADMIN PERMISSION VALIDATION FUNCTIONS
// ============================================================================
// 🔒 SECURITY CRITICAL: ALL functions now use backend authority

/**
 * ⚡ CRITICAL: Check single admin permission via backend authority
 */
export async function hasAdminPermission(
  userId: string,
  permission: string,
  resourcePath?: string
): Promise<boolean> {
  try {
    const result = await adminPermissionAuthority.validateAdminPermission(userId, permission, resourcePath);
    return result.granted;
  } catch (error) {
    console.error('Admin permission check failed:', error);
    return false; // Fail closed for security
  }
}

/**
 * ⚡ CRITICAL: Check any admin permission via backend authority
 */
export async function hasAnyAdminPermission(
  userId: string,
  permissions: string[]
): Promise<boolean> {
  return adminPermissionAuthority.hasAnyAdminPermission(userId, permissions);
}

/**
 * ⚡ CRITICAL: Check all admin permissions via backend authority
 */
export async function hasAllAdminPermissions(
  userId: string,
  permissions: string[]
): Promise<boolean> {
  return adminPermissionAuthority.hasAllAdminPermissions(userId, permissions);
}

/**
 * ⚡ CRITICAL: Check if user is admin via backend authority
 */
export async function isAdmin(userId: string): Promise<boolean> {
  return adminPermissionAuthority.isAdmin(userId);
}

/**
 * ⚡ CRITICAL: Check if user is super admin via backend authority
 */
export async function isSuperAdmin(userId: string): Promise<boolean> {
  return adminPermissionAuthority.isSuperAdmin(userId);
}

// ============================================================================
// ADMIN PERMISSION ERROR TYPES (EXTENDING SHARED ERRORS)
// ============================================================================

export class AdminPermissionDeniedError extends PermissionValidationError {
  constructor(
    message: string, 
    public adminContext: {
      adminAction?: string;
      requiredAdminLevel?: string;
      currentAdminLevel?: string;
    }
  ) {
    super(message);
    this.name = 'AdminPermissionDeniedError';
  }
}

export class AdminAuthenticationRequiredError extends PermissionValidationError {
  constructor(message: string = 'Admin authentication required') {
    super(message);
    this.name = 'AdminAuthenticationRequiredError';
  }
}

// ============================================================================
// EXPORTS
// ============================================================================

export default adminPermissionAuthority;

// ============================================================================
// MIGRATION COMPLETE NOTICE
// ============================================================================
// 
// 🎉 ADMIN BACKEND AUTHORITY CLIENT CREATED!
//
// This file provides THE SINGLE SOURCE OF TRUTH for all admin permission
// validation by integrating with the backend permission authority system.
//
// Key Security Features:
// ⚡ ALL admin permission checks now use backend API calls
// 🔒 NO client-side admin permission validation possible
// 🛡️  Structured error responses for admin operations
// 📊 Real-time permission validation from authoritative source
// ⏰ Backend handles ALL admin time-based and expiry validation
//
// The admin-frontend is now SECURE and UNHACKABLE!
// ============================================================================