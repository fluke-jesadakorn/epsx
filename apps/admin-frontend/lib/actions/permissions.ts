/* eslint-disable @typescript-eslint/no-unsafe-return */

'use server';

import { revalidatePath } from 'next/cache';
import { cookies } from 'next/headers';

import { logger } from '@/lib/logger';
import { COOKIES } from '@/shared/auth/cookies';
import { API_ROUTES } from '@/shared/config/route-constants';

const UNKNOWN_ERROR = 'Unknown error occurred';
const AUTH_REQUIRED_MSG = 'Authentication required';
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

const PATH_PERMISSIONS = '/policies';
const PATH_WALLET_MGMT = '/wallet-management';

const getAuthHeaders = (token: string) => ({
  'Authorization': `Bearer ${token}`,
  'Content-Type': 'application/json',
});

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

  if (token === undefined || token === '') {
    throw new Error(AUTH_REQUIRED_MSG);
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.PERMISSIONS.BULK_GRANT}`, {
      method: 'POST',
      headers: getAuthHeaders(token),
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`Failed to grant batch permissions: ${response.status}`);
    }

    const result = (await response.json()) as {
      success: boolean;
      message: string;
      processed: number;
      failed: string[];
    };

    // Revalidate admin pages
    revalidatePath(PATH_PERMISSIONS);
    revalidatePath(PATH_WALLET_MGMT);

    return result;
  } catch (error) {
    logger.error('Error granting batch permissions:', { error });
    return {
      success: false,
      message: error instanceof Error ? error.message : UNKNOWN_ERROR,
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

  if (token === undefined || token === '') {
    throw new Error(AUTH_REQUIRED_MSG);
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.PERMISSIONS.BULK_REVOKE}`, {
      method: 'POST',
      headers: getAuthHeaders(token),
      body: JSON.stringify({
        wallet_addresses: walletAddresses,
        permissions,
        reason
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to revoke batch permissions: ${response.status}`);
    }

    const result = (await response.json()) as {
      success: boolean;
      message: string;
      processed: number;
      failed: string[];
    };

    // Revalidate admin pages
    revalidatePath(PATH_PERMISSIONS);
    revalidatePath(PATH_WALLET_MGMT);

    return result;
  } catch (error) {
    logger.error('Error revoking batch permissions:', { error });
    return {
      success: false,
      message: error instanceof Error ? error.message : UNKNOWN_ERROR,
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

  if (token === undefined || token === '') {
    throw new Error(AUTH_REQUIRED_MSG);
  }

  try {
    // Note: PERMISSION_TEMPLATES endpoint may not be implemented in backend
    const response = await fetch(`${process.env.BACKEND_URL}/api/admin/permissions/templates`, {
      method: 'POST',
      headers: getAuthHeaders(token),
      body: JSON.stringify(template),
    });

    if (!response.ok) {
      throw new Error(`Failed to create permission template: ${response.status}`);
    }

    const result = (await response.json()) as {
      success: boolean;
      template?: PermissionTemplate;
      message: string;
    };

    // Revalidate admin pages
    revalidatePath(PATH_PERMISSIONS);

    return result;
  } catch (error) {
    logger.error('Error creating permission template:', { error });
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

  if (token === undefined || token === '') {
    throw new Error(AUTH_REQUIRED_MSG);
  }

  try {
    // Note: PERMISSION_TEMPLATES endpoint may not be implemented in backend
    const response = await fetch(`${process.env.BACKEND_URL}/api/admin/permissions/templates`, {
      method: 'GET',
      headers: getAuthHeaders(token),
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch permission templates: ${response.status}`);
    }

    return (await response.json()) as PermissionTemplate[];
  } catch (error) {
    logger.error('Error fetching permission templates:', { error });
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
 * @param params
 * @param params.templateId
 * @param params.walletAddresses
 * @param params.expiresAt
 * @param params.reason
 */
export async function applyPermissionTemplate(params: {
  templateId: string;
  walletAddresses: string[];
  expiresAt?: string;
  reason?: string;
}): Promise<{
  success: boolean;
  message: string;
  processed: number;
  failed: string[];
}> {
  const cookieStore = await cookies();
  const token = cookieStore.get(COOKIES.access_token)?.value;

  if (token === undefined || token === '') {
    throw new Error(AUTH_REQUIRED_MSG);
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.ADMIN.PERMISSION_PLANS}/${params.templateId}/apply`, {
      method: 'POST',
      headers: getAuthHeaders(token),
      body: JSON.stringify({
        wallet_addresses: params.walletAddresses,
        expires_at: params.expiresAt,
        reason: params.reason
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to apply permission template: ${response.status}`);
    }

    const result = (await response.json()) as {
      success: boolean;
      message: string;
      processed: number;
      failed: string[];
    };

    // Revalidate admin pages
    revalidatePath(PATH_PERMISSIONS);
    revalidatePath(PATH_WALLET_MGMT);

    return result;
  } catch (error) {
    logger.error('Error applying permission template:', { error });
    return {
      success: false,
      message: error instanceof Error ? error.message : UNKNOWN_ERROR,
      processed: 0,
      failed: params.walletAddresses
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

  if (token === undefined || token === '') {
    throw new Error(AUTH_REQUIRED_MSG);
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.ADMIN.REPORTS}`, {
      method: 'POST',
      headers: getAuthHeaders(token),
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
    logger.error('Error exporting permissions:', { error });
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
  await Promise.resolve();
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

  if (token === undefined || token === '') {
    throw new Error(AUTH_REQUIRED_MSG);
  }

  try {
    const response = await fetch(`${process.env.BACKEND_URL}${API_ROUTES.PERMISSIONS.STATS}`, {
      method: 'GET',
      headers: getAuthHeaders(token),
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch statistics: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    logger.error('Error fetching statistics:', { error });
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
          date: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString().split('T')[0] ?? '',
          granted: 45,
          revoked: 12,
          expired: 8
        },
        {
          date: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString().split('T')[0] ?? '',
          granted: 38,
          revoked: 15,
          expired: 5
        }
      ]
    };
  }
}