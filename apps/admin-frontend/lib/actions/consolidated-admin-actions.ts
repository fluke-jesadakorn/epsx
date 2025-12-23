'use server';

import { cookies } from 'next/headers';

// ============================================================================
// TYPES
// ============================================================================

interface CreateNotificationParams {
  title: string;
  message: string;
  priority: 'low' | 'medium' | 'high';
  type: 'info' | 'warning' | 'error' | 'success';
  walletAddress?: string;
  broadcast?: boolean;
  actionUrl?: string;
  imageUrl?: string;
  expiresAt?: string;
}

interface ActionResult<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async function getAdminSession() {
  const cookieStore = await cookies();
  const sessionToken = cookieStore.get('admin_session_token');

  if (!sessionToken) {
    throw new Error('Unauthorized: No admin session found');
  }

  return sessionToken.value;
}

async function makeAdminApiRequest(
  endpoint: string,
  options: RequestInit = {}
): Promise<Response> {
  const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
  const sessionToken = await getAdminSession();

  const response = await fetch(`${backendUrl}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${sessionToken}`,
      'X-API-Version': 'v1',
      'X-Access-Level': 'admin',
      'X-Admin-Context': 'true',
      ...options.headers,
    },
  });

  return response;
}

// ============================================================================
// NOTIFICATION ACTIONS
// ============================================================================

export async function createAdminNotification(
  params: CreateNotificationParams
): Promise<ActionResult> {
  try {
    // Map frontend priority to backend priority
    const priorityMap: Record<string, string> = {
      'low': 'low',
      'medium': 'normal',
      'high': 'critical'
    };

    // Map frontend type to backend notification type
    const typeMap: Record<string, string> = {
      'info': 'system',
      'warning': 'security',
      'error': 'security',
      'success': 'general'
    };

    const requestBody = {
      recipient_wallet_address: params.walletAddress,
      broadcast: params.broadcast || false,
      notification_type: typeMap[params.type] || 'system',
      priority: priorityMap[params.priority] || 'normal',
      title: params.title,
      message: params.message,
      action_url: params.actionUrl,
      image_url: params.imageUrl,
      expires_at: params.expiresAt,
    };

    const response = await makeAdminApiRequest('/api/admin/notifications/send', {
      method: 'POST',
      body: JSON.stringify(requestBody),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ message: 'Unknown error' }));
      return {
        success: false,
        error: errorData.message || `HTTP ${response.status}: ${response.statusText}`,
      };
    }

    const data = await response.json();

    return {
      success: true,
      data: data.data,
    };
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error('Failed to create admin notification:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Failed to create notification',
    };
  }
}

export async function broadcastNotification(
  params: Omit<CreateNotificationParams, 'walletAddress' | 'broadcast'>
): Promise<ActionResult> {
  return createAdminNotification({
    ...params,
    broadcast: true,
  });
}

export async function sendNotification(
  params: CreateNotificationParams
): Promise<ActionResult> {
  return createAdminNotification(params);
}

export async function cleanupExpiredPermissionsAction(): Promise<ActionResult> {
  try {
    const response = await makeAdminApiRequest('/api/v1/admin/permissions/cleanup', {
      method: 'POST',
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ message: 'Unknown error' }));
      return {
        success: false,
        error: errorData.message || `HTTP ${response.status}: ${response.statusText}`,
      };
    }

    const data = await response.json();
    return {
      success: true,
      data: data.data,
    };
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error('Failed to cleanup permissions:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Failed to cleanup permissions',
    };
  }
}
