/**
 * Consolidated Permission Actions
 * Combines: permission-export-import-actions.ts, temporary-permission-actions.ts, 
 * app/actions/permission-actions.ts (embedded-permission-actions.ts removed)
 */

'use server';

import { revalidatePath } from 'next/cache';
import { makeAuthenticatedRequest } from './shared-utils';
import { createSuccessResult, createErrorResult, type ActionResult } from '@/lib/action-utils';

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

export interface PermissionProfile {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  isSystemProfile: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface TemporaryPermission {
  id: string;
  userId: string;
  permission: string;
  grantedAt: string;
  expiresAt: string;
  grantedBy: string;
  reason?: string;
  isActive: boolean;
}

export interface PermissionExportData {
  users: Array<{
    userId: string;
    email: string;
    permissions: string[];
    roles: string[];
    profiles: string[];
  }>;
  profiles: PermissionProfile[];
  metadata: {
    exportedAt: string;
    exportedBy: string;
    version: string;
  };
}

export interface PermissionImportResult {
  successful: number;
  failed: number;
  errors: string[];
  warnings: string[];
}

// ============================================================================
// PERMISSION PROFILE OPERATIONS
// ============================================================================

/**
 * Get all permission profiles
 */
export async function getPermissionProfiles(): Promise<ActionResult<PermissionProfile[]>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/permissions/profiles');
    return createSuccessResult(response.profiles || []);
  } catch (error) {
    console.error('Failed to fetch permission profiles:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch permission profiles');
  }
}

/**
 * Create permission profile
 */
export async function createPermissionProfile(profile: {
  name: string;
  description: string;
  permissions: string[];
}): Promise<ActionResult<PermissionProfile>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/permissions/profiles', {
      method: 'POST',
      body: JSON.stringify(profile)
    });

    revalidatePath('/permissions');
    return createSuccessResult(response, 'Permission profile created successfully');
  } catch (error) {
    console.error('Failed to create permission profile:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to create permission profile');
  }
}

/**
 * Update permission profile
 */
export async function updatePermissionProfile(
  profileId: string,
  updates: {
    name?: string;
    description?: string;
    permissions?: string[];
  }
): Promise<ActionResult<PermissionProfile>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/permissions/profiles/${profileId}`, {
      method: 'PUT',
      body: JSON.stringify(updates)
    });

    revalidatePath('/permissions');
    return createSuccessResult(response, 'Permission profile updated successfully');
  } catch (error) {
    console.error('Failed to update permission profile:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to update permission profile');
  }
}

/**
 * Delete permission profile
 */
export async function deletePermissionProfile(profileId: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/permissions/profiles/${profileId}`, {
      method: 'DELETE'
    });

    revalidatePath('/permissions');
    return createSuccessResult(undefined, 'Permission profile deleted successfully');
  } catch (error) {
    console.error('Failed to delete permission profile:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to delete permission profile');
  }
}

// ============================================================================
// TEMPORARY PERMISSION OPERATIONS
// ============================================================================

/**
 * Grant temporary permission
 */
export async function grantTemporaryPermission(grant: {
  userId: string;
  permission: string;
  expiresAt: string;
  reason?: string;
}): Promise<ActionResult<TemporaryPermission>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/permissions/temporary', {
      method: 'POST',
      body: JSON.stringify(grant)
    });

    revalidatePath('/permissions');
    revalidatePath(`/users/${grant.userId}`);
    return createSuccessResult(response, 'Temporary permission granted successfully');
  } catch (error) {
    console.error('Failed to grant temporary permission:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to grant temporary permission');
  }
}

/**
 * Get temporary permissions for user
 */
export async function getUserTemporaryPermissions(userId: string): Promise<ActionResult<TemporaryPermission[]>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/permissions/temporary/user/${userId}`);
    return createSuccessResult(response.permissions || []);
  } catch (error) {
    console.error('Failed to fetch temporary permissions:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch temporary permissions');
  }
}

/**
 * Revoke temporary permission
 */
export async function revokeTemporaryPermission(permissionId: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/permissions/temporary/${permissionId}`, {
      method: 'DELETE'
    });

    revalidatePath('/permissions');
    return createSuccessResult(undefined, 'Temporary permission revoked successfully');
  } catch (error) {
    console.error('Failed to revoke temporary permission:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to revoke temporary permission');
  }
}

/**
 * Get all active temporary permissions
 */
export async function getActiveTemporaryPermissions(): Promise<ActionResult<TemporaryPermission[]>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/permissions/temporary/active');
    return createSuccessResult(response.permissions || []);
  } catch (error) {
    console.error('Failed to fetch active temporary permissions:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch active temporary permissions');
  }
}

/**
 * Get expiring temporary permissions
 */
export async function getExpiringTemporaryPermissions(daysAhead: number = 7): Promise<ActionResult<TemporaryPermission[]>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/permissions/temporary/expiring?days=${daysAhead}`);
    return createSuccessResult(response.permissions || []);
  } catch (error) {
    console.error('Failed to fetch expiring temporary permissions:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch expiring temporary permissions');
  }
}

// ============================================================================
// EMBEDDED TIMESTAMP PERMISSION OPERATIONS
// ============================================================================

// Embedded permission functions removed - migrated to group-based permissions

// ============================================================================
// PERMISSION EXPORT/IMPORT OPERATIONS
// ============================================================================

/**
 * Export permissions data
 */
export async function exportPermissions(options: {
  includeUsers?: boolean;
  includeProfiles?: boolean;
  format?: 'json' | 'csv';
}): Promise<ActionResult<{
  downloadUrl: string;
  filename: string;
}>> {
  try {
    const params = new URLSearchParams({
      includeUsers: String(options.includeUsers ?? true),
      includeProfiles: String(options.includeProfiles ?? true),
      format: options.format ?? 'json'
    });

    const response = await makeAuthenticatedRequest(`/admin/permissions/export?${params.toString()}`);
    return createSuccessResult(response);
  } catch (error) {
    console.error('Failed to export permissions:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to export permissions');
  }
}

/**
 * Import permissions data
 */
export async function importPermissions(
  file: File,
  options: {
    overwrite?: boolean;
    validateOnly?: boolean;
  }
): Promise<ActionResult<PermissionImportResult>> {
  try {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('options', JSON.stringify(options));

    const response = await fetch('/api/admin/permissions/import', {
      method: 'POST',
      body: formData
    });

    if (!response.ok) {
      throw new Error(`Import failed: ${response.statusText}`);
    }

    const result = await response.json();
    
    if (!options.validateOnly) {
      revalidatePath('/permissions');
      revalidatePath('/users');
    }

    return createSuccessResult(result, 'Permissions import completed');
  } catch (error) {
    console.error('Failed to import permissions:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to import permissions');
  }
}

/**
 * Validate permissions import file
 */
export async function validatePermissionsImport(file: File): Promise<ActionResult<{
  valid: boolean;
  errors: string[];
  warnings: string[];
  summary: {
    userCount: number;
    profileCount: number;
    permissionCount: number;
  };
}>> {
  try {
    const formData = new FormData();
    formData.append('file', file);

    const response = await fetch('/api/admin/permissions/validate', {
      method: 'POST',
      body: formData
    });

    if (!response.ok) {
      throw new Error(`Validation failed: ${response.statusText}`);
    }

    const result = await response.json();
    return createSuccessResult(result);
  } catch (error) {
    console.error('Failed to validate permissions import:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to validate import file');
  }
}

// ============================================================================
// PERMISSION ANALYTICS AND REPORTING
// ============================================================================

/**
 * Get permission usage statistics
 */
export async function getPermissionStats(): Promise<ActionResult<{
  totalPermissions: number;
  mostUsedPermissions: Array<{
    permission: string;
    userCount: number;
  }>;
  leastUsedPermissions: Array<{
    permission: string;
    userCount: number;
  }>;
  temporaryPermissionCount: number;
  embeddedPermissionCount: number;
}>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/permissions/stats');
    return createSuccessResult(response);
  } catch (error) {
    console.error('Failed to fetch permission stats:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch permission statistics');
  }
}

/**
 * Get permission health report
 */
export async function getPermissionHealthReport(): Promise<ActionResult<{
  overPrivilegedUsers: number;
  unusedPermissions: string[];
  expiringSoon: number;
  securityRisks: Array<{
    type: string;
    description: string;
    severity: 'low' | 'medium' | 'high';
    affectedUsers: number;
  }>;
}>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/permissions/health');
    return createSuccessResult(response);
  } catch (error) {
    console.error('Failed to fetch permission health report:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch permission health report');
  }
}