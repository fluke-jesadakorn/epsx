'use server';

import { createAdminApiClient } from '@/lib/api-client';
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

async function getAdminClient() {
  const cookieStore = await cookies();
  const sessionToken = cookieStore.get('admin_session_token')?.value;

  // We pass the token explicitly to ensure it's used
  return createAdminApiClient({
    serverSide: true,
    token: sessionToken,
  });
}

// ============================================================================
// NOTIFICATION ACTIONS
// ============================================================================

export async function createAdminNotification(
  params: CreateNotificationParams
): Promise<ActionResult> {
  try {
    const client = await getAdminClient();

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

    const response = await client.post<any>('/api/admin/notifications/send', requestBody);

    if (!response.success) {
      return {
        success: false,
        error: response.error || 'Failed to create notification',
      };
    }

    return {
      success: true,
      data: response.data,
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
    const client = await getAdminClient();

    const response = await client.post<any>('/api/admin/permissions/cleanup');

    if (!response.success) {
      return {
        success: false,
        error: response.error || 'Failed to cleanup permissions',
      };
    }

    return {
      success: true,
      data: response.data,
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
