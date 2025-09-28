'use server';

import { createApiClient, isApiError } from '@/lib/api-client';
import { requireAuth, requirePermission } from '../auth';
import { safeError } from '@/lib/utils/logging';
import { redirect } from 'next/navigation';

// ============================================================================
// System and General Server Actions
// ============================================================================

const getClient = () => createApiClient();

/**
 * Navigation helper - navigate to page with error handling
 */
export async function navigateToPage(page: number, basePath: string = ''): Promise<void> {
  try {
    const targetPath = basePath ? `${basePath}?page=${page}` : `?page=${page}`;
    redirect(targetPath);
  } catch (error) {
    console.error('Navigation error:', error);
    throw error;
  }
}

/**
 * Get system status
 */
export async function getSystemStatus(): Promise<any> {
  try {
    const client = getClient();
    const result = await client.serverGetSystemStatus();
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to get system status');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get system status error:', error);
    throw error;
  }
}

/**
 * Get feature flags
 */
export async function getFeatureFlags(): Promise<Record<string, boolean>> {
  try {
    const client = getClient();
    const result = await client.serverGetFeatureFlags();
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to get feature flags');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get feature flags error:', error);
    throw error;
  }
}

/**
 * Check feature access for user
 */
export async function checkSystemFeatureAccess(feature: string): Promise<{
  hasAccess: boolean;
  tier: string;
  feature: string;
  reason?: string;
}> {
  try {
    const user = await requireAuth();
    
    const client = getClient();
    const result = await client.serverCheckFeatureAccess(feature);
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to check feature access');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Check feature access error:', error);
    throw error;
  }
}

/**
 * Log client error to server
 */
export async function logClientError(error: {
  message: string;
  stack?: string;
  url?: string;
  userId?: string;
}): Promise<void> {
  try {
    const client = getClient();
    const result = await client.serverLogClientError(error);
    
    if (isApiError(result)) {
      console.error('Failed to log client error to server:', result.message);
    }
  } catch (serverError) {
    console.error('Server error logging failed:', serverError);
    // Don't throw here - we don't want logging errors to break the app
  }
}

/**
 * Get application configuration
 */
export async function getAppConfig(): Promise<any> {
  try {
    const client = getClient();
    const result = await client.serverGetAppConfig();
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to get app config');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get app config error:', error);
    throw error;
  }
}

/**
 * Update application settings (admin only)
 */
export async function updateAppSettings(settings: Record<string, any>): Promise<void> {
  try {
    await requirePermission('admin:settings:write');
    
    const client = getClient();
    const result = await client.serverUpdateAppSettings(settings);
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to update app settings');
    }
  } catch (error) {
    console.error('Update app settings error:', error);
    throw error;
  }
}

/**
 * Clear application cache (admin only)
 */
export async function clearAppCache(cacheType?: string): Promise<void> {
  try {
    await requirePermission('admin:system:manage');
    
    const client = getClient();
    const result = await client.serverClearAppCache(cacheType);
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to clear app cache');
    }
  } catch (error) {
    console.error('Clear app cache error:', error);
    throw error;
  }
}

/**
 * Get system logs (admin only)
 */
export async function getSystemLogs(params?: {
  level?: string;
  limit?: number;
  offset?: number;
  startDate?: string;
  endDate?: string;
}): Promise<any[]> {
  try {
    await requirePermission('admin:logs:read');
    
    const client = getClient();
    const result = await client.serverGetSystemLogs(params);
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to get system logs');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get system logs error:', error);
    throw error;
  }
}

/**
 * Export system data (admin only)
 */
export async function exportSystemData(dataType: string, format: 'json' | 'csv' = 'json'): Promise<Blob> {
  try {
    await requirePermission('admin:data:export');
    
    const client = getClient();
    const result = await client.serverExportSystemData(dataType, format);
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to export system data');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Export system data error:', error);
    throw error;
  }
}

/**
 * Send test notification (admin only)
 */
export async function sendTestNotification(userId: string, type: string = 'test'): Promise<void> {
  try {
    await requirePermission('admin:notifications:send');
    
    const client = getClient();
    const result = await client.serverSendTestNotification(userId, type);
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to send test notification');
    }
  } catch (error) {
    console.error('Send test notification error:', error);
    throw error;
  }
}