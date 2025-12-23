/**
 * User Data Transformation Layer
 * Transforms backend UserSummary data to frontend User interface
 */

import type { User } from '@/types/core';

// Backend response interface (matches backend UserSummary)
export interface BackendUserSummary {
  // Identity fields
  id: string;
  wallet_address: string; // Primary identifier for wallet authentication
  email?: string; // Optional for wallet authentication
  display_name?: string;

  // Status and group fields
  group?: string;
  status?: string;
  is_active: boolean;
  email_verified?: boolean;

  // Permission fields (tier derived from permissions)
  permissions: string[];
  subscription_tier?: string; // Legacy field name from API (deprecated)

  // Timestamp fields
  created_at: string;
  updated_at: string;
  last_login_at?: string;
}

// Backend API response interface
export interface BackendUsersResponse {
  users: BackendUserSummary[];
  total_count: number;
}

/**
 * Transform backend user data to frontend User interface
 * Uses backend-provided group directly - no client-side derivation
 * @param backendUser
 */
export function transformBackendUser(backendUser: BackendUserSummary): User {
  // Use backend-provided values directly
  const group = backendUser.group || 'user';
  const permissions = backendUser.permissions || [];
  const platforms = derivePlatforms(permissions);

  return {
    // Identity mapping
    id: backendUser.id,
    wallet_address: backendUser.wallet_address,
    email: backendUser.email || undefined,
    displayName: backendUser.display_name || undefined,
    name: backendUser.display_name || undefined,
    firstName: backendUser.display_name?.split(' ')[0] || undefined,
    lastName: backendUser.display_name?.split(' ').slice(1).join(' ') || undefined,

    // Group from backend (no client-side derivation)
    group: group as 'admin' | 'user' | 'premium_user',
    status: mapBackendStatus(backendUser.status, backendUser.is_active),
    isActive: backendUser.is_active,

    // Timestamps
    createdAt: backendUser.created_at || new Date().toISOString(),
    updatedAt: backendUser.updated_at || new Date().toISOString(),
    lastLoginAt: backendUser.last_login_at,

    // Authentication context
    sub: backendUser.id,

    // Permissions from backend (no derivation)
    permissions,
    // Platform context
    platforms,
    primaryPlatform: platforms[0] || 'epsx',
    platformContext: 'epsx',

    // Permission checking (local only - backend enforces)
    hasAllPermissions: (requiredPermissions: string[]) => requiredPermissions.every(rp => permissions.includes(rp)),
  };
}

/**
 * Transform backend users response to frontend format
 * @param backendResponse
 */
export function transformBackendUsersResponse(
  backendResponse: BackendUsersResponse
): { users: User[]; totalCount: number } {
  return {
    users: backendResponse.users.map(transformBackendUser),
    totalCount: backendResponse.total_count,
  };
}

/**
 * Map backend status to frontend status enum
 * @param backendStatus
 * @param isActive
 */
function mapBackendStatus(backendStatus?: string, isActive?: boolean): 'active' | 'inactive' | 'suspended' | 'deleted' {
  if (backendStatus) {
    switch (backendStatus.toLowerCase()) {
      case 'active':
        return 'active';
      case 'inactive':
        return 'inactive';
      case 'suspended':
        return 'suspended';
      case 'deleted':
        return 'deleted';
      default:
        return isActive ? 'active' : 'inactive';
    }
  }
  return isActive ? 'active' : 'inactive';
}

/**
 * Derive platforms from user permissions
 * @param permissions
 */
function derivePlatforms(permissions: string[]): string[] {
  const platformSet = new Set<string>();

  permissions.forEach(permission => {
    const parts = permission.split(':');
    if (parts.length >= 1) {
      const platform = parts[0];
      if (platform && platform !== 'admin') {
        platformSet.add(platform);
      }
    }
  });

  return Array.from(platformSet).length > 0 ? Array.from(platformSet) : ['epsx'];
}

/**
 * Create mock user data for development/testing
 * @param overrides
 */
export function createMockUser(overrides: Partial<BackendUserSummary> = {}): User {
  const mockBackendUser: BackendUserSummary = {
    id: 'mock-user-id',
    wallet_address: '0x1234567890123456789012345678901234567890',
    email: 'user@example.com',
    display_name: 'Mock User',
    group: 'user',
    status: 'active',
    is_active: true,
    email_verified: true,
    permissions: ['epsx:analytics:view'],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    last_login_at: new Date().toISOString(),
    ...overrides,
  };

  return transformBackendUser(mockBackendUser);
}

/**
 * Validate backend user data
 * @param data
 */
export function validateBackendUser(data: any): data is BackendUserSummary {
  // Basic required fields
  const hasRequiredFields = (
    typeof data === 'object' &&
    data !== null &&
    typeof data.id === 'string' &&
    typeof data.wallet_address === 'string' &&
    typeof data.is_active === 'boolean' &&
    Array.isArray(data.permissions) &&
    typeof data.created_at === 'string' &&
    typeof data.updated_at === 'string'
  );

  if (!hasRequiredFields) {
    return false;
  }

  // Allow optional fields to be missing or have correct types
  const optionalFieldsValid = (
    (data.email === undefined || typeof data.email === 'string') &&
    (data.display_name === undefined || typeof data.display_name === 'string') &&
    (data.role === undefined || typeof data.role === 'string') &&
    (data.status === undefined || typeof data.status === 'string') &&
    (data.email_verified === undefined || typeof data.email_verified === 'boolean') &&
    (data.last_login_at === undefined || typeof data.last_login_at === 'string')
  );

  if (!optionalFieldsValid) {
    return false;
  }

  return true;
}