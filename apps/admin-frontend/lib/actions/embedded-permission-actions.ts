'use server';

import { revalidatePath } from 'next/cache';
import { getServerSession } from '@/lib/auth';
import { logger } from '@/lib/logger';
import { env } from '@/config/env';
import type { UserOperationResult } from '@/lib/types/unified-user';

// Get bearer token from custom JWT session
const getBearerToken = async () => {
  const session = await getServerSession();
  return (session as any)?.accessToken || null;
};

const BACKEND_URL = env.BACKEND_URL;

// ====================================
// Embedded Timestamp Permission Types
// ====================================

export interface EmbeddedPermissionData {
  userId: string;
  basePermission: string;
  platform: string;
  resource: string;
  action: string;
  expiryTimestamp: number; // Unix timestamp
  reason?: string;
  metadata?: Record<string, any>;
}

export interface BulkEmbeddedPermissionData {
  userIds: string[];
  permissions: Array<{
    basePermission: string;
    platform: string;
    resource: string;
    action: string;
    expiryTimestamp: number;
  }>;
  reason?: string;
  metadata?: Record<string, any>;
}

export interface EmbeddedPermissionValidationResult {
  valid: string[];
  expired: Array<{
    permission: string;
    basePermission: string;
    expiredAt: number;
    expiredFor: number; // milliseconds
  }>;
  expiringSoon: Array<{
    permission: string;
    basePermission: string;
    expiresAt: number;
    expiresIn: number; // milliseconds
  }>;
  summary: {
    total: number;
    validCount: number;
    expiredCount: number;
    expiringSoonCount: number;
  };
}

export interface PermissionExpiryStatus {
  userId: string;
  permissions: Array<{
    permission: string;
    basePermission: string;
    expiresAt?: number;
    isExpired: boolean;
    timeRemaining?: number; // milliseconds
    expiresIn?: string; // human readable
  }>;
  health: {
    hasExpired: boolean;
    hasExpiringSoon: boolean;
    nextExpiry: number | null;
    timeUntilNextExpiry: number | null; // milliseconds
  };
}

export interface EmbeddedPermissionHistoryEntry {
  id: string;
  userId: string;
  action: 'granted' | 'revoked' | 'extended' | 'expired' | 'converted';
  permission: string;
  basePermission: string;
  platform: string;
  resource: string;
  actionType: string;
  expiryTimestamp?: number;
  previousExpiry?: number;
  newExpiry?: number;
  reason?: string;
  grantedBy: string;
  timestamp: Date;
  metadata?: Record<string, any>;
}

// ====================================
// Helper Functions
// ====================================

function createEmbeddedPermission(
  basePermission: string, 
  expiryTimestamp: number
): string {
  return `${basePermission}:${expiryTimestamp}`;
}

function parseEmbeddedPermission(permission: string): {
  basePermission: string;
  timestamp?: number;
  isEmbedded: boolean;
} {
  const parts = permission.split(':');
  if (parts.length >= 4) {
    const lastPart = parts[parts.length - 1];
    const timestamp = parseInt(lastPart, 10);
    if (!isNaN(timestamp)) {
      const basePermission = parts.slice(0, -1).join(':');
      return { basePermission, timestamp, isEmbedded: true };
    }
  }
  return { basePermission: permission, isEmbedded: false };
}

function isPermissionExpired(permission: string): boolean {
  const { timestamp, isEmbedded } = parseEmbeddedPermission(permission);
  if (!isEmbedded || !timestamp) return false;
  return Date.now() > timestamp * 1000;
}

async function makeApiRequest<T>(endpoint: string, options: RequestInit = {}): Promise<UserOperationResult<T>> {
  try {
    const token = await getBearerToken();
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } };
    }

    const response = await fetch(`${BACKEND_URL}${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      const errorText = await response.text();
      logger.action.error(`API request failed: ${endpoint}`, `HTTP ${response.status}: ${response.statusText}`, { 
        details: errorText,
        endpoint,
        status: response.status 
      });
      
      return {
        success: false,
        error: {
          code: `HTTP_${response.status}`,
          message: `API request failed: ${response.status} ${response.statusText}`,
        },
      };
    }

    const data = await response.json();
    return { success: true, data };
  } catch (error) {
    logger.action.error(`API request error: ${endpoint}`, error);
    return {
      success: false,
      error: {
        code: 'NETWORK_ERROR',
        message: error instanceof Error ? error.message : 'Network error occurred',
      },
    };
  }
}

// ====================================
// Main Server Actions
// ====================================

/**
 * Grant an embedded timestamp permission to a user
 */
export async function grantEmbeddedTimestampPermission(
  data: EmbeddedPermissionData
): Promise<UserOperationResult<{ permission: string; expiresAt: number }>> {
  try {
    logger.action.start('grantEmbeddedTimestampPermission', {
      userId: data.userId,
      basePermission: data.basePermission,
      expiryTimestamp: data.expiryTimestamp
    });

    // Create the embedded permission string
    const embeddedPermission = createEmbeddedPermission(data.basePermission, data.expiryTimestamp);

    const result = await makeApiRequest<{ permission: string; expiresAt: number }>(
      `/api/v1/admin/users/${data.userId}/embedded-permissions`,
      {
        method: 'POST',
        body: JSON.stringify({
          embedded_permission: embeddedPermission,
          base_permission: data.basePermission,
          platform: data.platform,
          resource: data.resource,
          action: data.action,
          expiry_timestamp: data.expiryTimestamp,
          reason: data.reason,
          metadata: data.metadata,
        }),
      }
    );

    if (!result.success) {
      return result;
    }

    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`);
    revalidatePath(`/users/${data.userId}/permissions`);
    revalidatePath('/users');

    logger.admin.permission(
      `Embedded timestamp permission granted: ${embeddedPermission}`, 
      { 
        userId: data.userId,
        expiresAt: new Date(data.expiryTimestamp * 1000).toISOString()
      }
    );
    logger.action.success('grantEmbeddedTimestampPermission', data);

    return result;
  } catch (error) {
    logger.action.error('grantEmbeddedTimestampPermission', error, data);
    return {
      success: false,
      error: {
        code: 'GRANT_ERROR',
        message: 'Failed to grant embedded timestamp permission',
      },
    };
  }
}

/**
 * Bulk grant embedded timestamp permissions to multiple users
 */
export async function grantBulkEmbeddedPermissions(
  data: BulkEmbeddedPermissionData
): Promise<UserOperationResult<{
  successful: Array<{ userId: string; permissions: string[] }>;
  failed: Array<{ userId: string; error: string }>;
  summary: { total: number; successful: number; failed: number };
}>> {
  try {
    logger.action.start('grantBulkEmbeddedPermissions', {
      userCount: data.userIds.length,
      permissionCount: data.permissions.length
    });

    // Create embedded permission strings
    const embeddedPermissions = data.permissions.map(p => ({
      ...p,
      embeddedPermission: createEmbeddedPermission(p.basePermission, p.expiryTimestamp)
    }));

    const result = await makeApiRequest<{
      successful: Array<{ userId: string; permissions: string[] }>;
      failed: Array<{ userId: string; error: string }>;
      summary: { total: number; successful: number; failed: number };
    }>('/api/v1/admin/users/bulk/embedded-permissions', {
      method: 'POST',
      body: JSON.stringify({
        user_ids: data.userIds,
        permissions: embeddedPermissions,
        reason: data.reason,
        metadata: data.metadata,
      }),
    });

    if (!result.success) {
      return result;
    }

    // Revalidate user pages for all affected users
    data.userIds.forEach(userId => {
      revalidatePath(`/users/${userId}`);
      revalidatePath(`/users/${userId}/permissions`);
    });
    revalidatePath('/users');

    logger.admin.permission(
      `Bulk embedded permissions granted to ${data.userIds.length} users`,
      { userCount: data.userIds.length, permissionCount: data.permissions.length }
    );
    logger.action.success('grantBulkEmbeddedPermissions', {
      userCount: data.userIds.length,
      permissionCount: data.permissions.length
    });

    return result;
  } catch (error) {
    logger.action.error('grantBulkEmbeddedPermissions', error, data);
    return {
      success: false,
      error: {
        code: 'BULK_GRANT_ERROR',
        message: 'Failed to grant bulk embedded permissions',
      },
    };
  }
}

/**
 * Validate and filter expired embedded permissions for a user
 */
export async function validateEmbeddedPermissions(
  userId: string,
  permissions?: string[]
): Promise<UserOperationResult<EmbeddedPermissionValidationResult>> {
  try {
    logger.action.start('validateEmbeddedPermissions', { userId, permissionCount: permissions?.length });

    const result = await makeApiRequest<EmbeddedPermissionValidationResult>(
      `/api/v1/admin/users/${userId}/embedded-permissions/validate`,
      {
        method: 'POST',
        body: JSON.stringify({
          permissions: permissions || []
        }),
      }
    );

    if (!result.success) {
      return result;
    }

    logger.action.success('validateEmbeddedPermissions', {
      userId,
      validation: result.data?.summary
    });

    return result;
  } catch (error) {
    logger.action.error('validateEmbeddedPermissions', error, { userId });
    return {
      success: false,
      error: {
        code: 'VALIDATION_ERROR',
        message: 'Failed to validate embedded permissions',
      },
    };
  }
}

/**
 * Get permission expiry status for a user
 */
export async function getPermissionExpiryStatus(
  userId: string
): Promise<UserOperationResult<PermissionExpiryStatus>> {
  try {
    logger.action.start('getPermissionExpiryStatus', { userId });

    const result = await makeApiRequest<PermissionExpiryStatus>(
      `/api/v1/admin/users/${userId}/permissions/expiry-status`
    );

    if (!result.success) {
      return result;
    }

    logger.action.success('getPermissionExpiryStatus', {
      userId,
      permissionCount: result.data?.permissions.length,
      hasExpired: result.data?.health.hasExpired
    });

    return result;
  } catch (error) {
    logger.action.error('getPermissionExpiryStatus', error, { userId });
    return {
      success: false,
      error: {
        code: 'EXPIRY_STATUS_ERROR',
        message: 'Failed to get permission expiry status',
      },
    };
  }
}

/**
 * Convert traditional permission to embedded timestamp format
 */
export async function convertToEmbeddedFormat(data: {
  userId: string;
  permissionId: string;
  expiryTimestamp: number;
  reason?: string;
}): Promise<UserOperationResult<{ newPermission: string; oldPermission: string }>> {
  try {
    logger.action.start('convertToEmbeddedFormat', {
      userId: data.userId,
      permissionId: data.permissionId,
      expiryTimestamp: data.expiryTimestamp
    });

    const result = await makeApiRequest<{ newPermission: string; oldPermission: string }>(
      `/api/v1/admin/users/${data.userId}/permissions/${data.permissionId}/convert-embedded`,
      {
        method: 'POST',
        body: JSON.stringify({
          expiry_timestamp: data.expiryTimestamp,
          reason: data.reason,
        }),
      }
    );

    if (!result.success) {
      return result;
    }

    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`);
    revalidatePath(`/users/${data.userId}/permissions`);
    revalidatePath('/users');

    logger.admin.permission(
      `Permission converted to embedded format: ${result.data?.newPermission}`,
      { userId: data.userId, oldPermission: result.data?.oldPermission }
    );
    logger.action.success('convertToEmbeddedFormat', data);

    return result;
  } catch (error) {
    logger.action.error('convertToEmbeddedFormat', error, data);
    return {
      success: false,
      error: {
        code: 'CONVERSION_ERROR',
        message: 'Failed to convert permission to embedded format',
      },
    };
  }
}

/**
 * Revoke an embedded timestamp permission
 */
export async function revokeEmbeddedPermission(data: {
  userId: string;
  permission: string;
  reason?: string;
}): Promise<UserOperationResult<void>> {
  try {
    logger.action.start('revokeEmbeddedPermission', {
      userId: data.userId,
      permission: data.permission
    });

    const result = await makeApiRequest<void>(
      `/api/v1/admin/users/${data.userId}/embedded-permissions/revoke`,
      {
        method: 'POST',
        body: JSON.stringify({
          permission: data.permission,
          reason: data.reason,
        }),
      }
    );

    if (!result.success) {
      return result;
    }

    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`);
    revalidatePath(`/users/${data.userId}/permissions`);
    revalidatePath('/users');

    logger.admin.permission(
      `Embedded permission revoked: ${data.permission}`,
      { userId: data.userId }
    );
    logger.action.success('revokeEmbeddedPermission', data);

    return result;
  } catch (error) {
    logger.action.error('revokeEmbeddedPermission', error, data);
    return {
      success: false,
      error: {
        code: 'REVOKE_ERROR',
        message: 'Failed to revoke embedded permission',
      },
    };
  }
}

/**
 * Extend an embedded timestamp permission's expiry
 */
export async function extendEmbeddedPermission(data: {
  userId: string;
  permission: string;
  newExpiryTimestamp: number;
  reason?: string;
}): Promise<UserOperationResult<{ oldPermission: string; newPermission: string; extension: number }>> {
  try {
    logger.action.start('extendEmbeddedPermission', {
      userId: data.userId,
      permission: data.permission,
      newExpiryTimestamp: data.newExpiryTimestamp
    });

    const result = await makeApiRequest<{ 
      oldPermission: string; 
      newPermission: string; 
      extension: number 
    }>(`/api/v1/admin/users/${data.userId}/embedded-permissions/extend`, {
      method: 'POST',
      body: JSON.stringify({
        permission: data.permission,
        new_expiry_timestamp: data.newExpiryTimestamp,
        reason: data.reason,
      }),
    });

    if (!result.success) {
      return result;
    }

    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`);
    revalidatePath(`/users/${data.userId}/permissions`);
    revalidatePath('/users');

    logger.admin.permission(
      `Embedded permission extended: ${result.data?.newPermission}`,
      { 
        userId: data.userId,
        oldPermission: result.data?.oldPermission,
        extensionHours: result.data ? Math.round(result.data.extension / (1000 * 60 * 60)) : 0
      }
    );
    logger.action.success('extendEmbeddedPermission', data);

    return result;
  } catch (error) {
    logger.action.error('extendEmbeddedPermission', error, data);
    return {
      success: false,
      error: {
        code: 'EXTEND_ERROR',
        message: 'Failed to extend embedded permission',
      },
    };
  }
}

/**
 * Get history of embedded permission operations for a user
 */
export async function getEmbeddedPermissionHistory(
  userId: string,
  options: {
    limit?: number;
    offset?: number;
    action?: 'granted' | 'revoked' | 'extended' | 'expired' | 'converted';
    platform?: string;
  } = {}
): Promise<UserOperationResult<{
  history: EmbeddedPermissionHistoryEntry[];
  pagination: { total: number; limit: number; offset: number; hasMore: boolean };
}>> {
  try {
    logger.action.start('getEmbeddedPermissionHistory', { userId, options });

    const queryParams = new URLSearchParams();
    if (options.limit) queryParams.append('limit', options.limit.toString());
    if (options.offset) queryParams.append('offset', options.offset.toString());
    if (options.action) queryParams.append('action', options.action);
    if (options.platform) queryParams.append('platform', options.platform);

    const queryString = queryParams.toString();
    const endpoint = `/api/v1/admin/users/${userId}/embedded-permissions/history${queryString ? `?${queryString}` : ''}`;

    const result = await makeApiRequest<{
      history: EmbeddedPermissionHistoryEntry[];
      pagination: { total: number; limit: number; offset: number; hasMore: boolean };
    }>(endpoint);

    if (!result.success) {
      return result;
    }

    logger.action.success('getEmbeddedPermissionHistory', {
      userId,
      historyCount: result.data?.history.length,
      total: result.data?.pagination.total
    });

    return result;
  } catch (error) {
    logger.action.error('getEmbeddedPermissionHistory', error, { userId, options });
    return {
      success: false,
      error: {
        code: 'HISTORY_ERROR',
        message: 'Failed to get embedded permission history',
      },
    };
  }
}

/**
 * Cleanup expired embedded permissions across all users
 */
export async function cleanupExpiredEmbeddedPermissions(options: {
  dryRun?: boolean;
  batchSize?: number;
} = {}): Promise<UserOperationResult<{
  cleaned: number;
  failed: number;
  details: Array<{
    userId: string;
    permission: string;
    expiredAt: number;
    status: 'cleaned' | 'failed';
    error?: string;
  }>;
}>> {
  try {
    logger.action.start('cleanupExpiredEmbeddedPermissions', options);

    const result = await makeApiRequest<{
      cleaned: number;
      failed: number;
      details: Array<{
        userId: string;
        permission: string;
        expiredAt: number;
        status: 'cleaned' | 'failed';
        error?: string;
      }>;
    }>('/api/v1/admin/embedded-permissions/cleanup-expired', {
      method: 'POST',
      body: JSON.stringify({
        dry_run: options.dryRun || false,
        batch_size: options.batchSize || 100,
      }),
    });

    if (!result.success) {
      return result;
    }

    if (!options.dryRun && result.data && result.data.cleaned > 0) {
      // Revalidate all user pages if we actually cleaned permissions
      revalidatePath('/users');
    }

    logger.admin.system(
      `Embedded permission cleanup: ${result.data?.cleaned} cleaned, ${result.data?.failed} failed`,
      { 
        dryRun: options.dryRun,
        cleaned: result.data?.cleaned,
        failed: result.data?.failed
      }
    );
    logger.action.success('cleanupExpiredEmbeddedPermissions', {
      cleaned: result.data?.cleaned,
      failed: result.data?.failed,
      dryRun: options.dryRun
    });

    return result;
  } catch (error) {
    logger.action.error('cleanupExpiredEmbeddedPermissions', error, options);
    return {
      success: false,
      error: {
        code: 'CLEANUP_ERROR',
        message: 'Failed to cleanup expired embedded permissions',
      },
    };
  }
}