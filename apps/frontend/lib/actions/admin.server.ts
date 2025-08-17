'use server';

import { createApiClient, isApiError } from '@/lib/api-client';

const getClient = () => createApiClient();

/**
 * Server Action to get admin configuration
 */
export async function getAdminConfig(): Promise<{ adminUrl: string }> {
  try {
    const client = getClient();
    const result = await client.serverGetAdminConfig();
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to get admin config');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Admin config error:', error);
    throw error;
  }
}

/**
 * Server Action to get VAPID key for push notifications
 */
export async function getVapidKey(): Promise<{ vapidPublicKey: string }> {
  try {
    const client = getClient();
    const result = await client.serverGetVapidKey();
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to get VAPID key');
    }
    
    return result.data!;
  } catch (error) {
    console.error('VAPID key error:', error);
    throw error;
  }
}