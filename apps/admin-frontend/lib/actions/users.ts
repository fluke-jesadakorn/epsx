/**
 * Unified User Actions - Server Actions for user management operations
 * Consolidates user operations from multiple fragmented systems
 */

'use server'

import { revalidatePath } from 'next/cache'
import { redirect } from 'next/navigation'
import { getSession } from '@/lib/auth/session'
import { env } from '@/config/env'
import type { 
  UnifiedUserData, 
  UserProfileUpdateData,
  UserStatusUpdateData,
  UserRoleUpdateData,
  ModuleAccessUpdateData,
  BillingUpdateData,
  UserOperationResult 
} from '@/lib/types/unified-user'

/**
 * Profile update data interface
 */
export interface ProfileUpdateData {
  name?: string
  email?: string
  phone?: string | null
  timezone?: string
  language?: string
  status?: 'active' | 'inactive' | 'suspended'
  email_verified?: boolean
  two_factor_enabled?: boolean
}

const BACKEND_URL = env.BACKEND_URL

// Get bearer token from session
const getBearerToken = async () => {
  const session = await getSession();
  console.log('🔍 Session for token extraction:', {
    isLoggedIn: session.isLoggedIn,
    hasAccessToken: !!session.accessToken,
    userEmail: session.user?.email,
    tokenLength: session.accessToken?.length
  });
  return session.accessToken || null;
};

/**
 * User filters for enhanced user list
 */
export interface UserListFilters {
  search: string
  status: string
  role: string
  page: number
  limit: number
  sortBy: string
  sortOrder: 'asc' | 'desc'
}

/**
 * Temporary Permission interfaces
 */
export interface AssignTemporaryPermissionData {
  userId: string
  resource: string
  action: string
  expires: Date
  reason?: string
}

export interface UserListResult {
  users: UnifiedUserData[]
  total: number
  page: number
  totalPages: number
  limit: number
}

/**
 * Get paginated list of users with filtering and search
 */
export async function getUsersList(filters?: Partial<UserListFilters>): Promise<UserOperationResult<UserListResult>> {
  try {
    // Get auth token directly without redirect - we'll handle auth gracefully
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const searchParams = new URLSearchParams()
    
    if (filters?.page) searchParams.set('offset', ((filters.page - 1) * (filters.limit || 50)).toString())
    if (filters?.limit) searchParams.set('limit', filters.limit.toString())
    if (filters?.role && filters.role !== 'all') searchParams.set('role_filter', filters.role)
    if (filters?.search) searchParams.set('search', filters.search)
    
    const url = `${BACKEND_URL}/api/v1/admin/users${searchParams.toString() ? '?' + searchParams.toString() : ''}`
    
    const response = await fetch(url, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    })
    
    if (!response.ok) {
      return { 
        success: false, 
        error: { 
          code: 'FETCH_ERROR', 
          message: `Failed to fetch users list: ${response.statusText}` 
        } 
      }
    }
    
    const result = await response.json()
    
    const usersList: UserListResult = {
      users: result.users || [],
      total: result.total || 0,
      page: Math.floor((result.offset || 0) / (result.limit || 50)) + 1,
      totalPages: Math.ceil((result.total || 0) / (result.limit || 50)),
      limit: result.limit || 50,
    }
    
    return { success: true, data: usersList }
    
  } catch (error) {
    console.error('Get users list error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Get unified user data by ID
 */
export async function getUnifiedUserData(userId: string): Promise<UserOperationResult<UnifiedUserData>> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    // In a real implementation, this would fetch from multiple endpoints
    // For now, we'll structure it to show the intended API
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/unified`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    })
    
    if (!response.ok) {
      return { 
        success: false, 
        error: { 
          code: 'FETCH_ERROR', 
          message: `Failed to fetch user data: ${response.statusText}` 
        } 
      }
    }
    
    const rawData = await response.json()
    
    // Transform backend response to match frontend interface
    const userData = {
      // Map user profile data
      id: rawData.user.id,
      email: rawData.user.email,
      displayName: rawData.user.display_name || rawData.user.email.split('@')[0], // Fallback to email prefix
      firstName: null,
      lastName: null,
      avatar: rawData.user.profile_picture,
      emailVerified: true, // Assuming verified if in system
      createdAt: new Date(rawData.user.created_at),
      updatedAt: new Date(rawData.user.updated_at),
      lastLogin: rawData.user.last_login ? new Date(rawData.user.last_login) : undefined,
      
      // Map account info
      status: rawData.user.is_active ? 'active' : 'inactive' as const,
      phoneNumber: null,
      timezone: null,
      language: 'en',
      twoFactorEnabled: false, // Placeholder
      
      // Map permissions & roles
      roles: rawData.permissions.roles.map((role: string) => ({
        id: role,
        name: role,
        description: `${role} role`,
        permissions: [],
        createdAt: new Date()
      })),
      customPermissions: [],
      permissionProfiles: [],
      
      // Map modules
      moduleAccess: rawData.modules.enabled_modules || [],
      moduleQuotas: [
        {
          moduleId: 'api',
          quotaType: 'api_calls',
          limit: rawData.modules.quotas.api_calls_per_day,
          used: rawData.modules.quotas.api_calls_used,
          period: 'daily'
        }
      ],
      
      // Map billing
      billing: {
        tier: rawData.billing.subscription.tier,
        status: rawData.billing.subscription.status,
        nextBillingDate: rawData.billing.subscription.next_billing ? new Date(rawData.billing.subscription.next_billing) : undefined,
        amount: rawData.billing.subscription.amount,
        currency: rawData.billing.subscription.currency
      },
      stockRankingPackages: [],
      
      // Developer access
      apiKeys: [],
      
      // Activity
      recentActivity: [],
      loginHistory: [],
      usageMetrics: {
        apiCallsThisMonth: rawData.modules.quotas.api_calls_used,
        storageUsed: 0,
        lastActiveDate: rawData.activity.last_activity ? new Date(rawData.activity.last_activity) : new Date(),
        sessionsThisMonth: rawData.activity.total_logins,
        averageSessionDuration: 0
      }
    }
    
    return { success: true, data: userData }
    
  } catch (error) {
    console.error('Get unified user data error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Update user profile information
 */
export async function updateUserProfile(
  userId: string, 
  data: UserProfileUpdateData
): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/profile`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    })
    
    if (!response.ok) {
      return { 
        success: false, 
        error: { 
          code: 'UPDATE_ERROR', 
          message: `Failed to update profile: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${userId}`)
    revalidatePath(`/users/${userId}/overview`)
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Update user profile error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Update user status (active, disabled, etc.)
 */
export async function updateUserStatus(
  userId: string, 
  data: UserStatusUpdateData
): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/status`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    })
    
    if (!response.ok) {
      return { 
        success: false, 
        error: { 
          code: 'UPDATE_ERROR', 
          message: `Failed to update status: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${userId}`)
    revalidatePath(`/users/${userId}/overview`)
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Update user status error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Update user roles and permissions
 */
export async function updateUserRoles(
  userId: string, 
  data: UserRoleUpdateData
): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/roles`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    })
    
    if (!response.ok) {
      return { 
        success: false, 
        error: { 
          code: 'UPDATE_ERROR', 
          message: `Failed to update roles: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${userId}`)
    revalidatePath(`/users/${userId}/permissions`)
    revalidatePath('/users')
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Update user roles error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Update module access and quotas
 */
export async function updateModuleAccess(
  userId: string, 
  data: ModuleAccessUpdateData
): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/modules`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    })
    
    if (!response.ok) {
      return { 
        success: false, 
        error: { 
          code: 'UPDATE_ERROR', 
          message: `Failed to update modules: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${userId}`)
    revalidatePath(`/users/${userId}/modules`)
    revalidatePath('/users')
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Update module access error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Update billing information and packages
 */
export async function updateUserBilling(
  userId: string, 
  data: BillingUpdateData
): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/billing`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    })
    
    if (!response.ok) {
      return { 
        success: false, 
        error: { 
          code: 'UPDATE_ERROR', 
          message: `Failed to update billing: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${userId}`)
    revalidatePath(`/users/${userId}/packages`)
    revalidatePath('/users')
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Update user billing error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Create new user
 */
export async function createUser(formData: FormData): Promise<UserOperationResult<{ userId: string }>> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const userData = {
      email: formData.get('email') as string,
      display_name: formData.get('displayName') as string,
      role: formData.get('role') as string,
      password: formData.get('password') as string,
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(userData),
    })
    
    if (!response.ok) {
      return { 
        success: false, 
        error: { 
          code: 'CREATE_ERROR', 
          message: `Failed to create user: ${response.statusText}` 
        } 
      }
    }
    
    const result = await response.json()
    
    // Revalidate user list
    revalidatePath('/users')
    
    // Redirect to new user profile
    redirect(`/users/${result.user_id}`)
    
  } catch (error) {
    console.error('Create user error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Delete user (soft delete)
 */
export async function deleteUser(userId: string): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    })
    
    if (!response.ok) {
      return { 
        success: false, 
        error: { 
          code: 'DELETE_ERROR', 
          message: `Failed to delete user: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user list
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Delete user error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

// ========================================
// ROLE MANAGEMENT ACTIONS
// ========================================

/**
 * Assign role to user
 */
export async function assignUserRole(data: {
  userId: string
  role: string
  reason?: string
}): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/casbin/roles`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user: data.userId,
        role: data.role
      }),
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      let errorMessage = `Failed to assign role: ${response.status}`
      
      if (response.status === 409) {
        errorMessage = 'User already has this role'
      } else if (response.status === 403) {
        errorMessage = 'Insufficient permissions to assign this role'
      } else if (response.status === 404) {
        errorMessage = 'User or role not found'
      } else if (errorText) {
        errorMessage = `Failed to assign role: ${errorText}`
      }
      
      return { 
        success: false, 
        error: { 
          code: 'ROLE_ASSIGN_ERROR', 
          message: errorMessage
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Assign user role error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Remove role from user
 */
export async function removeUserRole(data: {
  userId: string
  role: string
  reason?: string
}): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/casbin/roles`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user: data.userId,
        role: data.role
      }),
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      let errorMessage = `Failed to remove role: ${response.status}`
      
      if (response.status === 404) {
        errorMessage = 'User does not have this role'
      } else if (response.status === 403) {
        errorMessage = 'Insufficient permissions to remove this role'
      } else if (response.status === 400) {
        errorMessage = 'Cannot remove required role'
      } else if (errorText) {
        errorMessage = `Failed to remove role: ${errorText}`
      }
      
      return { 
        success: false, 
        error: { 
          code: 'ROLE_REMOVE_ERROR', 
          message: errorMessage
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Remove user role error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

// ========================================
// PERMISSION PROFILES MANAGEMENT ACTIONS
// ========================================

/**
 * Assign permission profile to user
 */
export async function assignPermissionProfile(data: {
  userId: string
  profileId: string
  reason?: string
}): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/permission-profiles/assign`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        profile_id: data.profileId,
        user_ids: [data.userId]
      }),
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      let errorMessage = `Failed to assign permission profile: ${response.status}`
      
      if (response.status === 409) {
        errorMessage = 'User already has this permission profile'
      } else if (response.status === 403) {
        errorMessage = 'Insufficient permissions to assign this profile'
      } else if (response.status === 404) {
        errorMessage = 'Permission profile not found'
      } else if (errorText) {
        errorMessage = `Failed to assign permission profile: ${errorText}`
      }
      
      return { 
        success: false, 
        error: { 
          code: 'PROFILE_ASSIGN_ERROR', 
          message: errorMessage
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Assign permission profile error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

// ========================================
// CUSTOM PERMISSIONS MANAGEMENT ACTIONS
// ========================================

/**
 * Add custom permission to user
 */
export async function addCustomPermission(data: {
  userId: string
  resource: string
  action: string
  reason?: string
}): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/casbin/policies`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        subject: data.userId,
        object: data.resource,
        action: data.action
      }),
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      let errorMessage = `Failed to add permission: ${response.status}`
      
      if (response.status === 409) {
        errorMessage = 'User already has this permission'
      } else if (response.status === 403) {
        errorMessage = 'Insufficient permissions to grant this permission'
      } else if (response.status === 400) {
        errorMessage = 'Invalid permission resource or action'
      } else if (errorText) {
        errorMessage = `Failed to add permission: ${errorText}`
      }
      
      return { 
        success: false, 
        error: { 
          code: 'PERMISSION_ADD_ERROR', 
          message: errorMessage
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Add custom permission error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Remove custom permission from user
 */
export async function removeCustomPermission(data: {
  userId: string
  resource: string
  action: string
  reason?: string
}): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/casbin/policies`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        subject: data.userId,
        object: data.resource,
        action: data.action
      }),
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      return { 
        success: false, 
        error: { 
          code: 'PERMISSION_REMOVE_ERROR', 
          message: `Failed to remove permission: ${response.status} ${errorText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Remove custom permission error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

// ========================================
// PERMISSION HISTORY & AUDIT TRAIL
// ========================================

export interface PermissionHistoryEntry {
  id: string
  userId: string
  action: 'granted' | 'revoked' | 'modified'
  type: 'role' | 'permission' | 'profile'
  resource?: string
  permission?: string
  role?: string
  profileId?: string
  grantedBy: string
  grantedAt: Date
  reason?: string
  expires?: Date
}

/**
 * Get permission history for a user
 */
export async function getPermissionHistory(userId: string, limit = 50): Promise<UserOperationResult<PermissionHistoryEntry[]>> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/activity?limit=${limit}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      return { 
        success: false, 
        error: { 
          code: 'FETCH_ERROR', 
          message: `Failed to fetch permission history: ${response.status} ${errorText}` 
        } 
      }
    }
    
    const result = await response.json()
    // Transform user activity data to permission history format
    const history: PermissionHistoryEntry[] = (result.activities || [])
      .filter((activity: any) => 
        activity.action?.includes('permission') || 
        activity.action?.includes('role') || 
        activity.action?.includes('profile')
      )
      .map((activity: any) => ({
        id: activity.id,
        action: activity.action?.includes('granted') ? 'granted' : 
               activity.action?.includes('revoked') ? 'revoked' : 'modified',
        type: activity.action?.includes('role') ? 'role' : 
              activity.action?.includes('profile') ? 'profile' : 'permission',
        resource: activity.resource || '',
        permission: activity.details?.permission || '',
        role: activity.details?.role || '',
        profileId: activity.details?.profile_id || '',
        reason: activity.details?.reason || '',
        grantedBy: activity.performed_by || 'System',
        grantedAt: new Date(activity.created_at || activity.timestamp),
        expires: activity.expires_at ? new Date(activity.expires_at) : undefined
      }))
    
    return { success: true, data: history }
    
  } catch (error) {
    console.error('Get permission history error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

// ========================================
// BULK OPERATIONS
// ========================================

/**
 * Bulk assign permissions to multiple users
 */
export async function bulkAssignPermissions(data: {
  userIds: string[]
  permissions: { resource: string; action: string }[]
  reason?: string
}): Promise<UserOperationResult<{ succeeded: string[]; failed: { userId: string; error: string }[] }>> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/casbin/bulk-assign`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_ids: data.userIds,
        policies: data.permissions.map(p => ({
          object: p.resource,
          action: p.action
        })),
        reason: data.reason
      }),
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      return { 
        success: false, 
        error: { 
          code: 'BULK_ASSIGN_ERROR', 
          message: `Failed to bulk assign permissions: ${response.status} ${errorText}` 
        } 
      }
    }
    
    const result = await response.json()
    
    // Revalidate all affected user pages
    data.userIds.forEach(userId => {
      revalidatePath(`/users/${userId}`)
      revalidatePath(`/users/${userId}/permissions`)
    })
    revalidatePath('/users')
    
    return { success: true, data: result }
    
  } catch (error) {
    console.error('Bulk assign permissions error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Bulk remove permissions from multiple users
 */
export async function bulkRemovePermissions(data: {
  userIds: string[]
  permissions: { resource: string; action: string }[]
  reason?: string
}): Promise<UserOperationResult<{ succeeded: string[]; failed: { userId: string; error: string }[] }>> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/casbin/bulk-remove`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_ids: data.userIds,
        policies: data.permissions.map(p => ({
          object: p.resource,
          action: p.action
        })),
        reason: data.reason
      }),
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      return { 
        success: false, 
        error: { 
          code: 'BULK_REMOVE_ERROR', 
          message: `Failed to bulk remove permissions: ${response.status} ${errorText}` 
        } 
      }
    }
    
    const result = await response.json()
    
    // Revalidate all affected user pages
    data.userIds.forEach(userId => {
      revalidatePath(`/users/${userId}`)
      revalidatePath(`/users/${userId}/permissions`)
    })
    revalidatePath('/users')
    
    return { success: true, data: result }
    
  } catch (error) {
    console.error('Bulk remove permissions error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

// ========================================
// PERMISSION EXPIRATION & SCHEDULING
// ========================================

/**
 * Assign temporary permission with expiration
 */
export async function assignTemporaryPermission(data: {
  userId: string
  resource: string
  action: string
  expires: Date
  reason?: string
}): Promise<UserOperationResult> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/casbin/temporary-policies`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        subject: data.userId,
        object: data.resource,
        action: data.action,
        expires_at: data.expires.toISOString(),
        reason: data.reason
      }),
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      return { 
        success: false, 
        error: { 
          code: 'TEMP_PERMISSION_ERROR', 
          message: `Failed to assign temporary permission: ${response.status} ${errorText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    return { success: true }
    
  } catch (error) {
    console.error('Assign temporary permission error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Get all permissions that are expiring soon
 */
export async function getExpiringPermissions(days = 7): Promise<UserOperationResult<any[]>> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/casbin/expiring-permissions?days=${days}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      return { 
        success: false, 
        error: { 
          code: 'FETCH_ERROR', 
          message: `Failed to fetch expiring permissions: ${response.status} ${errorText}` 
        } 
      }
    }
    
    const result = await response.json()
    return { success: true, data: result.permissions || [] }
    
  } catch (error) {
    console.error('Get expiring permissions error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

// ========================================
// PERMISSION VALIDATION & SAFETY
// ========================================

/**
 * Validate permission assignment for conflicts
 */
export async function validatePermissionAssignment(data: {
  userId: string
  resource: string
  action: string
}): Promise<UserOperationResult<{ conflicts: any[]; warnings: string[] }>> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/casbin/validate-assignment`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        subject: data.userId,
        object: data.resource,
        action: data.action
      }),
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      return { 
        success: false, 
        error: { 
          code: 'VALIDATION_ERROR', 
          message: `Failed to validate permission: ${response.status} ${errorText}` 
        } 
      }
    }
    
    const result = await response.json()
    return { success: true, data: { conflicts: result.conflicts || [], warnings: result.warnings || [] } }
    
  } catch (error) {
    console.error('Validate permission assignment error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Get permission impact analysis for a user
 */
export async function getPermissionImpact(userId: string): Promise<UserOperationResult<{ canAccess: string[]; cannotAccess: string[]; totalResources: number }>> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/permission-impact`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    })
    
    if (!response.ok) {
      let errorText = 'Unknown error'
      try {
        errorText = await response.text()
      } catch {
        // Use default error text if parsing fails
      }
      return { 
        success: false, 
        error: { 
          code: 'FETCH_ERROR', 
          message: `Failed to get permission impact: ${response.status} ${errorText}` 
        } 
      }
    }
    
    const result = await response.json()
    return { success: true, data: result }
    
  } catch (error) {
    console.error('Get permission impact error:', error)
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

