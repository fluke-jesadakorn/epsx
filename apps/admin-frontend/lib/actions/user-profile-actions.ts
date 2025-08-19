/**
 * User Profile Server Actions - Focused on profile operations
 * Extracted from user-actions.ts for better maintainability
 */

'use server'

import { revalidatePath } from 'next/cache'
import { getBearerToken } from '@/lib/actions/server-auth'
import { logger } from '@/lib/logger'
import { env } from '@/config/env'
import type { 
  UnifiedUserData, 
  UserProfileUpdateData,
  UserStatusUpdateData,
  UserOperationResult 
} from '@/lib/types/unified-user'

const BACKEND_URL = env.BACKEND_URL

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

/**
 * Get unified user data by ID
 */
export async function getUnifiedUserData(userId: string): Promise<UserOperationResult<UnifiedUserData>> {
  try {
    logger.action.start('getUnifiedUserData', { userId })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/unified`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    })
    
    if (!response.ok) {
      logger.action.error('getUnifiedUserData', `Failed to fetch user data: ${response.statusText}`, { userId })
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
    const userData: UnifiedUserData = {
      // Map user profile data
      id: rawData.user.id,
      email: rawData.user.email,
      displayName: rawData.user.display_name || rawData.user.email.split('@')[0],
      firstName: null,
      lastName: null,
      avatar: rawData.user.profile_picture,
      emailVerified: true,
      createdAt: new Date(rawData.user.created_at),
      updatedAt: new Date(rawData.user.updated_at),
      lastLogin: rawData.user.last_login ? new Date(rawData.user.last_login) : undefined,
      
      // Map account info
      status: rawData.user.is_active ? 'active' : 'inactive' as const,
      phoneNumber: null,
      timezone: null,
      language: 'en',
      twoFactorEnabled: false,
      
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
    
    logger.action.success('getUnifiedUserData', { userId })
    return { success: true, data: userData }
    
  } catch (error) {
    logger.action.error('getUnifiedUserData', error, { userId })
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
    logger.action.start('updateUserProfile', { userId, fields: Object.keys(data) })
    
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
      logger.action.error('updateUserProfile', `Failed to update profile: ${response.statusText}`, { userId })
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
    
    logger.admin.userOperation('Profile updated', { userId })
    logger.action.success('updateUserProfile', { userId })
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('updateUserProfile', error, { userId })
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
    logger.action.start('updateUserStatus', { userId, status: data.status })
    
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
      logger.action.error('updateUserStatus', `Failed to update status: ${response.statusText}`, { userId })
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
    
    logger.admin.userOperation(`Status updated to ${data.status}`, { userId })
    logger.action.success('updateUserStatus', { userId, status: data.status })
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('updateUserStatus', error, { userId })
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
    const email = formData.get('email') as string
    const name = formData.get('name') as string
    const role = formData.get('role') as string
    
    logger.action.start('createUser', { email, role })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const userData = {
      email,
      name,
      role: role || 'user',
      status: 'active',
      email_verified: false
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
      logger.action.error('createUser', `Failed to create user: ${response.statusText}`, { email })
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
    
    logger.admin.userOperation('User created', { userId: result.user_id, email })
    logger.action.success('createUser', { userId: result.user_id, email })
    
    return { success: true, data: { userId: result.user_id } }
    
  } catch (error) {
    logger.action.error('createUser', error)
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
    logger.action.start('deleteUser', { userId })
    
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
      logger.action.error('deleteUser', `Failed to delete user: ${response.statusText}`, { userId })
      return { 
        success: false, 
        error: { 
          code: 'DELETE_ERROR', 
          message: `Failed to delete user: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath('/users')
    
    logger.admin.userOperation('User deleted', { userId })
    logger.action.success('deleteUser', { userId })
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('deleteUser', error, { userId })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}