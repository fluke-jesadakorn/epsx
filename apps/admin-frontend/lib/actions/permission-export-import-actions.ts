'use server';

import { revalidatePath } from 'next/cache';
import { getServerSession } from '@/lib/auth';

// Get bearer token from NextAuth session
const getBearerToken = async () => {
  const session = await getServerSession();
  return (session as any)?.accessToken || null;
};

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
  };
}

async function makeApiRequest<T>(url: string, options: RequestInit = {}): Promise<ApiResponse<T>> {
  try {
    const token = await getBearerToken();
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } };
    }

    const fullUrl = url.startsWith('/') ? `${BACKEND_URL}/api/v1${url}` : url;
    
    const response = await fetch(fullUrl, {
      ...options,
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      return {
        success: false,
        error: {
          code: 'API_ERROR',
          message: `API request failed: ${response.statusText}`,
        },
      };
    }

    const data = await response.json();
    return { success: true, data };
  } catch (error) {
    console.error('API request error:', error);
    return {
      success: false,
      error: {
        code: 'NETWORK_ERROR',
        message: 'Network request failed',
      },
    };
  }
}

export interface PermissionExportData {
  userId: string;
  userEmail: string;
  exportedAt: string;
  exportedBy: string;
  version: string;
  format: 'json' | 'csv' | 'xlsx';
  includeHistory?: boolean;
  includeTemporary?: boolean;
  permissions: {
    roles: Array<{
      name: string;
      isActive: boolean;
      assignedAt?: string;
      assignedBy?: string;
    }>;
    customPermissions: Array<{
      resource: string;
      action: string;
      assignedAt?: string;
      assignedBy?: string;
    }>;
    profiles: Array<{
      id: string;
      name: string;
      isActive: boolean;
      assignedAt?: string;
      assignedBy?: string;
    }>;
    temporaryPermissions?: Array<{
      id: string;
      permission: string;
      resource: string;
      action: string;
      grantedAt: string;
      expiresAt: string;
      status: string;
    }>;
    permissionHistory?: Array<{
      action: string;
      resource: string;
      permission: string;
      timestamp: string;
      performedBy: string;
    }>;
  };
}

export interface PermissionImportData {
  userId: string;
  importData: PermissionExportData;
  replaceExisting: boolean;
  importOptions: {
    includeRoles: boolean;
    includeCustomPermissions: boolean;
    includeProfiles: boolean;
    includeTemporary: boolean;
    dryRun?: boolean;
  };
}

export interface BulkExportData {
  userIds: string[];
  format: 'json' | 'csv' | 'xlsx';
  includeHistory?: boolean;
  includeTemporary?: boolean;
  groupBy?: 'user' | 'role' | 'profile';
}

export interface ExportSummary {
  totalUsers: number;
  totalRoles: number;
  totalCustomPermissions: number;
  totalProfiles: number;
  totalTemporaryPermissions: number;
  exportedAt: string;
  exportedBy: string;
  format: string;
}

export interface ImportValidationResult {
  isValid: boolean;
  warnings: string[];
  errors: string[];
  preview: {
    rolesToAdd: number;
    rolesToRemove: number;
    permissionsToAdd: number;
    permissionsToRemove: number;
    profilesToAdd: number;
    profilesToRemove: number;
  };
}

// Export single user permissions
export async function exportUserPermissions(
  userId: string,
  format: 'json' | 'csv' | 'xlsx' = 'json',
  options?: {
    includeHistory?: boolean;
    includeTemporary?: boolean;
  }
): Promise<ApiResponse<PermissionExportData>> {
  const params = new URLSearchParams({
    format,
    ...(options?.includeHistory && { include_history: 'true' }),
    ...(options?.includeTemporary && { include_temporary: 'true' }),
  });

  return makeApiRequest<PermissionExportData>(`/admin/users/${userId}/permissions/export?${params}`, {
    method: 'GET',
  });
}

// Export multiple users' permissions (bulk export)
export async function bulkExportUserPermissions(
  data: BulkExportData
): Promise<ApiResponse<{ downloadUrl: string; summary: ExportSummary }>> {
  return makeApiRequest<{ downloadUrl: string; summary: ExportSummary }>('/admin/permissions/bulk-export', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

// Validate import data before applying
export async function validatePermissionImport(
  userId: string,
  importData: PermissionExportData,
  options: {
    includeRoles: boolean;
    includeCustomPermissions: boolean;
    includeProfiles: boolean;
    includeTemporary: boolean;
  }
): Promise<ApiResponse<ImportValidationResult>> {
  return makeApiRequest<ImportValidationResult>(`/admin/users/${userId}/permissions/validate-import`, {
    method: 'POST',
    body: JSON.stringify({
      import_data: importData,
      options,
    }),
  });
}

// Import permissions for a user
export async function importUserPermissions(
  data: PermissionImportData
): Promise<ApiResponse<{
  summary: {
    rolesAdded: number;
    rolesRemoved: number;
    permissionsAdded: number;
    permissionsRemoved: number;
    profilesAdded: number;
    profilesRemoved: number;
  };
  warnings: string[];
}>> {
  const result = await makeApiRequest<{
    summary: {
      rolesAdded: number;
      rolesRemoved: number;
      permissionsAdded: number;
      permissionsRemoved: number;
      profilesAdded: number;
      profilesRemoved: number;
    };
    warnings: string[];
  }>(`/admin/users/${data.userId}/permissions/import`, {
    method: 'POST',
    body: JSON.stringify({
      import_data: data.importData,
      replace_existing: data.replaceExisting,
      import_options: data.importOptions,
    }),
  });

  if (result.success) {
    revalidatePath(`/users/${data.userId}/permissions`);
    revalidatePath('/users');
  }

  return result;
}

// Export permission templates for reuse
export async function exportPermissionTemplates(
  templateIds?: string[]
): Promise<ApiResponse<Array<{
  id: string;
  name: string;
  description: string;
  permissions: {
    roles: string[];
    customPermissions: Array<{ resource: string; action: string }>;
    profiles: string[];
  };
  createdAt: string;
  createdBy: string;
}>>> {
  const params = templateIds?.length ? `?template_ids=${templateIds.join(',')}` : '';
  
  return makeApiRequest<Array<{
    id: string;
    name: string;
    description: string;
    permissions: {
      roles: string[];
      customPermissions: Array<{ resource: string; action: string }>;
      profiles: string[];
    };
    createdAt: string;
    createdBy: string;
  }>>(`/admin/permission-templates/export${params}`, {
    method: 'GET',
  });
}

// Import permission templates
export async function importPermissionTemplates(
  templates: Array<{
    name: string;
    description: string;
    permissions: {
      roles: string[];
      customPermissions: Array<{ resource: string; action: string }>;
      profiles: string[];
    };
  }>
): Promise<ApiResponse<{
  created: number;
  skipped: number;
  errors: Array<{ template: string; error: string }>;
}>> {
  return makeApiRequest<{
    created: number;
    skipped: number;
    errors: Array<{ template: string; error: string }>;
  }>('/admin/permission-templates/import', {
    method: 'POST',
    body: JSON.stringify({ templates }),
  });
}

// Generate permission audit report
export async function generatePermissionAuditReport(
  options: {
    userIds?: string[];
    dateFrom?: string;
    dateTo?: string;
    format: 'json' | 'csv' | 'xlsx' | 'pdf';
    includeChanges?: boolean;
    includeCurrentState?: boolean;
    groupBy?: 'user' | 'date' | 'action';
  }
): Promise<ApiResponse<{ downloadUrl: string; reportId: string }>> {
  return makeApiRequest<{ downloadUrl: string; reportId: string }>('/admin/permissions/audit-report', {
    method: 'POST',
    body: JSON.stringify(options),
  });
}

// Backup all system permissions
export async function createSystemPermissionBackup(
  options?: {
    format?: 'json' | 'sql';
    includeHistory?: boolean;
    includeTemporary?: boolean;
    compression?: 'none' | 'gzip' | 'zip';
  }
): Promise<ApiResponse<{
  backupId: string;
  downloadUrl: string;
  size: number;
  checksum: string;
  createdAt: string;
}>> {
  return makeApiRequest<{
    backupId: string;
    downloadUrl: string;
    size: number;
    checksum: string;
    createdAt: string;
  }>('/admin/permissions/system-backup', {
    method: 'POST',
    body: JSON.stringify(options || {}),
  });
}

// Restore from system backup
export async function restoreFromSystemBackup(
  backupId: string,
  options?: {
    dryRun?: boolean;
    restoreUsers?: boolean;
    restoreRoles?: boolean;
    restoreProfiles?: boolean;
    restoreHistory?: boolean;
  }
): Promise<ApiResponse<{
  restored: {
    users: number;
    roles: number;
    profiles: number;
    permissions: number;
  };
  warnings: string[];
  errors: string[];
}>> {
  const result = await makeApiRequest<{
    restored: {
      users: number;
      roles: number;
      profiles: number;
      permissions: number;
    };
    warnings: string[];
    errors: string[];
  }>(`/admin/permissions/system-backup/${backupId}/restore`, {
    method: 'POST',
    body: JSON.stringify(options || {}),
  });

  if (result.success && !options?.dryRun) {
    revalidatePath('/users');
    revalidatePath('/analytics');
  }

  return result;
}