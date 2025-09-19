'use server';

import { createApiClient, isApiError } from '@/lib/api-client';
import { requirePermission } from '../auth';
import { safeError } from '@/lib/utils/logging';

// ============================================================================
// Admin Server Actions
// ============================================================================

const getClient = () => createApiClient();

/**
 * Server Action to get admin configuration
 */
export async function getAdminConfig(): Promise<{ adminUrl: string }> {
  try {
    // Require admin permissions
    await requirePermission('admin:config:read');
    
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
    // Require admin permissions
    await requirePermission('admin:notifications:manage');
    
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

/**
 * Server Action to get system health
 */
export async function getSystemHealth(): Promise<any> {
  try {
    // Require admin permissions
    await requirePermission('admin:system:read');
    
    const client = getClient();
    const result = await client.serverGetSystemHealth();
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to get system health');
    }
    
    return result.data!;
  } catch (error) {
    console.error('System health error:', error);
    throw error;
  }
}

/**
 * Server Action to manage user permissions
 */
export async function manageUserPermissions(userId: string, permissions: string[]): Promise<void> {
  try {
    // Require admin permissions
    await requirePermission('admin:users:manage');
    
    const client = getClient();
    const result = await client.serverManageUserPermissions(userId, permissions);
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to manage user permissions');
    }
  } catch (error) {
    console.error('Manage user permissions error:', error);
    throw error;
  }
}

/**
 * Server Action to get all users (admin only)
 */
export async function getAllUsers(): Promise<any[]> {
  try {
    // Require admin permissions
    await requirePermission('admin:users:read');
    
    const client = getClient();
    const result = await client.serverGetAllUsers();
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to get all users');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get all users error:', error);
    throw error;
  }
}

/**
 * Server Action to get system metrics
 */
export async function getSystemMetrics(): Promise<any> {
  try {
    // Require admin permissions
    await requirePermission('admin:system:read');
    
    const client = getClient();
    const result = await client.serverGetSystemMetrics();
    
    if (isApiError(result)) {
      throw new Error(result.error || 'Failed to get system metrics');
    }
    
    return result.data!;
  } catch (error) {
    console.error('System metrics error:', error);
    throw error;
  }
}