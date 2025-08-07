'use server';

import type { ActionResult, AssignmentResult, StockRankingAssignmentUpdateRequest } from '@epsx/api-client';
import { createApiClient, isApiError } from '@epsx/api-client';
import { revalidatePath } from 'next/cache';
import { config } from '../config';
import { auth } from '../../auth';

// Get backend URL server-side only
const getApiClient = async () => {
  if (!config.isServer()) {
    throw new Error('API client can only be created on server-side');
  }
  
  // Get NextAuth session for authenticated API calls
  const session = await auth();
  const headers: Record<string, string> = {};
  
  if (session?.session_id) {
    headers['Authorization'] = `Bearer ${session.session_id}`;
    console.log('✅ [AdminActions] Adding NextAuth session token to API client');
  } else {
    console.warn('⚠️ [AdminActions] No NextAuth session found for API client');
  }
  
  return createApiClient({ 
    baseURL: process.env.BACKEND_URL || 'http://localhost:8080',
    headers 
  });
};

// Server action to get users with proper authentication
export async function getUsersAction(): Promise<ActionResult<any[]>> {
  try {
    const session = await auth();
    if (!session) {
      return { success: false, error: 'No authentication session found' };
    }

    console.log('🔐 [AdminActions] Getting users with session:', {
      hasSession: !!session,
      hasSessionId: !!session.session_id,
      userEmail: session.user?.email
    });

    const apiClient = await getApiClient();
    const response = await apiClient.get('/api/v1/admin/users');

    if (isApiError(response)) {
      console.error('Failed to fetch users', { error: response.error, details: response.details });
      return { 
        success: false, 
        error: response.error || 'Failed to fetch users'
      };
    }

    return { success: true, data: response.data };
  } catch (error) {
    console.error('Users fetch error', { 
      error: error instanceof Error ? error.message : String(error)
    });
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
      console.error('Failed to assign permission profile', { error: response.error, details: response.details, profileId, userId }, 'AdminActionLayer');
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
    console.error('Permission profile assignment error', { 
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