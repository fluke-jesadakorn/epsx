'use server';

import type { ActionResult } from '@/types/api';
import { UnifiedAdminClient } from '@/lib/api/unified-admin-client';
import { revalidatePath } from 'next/cache';
import { config } from '../config';
import { getServerSession } from '@/lib/server/auth';
import { logger } from '@/lib/logger';
import { env } from '@/config/env';

// Get bearer token from custom JWT session
export const getBearerToken = async () => {
  const session = await getServerSession();
  return (session as any)?.accessToken || null;
};

// Get backend URL server-side only
const getApiClient = async () => {
  if (!config.isServer()) {
    throw new Error('API client can only be created on server-side');
  }
  
  // Get bearer token for authenticated API calls
  const token = await getBearerToken();
  const headers: Record<string, string> = {};
  
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
    logger.admin.audit('Adding bearer token to API client');
  } else {
    logger.warn('No bearer token found for API client', { component: 'admin-actions' });
  }
  
  return createApiClient({ 
    baseURL: env.BACKEND_URL,
    headers 
  });
};

// Server action to get users with proper authentication
export async function getUsersAction(): Promise<ActionResult<any[]>> {
  try {
    const session = await getServerSession();
    if (!session?.user) {
      return { success: false, error: 'Not authenticated' };
    }

    const userPermissions = (session.user as any)?.permissions as string[] || [];
    
    // Check if user has admin permissions
    const hasAdminPermission = userPermissions.some(p => 
      p.includes(':manage') || 
      p.includes(':admin') || 
      p === '*'
    );
    
    logger.admin.userOperation('Getting users list', {
      userId: session.user.id,
      email: session.user.email,
      isAdmin: hasAdminPermission,
      permissions: userPermissions
    });

    const apiClient = await getApiClient();
    const response = await apiClient.get('/api/v1/admin/users');

    if (isApiError(response)) {
      logger.action.error('getUsersAction', response.error, { details: response.details });
      return { 
        success: false, 
        error: response.error || 'Failed to fetch users'
      };
    }

    return { success: true, data: response.data };
  } catch (error) {
    logger.action.error('getUsersAction', error);
    return { 
      success: false, 
      error: 'Failed to fetch users' 
    };
  }
}

// Permission Profile Actions
export async function assignPermissionProfileAction(formData: FormData): Promise<ActionResult<AssignmentResult>> {
  const profileId = formData.get('profileId') as string;
  const userId = formData.get('userId') as string;
  const expiresAt = formData.get('expiresAt') as string;

  try {

    const response = await (await getApiClient()).assignAdminPermissionProfile({
      profile_id: profileId,
      user_id: userId,
      expires_at: expiresAt || undefined,
    });

    if (isApiError(response)) {
      logger.action.error('assignPermissionProfileAction', response.error, { details: response.details, profileId, userId });
      return { 
        success: false, 
        error: response.error || 'Failed to assign permission profile'
      };
    }

    
    // Revalidate related pages
    revalidatePath('/admin/permission-profiles');
    revalidatePath('/users');
    
    return { success: true, data: response.data };
  } catch (error) {
    logger.action.error('assignPermissionProfileAction', error, { profileId, userId });
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

    const response = await getApiClient().assignBulkStockRanking({
      user_ids: userIds,
      package_tier: packageTier,
      expires_at: expiresAt || undefined,
    });

    if (isApiError(response)) {
      console.error('Failed to assign bulk stock ranking', { error: response.error, details: response.details, userIds, packageTier }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to assign stock ranking'
      };
    }

    
    // Revalidate related pages
    revalidatePath('/admin/stock-ranking');
    revalidatePath('/users');
    
    return { success: true, data: response.data };
  } catch (error) {
    console.error('Bulk stock ranking assignment error', { 
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

    const response = await getApiClient().revokeStockRankingAssignment(assignmentId);

    if (isApiError(response)) {
      console.error('Failed to revoke stock ranking assignment', { error: response.error, details: response.details, assignmentId }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to revoke assignment'
      };
    }

    
    // Revalidate related pages
    revalidatePath('/admin/stock-ranking');
    
    return { success: true, data: response.data };
  } catch (error) {
    console.error('Stock ranking assignment revocation error', { 
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

    const response = await getApiClient().extendStockRankingAssignment(assignmentId, {
      new_expires_at: newExpiresAt,
    });

    if (isApiError(response)) {
      console.error('Failed to extend stock ranking assignment', { error: response.error, details: response.details, assignmentId, newExpiresAt }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to extend assignment'
      };
    }

    
    // Revalidate related pages
    revalidatePath('/admin/stock-ranking');
    
    return { success: true, data: response.data };
  } catch (error) {
    console.error('Stock ranking assignment extension error', { 
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

    const response = await getApiClient().updateStockRankingAssignment(assignmentId, updateData);

    if (isApiError(response)) {
      console.error('Failed to update stock ranking assignment', { error: response.error, details: response.details, assignmentId, updateData }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to update assignment'
      };
    }

    
    // Revalidate related pages
    revalidatePath('/admin/stock-ranking');
    
    return { success: true, data: response.data };
  } catch (error) {
    console.error('Stock ranking assignment update error', { 
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

    const response = await getApiClient().softDeleteUser(userId, {
      reason: reason || 'Deleted via admin interface',
    });

    if (isApiError(response)) {
      console.error('Failed to soft delete user', { error: response.error, details: response.details, userId, reason }, 'AdminActionLayer');
      return { 
        success: false, 
        error: response.error || 'Failed to delete user'
      };
    }

    
    // Revalidate related pages
    revalidatePath('/users');
    
    return { success: true, data: response.data };
  } catch (error) {
    console.error('User soft delete error', { 
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