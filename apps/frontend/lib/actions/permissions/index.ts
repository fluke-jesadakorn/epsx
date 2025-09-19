'use server';

import { createApiClient, isApiError } from '@/lib/api-client';
import { requireAuth, requirePermission } from '../auth';
import { safeError } from '@/lib/utils/logging';

// ============================================================================
// Permission Management Server Actions
// ============================================================================

const getClient = () => createApiClient();

/**
 * Grant permission to user
 */
export async function grantPermission(userId: string, permission: string, expiresAt?: number): Promise<void> {
  try {
    await requirePermission('admin:permissions:grant');
    
    const client = getClient();
    const result = await client.serverGrantPermission(userId, permission, expiresAt);
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to grant permission');
    }
  } catch (error) {
    console.error('Grant permission error:', error);
    throw error;
  }
}

/**
 * Revoke permission from user
 */
export async function revokePermission(userId: string, permission: string): Promise<void> {
  try {
    await requirePermission('admin:permissions:revoke');
    
    const client = getClient();
    const result = await client.serverRevokePermission(userId, permission);
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to revoke permission');
    }
  } catch (error) {
    console.error('Revoke permission error:', error);
    throw error;
  }
}

/**
 * Get user permissions
 */
export async function getUserPermissionsList(userId: string): Promise<string[]> {
  try {
    await requirePermission('admin:permissions:read');
    
    const client = getClient();
    const result = await client.serverGetUserPermissions(userId);
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to get user permissions');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get user permissions error:', error);
    throw error;
  }
}

/**
 * Set user permissions (replace all)
 */
export async function setUserPermissions(userId: string, permissions: string[]): Promise<void> {
  try {
    await requirePermission('admin:permissions:manage');
    
    const client = getClient();
    const result = await client.serverSetUserPermissions(userId, permissions);
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to set user permissions');
    }
  } catch (error) {
    console.error('Set user permissions error:', error);
    throw error;
  }
}

/**
 * Get available permissions
 */
export async function getAvailablePermissions(): Promise<string[]> {
  try {
    await requirePermission('admin:permissions:read');
    
    const client = getClient();
    const result = await client.serverGetAvailablePermissions();
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to get available permissions');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get available permissions error:', error);
    throw error;
  }
}

/**
 * Validate permission for user
 */
export async function validateUserPermission(userId: string, permission: string): Promise<boolean> {
  try {
    await requirePermission('admin:permissions:read');
    
    const client = getClient();
    const result = await client.serverValidateUserPermission(userId, permission);
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to validate user permission');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Validate user permission error:', error);
    throw error;
  }
}

/**
 * Grant embedded timestamp permission
 */
export async function grantEmbeddedPermission(
  userId: string, 
  permission: string, 
  expiresAt: number,
  metadata?: Record<string, any>
): Promise<void> {
  try {
    await requirePermission('admin:permissions:grant');
    
    const client = getClient();
    const result = await client.serverGrantEmbeddedPermission(userId, permission, expiresAt, metadata);
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to grant embedded permission');
    }
  } catch (error) {
    console.error('Grant embedded permission error:', error);
    throw error;
  }
}

/**
 * Get permission health information
 */
export async function getPermissionHealth(): Promise<any> {
  try {
    await requirePermission('admin:system:read');
    
    const client = getClient();
    const result = await client.serverGetPermissionHealth();
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to get permission health');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get permission health error:', error);
    throw error;
  }
}

/**
 * Refresh user permissions from database
 */
export async function refreshUserPermissions(userId?: string): Promise<string[]> {
  try {
    await requireAuth();
    
    // If no userId provided, refresh current user's permissions
    if (!userId) {
      // Current user can refresh their own permissions
    } else {
      // Admin required to refresh other user's permissions
      await requirePermission('admin:permissions:manage');
    }
    
    const client = getClient();
    const result = await client.serverRefreshUserPermissions(userId);
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to refresh user permissions');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Refresh user permissions error:', error);
    throw error;
  }
}

/**
 * Get permission audit log
 */
export async function getPermissionAuditLog(userId?: string, limit: number = 50): Promise<any[]> {
  try {
    await requirePermission('admin:permissions:audit');
    
    const client = getClient();
    const result = await client.serverGetPermissionAuditLog(userId, limit);
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to get permission audit log');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get permission audit log error:', error);
    throw error;
  }
}