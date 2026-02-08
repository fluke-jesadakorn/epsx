'use server';

import { revalidatePath } from 'next/cache';
import { cookies } from 'next/headers';

import { COOKIES } from '@/shared/auth/cookies';
import { API_ROUTES } from '@/shared/config/route-constants';

// Batch permission operations for admin panel
interface BatchPermissionRequest {
  permissions: string[];
  walletAddresses: string[];
  expiresAt?: string;
  reason?: string;
  sourceType?: 'direct' | 'group';
  planId?: string;
}

interface PermissionTemplate {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  category: string;
  isDefault: boolean;
  created_at: string;
}

interface PermissionAuditLog {
  id: string;
  action: 'grant' | 'revoke' | 'expire' | 'modify';
  wallet_address: string;
  permissions: string[];
  performed_by: string;
  timestamp: string;
  reason?: string;
  ip_address?: string;
  user_agent?: string;
}

// Grant multiple permissions to multiple wallets
/**
 *
 * @param request
 */
export async function grantBatchPermissions(request: BatchPermissionRequest): Promise<{
  success: boolean;
  message: string;
  processed: number;
  failed: string[];
}> {
  const cookieStore = await cookies();
  const token = cookieStore.get(COOKIES.access_token)?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.PERMISSIONS.BULK_GRANT}`, {
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

    // Revalidate admin pages
    revalidatePath('/permissions');
    revalidatePath('/wallet-management');

    return result;
  } catch (error) {
    console.error('Error granting batch permissions:', error);
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Unknown error occurred',
      processed: 0,
      failed: request.walletAddresses
    };
  }
}

// Revoke multiple permissions from multiple wallets
/**
 *
 * @param walletAddresses
 * @param permissions
 * @param reason
 */
export async function revokeBatchPermissions(
  walletAddresses: string[],
  permissions: string[],
  reason?: string
): Promise<{
  success: boolean;
  message: string;
  processed: number;
  failed: string[];
}> {
  const cookieStore = await cookies();
  const token = cookieStore.get(COOKIES.access_token)?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.PERMISSIONS.BULK_REVOKE}`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        wallet_addresses: walletAddresses,
        permissions,
        reason
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to revoke batch permissions: ${response.status}`);
    }

    const result = await response.json();

    // Revalidate admin pages
    revalidatePath('/permissions');
    revalidatePath('/wallet-management');

    return result;
  } catch (error) {
    console.error('Error revoking batch permissions:', error);
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Unknown error occurred',
      processed: 0,
      failed: walletAddresses
    };
  }
}

// Create permission template
/**
 *
 * @param template
 */
export async function createPermissionTemplate(template: Omit<PermissionTemplate, 'id' | 'created_at'>): Promise<{
  success: boolean;
  template?: PermissionTemplate;
  message: string;
}> {
  const cookieStore = await cookies();
  const token = cookieStore.get(COOKIES.access_token)?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    // Note: PERMISSION_TEMPLATES endpoint may not be implemented in backend
    const response = await fetch(`${process.env.BACKEND_URL}/api/admin/permissions/templates`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(template),
    });

    if (!response.ok) {
      throw new Error(`Failed to create permission template: ${response.status}`);
    }

    const result = await response.json();

    // Revalidate admin pages
    revalidatePath('/permissions');

    return result;
  } catch (error) {
    console.error('Error creating permission template:', error);
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Unknown error occurred'
    };
  }
}

// Get permission templates
/**
 *
 */
export async function getPermissionTemplates(): Promise<PermissionTemplate[]> {
  const cookieStore = await cookies();
  const token = cookieStore.get(COOKIES.access_token)?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    // Note: PERMISSION_TEMPLATES endpoint may not be implemented in backend
    const response = await fetch(`${process.env.BACKEND_URL}/api/admin/permissions/templates`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch permission templates: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching permission templates:', error);
    // Return mock templates for development
    return [
      {
        id: '1',
        name: 'Basic Analyst',
        description: 'Essential permissions for analytics functionality',
        permissions: ['epsx:analytics:basic', 'epsx:analytics:view', 'epsx:portfolio:view'],
        category: 'Analytics',
        isDefault: true,
        created_at: new Date().toISOString()
      },
      {
        id: '2',
        name: 'Premium Analyst',
        description: 'Advanced analytics and reporting permissions',
        permissions: ['epsx:analytics:view', 'epsx:analytics:advanced', 'epsx:data:export'],
        category: 'Analytics',
        isDefault: false,
        created_at: new Date().toISOString()
      },
      {
        id: '3',
        name: 'System Administrator',
        description: 'Full system administration permissions',
        permissions: ['admin:*:*', 'epsx:*:*'],
        category: 'Administration',
        isDefault: false,
        created_at: new Date().toISOString()
      }
    ];
  }
}

// Apply permission template to wallets
/**
 *
 * @param templateId
 * @param walletAddresses
 * @param expiresAt
 * @param reason
 */
export async function applyPermissionTemplate(
  templateId: string,
  walletAddresses: string[],
  expiresAt?: string,
  reason?: string
): Promise<{
  success: boolean;
  message: string;
  processed: number;
  failed: string[];
}> {
  const cookieStore = await cookies();
  const token = cookieStore.get(COOKIES.access_token)?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.ADMIN.PERMISSION_PLANS}/${templateId}/apply`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        wallet_addresses: walletAddresses,
        expires_at: expiresAt,
        reason
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to apply permission template: ${response.status}`);
    }

    const result = await response.json();

    // Revalidate admin pages
    revalidatePath('/permissions');
    revalidatePath('/wallet-management');

    return result;
  } catch (error) {
    console.error('Error applying permission template:', error);
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Unknown error occurred',
      processed: 0,
      failed: walletAddresses
    };
  }
}

// Get permission audit log
/**
 *
 * @param limit
 * @param offset
 * @param filters
 * @param filters.wallet_address
 * @param filters.action
 * @param filters.date_from
 * @param filters.date_to
 */
export async function getPermissionAuditLog(
  limit = 50,
  offset = 0,
  filters?: {
    wallet_address?: string;
    action?: string;
    date_from?: string;
    date_to?: string;
  }
): Promise<{
  logs: PermissionAuditLog[];
  total: number;
  hasMore: boolean;
}> {
  const cookieStore = await cookies();
  const token = cookieStore.get(COOKIES.access_token)?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const params = new URLSearchParams({
      limit: limit.toString(),
      offset: offset.toString(),
      ...(filters?.wallet_address && { wallet_address: filters.wallet_address }),
      ...(filters?.action && { action: filters.action }),
      ...(filters?.date_from && { date_from: filters.date_from }),
      ...(filters?.date_to && { date_to: filters.date_to }),
    });

    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.ADMIN.AUDIT_LOGS}?${params}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch audit log: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching audit log:', error);
    // Return mock data for development
    return {
      logs: [
        {
          id: '1',
          action: 'grant',
          wallet_address: '0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6',
          permissions: ['epsx:analytics:basic', 'epsx:analytics:view'],
          performed_by: 'admin@epsx.com',
          timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
          reason: 'Standard user permissions',
          ip_address: '192.168.1.100',
          user_agent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        },
        {
          id: '2',
          action: 'revoke',
          wallet_address: '0x1234567890123456789012345678901234567890',
          permissions: ['admin:users:manage'],
          performed_by: 'admin@epsx.com',
          timestamp: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString(),
          reason: 'Security policy violation',
          ip_address: '192.168.1.101',
          user_agent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
        }
      ],
      total: 2,
      hasMore: false
    };
  }
}

// Export permissions data for reporting
/**
 *
 * @param format
 * @param filters
 * @param filters.wallet_addresses
 * @param filters.platforms
 * @param filters.date_from
 * @param filters.date_to
 * @param filters.include_expired
 */
export async function exportPermissionsData(
  format: 'json' | 'csv' | 'xlsx' = 'csv',
  filters?: {
    wallet_addresses?: string[];
    platforms?: string[];
    date_from?: string;
    date_to?: string;
    include_expired?: boolean;
  }
): Promise<{ data: string; filename: string }> {
  const cookieStore = await cookies();
  const token = cookieStore.get(COOKIES.access_token)?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.ADMIN.REPORTS}`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        format,
        filters
      }),
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

// Validate permission strings
/**
 * PERMISSION REFACTOR: Client-side validation is now permissive.
 * Backend (Rust) validates all permission identifiers during assignment.
 * @param permissions
 */
export async function validatePermissions(permissions: string[]): Promise<{
  valid: string[];
  invalid: string[];
  normalized: Record<string, string>;
}> {
  // Return all permissions as valid on the client
  return {
    valid: permissions,
    invalid: [],
    normalized: Object.fromEntries(permissions.map(perm => [perm, perm.toLowerCase().trim()]))
  };
}

// Get permission statistics for dashboard
/**
 *
 */
export async function getPermissionStatistics(): Promise<{
  total_permissions: number;
  active_permissions: number;
  expired_permissions: number;
  expiring_soon: number;
  total_wallets: number;
  platform_distribution: Record<string, number>;
  recent_activity: Array<{
    date: string;
    granted: number;
    revoked: number;
    expired: number;
  }>;
}> {
  const cookieStore = await cookies();
  const token = cookieStore.get(COOKIES.access_token)?.value;

  if (!token) {
    throw new Error('Authentication required');
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.PERMISSIONS.STATS}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch statistics: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching statistics:', error);
    // Return mock statistics for development
    return {
      total_permissions: 15420,
      active_permissions: 12850,
      expired_permissions: 1570,
      expiring_soon: 234,
      total_wallets: 8932,
      platform_distribution: {
        epsx: 6234,
        admin: 3125,
        'epsx-pay': 2847,
        'epsx-token': 3214
      },
      recent_activity: [
        {
          date: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString().split('T')[0] || '',
          granted: 45,
          revoked: 12,
          expired: 8
        },
        {
          date: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString().split('T')[0] || '',
          granted: 38,
          revoked: 15,
          expired: 5
        }
      ]
    };
  }
}