'use server';

import { cookies } from 'next/headers';
import { revalidatePath } from 'next/cache';
import { env } from '../../config/env';

// Server action utilities for permission-specific operations
async function makePermissionRequest(endpoint: string, options: RequestInit = {}) {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;
  
  const headers = {
    'Content-Type': 'application/json',
    ...(token && { 'Authorization': `Bearer ${token}` }),
    ...options.headers,
  };

  const response = await fetch(`${env.NEXT_PUBLIC_BACKEND_URL}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`HTTP ${response.status}: ${errorText || 'Request failed'}`);
  }

  return response.json();
}

// Permission Template Management
export async function createPermissionTemplate(templateData: {
  name: string;
  description?: string;
  permissions: string[];
  platform: string;
  is_default?: boolean;
}) {
  try {
    const result = await makePermissionRequest('/api/v1/admin/permission-templates', {
      method: 'POST',
      body: JSON.stringify(templateData),
    });

    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error creating permission template:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to create template' };
  }
}

export async function updatePermissionTemplate(templateId: string, templateData: {
  name?: string;
  description?: string;
  permissions?: string[];
  is_default?: boolean;
}) {
  try {
    const result = await makePermissionRequest(`/api/v1/admin/permission-templates/${templateId}`, {
      method: 'PATCH',
      body: JSON.stringify(templateData),
    });

    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error updating permission template:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to update template' };
  }
}

export async function deletePermissionTemplate(templateId: string) {
  try {
    await makePermissionRequest(`/api/v1/admin/permission-templates/${templateId}`, {
      method: 'DELETE',
    });

    revalidatePath('/permissions');
    return { success: true };
  } catch (error) {
    console.error('Error deleting permission template:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to delete template' };
  }
}

// Permission Validation and Analysis
export async function validatePermissionStructure(permission: string) {
  try {
    const result = await makePermissionRequest('/api/v1/admin/permissions/validate', {
      method: 'POST',
      body: JSON.stringify({ permission }),
    });

    return { success: true, data: result };
  } catch (error) {
    console.error('Error validating permission:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to validate permission' };
  }
}

export async function analyzePermissionConflicts(userId: string, newPermissions: string[]) {
  try {
    const result = await makePermissionRequest('/api/v1/admin/permissions/analyze-conflicts', {
      method: 'POST',
      body: JSON.stringify({ 
        user_id: userId,
        new_permissions: newPermissions
      }),
    });

    return { success: true, data: result };
  } catch (error) {
    console.error('Error analyzing permission conflicts:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to analyze conflicts' };
  }
}

// Temporary Permission Management (with embedded timestamps)
export async function grantTemporaryPermission(userId: string, permission: string, expiresAt: Date) {
  try {
    const expiryTimestamp = Math.floor(expiresAt.getTime() / 1000);
    const temporaryPermission = `${permission}:${expiryTimestamp}`;

    const result = await makePermissionRequest(`/api/v1/admin/users/${userId}/permissions/temporary`, {
      method: 'POST',
      body: JSON.stringify({ 
        permission: temporaryPermission,
        expires_at: expiresAt.toISOString()
      }),
    });

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error granting temporary permission:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to grant temporary permission' };
  }
}

export async function extendTemporaryPermission(userId: string, permission: string, newExpiryDate: Date) {
  try {
    const expiryTimestamp = Math.floor(newExpiryDate.getTime() / 1000);
    const updatedPermission = `${permission}:${expiryTimestamp}`;

    const result = await makePermissionRequest(`/api/v1/admin/users/${userId}/permissions/extend`, {
      method: 'PATCH',
      body: JSON.stringify({ 
        permission: updatedPermission,
        new_expiry: newExpiryDate.toISOString()
      }),
    });

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error extending temporary permission:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to extend permission' };
  }
}

// Permission Audit and Cleanup
export async function cleanupExpiredPermissions() {
  try {
    const result = await makePermissionRequest('/api/v1/admin/permissions/cleanup-expired', {
      method: 'POST',
    });

    revalidatePath('/users');
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error cleaning up expired permissions:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to cleanup expired permissions' };
  }
}

export async function auditPermissionChanges(filters?: {
  user_id?: string;
  permission?: string;
  action?: 'granted' | 'revoked' | 'expired';
  start_date?: string;
  end_date?: string;
}) {
  try {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value) queryParams.append(key, value);
      });
    }

    const endpoint = `/api/v1/admin/permissions/audit${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    const result = await makePermissionRequest(endpoint);

    return { success: true, data: result };
  } catch (error) {
    console.error('Error auditing permission changes:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to audit permissions' };
  }
}

// Cross-Platform Permission Management
export async function copyPermissionsAcrossPlatforms(userId: string, fromPlatform: string, toPlatform: string) {
  try {
    const result = await makePermissionRequest('/api/v1/admin/permissions/copy-across-platforms', {
      method: 'POST',
      body: JSON.stringify({ 
        user_id: userId,
        from_platform: fromPlatform,
        to_platform: toPlatform
      }),
    });

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error copying permissions across platforms:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to copy permissions' };
  }
}

// Permission Recommendations
export async function getPermissionRecommendations(userId: string, context?: {
  role?: string;
  department?: string;
  similar_users?: string[];
}) {
  try {
    const result = await makePermissionRequest('/api/v1/admin/permissions/recommendations', {
      method: 'POST',
      body: JSON.stringify({ 
        user_id: userId,
        context
      }),
    });

    return { success: true, data: result };
  } catch (error) {
    console.error('Error getting permission recommendations:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to get recommendations' };
  }
}

// Bulk Permission Operations
export async function applyPermissionTemplate(userIds: string[], templateId: string, replaceExisting: boolean = false) {
  try {
    const result = await makePermissionRequest('/api/v1/admin/permissions/apply-template', {
      method: 'POST',
      body: JSON.stringify({ 
        user_ids: userIds,
        template_id: templateId,
        replace_existing: replaceExisting
      }),
    });

    revalidatePath('/users');
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error applying permission template:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to apply template' };
  }
}

export async function bulkUpdatePermissionExpiry(permissions: Array<{
  user_id: string;
  permission: string;
  new_expiry: Date;
}>) {
  try {
    const permissionUpdates = permissions.map(p => ({
      user_id: p.user_id,
      permission: p.permission,
      new_expiry: p.new_expiry.toISOString(),
      expiry_timestamp: Math.floor(p.new_expiry.getTime() / 1000)
    }));

    const result = await makePermissionRequest('/api/v1/admin/permissions/bulk-update-expiry', {
      method: 'PATCH',
      body: JSON.stringify({ updates: permissionUpdates }),
    });

    revalidatePath('/users');
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error bulk updating permission expiry:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to bulk update expiry' };
  }
}

// Permission Analytics
export async function generatePermissionReport(reportType: 'usage' | 'expiring' | 'conflicts' | 'audit', filters?: {
  platform?: string;
  date_range?: { start: string; end: string };
  user_ids?: string[];
}) {
  try {
    const result = await makePermissionRequest('/api/v1/admin/permissions/reports', {
      method: 'POST',
      body: JSON.stringify({ 
        report_type: reportType,
        filters
      }),
    });

    return { success: true, data: result };
  } catch (error) {
    console.error('Error generating permission report:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to generate report' };
  }
}

// Permission Import/Export
export async function exportPermissions(filters?: {
  platform?: string;
  user_ids?: string[];
  format?: 'json' | 'csv';
}) {
  try {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value) {
          if (Array.isArray(value)) {
            value.forEach(v => queryParams.append(key, v));
          } else {
            queryParams.append(key, value);
          }
        }
      });
    }

    const endpoint = `/api/v1/admin/permissions/export${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    const result = await makePermissionRequest(endpoint);

    return { success: true, data: result };
  } catch (error) {
    console.error('Error exporting permissions:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to export permissions' };
  }
}

export async function importPermissions(importData: {
  permissions: Array<{
    user_id: string;
    permission: string;
    expires_at?: string;
  }>;
  replace_existing?: boolean;
  validate_only?: boolean;
}) {
  try {
    const result = await makePermissionRequest('/api/v1/admin/permissions/import', {
      method: 'POST',
      body: JSON.stringify(importData),
    });

    if (!importData.validate_only) {
      revalidatePath('/users');
      revalidatePath('/permissions');
    }

    return { success: true, data: result };
  } catch (error) {
    console.error('Error importing permissions:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to import permissions' };
  }
}

// Basic permission grant/revoke functions for backward compatibility
export async function grantPermission(userId: string, permission: string) {
  try {
    const result = await makePermissionRequest(`/api/v1/admin/users/${userId}/permissions`, {
      method: 'POST',
      body: JSON.stringify({ permission }),
    });

    revalidatePath('/users');
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error granting permission:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to grant permission' };
  }
}

export async function revokePermission(userId: string, permission: string) {
  try {
    const result = await makePermissionRequest(`/api/v1/admin/users/${userId}/permissions/${encodeURIComponent(permission)}`, {
      method: 'DELETE',
    });

    revalidatePath('/users');
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error revoking permission:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to revoke permission' };
  }
}