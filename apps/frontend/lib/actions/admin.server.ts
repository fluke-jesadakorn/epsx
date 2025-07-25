'use server';

import { ApiClient } from '@epsx/api-client/api-client';

const client = new ApiClient();

/**
 * Server Action to get admin configuration
 */
export async function getAdminConfig(): Promise<{ adminUrl: string }> {
  try {
    const result = await client.serverGetAdminConfig();
    
    if (result.error || !result.data) {
      throw new Error(result.error || 'Failed to get admin config');
    }
    
    return result.data;
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
    const result = await client.serverGetVapidKey();
    
    if (result.error || !result.data) {
      throw new Error(result.error || 'Failed to get VAPID key');
    }
    
    return result.data;
  } catch (error) {
    console.error('VAPID key error:', error);
    throw error;
  }
}