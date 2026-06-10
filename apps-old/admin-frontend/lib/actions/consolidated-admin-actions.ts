'use server';

import { logger } from '@/lib/logger';
import { getAdminClient } from '../api-client';

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
// NOTIFICATION ACTIONS
// ============================================================================

/**
 *
 * @param params
 */
export async function createAdminNotification(
  params: CreateNotificationParams
): Promise<ActionResult> {
  try {
    const client = getAdminClient();

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
      broadcast: params.broadcast ?? false,
      notification_type: typeMap[params.type] ?? 'system',
      priority: priorityMap[params.priority] ?? 'normal',
      title: params.title,
      message: params.message,
      action_url: params.actionUrl,
      image_url: params.imageUrl,
      expires_at: params.expiresAt,
    };

    const response = await client.post('/api/admin/notifications/send', requestBody);

    if (!response.success) {
      return {
        success: false,
        error: typeof response.error === 'string' ? response.error : (response.error?.message ?? 'Failed to create notification'),
      };
    }

    return {
      success: true,
      data: response.data,
    };
  } catch (error) {
    logger.error('Failed to create admin notification:', { error });
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Failed to create notification',
    };
  }
}

/**
 *
 * @param params
 */
export async function broadcastNotification(
  params: Omit<CreateNotificationParams, 'walletAddress' | 'broadcast'>
): Promise<ActionResult> {
  return createAdminNotification({
    ...params,
    broadcast: true,
  });
}

/**
 *
 * @param params
 */
export async function sendNotification(
  params: CreateNotificationParams
): Promise<ActionResult> {
  return createAdminNotification(params);
}

/**
 *
 */
export async function cleanupExpiredPermissionsAction(): Promise<ActionResult> {
  try {
    const client = getAdminClient();

    const response = await client.post('/api/admin/permissions/cleanup');

    if (!response.success) {
      return {
        success: false,
        error: typeof response.error === 'string' ? response.error : (response.error?.message ?? 'Failed to cleanup permissions'),
      };
    }

    return {
      success: true,
      data: response.data,
    };
  } catch (error) {
    logger.error('Failed to cleanup permissions:', { error });
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Failed to cleanup permissions',
    };
  }
}
