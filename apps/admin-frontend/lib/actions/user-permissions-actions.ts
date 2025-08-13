/**
 * User Permission Server Actions - Focused on permission management
 * Extracted from unified-user-actions.ts for better maintainability
 */

'use server'

import { revalidatePath } from 'next/cache'
import { getBearerToken } from '@/lib/actions/server-auth'
import { logger } from '@/lib/logger'
import type { UserOperationResult } from '@/lib/types/unified-user'

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080'

export interface AssignTemporaryPermissionData {
  userId: string
  resource: string
  action: string
  expires: Date
  reason?: string
}

export interface PermissionHistoryEntry {
  id: string
  action: string
  resource: string
  granted: boolean
  timestamp: Date
  reason?: string
  grantedBy: string
  expires?: Date
}

export interface BulkPermissionData {
  userIds: string[]
  permissions: string[]
  reason?: string
  expires?: Date
}

/**
 * Assign user role
 */
export async function assignUserRole(data: {
  userId: string
  roleId: string
  reason?: string
}): Promise<UserOperationResult> {
  try {
    logger.action.start('assignUserRole', { userId: data.userId, roleId: data.roleId })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${data.userId}/roles`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        role_id: data.roleId,
        reason: data.reason
      }),
    })
    
    if (!response.ok) {
      logger.action.error('assignUserRole', `Failed to assign role: ${response.statusText}`, data)
      return { 
        success: false, 
        error: { 
          code: 'ASSIGN_ERROR', 
          message: `Failed to assign role: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    logger.admin.permission(`Role ${data.roleId} assigned`, { userId: data.userId })
    logger.action.success('assignUserRole', data)
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('assignUserRole', error, data)
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
 * Remove user role
 */
export async function removeUserRole(data: {
  userId: string
  roleId: string
  reason?: string
}): Promise<UserOperationResult> {
  try {
    logger.action.start('removeUserRole', { userId: data.userId, roleId: data.roleId })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${data.userId}/roles/${data.roleId}`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        reason: data.reason
      }),
    })
    
    if (!response.ok) {
      logger.action.error('removeUserRole', `Failed to remove role: ${response.statusText}`, data)
      return { 
        success: false, 
        error: { 
          code: 'REMOVE_ERROR', 
          message: `Failed to remove role: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    logger.admin.permission(`Role ${data.roleId} removed`, { userId: data.userId })
    logger.action.success('removeUserRole', data)
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('removeUserRole', error, data)
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
 * Assign permission profile to user
 */
export async function assignPermissionProfile(data: {
  userId: string
  profileId: string
  expires?: Date
  reason?: string
}): Promise<UserOperationResult> {
  try {
    logger.action.start('assignPermissionProfile', { userId: data.userId, profileId: data.profileId })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${data.userId}/permission-profiles`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        profile_id: data.profileId,
        expires_at: data.expires?.toISOString(),
        reason: data.reason
      }),
    })
    
    if (!response.ok) {
      logger.action.error('assignPermissionProfile', `Failed to assign permission profile: ${response.statusText}`, data)
      return { 
        success: false, 
        error: { 
          code: 'ASSIGN_ERROR', 
          message: `Failed to assign permission profile: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    logger.admin.permission(`Permission profile ${data.profileId} assigned`, { userId: data.userId })
    logger.action.success('assignPermissionProfile', data)
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('assignPermissionProfile', error, data)
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
 * Add custom permission to user
 */
export async function addCustomPermission(data: {
  userId: string
  resource: string
  action: string
  expires?: Date
  reason?: string
}): Promise<UserOperationResult> {
  try {
    logger.action.start('addCustomPermission', { 
      userId: data.userId, 
      resource: data.resource, 
      action: data.action 
    })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${data.userId}/permissions`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        resource: data.resource,
        action: data.action,
        expires_at: data.expires?.toISOString(),
        reason: data.reason
      }),
    })
    
    if (!response.ok) {
      logger.action.error('addCustomPermission', `Failed to add custom permission: ${response.statusText}`, data)
      return { 
        success: false, 
        error: { 
          code: 'ADD_ERROR', 
          message: `Failed to add custom permission: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    logger.admin.permission(`Custom permission ${data.resource}:${data.action} added`, { userId: data.userId })
    logger.action.success('addCustomPermission', data)
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('addCustomPermission', error, data)
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
  permissionId: string
  reason?: string
}): Promise<UserOperationResult> {
  try {
    logger.action.start('removeCustomPermission', { 
      userId: data.userId, 
      permissionId: data.permissionId 
    })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${data.userId}/permissions/${data.permissionId}`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        reason: data.reason
      }),
    })
    
    if (!response.ok) {
      logger.action.error('removeCustomPermission', `Failed to remove custom permission: ${response.statusText}`, data)
      return { 
        success: false, 
        error: { 
          code: 'REMOVE_ERROR', 
          message: `Failed to remove custom permission: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    logger.admin.permission(`Custom permission removed`, { userId: data.userId, permissionId: data.permissionId })
    logger.action.success('removeCustomPermission', data)
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('removeCustomPermission', error, data)
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
 * Get permission history for user
 */
export async function getPermissionHistory(userId: string, limit = 50): Promise<UserOperationResult<PermissionHistoryEntry[]>> {
  try {
    logger.action.start('getPermissionHistory', { userId, limit })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/permissions/history?limit=${limit}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      next: { revalidate: 60 }
    })
    
    if (!response.ok) {
      logger.action.error('getPermissionHistory', `Failed to fetch permission history: ${response.statusText}`, { userId })
      return { 
        success: false, 
        error: { 
          code: 'FETCH_ERROR', 
          message: `Failed to fetch permission history: ${response.statusText}` 
        } 
      }
    }
    
    const rawHistory = await response.json()
    
    const history: PermissionHistoryEntry[] = rawHistory.map((entry: any) => ({
      id: entry.id,
      action: entry.action,
      resource: entry.resource,
      granted: entry.granted,
      timestamp: new Date(entry.timestamp),
      reason: entry.reason,
      grantedBy: entry.granted_by,
      expires: entry.expires ? new Date(entry.expires) : undefined
    }))
    
    logger.action.success('getPermissionHistory', { userId, historyCount: history.length })
    
    return { success: true, data: history }
    
  } catch (error) {
    logger.action.error('getPermissionHistory', error, { userId })
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
 * Bulk assign permissions to multiple users
 */
export async function bulkAssignPermissions(data: BulkPermissionData): Promise<UserOperationResult> {
  try {
    logger.action.start('bulkAssignPermissions', { 
      userCount: data.userIds.length, 
      permissionCount: data.permissions.length 
    })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/bulk/assign-permissions`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_ids: data.userIds,
        permissions: data.permissions,
        expires_at: data.expires?.toISOString(),
        reason: data.reason
      }),
    })
    
    if (!response.ok) {
      logger.action.error('bulkAssignPermissions', `Bulk assign failed: ${response.statusText}`, data)
      return { 
        success: false, 
        error: { 
          code: 'BULK_ASSIGN_ERROR', 
          message: `Bulk assign failed: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages for affected users
    data.userIds.forEach(userId => {
      revalidatePath(`/users/${userId}`)
      revalidatePath(`/users/${userId}/permissions`)
    })
    revalidatePath('/users')
    
    logger.admin.permission(`Bulk assigned ${data.permissions.length} permissions to ${data.userIds.length} users`)
    logger.action.success('bulkAssignPermissions', { 
      userCount: data.userIds.length, 
      permissionCount: data.permissions.length 
    })
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('bulkAssignPermissions', error, data)
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
 * Assign temporary permission to user
 */
export async function assignTemporaryPermission(data: AssignTemporaryPermissionData): Promise<UserOperationResult> {
  try {
    logger.action.start('assignTemporaryPermission', { 
      userId: data.userId, 
      resource: data.resource, 
      action: data.action,
      expires: data.expires.toISOString()
    })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/${data.userId}/permissions/temporary`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        resource: data.resource,
        action: data.action,
        expires_at: data.expires.toISOString(),
        reason: data.reason
      }),
    })
    
    if (!response.ok) {
      logger.action.error('assignTemporaryPermission', `Failed to assign temporary permission: ${response.statusText}`, data)
      return { 
        success: false, 
        error: { 
          code: 'TEMP_ASSIGN_ERROR', 
          message: `Failed to assign temporary permission: ${response.statusText}` 
        } 
      }
    }
    
    // Revalidate user pages
    revalidatePath(`/users/${data.userId}`)
    revalidatePath(`/users/${data.userId}/permissions`)
    revalidatePath('/users')
    
    logger.admin.permission(`Temporary permission ${data.resource}:${data.action} assigned until ${data.expires.toISOString()}`, { userId: data.userId })
    logger.action.success('assignTemporaryPermission', data)
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('assignTemporaryPermission', error, data)
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
 * Get expiring permissions (within specified days)
 */
export async function getExpiringPermissions(days = 7): Promise<UserOperationResult<any[]>> {
  try {
    logger.action.start('getExpiringPermissions', { days })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/permissions/expiring?days=${days}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      next: { revalidate: 300 }
    })
    
    if (!response.ok) {
      logger.action.error('getExpiringPermissions', `Failed to fetch expiring permissions: ${response.statusText}`, { days })
      return { 
        success: false, 
        error: { 
          code: 'FETCH_ERROR', 
          message: `Failed to fetch expiring permissions: ${response.statusText}` 
        } 
      }
    }
    
    const permissions = await response.json()
    
    logger.action.success('getExpiringPermissions', { days, count: permissions.length })
    
    return { success: true, data: permissions }
    
  } catch (error) {
    logger.action.error('getExpiringPermissions', error, { days })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}