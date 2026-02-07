/**
 * User Data Transformation Layer
 * Transforms backend UserSummary data to frontend User interface
 */

import type { User } from '@/shared/types/domain/User';

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
    walletAddress: backendUser.wallet_address,
    email: backendUser.email || '',
    displayName: backendUser.display_name,
    name: backendUser.display_name || 'User',
    firstName: backendUser.display_name?.split(' ')[0],
    lastName: backendUser.display_name?.split(' ').slice(1).join(' '),
    role: (backendUser.group as unknown as 'user' | 'admin') || 'user',
    phoneNumber: undefined,
    timezone: undefined,
    language: undefined,
    twoFactorEnabled: false,
    avatar: undefined,
    lastActivityAt: undefined,

    // Group from backend (no client-side derivation)
    group: group as unknown as 'Basic Access Group',
    permissionGroup: group as unknown as 'Basic Access Group',
    status: mapBackendStatus(backendUser.status, backendUser.is_active),
    emailVerified: backendUser.email_verified || false,

    // Timestamps
    createdAt: new Date(backendUser.created_at),
    updatedAt: new Date(backendUser.updated_at),
    lastLogin: backendUser.last_login_at ? new Date(backendUser.last_login_at) : undefined,

    // Platform context
    permissions,
    platforms,
    primaryPlatform: platforms[0] || 'epsx',
    platformContext: 'epsx',
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
function mapBackendStatus(backendStatus?: string, isActive?: boolean): 'active' | 'disabled' | 'pending' | 'suspended' | 'trial' {
  if (backendStatus) {
    const status = backendStatus.toLowerCase();
    switch (status) {
      case 'active':
        return 'active';
      case 'inactive':
        return 'disabled';
      case 'suspended':
        return 'suspended';
      case 'deleted':
        return 'disabled';
      default:
        return isActive ? 'active' : 'disabled';
    }
  }
  return isActive ? 'active' : 'disabled';
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
 * Internal helper for optional field validation
 * @param d 
 */
function validateOptionalFields(d: Record<string, unknown>): boolean {
  return (
    (d.email === undefined || typeof d.email === 'string') &&
    (d.display_name === undefined || typeof d.display_name === 'string') &&
    (d.role === undefined || typeof d.role === 'string') &&
    (d.status === undefined || typeof d.status === 'string') &&
    (d.email_verified === undefined || typeof d.email_verified === 'boolean') &&
    (d.last_login_at === undefined || typeof d.last_login_at === 'string')
  );
}

/**
 * Validate backend user data
 * @param data
 */
export function validateBackendUser(data: unknown): data is BackendUserSummary {
  if (typeof data !== 'object' || data === null) {
    return false;
  }

  const d = data as Record<string, unknown>;

  // Basic required fields
  const hasRequiredFields = (
    typeof d.id === 'string' &&
    typeof d.wallet_address === 'string' &&
    typeof d.is_active === 'boolean' &&
    Array.isArray(d.permissions) &&
    typeof d.created_at === 'string' &&
    typeof d.updated_at === 'string'
  );

  if (!hasRequiredFields) {
    return false;
  }

  return validateOptionalFields(d);
}