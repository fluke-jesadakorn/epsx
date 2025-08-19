'use server';

import { revalidatePath } from 'next/cache';
import { env } from '@/config/env';

const API_BASE = env.BACKEND_URL;

export interface TemporaryPermission {
  id: string;
  user_id: string;
  permission: string;
  resource: string;
  action: string;
  granted_at: string;
  expires_at: string;
  auto_revoke: boolean;
  granted_by: string;
  reason?: string;
  conditions: Record<string, any>;
  status: 'active' | 'suspended' | 'expired' | 'revoked';
  revoked_at?: string;
  revoked_by?: string;
  revocation_reason?: string;
  created_at: string;
  updated_at: string;
  is_active: boolean;
  is_expired: boolean;
}

export interface CreateTemporaryPermissionData {
  user_id: string;
  permission: string;
  resource: string;
  action: string;
  expires_at: string;
  reason?: string;
  conditions?: Record<string, any>;
}

export interface UpdateTemporaryPermissionData {
  permission?: string;
  resource?: string;
  action?: string;
  expires_at?: string;
  reason?: string;
  conditions?: Record<string, any>;
  status?: 'active' | 'suspended' | 'expired' | 'revoked';
}

export interface ListTemporaryPermissionsParams {
  user_id?: string;
  permission?: string;
  resource?: string;
  action?: string;
  status?: 'active' | 'suspended' | 'expired' | 'revoked';
  active_only?: boolean;
  expires_before?: string;
  expires_after?: string;
  granted_by?: string;
  limit?: number;
  offset?: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    message: string;
    details?: string;
  };
}

export interface ListTemporaryPermissionsResponse {
  permissions: TemporaryPermission[];
  total: number;
  limit: number;
  offset: number;
}

async function makeApiRequest<T>(endpoint: string, options: RequestInit = {}): Promise<ApiResponse<T>> {
  try {
    const response = await fetch(`${API_BASE}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      const errorText = await response.text();
      return {
        success: false,
        error: {
          message: `HTTP ${response.status}: ${response.statusText}`,
          details: errorText,
        },
      };
    }

    const data = await response.json();
    return {
      success: true,
      data,
    };
  } catch (error) {
    return {
      success: false,
      error: {
        message: error instanceof Error ? error.message : 'Unknown error occurred',
      },
    };
  }
}

export async function createTemporaryPermission(
  data: CreateTemporaryPermissionData
): Promise<ApiResponse<TemporaryPermission>> {
  const result = await makeApiRequest<TemporaryPermission>('/admin/temporary-permissions', {
    method: 'POST',
    body: JSON.stringify(data),
  });

  if (result.success) {
    revalidatePath(`/users/${data.user_id}/permissions`);
    revalidatePath('/users');
  }

  return result;
}

export async function getTemporaryPermission(id: string): Promise<ApiResponse<TemporaryPermission>> {
  return makeApiRequest<TemporaryPermission>(`/admin/temporary-permissions/${id}`);
}

export async function listTemporaryPermissions(
  params: ListTemporaryPermissionsParams = {}
): Promise<ApiResponse<ListTemporaryPermissionsResponse>> {
  const queryParams = new URLSearchParams();
  
  Object.entries(params).forEach(([key, value]) => {
    if (value !== undefined && value !== null) {
      queryParams.append(key, String(value));
    }
  });

  const queryString = queryParams.toString();
  const endpoint = `/admin/temporary-permissions${queryString ? `?${queryString}` : ''}`;
  
  return makeApiRequest<ListTemporaryPermissionsResponse>(endpoint);
}

export async function getUserTemporaryPermissions(userId: string): Promise<ApiResponse<TemporaryPermission[]>> {
  return makeApiRequest<TemporaryPermission[]>(`/admin/users/${userId}/temporary-permissions`);
}

export async function updateTemporaryPermission(
  id: string,
  data: UpdateTemporaryPermissionData
): Promise<ApiResponse<TemporaryPermission>> {
  const result = await makeApiRequest<TemporaryPermission>(`/admin/temporary-permissions/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });

  if (result.success) {
    revalidatePath('/users');
  }

  return result;
}

export async function revokeTemporaryPermission(
  id: string,
  reason?: string
): Promise<ApiResponse<void>> {
  const result = await makeApiRequest<void>(`/admin/temporary-permissions/${id}/revoke`, {
    method: 'POST',
    body: JSON.stringify({ reason }),
  });

  if (result.success) {
    revalidatePath('/users');
  }

  return result;
}

export async function deleteTemporaryPermission(id: string): Promise<ApiResponse<void>> {
  const result = await makeApiRequest<void>(`/admin/temporary-permissions/${id}`, {
    method: 'DELETE',
  });

  if (result.success) {
    revalidatePath('/users');
  }

  return result;
}

export async function cleanupExpiredPermissions(): Promise<ApiResponse<{ message: string; cleaned_count: number }>> {
  return makeApiRequest<{ message: string; cleaned_count: number }>('/admin/temporary-permissions/cleanup-expired', {
    method: 'POST',
  });
}

// Bulk Operations

export interface BulkCreateTemporaryPermissionsData {
  permissions: CreateTemporaryPermissionData[];
}

export interface BulkCreateTemporaryPermissionsResponse {
  created: TemporaryPermission[];
  failed: Array<{
    id?: string;
    error: string;
    details?: string;
  }>;
  summary: {
    total_requested: number;
    successful: number;
    failed: number;
    execution_time_ms: number;
  };
}

export async function bulkCreateTemporaryPermissions(
  data: BulkCreateTemporaryPermissionsData
): Promise<ApiResponse<BulkCreateTemporaryPermissionsResponse>> {
  const result = await makeApiRequest<BulkCreateTemporaryPermissionsResponse>('/admin/temporary-permissions/bulk-create', {
    method: 'POST',
    body: JSON.stringify(data),
  });

  if (result.success) {
    revalidatePath('/users');
  }

  return result;
}

export interface BulkRevokeTemporaryPermissionsData {
  permission_ids: string[];
  reason?: string;
}

export interface BulkRevokeTemporaryPermissionsResponse {
  revoked: string[];
  failed: Array<{
    id?: string;
    error: string;
    details?: string;
  }>;
  summary: {
    total_requested: number;
    successful: number;
    failed: number;
    execution_time_ms: number;
  };
}

export async function bulkRevokeTemporaryPermissions(
  data: BulkRevokeTemporaryPermissionsData
): Promise<ApiResponse<BulkRevokeTemporaryPermissionsResponse>> {
  const result = await makeApiRequest<BulkRevokeTemporaryPermissionsResponse>('/admin/temporary-permissions/bulk-revoke', {
    method: 'POST',
    body: JSON.stringify(data),
  });

  if (result.success) {
    revalidatePath('/users');
  }

  return result;
}

export interface BulkUpdateTemporaryPermissionsData {
  updates: Array<{
    id: string;
    updates: UpdateTemporaryPermissionData;
  }>;
}

export interface BulkUpdateTemporaryPermissionsResponse {
  updated: TemporaryPermission[];
  failed: Array<{
    id?: string;
    error: string;
    details?: string;
  }>;
  summary: {
    total_requested: number;
    successful: number;
    failed: number;
    execution_time_ms: number;
  };
}

export async function bulkUpdateTemporaryPermissions(
  data: BulkUpdateTemporaryPermissionsData
): Promise<ApiResponse<BulkUpdateTemporaryPermissionsResponse>> {
  const result = await makeApiRequest<BulkUpdateTemporaryPermissionsResponse>('/admin/temporary-permissions/bulk-update', {
    method: 'POST',
    body: JSON.stringify(data),
  });

  if (result.success) {
    revalidatePath('/users');
  }

  return result;
}