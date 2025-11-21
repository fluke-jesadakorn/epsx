'use server';

import { revalidatePath } from 'next/cache';
import { cookies } from 'next/headers';

// Permission analytics data
interface PermissionAnalytics {
  totalPermissions: number;
  activePermissions: number;
  expiredPermissions: number;
  expiringSoon: number;
  platformDistribution: Record<string, number>;
  permissionHistory: PermissionHistoryItem[];
  usageStats: {
    mostUsedPermissions: Array<{ permission: string; usage: number }>;
    recentlyGranted: Array<{ permission: string; grantedAt: string }>;
  };
}

interface PermissionHistoryItem {
  id: string;
  permission: string;
  action: 'granted' | 'revoked' | 'expired';
  timestamp: string;
  reason?: string;
  grantedBy?: string;
  expiresAt?: string;
}

interface BatchPermissionRequest {
  permissions: string[];
  walletAddresses: string[];
  expiresAt?: string;
  reason?: string;
}

// Get permission analytics
export async function getPermissionAnalytics(): Promise<PermissionAnalytics> {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/permissions/analytics`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch analytics: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching permission analytics:', error);

    // Return mock data for development
    return {
      totalPermissions: 12,
      activePermissions: 10,
      expiredPermissions: 2,
      expiringSoon: 1,
      platformDistribution: {
        epsx: 6,
        admin: 3,
        'epsx-pay': 2,
        'epsx-token': 1,
      },
      permissionHistory: [
        {
          id: '1',
          permission: 'epsx:analytics:view',
          action: 'granted',
          timestamp: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
          reason: 'Default access level',
          grantedBy: 'system',
          expiresAt: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString(),
        },
        {
          id: '2',
          permission: 'admin:users:view',
          action: 'granted',
          timestamp: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString(),
          reason: 'Admin access granted',
          grantedBy: 'admin@epsx.com',
        },
        {
          id: '3',
          permission: 'epsx:trading:basic',
          action: 'expired',
          timestamp: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString(),
          reason: 'Temporary access expired',
        },
      ],
      usageStats: {
        mostUsedPermissions: [
          { permission: 'epsx:analytics:view', usage: 145 },
          { permission: 'admin:users:view', usage: 89 },
          { permission: 'epsx:trading:basic', usage: 67 },
        ],
        recentlyGranted: [
          { permission: 'epsx:notifications:manage', grantedAt: new Date(Date.now() - 1 * 60 * 60 * 1000).toISOString() },
          { permission: 'epsx:api:read', grantedAt: new Date(Date.now() - 3 * 60 * 60 * 1000).toISOString() },
        ],
      },
    };
  }
}

// Get permission history
export async function getPermissionHistory(limit: number = 20): Promise<PermissionHistoryItem[]> {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/permissions/history?limit=${limit}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch permission history: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching permission history:', error);
    return [];
  }
}

// Batch permission operations
export async function grantBatchPermissions(request: BatchPermissionRequest): Promise<{ success: boolean; message: string }> {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/permissions/batch/grant`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`Failed to grant batch permissions: ${response.status}`);
    }

    const result = await response.json();

    // Revalidate the permissions page
    revalidatePath('/permissions');

    return result;
  } catch (error) {
    console.error('Error granting batch permissions:', error);
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Unknown error occurred'
    };
  }
}

export async function revokeBatchPermissions(permissions: string[], reason?: string): Promise<{ success: boolean; message: string }> {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/permissions/batch/revoke`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ permissions, reason }),
    });

    if (!response.ok) {
      throw new Error(`Failed to revoke batch permissions: ${response.status}`);
    }

    const result = await response.json();

    // Revalidate the permissions page
    revalidatePath('/permissions');

    return result;
  } catch (error) {
    console.error('Error revoking batch permissions:', error);
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Unknown error occurred'
    };
  }
}

// Export permissions data
export async function exportPermissionsData(format: 'json' | 'csv' = 'json'): Promise<{ data: string; filename: string }> {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/permissions/export?format=${format}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to export permissions: ${response.status}`);
    }

    const data = await response.text();
    const timestamp = new Date().toISOString().split('T')[0];
    const filename = `permissions-export-${timestamp}.${format}`;

    return { data, filename };
  } catch (error) {
    console.error('Error exporting permissions:', error);
    throw error;
  }
}

// Check multiple permissions at once
export async function checkMultiplePermissions(permissions: string[]): Promise<Record<string, boolean>> {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/permissions/check-batch`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ permissions }),
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to check permissions: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error checking permissions:', error);
    // Return false for all permissions on error
    return Object.fromEntries(permissions.map(perm => [perm, false]));
  }
}

// Get permission recommendations
export async function getPermissionRecommendations(): Promise<Array<{ permission: string; description: string; category: string }>> {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/permissions/recommendations`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to get recommendations: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error getting recommendations:', error);
    // Return mock recommendations
    return [
      {
        permission: 'epsx:analytics:advanced',
        description: 'Advanced analytics features and custom reports',
        category: 'Analytics'
      },
      {
        permission: 'epsx:trading:pro',
        description: 'Professional trading tools and real-time data',
        category: 'Trading'
      },
      {
        permission: 'epsx:api:write',
        description: 'API write access for automation and integrations',
        category: 'API'
      },
    ];
  }
}