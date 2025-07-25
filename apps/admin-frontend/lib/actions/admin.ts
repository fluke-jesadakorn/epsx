'use server';

import { createApiClient, isApiError, type ActionResult, type AssignmentResult, type StockRankingAssignmentRequest, type StockRankingAssignmentExtendRequest, type StockRankingAssignmentUpdateRequest, type UserSoftDeleteRequest } from '@epsx/api-client';
import { adminLogger } from '../logger';
import { revalidatePath } from 'next/cache';
import { config } from '../config';

// Get API client server-side only
const getApiClient = () => {
  const backendUrl = config.getBackendUrl();
  return createApiClient(backendUrl);
};

// Permission Profile Actions
export async function assignPermissionProfileAction(formData: FormData): Promise<ActionResult<AssignmentResult>> {
  const profileId = formData.get('profileId') as string;
  const userId = formData.get('userId') as string;
  const expiresAt = formData.get('expiresAt') as string;

  try {
    adminLogger.info('Assigning permission profile', { profileId, userId, expiresAt }, 'AdminActionLayer');

    const response = await getApiClient().assignAdminPermissionProfile({
      profile_id: profileId,
      user_id: userId,
      expires_at: expiresAt || undefined,
    });

    if (isApiError(response)) {
      adminLogger.error('Failed to assign permission profile', { error: response.error, details: response.details, profileId, userId }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to assign permission profile'
      };
    }

    adminLogger.info('Permission profile assigned successfully', { profileId, userId }, 'AdminActionLayer');
    
    // Revalidate related pages
    revalidatePath('/admin/permission-profiles');
    revalidatePath('/admin/users');
    
    return { success: true, data: response.data };
  } catch (error) {
    adminLogger.error('Permission profile assignment error', { 
      error: error instanceof Error ? error.message : String(error),
      profileId,
      userId
    }, 'AdminActionLayer');
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Assignment failed' 
    };
  }
}

// Stock Ranking Actions
export async function assignBulkStockRankingAction(formData: FormData): Promise<ActionResult<AssignmentResult>> {
  const userIds = JSON.parse(formData.get('userIds') as string);
  const packageTier = formData.get('packageTier') as string;
  const expiresAt = formData.get('expiresAt') as string;

  try {
    adminLogger.info('Assigning bulk stock ranking', { userIds, packageTier, expiresAt }, 'AdminActionLayer');

    const response = await getApiClient().assignBulkStockRanking({
      user_ids: userIds,
      package_tier: packageTier,
      expires_at: expiresAt || undefined,
    });

    if (isApiError(response)) {
      adminLogger.error('Failed to assign bulk stock ranking', { error: response.error, details: response.details, userIds, packageTier }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to assign stock ranking'
      };
    }

    adminLogger.info('Bulk stock ranking assigned successfully', { userIds, packageTier }, 'AdminActionLayer');
    
    // Revalidate related pages
    revalidatePath('/admin/stock-ranking');
    revalidatePath('/admin/users');
    
    return { success: true, data: response.data };
  } catch (error) {
    adminLogger.error('Bulk stock ranking assignment error', { 
      error: error instanceof Error ? error.message : String(error),
      userIds,
      packageTier
    }, 'AdminActionLayer');
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Assignment failed' 
    };
  }
}

export async function revokeStockRankingAssignmentAction(assignmentId: string): Promise<ActionResult<AssignmentResult>> {
  try {
    adminLogger.info('Revoking stock ranking assignment', { assignmentId }, 'AdminActionLayer');

    const response = await getApiClient().revokeStockRankingAssignment(assignmentId);

    if (isApiError(response)) {
      adminLogger.error('Failed to revoke stock ranking assignment', { error: response.error, details: response.details, assignmentId }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to revoke assignment'
      };
    }

    adminLogger.info('Stock ranking assignment revoked successfully', { assignmentId }, 'AdminActionLayer');
    
    // Revalidate related pages
    revalidatePath('/admin/stock-ranking');
    
    return { success: true, data: response.data };
  } catch (error) {
    adminLogger.error('Stock ranking assignment revocation error', { 
      error: error instanceof Error ? error.message : String(error),
      assignmentId
    }, 'AdminActionLayer');
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Revocation failed' 
    };
  }
}

export async function extendStockRankingAssignmentAction(assignmentId: string, formData: FormData): Promise<ActionResult<AssignmentResult>> {
  const newExpiresAt = formData.get('newExpiresAt') as string;

  try {
    adminLogger.info('Extending stock ranking assignment', { assignmentId, newExpiresAt }, 'AdminActionLayer');

    const response = await getApiClient().extendStockRankingAssignment(assignmentId, {
      new_expires_at: newExpiresAt,
    });

    if (isApiError(response)) {
      adminLogger.error('Failed to extend stock ranking assignment', { error: response.error, details: response.details, assignmentId, newExpiresAt }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to extend assignment'
      };
    }

    adminLogger.info('Stock ranking assignment extended successfully', { assignmentId, newExpiresAt }, 'AdminActionLayer');
    
    // Revalidate related pages
    revalidatePath('/admin/stock-ranking');
    
    return { success: true, data: response.data };
  } catch (error) {
    adminLogger.error('Stock ranking assignment extension error', { 
      error: error instanceof Error ? error.message : String(error),
      assignmentId,
      newExpiresAt
    }, 'AdminActionLayer');
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Extension failed' 
    };
  }
}

export async function updateStockRankingAssignmentAction(assignmentId: string, formData: FormData): Promise<ActionResult<AssignmentResult>> {
  const updateData: StockRankingAssignmentUpdateRequest = {
    status: formData.get('status') as string,
    expires_at: formData.get('expiresAt') as string,
    package_tier: formData.get('packageTier') as string,
  };

  try {
    adminLogger.info('Updating stock ranking assignment', { assignmentId, updateData }, 'AdminActionLayer');

    const response = await getApiClient().updateStockRankingAssignment(assignmentId, updateData);

    if (isApiError(response)) {
      adminLogger.error('Failed to update stock ranking assignment', { error: response.error, details: response.details, assignmentId, updateData }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to update assignment'
      };
    }

    adminLogger.info('Stock ranking assignment updated successfully', { assignmentId }, 'AdminActionLayer');
    
    // Revalidate related pages
    revalidatePath('/admin/stock-ranking');
    
    return { success: true, data: response.data };
  } catch (error) {
    adminLogger.error('Stock ranking assignment update error', { 
      error: error instanceof Error ? error.message : String(error),
      assignmentId,
      updateData
    }, 'AdminActionLayer');
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Update failed' 
    };
  }
}

// User Management Actions
export async function softDeleteUserAction(formData: FormData): Promise<ActionResult<{ message: string }>> {
  const userId = formData.get('userId') as string;
  const reason = formData.get('reason') as string;

  try {
    adminLogger.info('Soft deleting user', { userId, reason }, 'AdminActionLayer');

    const response = await getApiClient().softDeleteUser(userId, {
      reason: reason || 'Deleted via admin interface',
    });

    if (isApiError(response)) {
      adminLogger.error('Failed to soft delete user', { error: response.error, details: response.details, userId, reason }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to delete user'
      };
    }

    adminLogger.info('User soft deleted successfully', { userId }, 'AdminActionLayer');
    
    // Revalidate related pages
    revalidatePath('/admin/users');
    
    return { success: true, data: response.data };
  } catch (error) {
    adminLogger.error('User soft delete error', { 
      userId,
      reason,
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminActionLayer');
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'User deletion failed' 
    };
  }
}