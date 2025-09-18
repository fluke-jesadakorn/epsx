'use server'

/**
 * Admin Server Actions - Interactive Operations
 * Handles user interactions with proper server-side validation
 */

import { cookies } from 'next/headers'
import { revalidatePath } from 'next/cache'
import { URL, URLContext, Service } from '../../../../shared/utils/url-resolver';

const API_BASE_URL = URL.get(Service.BACKEND, URLContext.SERVER);

// Server-side authenticated fetch for actions
async function actionFetch(endpoint: string, options: RequestInit = {}): Promise<any> {
  const cookieStore = cookies()
  const token = cookieStore.get('epsx_admin_jwt')?.value
  
  const headers = {
    'Content-Type': 'application/json',
    ...(token && { 'Authorization': `Bearer ${token}` }),
    ...options.headers,
  }

  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    ...options,
    headers,
  })

  if (!response.ok) {
    const error = await response.text()
    console.error(`Action API Error: ${response.status} ${response.statusText}`, error)
    throw new Error(`API Error: ${response.status} ${response.statusText}`)
  }

  return response.json()
}

// User Management Actions
export async function createUserAction(formData: FormData) {
  try {
    const email = formData.get('email') as string
    const permissions = formData.getAll('permissions') as string[]
    const displayName = formData.get('displayName') as string

    if (!email || !permissions.length) {
      return { success: false, error: 'Email and permissions are required' }
    }

    const result = await actionFetch('/api/v1/admin/users', {
      method: 'POST',
      body: JSON.stringify({
        email,
        permissions,
        display_name: displayName || undefined
      })
    })

    revalidatePath('/users')
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to create user:', error)
    return { success: false, error: 'Failed to create user' }
  }
}

export async function updateUserAction(userId: string, formData: FormData) {
  try {
    const permissions = formData.getAll('permissions') as string[]
    const email = formData.get('email') as string

    const updates: any = {}
    if (permissions.length > 0) updates.permissions = permissions
    if (email) updates.email = email

    if (Object.keys(updates).length === 0) {
      return { success: false, error: 'No updates provided' }
    }

    const result = await actionFetch(`/api/v1/admin/users/${userId}`, {
      method: 'PUT',
      body: JSON.stringify(updates)
    })

    revalidatePath('/users')
    revalidatePath(`/users/${userId}`)
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to update user:', error)
    return { success: false, error: 'Failed to update user' }
  }
}

export async function deleteUserAction(userId: string) {
  try {
    const result = await actionFetch(`/api/v1/admin/users/${userId}`, {
      method: 'DELETE'
    })

    revalidatePath('/users')
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to delete user:', error)
    return { success: false, error: 'Failed to delete user' }
  }
}

// Permission Management Actions
export async function grantPermissionAction(userId: string, formData: FormData) {
  try {
    const permission = formData.get('permission') as string
    const expiresAt = formData.get('expiresAt') as string
    const reason = formData.get('reason') as string

    if (!permission) {
      return { success: false, error: 'Permission is required' }
    }

    const data: any = { permission }
    if (expiresAt) data.expires_at = parseInt(expiresAt)
    if (reason) data.reason = reason

    const result = await actionFetch(`/api/v1/admin/users/${userId}/embedded-permissions`, {
      method: 'POST',
      body: JSON.stringify(data)
    })

    revalidatePath('/permissions')
    revalidatePath(`/users/${userId}`)
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to grant permission:', error)
    return { success: false, error: 'Failed to grant permission' }
  }
}

export async function extendPermissionAction(userId: string, formData: FormData) {
  try {
    const permission = formData.get('permission') as string
    const extendByHours = formData.get('extendByHours') as string

    if (!permission || !extendByHours) {
      return { success: false, error: 'Permission and extension duration are required' }
    }

    const result = await actionFetch(`/api/v1/admin/users/${userId}/embedded-permissions/extend`, {
      method: 'POST',
      body: JSON.stringify({
        permission,
        extend_by_hours: parseInt(extendByHours)
      })
    })

    revalidatePath('/permissions')
    revalidatePath(`/users/${userId}`)
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to extend permission:', error)
    return { success: false, error: 'Failed to extend permission' }
  }
}

export async function revokePermissionAction(userId: string, permission: string) {
  try {
    const result = await actionFetch(`/api/v1/admin/users/${userId}/embedded-permissions/revoke`, {
      method: 'POST',
      body: JSON.stringify({ permission })
    })

    revalidatePath('/permissions')
    revalidatePath(`/users/${userId}`)
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to revoke permission:', error)
    return { success: false, error: 'Failed to revoke permission' }
  }
}

// Notification Actions
export async function markNotificationReadAction(notificationId: string) {
  try {
    const result = await actionFetch(`/api/v1/notifications/read/${notificationId}`, {
      method: 'POST'
    })

    revalidatePath('/notifications')
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to mark notification as read:', error)
    return { success: false, error: 'Failed to mark notification as read' }
  }
}

export async function markAllNotificationsReadAction() {
  try {
    const result = await actionFetch('/api/v1/notifications/read-all', {
      method: 'POST'
    })

    revalidatePath('/notifications')
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to mark all notifications as read:', error)
    return { success: false, error: 'Failed to mark all notifications as read' }
  }
}

// System Configuration Actions
export async function updateSystemConfigAction(formData: FormData) {
  try {
    const config: any = {}
    
    // Extract configuration values from form
    for (const [key, value] of formData.entries()) {
      if (key.startsWith('config_')) {
        const configKey = key.replace('config_', '')
        config[configKey] = value === 'on' ? true : value // Handle checkboxes
      }
    }

    if (Object.keys(config).length === 0) {
      return { success: false, error: 'No configuration updates provided' }
    }

    const result = await actionFetch('/api/v1/settings/system', {
      method: 'PUT',
      body: JSON.stringify(config)
    })

    revalidatePath('/system')
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to update system configuration:', error)
    return { success: false, error: 'Failed to update system configuration' }
  }
}

export async function updateFeatureFlagsAction(formData: FormData) {
  try {
    const flags: Record<string, boolean> = {}
    
    // Extract feature flags from form
    for (const [key, value] of formData.entries()) {
      if (key.startsWith('flag_')) {
        const flagKey = key.replace('flag_', '')
        flags[flagKey] = value === 'on'
      }
    }

    if (Object.keys(flags).length === 0) {
      return { success: false, error: 'No feature flags provided' }
    }

    const result = await actionFetch('/api/v1/settings/feature-flags', {
      method: 'PUT',
      body: JSON.stringify(flags)
    })

    revalidatePath('/system')
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to update feature flags:', error)
    return { success: false, error: 'Failed to update feature flags' }
  }
}

// Bulk Operations
export async function bulkUpdateUsersAction(formData: FormData) {
  try {
    const userIds = formData.getAll('userIds') as string[]
    const newPermissions = formData.getAll('newPermissions') as string[]
    const batchId = formData.get('batchId') as string

    if (!userIds.length) {
      return { success: false, error: 'No users selected' }
    }

    const data: any = { user_ids: userIds }
    if (newPermissions.length > 0) data.new_permissions = newPermissions
    if (batchId) data.batch_id = batchId

    const result = await actionFetch('/api/v1/admin/users/bulk-update', {
      method: 'POST',
      body: JSON.stringify(data)
    })

    revalidatePath('/users')
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to bulk update users:', error)
    return { success: false, error: 'Failed to bulk update users' }
  }
}

// Utility action for cleaning up expired permissions
export async function cleanupExpiredPermissionsAction() {
  try {
    const result = await actionFetch('/api/v1/admin/embedded-permissions/cleanup-expired', {
      method: 'POST'
    })

    revalidatePath('/permissions')
    return { success: true, data: result }
  } catch (error) {
    console.error('Failed to cleanup expired permissions:', error)
    return { success: false, error: 'Failed to cleanup expired permissions' }
  }
}