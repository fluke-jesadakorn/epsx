'use server'

import { revalidatePath } from 'next/cache'
import { getSession } from '@/lib/auth/session'

const BACKEND_BASE_URL = process.env.BACKEND_URL || 'http://localhost:8080'

// Bulk grant permissions to multiple users
export async function bulkGrantPermissionsAction(data: {
  userIds: string[]
  permissions: string[]
  reason?: string
}) {
  try {
    const session = await getSession()
    
    if (!session?.accessToken) {
      return { success: false, error: 'Authentication required' }
    }

    const response = await fetch(`${BACKEND_BASE_URL}/api/v1/admin/users/bulk/permissions/grant`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${session.accessToken}`,
      },
      body: JSON.stringify({
        user_ids: data.userIds,
        permissions: data.permissions,
        reason: data.reason,
        notify_users: false
      }),
    })

    if (!response.ok) {
      const errorText = await response.text()
      return { 
        success: false, 
        error: `HTTP ${response.status}: ${errorText}` 
      }
    }

    const result = await response.json()
    
    // Revalidate the users page to refresh the data
    revalidatePath('/users')
    
    return { success: true, data: result }

  } catch (error) {
    console.error('Bulk grant permissions server action failed:', error)
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'An unexpected error occurred' 
    }
  }
}

// Bulk revoke permissions from multiple users
export async function bulkRevokePermissionsAction(data: {
  userIds: string[]
  permissions: string[]
  reason?: string
}) {
  try {
    const session = await getSession()
    
    if (!session?.accessToken) {
      return { success: false, error: 'Authentication required' }
    }

    const response = await fetch(`${BACKEND_BASE_URL}/api/v1/admin/users/bulk/permissions/revoke`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${session.accessToken}`,
      },
      body: JSON.stringify({
        user_ids: data.userIds,
        permissions: data.permissions,
        reason: data.reason,
        notify_users: false
      }),
    })

    if (!response.ok) {
      const errorText = await response.text()
      return { 
        success: false, 
        error: `HTTP ${response.status}: ${errorText}` 
      }
    }

    const result = await response.json()
    
    // Revalidate the users page to refresh the data
    revalidatePath('/users')
    
    return { success: true, data: result }

  } catch (error) {
    console.error('Bulk revoke permissions server action failed:', error)
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'An unexpected error occurred' 
    }
  }
}

// Bulk assign roles to multiple users
export async function bulkAssignRolesAction(data: {
  userIds: string[]
  role: string
  mergePermissions: boolean
  reason?: string
}) {
  try {
    const session = await getSession()
    
    if (!session?.accessToken) {
      return { success: false, error: 'Authentication required' }
    }

    const response = await fetch(`${BACKEND_BASE_URL}/api/v1/admin/users/bulk/roles/assign`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${session.accessToken}`,
      },
      body: JSON.stringify({
        user_ids: data.userIds,
        role: data.role,
        merge_permissions: data.mergePermissions,
        reason: data.reason
      }),
    })

    if (!response.ok) {
      const errorText = await response.text()
      return { 
        success: false, 
        error: `HTTP ${response.status}: ${errorText}` 
      }
    }

    const result = await response.json()
    
    // Revalidate the users page to refresh the data
    revalidatePath('/users')
    
    return { success: true, data: result }

  } catch (error) {
    console.error('Bulk assign roles server action failed:', error)
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'An unexpected error occurred' 
    }
  }
}

// Bulk apply permission template to multiple users
export async function bulkApplyTemplateAction(data: {
  userIds: string[]
  templateId: string
  mergePermissions?: boolean
  reason?: string
}) {
  try {
    const session = await getSession()
    
    if (!session?.accessToken) {
      return { success: false, error: 'Authentication required' }
    }

    const response = await fetch(`${BACKEND_BASE_URL}/api/v1/admin/users/bulk/templates/apply`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${session.accessToken}`,
      },
      body: JSON.stringify({
        user_ids: data.userIds,
        template_id: data.templateId,
        merge_permissions: data.mergePermissions,
        reason: data.reason
      }),
    })

    if (!response.ok) {
      const errorText = await response.text()
      return { 
        success: false, 
        error: `HTTP ${response.status}: ${errorText}` 
      }
    }

    const result = await response.json()
    
    // Revalidate the users page to refresh the data
    revalidatePath('/users')
    
    return { success: true, data: result }

  } catch (error) {
    console.error('Bulk apply template server action failed:', error)
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'An unexpected error occurred' 
    }
  }
}

// Bulk validate permissions for multiple users
export async function bulkValidatePermissionsAction(data: {
  userIds: string[]
  checkExpired?: boolean
  checkConflicting?: boolean
}) {
  try {
    const session = await getSession()
    
    if (!session?.accessToken) {
      return { success: false, error: 'Authentication required' }
    }

    const response = await fetch(`${BACKEND_BASE_URL}/api/v1/admin/users/bulk/permissions/validate`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${session.accessToken}`,
      },
      body: JSON.stringify({
        user_ids: data.userIds,
        check_expired: data.checkExpired ?? true,
        check_conflicting: data.checkConflicting ?? true
      }),
    })

    if (!response.ok) {
      const errorText = await response.text()
      return { 
        success: false, 
        error: `HTTP ${response.status}: ${errorText}` 
      }
    }

    const result = await response.json()
    
    return { success: true, data: result }

  } catch (error) {
    console.error('Bulk validate permissions server action failed:', error)
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'An unexpected error occurred' 
    }
  }
}