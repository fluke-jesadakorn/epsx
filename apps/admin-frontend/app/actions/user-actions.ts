'use server';

import { cookies } from 'next/headers';
import { revalidatePath } from 'next/cache';
import { redirect } from 'next/navigation';
import { env } from '../../config/env';

// Server action utilities for user-specific operations
async function makeUserRequest(endpoint: string, options: RequestInit = {}) {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;
  
  const headers = {
    'Content-Type': 'application/json',
    ...(token && { 'Authorization': `Bearer ${token}` }),
    ...options.headers,
  };

  const response = await fetch(`${env.NEXT_PUBLIC_BACKEND_URL}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`HTTP ${response.status}: ${errorText || 'Request failed'}`);
  }

  return response.json();
}

// User Profile Management
export async function updateUserProfile(userId: string, profileData: {
  name?: string;
  email?: string;
  phone?: string;
  timezone?: string;
  language?: string;
}) {
  try {
    const result = await makeUserRequest(`/api/v1/users/${userId}/profile`, {
      method: 'PATCH',
      body: JSON.stringify(profileData),
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath(`/users/${userId}/overview`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error updating user profile:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to update profile' };
  }
}

// User Status Management
export async function activateUser(userId: string) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/activate`, {
      method: 'PATCH',
    });

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error activating user:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to activate user' };
  }
}

export async function deactivateUser(userId: string) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/deactivate`, {
      method: 'PATCH',
    });

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error deactivating user:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to deactivate user' };
  }
}

// User Session Management
export async function terminateUserSessions(userId: string) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/sessions`, {
      method: 'DELETE',
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath(`/users/${userId}/activity`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error terminating user sessions:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to terminate sessions' };
  }
}

export async function terminateSpecificSession(userId: string, sessionId: string) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/sessions/${sessionId}`, {
      method: 'DELETE',
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath(`/users/${userId}/activity`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error terminating specific session:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to terminate session' };
  }
}

// User Subscription Management
export async function updateUserSubscription(userId: string, subscriptionData: {
  tier?: string;
  expires_at?: string;
  auto_renew?: boolean;
}) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/subscription`, {
      method: 'PATCH',
      body: JSON.stringify(subscriptionData),
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath(`/users/${userId}/overview`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error updating user subscription:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to update subscription' };
  }
}

// User API Key Management
export async function generateUserApiKey(userId: string, keyData: {
  name: string;
  permissions?: string[];
  expires_at?: string;
}) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/api-keys`, {
      method: 'POST',
      body: JSON.stringify(keyData),
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath(`/users/${userId}/overview`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error generating user API key:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to generate API key' };
  }
}

export async function revokeUserApiKey(userId: string, keyId: string) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/api-keys/${keyId}`, {
      method: 'DELETE',
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath(`/users/${userId}/overview`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error revoking user API key:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to revoke API key' };
  }
}

// User Password Management
export async function resetUserPassword(userId: string, sendEmail: boolean = true) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/password-reset`, {
      method: 'POST',
      body: JSON.stringify({ send_email: sendEmail }),
    });

    revalidatePath(`/users/${userId}`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error resetting user password:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to reset password' };
  }
}

// User Impersonation (for admin debugging)
export async function impersonateUser(userId: string) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/impersonate`, {
      method: 'POST',
    });

    // Redirect to impersonation session
    if (result.impersonation_token) {
      redirect(`/impersonate?token=${result.impersonation_token}&user_id=${userId}`);
    }

    return { success: true, data: result };
  } catch (error) {
    console.error('Error impersonating user:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to impersonate user' };
  }
}

// User Data Export
export async function exportUserData(userId: string, format: 'json' | 'csv' = 'json') {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/export?format=${format}`, {
      method: 'GET',
    });

    return { success: true, data: result };
  } catch (error) {
    console.error('Error exporting user data:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to export user data' };
  }
}

// User Activity Logging
export async function addUserNote(userId: string, note: string, category: string = 'general') {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/notes`, {
      method: 'POST',
      body: JSON.stringify({ 
        note, 
        category,
        created_at: new Date().toISOString()
      }),
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath(`/users/${userId}/activity`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error adding user note:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to add note' };
  }
}

// User Verification
export async function verifyUserEmail(userId: string) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/verify-email`, {
      method: 'PATCH',
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath(`/users/${userId}/overview`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error verifying user email:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to verify email' };
  }
}

export async function resendVerificationEmail(userId: string) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/resend-verification`, {
      method: 'POST',
    });

    return { success: true, data: result };
  } catch (error) {
    console.error('Error resending verification email:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to resend verification' };
  }
}

// User Security Actions
export async function flagUserAccount(userId: string, reason: string) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/flag`, {
      method: 'POST',
      body: JSON.stringify({ 
        reason,
        flagged_at: new Date().toISOString()
      }),
    });

    revalidatePath(`/users/${userId}`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error flagging user account:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to flag account' };
  }
}

export async function unflagUserAccount(userId: string) {
  try {
    const result = await makeUserRequest(`/api/v1/admin/users/${userId}/unflag`, {
      method: 'DELETE',
    });

    revalidatePath(`/users/${userId}`);
    return { success: true, data: result };
  } catch (error) {
    console.error('Error unflagging user account:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to unflag account' };
  }
}

// Bulk User Operations
export async function bulkUpdateUsers(userIds: string[], updates: {
  is_active?: boolean;
  subscription_tier?: string;
  role?: string;
}) {
  try {
    const result = await makeUserRequest('/api/v1/admin/users/bulk/update', {
      method: 'PATCH',
      body: JSON.stringify({ 
        user_ids: userIds,
        updates
      }),
    });

    revalidatePath('/users');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error bulk updating users:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to bulk update users' };
  }
}

export async function bulkDeleteUsers(userIds: string[]) {
  try {
    const result = await makeUserRequest('/api/v1/admin/users/bulk/delete', {
      method: 'DELETE',
      body: JSON.stringify({ user_ids: userIds }),
    });

    revalidatePath('/users');
    return { success: true, data: result };
  } catch (error) {
    console.error('Error bulk deleting users:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Failed to bulk delete users' };
  }
}