'use server';

import { revalidatePath } from 'next/cache';
import { EmbeddedPermissionsApi } from '@/lib/api/embedded-permissions';
import type {
  EmbeddedPermissionRequest,
  BulkEmbeddedPermissionRequest,
  ValidatePermissionsRequest,
  RevokePermissionRequest,
  ExtendPermissionRequest,
  CleanupExpiredRequest
} from '@/types/admin/embedded-permissions';

export async function grantEmbeddedPermissionAction(
  userId: string,
  request: EmbeddedPermissionRequest
) {
  try {
    const result = await EmbeddedPermissionsApi.grantPermission(userId, request);
    revalidatePath('/users/permissions');
    revalidatePath(`/users/${userId}`);
    return { success: true, data: result };
  } catch (error: any) {
    console.error('Failed to grant embedded permission:', error);
    return { 
      success: false, 
      error: error.message || 'Failed to grant permission',
      details: error.details 
    };
  }
}

export async function grantBulkEmbeddedPermissionsAction(
  request: BulkEmbeddedPermissionRequest
) {
  try {
    const result = await EmbeddedPermissionsApi.grantBulkPermissions(request);
    revalidatePath('/users/permissions');
    // Revalidate individual user pages
    request.user_ids.forEach(userId => {
      revalidatePath(`/users/${userId}`);
    });
    return { success: true, data: result };
  } catch (error: any) {
    console.error('Failed to grant bulk embedded permissions:', error);
    return { 
      success: false, 
      error: error.message || 'Failed to grant bulk permissions',
      details: error.details 
    };
  }
}

export async function validateEmbeddedPermissionsAction(
  userId: string,
  request: ValidatePermissionsRequest
) {
  try {
    const result = await EmbeddedPermissionsApi.validatePermissions(userId, request);
    return { success: true, data: result };
  } catch (error: any) {
    console.error('Failed to validate embedded permissions:', error);
    return { 
      success: false, 
      error: error.message || 'Failed to validate permissions',
      details: error.details 
    };
  }
}

export async function getPermissionExpiryStatusAction(userId: string) {
  try {
    const result = await EmbeddedPermissionsApi.getExpiryStatus(userId);
    return { success: true, data: result };
  } catch (error: any) {
    console.error('Failed to get permission expiry status:', error);
    return { 
      success: false, 
      error: error.message || 'Failed to get expiry status',
      details: error.details 
    };
  }
}

export async function revokeEmbeddedPermissionAction(
  userId: string,
  request: RevokePermissionRequest
) {
  try {
    await EmbeddedPermissionsApi.revokePermission(userId, request);
    revalidatePath('/users/permissions');
    revalidatePath(`/users/${userId}`);
    return { success: true };
  } catch (error: any) {
    console.error('Failed to revoke embedded permission:', error);
    return { 
      success: false, 
      error: error.message || 'Failed to revoke permission',
      details: error.details 
    };
  }
}

export async function extendEmbeddedPermissionAction(
  userId: string,
  request: ExtendPermissionRequest
) {
  try {
    const result = await EmbeddedPermissionsApi.extendPermission(userId, request);
    revalidatePath('/users/permissions');
    revalidatePath(`/users/${userId}`);
    return { success: true, data: result };
  } catch (error: any) {
    console.error('Failed to extend embedded permission:', error);
    return { 
      success: false, 
      error: error.message || 'Failed to extend permission',
      details: error.details 
    };
  }
}

export async function cleanupExpiredPermissionsAction(
  request: CleanupExpiredRequest = {}
) {
  try {
    const result = await EmbeddedPermissionsApi.cleanupExpired(request);
    revalidatePath('/users/permissions');
    revalidatePath('/users');
    return { success: true, data: result };
  } catch (error: any) {
    console.error('Failed to cleanup expired permissions:', error);
    return { 
      success: false, 
      error: error.message || 'Failed to cleanup expired permissions',
      details: error.details 
    };
  }
}

// Utility action for creating embedded permission strings
export async function createEmbeddedPermissionAction(
  basePermission: string,
  hours: number
) {
  const expiryTimestamp = Math.floor(Date.now() / 1000) + (hours * 3600);
  return `${basePermission}:${expiryTimestamp}`;
}

// Batch permission operations for common use cases
export async function grantTemporaryPermissionAction(
  userId: string,
  basePermission: string,
  hours: number,
  reason?: string
) {
  const expiryTimestamp = Math.floor(Date.now() / 1000) + (hours * 3600);
  const parts = basePermission.split(':');
  
  if (parts.length !== 3) {
    return {
      success: false,
      error: 'Invalid permission format. Expected format: platform:resource:action'
    };
  }

  const request: EmbeddedPermissionRequest = {
    embedded_permission: `${basePermission}:${expiryTimestamp}`,
    base_permission: basePermission,
    platform: parts[0],
    resource: parts[1],
    action: parts[2],
    expiry_timestamp: expiryTimestamp,
    reason
  };

  return grantEmbeddedPermissionAction(userId, request);
}

export async function extendPermissionByHoursAction(
  userId: string,
  permission: string,
  additionalHours: number,
  reason?: string
) {
  const newExpiryTimestamp = Math.floor(Date.now() / 1000) + (additionalHours * 3600);
  
  const request: ExtendPermissionRequest = {
    permission,
    new_expiry_timestamp: newExpiryTimestamp,
    reason
  };

  return extendEmbeddedPermissionAction(userId, request);
}