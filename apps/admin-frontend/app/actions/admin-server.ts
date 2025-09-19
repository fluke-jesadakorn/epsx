'use server';

import { revalidatePath } from 'next/cache';
import { cookies } from 'next/headers';
import { env } from '../../config/env';

// Server action utilities
async function makeServerRequest(endpoint: string, options: RequestInit = {}) {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;

  const headers = {
    'Content-Type': 'application/json',
    ...(token && { Authorization: `Bearer ${token}` }),
    ...options.headers,
  };

  const response = await fetch(`${env.BACKEND_URL}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(
      `HTTP ${response.status}: ${errorText || 'Request failed'}`
    );
  }

  return response.json();
}

interface CreateUserData {
  email: string;
  name?: string;
  role?: string;
  permissions?: string[];
}

interface UpdateUserData {
  name?: string;
  role?: string;
  permissions?: string[];
  is_active?: boolean;
}

// User Management Server Actions
export async function createUser(data: CreateUserData) {
  try {
    const result = await makeServerRequest('/api/v1/admin/users', {
      method: 'POST',
      body: JSON.stringify(data),
    });

    revalidatePath('/users');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error creating user:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Failed to create user',
    };
  }
}

export async function updateUser(userId: string, data: UpdateUserData) {
  try {
    const result = await makeServerRequest(`/api/v1/admin/users/${userId}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error updating user:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Failed to update user',
    };
  }
}

export async function deleteUser(userId: string) {
  try {
    await makeServerRequest(`/api/v1/admin/users/${userId}`, {
      method: 'DELETE',
    });

    revalidatePath('/users');
    return { success: true };
  } catch (error) {
    console.error('Error deleting user:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Failed to delete user',
    };
  }
}

export async function fetchUserDetails() {
  try {
    const result = await makeServerRequest('/api/v1/admin/users');
    return result.users || [];
  } catch (error) {
    console.error('Error fetching users:', error);
    return [];
  }
}

export async function updateUserRole(userId: string, role: string) {
  try {
    const result = await makeServerRequest(
      `/api/v1/admin/users/${userId}/role`,
      {
        method: 'PATCH',
        body: JSON.stringify({ role }),
      }
    );

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
    return {
      success: true,
      message: 'User role updated successfully',
      data: result,
    };
  } catch (error) {
    console.error('Error updating user role:', error);
    return {
      success: false,
      error:
        error instanceof Error ? error.message : 'Failed to update user role',
    };
  }
}

// Permission Management Server Actions
export async function grantPermission(
  userId: string,
  permission: string,
  expiresAt?: number
) {
  try {
    const permissionData = expiresAt
      ? `${permission}:${expiresAt}`
      : permission;

    const result = await makeServerRequest(
      `/api/v1/admin/users/${userId}/permissions`,
      {
        method: 'POST',
        body: JSON.stringify({
          permissions: [permissionData],
          action: 'grant',
        }),
      }
    );

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error granting permission:', error);
    return {
      success: false,
      error:
        error instanceof Error ? error.message : 'Failed to grant permission',
    };
  }
}

export async function revokePermission(userId: string, permission: string) {
  try {
    const result = await makeServerRequest(
      `/api/v1/admin/users/${userId}/permissions`,
      {
        method: 'DELETE',
        body: JSON.stringify({
          permissions: [permission],
          action: 'revoke',
        }),
      }
    );

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error revoking permission:', error);
    return {
      success: false,
      error:
        error instanceof Error ? error.message : 'Failed to revoke permission',
    };
  }
}

export async function bulkGrantPermissions(
  userIds: string[],
  permissions: string[]
) {
  try {
    const result = await makeServerRequest(
      '/api/v1/admin/users/bulk/permissions',
      {
        method: 'POST',
        body: JSON.stringify({
          user_ids: userIds,
          permissions,
          action: 'grant',
        }),
      }
    );

    revalidatePath('/users');
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error bulk granting permissions:', error);
    return {
      success: false,
      error:
        error instanceof Error ? error.message : 'Failed to grant permissions',
    };
  }
}

export async function bulkRevokePermissions(
  userIds: string[],
  permissions: string[]
) {
  try {
    const result = await makeServerRequest(
      '/api/v1/admin/users/bulk/permissions',
      {
        method: 'DELETE',
        body: JSON.stringify({
          user_ids: userIds,
          permissions,
          action: 'revoke',
        }),
      }
    );

    revalidatePath('/users');
    revalidatePath('/permissions');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error bulk revoking permissions:', error);
    return {
      success: false,
      error:
        error instanceof Error ? error.message : 'Failed to revoke permissions',
    };
  }
}

// Notification Server Actions
export async function sendNotification(
  userIdOrEmail: string,
  title: string,
  message: string,
  priority: 'normal' | 'high' = 'normal'
) {
  try {
    const result = await makeServerRequest('/api/v1/admin/notifications', {
      method: 'POST',
      body: JSON.stringify({
        title,
        message,
        notification_type: 'admin_message',
        category: 'admin',
        priority,
        target_user_email: userIdOrEmail.includes('@') ? userIdOrEmail : null,
        target_users: userIdOrEmail.includes('@') ? null : [userIdOrEmail],
        channels: ['push', 'in_app'],
      }),
    });

    revalidatePath('/notifications');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error sending notification:', error);
    return {
      success: false,
      error:
        error instanceof Error ? error.message : 'Failed to send notification',
    };
  }
}

export async function sendBroadcastNotification(
  title: string,
  message: string,
  priority: 'normal' | 'high' = 'normal'
) {
  try {
    const result = await makeServerRequest('/api/v1/admin/notifications', {
      method: 'POST',
      body: JSON.stringify({
        title,
        message,
        notification_type: 'admin_broadcast',
        category: 'admin',
        priority,
        target_users: null,
        channels: ['push', 'in_app'],
      }),
    });

    revalidatePath('/notifications');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error sending broadcast notification:', error);
    return {
      success: false,
      error:
        error instanceof Error
          ? error.message
          : 'Failed to send broadcast notification',
    };
  }
}

export async function markNotificationAsRead(notificationId: string) {
  try {
    const result = await makeServerRequest(
      `/api/v1/notifications/${notificationId}/read`,
      {
        method: 'PATCH',
      }
    );

    revalidatePath('/notifications');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error marking notification as read:', error);
    return {
      success: false,
      error:
        error instanceof Error
          ? error.message
          : 'Failed to mark notification as read',
    };
  }
}

export async function getUserStats() {
  try {
    const result = await makeServerRequest(
      '/api/v1/admin/analytics/user-statistics'
    );
    return result;
  } catch (error) {
    console.error('Error getting user stats:', error);
    throw error;
  }
}

// IAM Server Actions
async function makeIAMRequest(endpoint: string, options: RequestInit = {}) {
  const { cookies } = await import('next/headers');
  const cookieStore = await cookies();
  const allCookies = cookieStore.getAll();
  const cookieHeader = allCookies.map(c => `${c.name}=${c.value}`).join('; ');

  const backendUrl = env.BACKEND_URL;
  const url = `${backendUrl}/api/v1/iam/${endpoint}`;

  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      Cookie: cookieHeader,
      ...options.headers,
    },
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(
      `HTTP ${response.status}: ${errorText || 'Request failed'}`
    );
  }

  return response.json();
}

interface IAMUserFilters {
  role?: string;
  status?: string;
  search?: string;
  limit?: number;
  offset?: number;
}

export async function getIAMUsers(filters?: IAMUserFilters) {
  try {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined) {
          queryParams.append(key, String(value));
        }
      });
    }

    const endpoint = `users${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    return await makeIAMRequest(endpoint);
  } catch (error) {
    console.error('Error fetching IAM users:', error);
    return [];
  }
}

export async function getIAMRoles() {
  try {
    return await makeIAMRequest('roles');
  } catch (error) {
    console.error('Error fetching IAM roles:', error);
    return [];
  }
}

export async function getIAMPolicies() {
  try {
    return await makeIAMRequest('policies');
  } catch (error) {
    console.error('Error fetching IAM policies:', error);
    return [];
  }
}
