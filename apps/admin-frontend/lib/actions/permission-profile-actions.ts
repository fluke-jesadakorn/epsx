/**
 * Server Actions for Permission Profile Management
 */

'use server'

import { revalidatePath } from 'next/cache'
import { requireAdminAuth } from '@/lib/auth/server-auth'
import { env } from '@/config/env'
import type { 
  PermissionProfile, 
  PermissionProfileQuery,
  ListPermissionProfilesResponse,
  ValidateAssignmentRequest,
  ValidateAssignmentResponse,
  BulkValidateAssignmentRequest,
  BulkValidateAssignmentResponse,
  UnassignProfileRequest,
  UnassignProfileResponse,
  ApiResponse 
} from '@/lib/types/permission-profiles'

const API_BASE_URL = env.BACKEND_URL

// Helper function to make authenticated API requests
async function makeAuthenticatedRequest<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  try {
    // Get the current admin user for auth
    const currentUser = await requireAdminAuth()
    
    const response = await fetch(`${API_BASE_URL}/api/v1/admin${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        // TODO: Add proper JWT token from session
        'Authorization': `Bearer admin-token-placeholder`,
        ...options.headers,
      },
    })

    if (!response.ok) {
      let errorData = {}
      try {
        errorData = await response.json()
      } catch {
        // Ignore JSON parsing errors, use empty object
      }
      return {
        success: false,
        error: {
          message: errorData.message || `HTTP ${response.status}: ${response.statusText}`,
          code: response.status.toString()
        }
      }
    }

    const data = await response.json()
    return {
      success: true,
      data
    }
  } catch (error) {
    return {
      success: false,
      error: {
        message: error instanceof Error ? error.message : 'Network error occurred',
        code: 'NETWORK_ERROR'
      }
    }
  }
}

/**
 * List permission profiles with filtering and pagination
 */
export async function listPermissionProfiles(
  query: PermissionProfileQuery = {}
): Promise<ApiResponse<ListPermissionProfilesResponse>> {
  const searchParams = new URLSearchParams()
  
  if (query.page) searchParams.set('page', query.page.toString())
  if (query.limit) searchParams.set('limit', query.limit.toString())
  if (query.category) searchParams.set('category', query.category)
  if (query.activeOnly !== undefined) searchParams.set('active_only', query.activeOnly.toString())
  if (query.name) searchParams.set('name', query.name)

  const result = await makeAuthenticatedRequest<ListPermissionProfilesResponse>(
    `/permission-profiles?${searchParams.toString()}`
  )

  if (result.success) {
    revalidatePath('/admin/permission-profiles')
  }

  return result
}

/**
 * Get a specific permission profile
 */
export async function getPermissionProfile(id: string): Promise<ApiResponse<PermissionProfile>> {
  const result = await makeAuthenticatedRequest<PermissionProfile>(
    `/permission-profiles/${id}`
  )

  return result
}

/**
 * Create a new permission profile
 */
export async function createPermissionProfile(
  data: Partial<PermissionProfile>
): Promise<ApiResponse<PermissionProfile>> {
  const result = await makeAuthenticatedRequest<PermissionProfile>(
    '/permission-profiles',
    {
      method: 'POST',
      body: JSON.stringify({
        name: data.name,
        description: data.description,
        category: data.category,
        permissions: data.permissions || [],
        target_tier: data.targetTier || 'free'
      })
    }
  )

  if (result.success) {
    revalidatePath('/admin/permission-profiles')
    revalidatePath('/admin/users')
  }

  return result
}

/**
 * Update an existing permission profile
 */
export async function updatePermissionProfile(
  id: string,
  data: Partial<PermissionProfile>
): Promise<ApiResponse<PermissionProfile>> {
  const updateData: Record<string, unknown> = {}
  
  if (data.name !== undefined) updateData.name = data.name
  if (data.description !== undefined) updateData.description = data.description
  if (data.category !== undefined) updateData.category = data.category
  if (data.permissions !== undefined) updateData.permissions = data.permissions
  if (data.targetTier !== undefined) updateData.target_tier = data.targetTier
  if (data.isActive !== undefined) updateData.is_active = data.isActive

  const result = await makeAuthenticatedRequest<PermissionProfile>(
    `/permission-profiles/${id}`,
    {
      method: 'PUT',
      body: JSON.stringify(updateData)
    }
  )

  if (result.success) {
    revalidatePath('/admin/permission-profiles')
    revalidatePath('/admin/users')
  }

  return result
}

/**
 * Delete (soft delete) a permission profile
 */
export async function deletePermissionProfile(id: string): Promise<ApiResponse<void>> {
  const result = await makeAuthenticatedRequest<void>(
    `/permission-profiles/${id}`,
    {
      method: 'DELETE'
    }
  )

  if (result.success) {
    revalidatePath('/admin/permission-profiles')
    revalidatePath('/admin/users')
  }

  return result
}

/**
 * Assign a permission profile to a user
 */
export async function assignPermissionProfile(data: {
  userId: string
  profileId: string
  assignedBy?: string
  reason?: string
  mergePermissions?: boolean
  expiresAt?: string
}): Promise<ApiResponse<any>> {
  const result = await makeAuthenticatedRequest(
    '/permission-profiles/assign',
    {
      method: 'POST',
      body: JSON.stringify({
        profile_id: data.profileId,
        user_ids: [data.userId],
        reason: data.reason,
        merge_permissions: data.mergePermissions ?? true,
        expires_at: data.expiresAt,
        notify_users: false
      })
    }
  )

  if (result.success) {
    revalidatePath(`/admin/users/${data.userId}`)
    revalidatePath(`/admin/users/${data.userId}/permissions`)
  }

  return result
}

/**
 * Unassign a permission profile from a user
 */
export async function unassignPermissionProfile(
  data: UnassignProfileRequest
): Promise<ApiResponse<UnassignProfileResponse>> {
  const result = await makeAuthenticatedRequest<UnassignProfileResponse>(
    '/permission-profiles/unassign',
    {
      method: 'DELETE',
      body: JSON.stringify({
        user_id: data.userId,
        profile_id: data.profileId,
        reason: data.reason
      })
    }
  )

  if (result.success) {
    revalidatePath(`/admin/users/${data.userId}`)
    revalidatePath(`/admin/users/${data.userId}/permissions`)
  }

  return result
}

/**
 * Validate permission profile assignment for a single user
 */
export async function validatePermissionProfileAssignment(
  data: ValidateAssignmentRequest
): Promise<ApiResponse<ValidateAssignmentResponse>> {
  const result = await makeAuthenticatedRequest<ValidateAssignmentResponse>(
    '/permission-profiles/validate-assignment',
    {
      method: 'POST',
      body: JSON.stringify({
        user_id: data.userId,
        profile_id: data.profileId
      })
    }
  )

  return result
}

/**
 * Bulk validate permission profile assignments
 */
export async function bulkValidatePermissionProfileAssignment(
  data: BulkValidateAssignmentRequest
): Promise<ApiResponse<BulkValidateAssignmentResponse>> {
  const result = await makeAuthenticatedRequest<BulkValidateAssignmentResponse>(
    '/permission-profiles/bulk-validate',
    {
      method: 'POST',
      body: JSON.stringify({
        user_ids: data.userIds,
        profile_id: data.profileId
      })
    }
  )

  return result
}

/**
 * Get available permission profile categories
 */
export async function getPermissionProfileCategories(): Promise<ApiResponse<{ categories: Array<{ id: string, name: string, description: string }> }>> {
  const result = await makeAuthenticatedRequest<{ categories: Array<{ id: string, name: string, description: string }> }>(
    '/permission-profiles/categories'
  )

  return result
}

/**
 * Get available permission profile tiers
 */
export async function getPermissionProfileTiers(): Promise<ApiResponse<{ tiers: Array<{ id: string, name: string, description: string }> }>> {
  const result = await makeAuthenticatedRequest<{ tiers: Array<{ id: string, name: string, description: string }> }>(
    '/permission-profiles/tiers'
  )

  return result
}