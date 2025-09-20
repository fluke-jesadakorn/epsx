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
  firebase_uid?: string;
  email?: string; // Optional for wallet authentication
  display_name?: string;
  
  // Status and role fields
  role?: string;
  status?: string;
  is_active: boolean;
  email_verified?: boolean;
  
  // Permission and tier fields
  permissions: string[];
  package_tier?: string;
  subscription_tier?: string; // Alternative field name from API
  
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
 */
export function transformBackendUser(backendUser: BackendUserSummary): User {
  // Derive platforms from permissions
  const platforms = derivePlatforms(backendUser.permissions || []);
  
  return {
    // Identity mapping
    id: backendUser.id,
    wallet_address: backendUser.wallet_address,
    email: backendUser.email || undefined,
    displayName: backendUser.display_name || undefined,
    name: backendUser.display_name || undefined,
    firstName: backendUser.display_name?.split(' ')[0] || undefined,
    lastName: backendUser.display_name?.split(' ').slice(1).join(' ') || undefined,
    
    // Role and status mapping
    role: mapBackendRole(backendUser.role || backendUser.subscription_tier || 'user'),
    status: mapBackendStatus(backendUser.status, backendUser.is_active),
    isActive: backendUser.is_active,
    
    // Timestamps
    createdAt: backendUser.created_at || new Date().toISOString(),
    updatedAt: backendUser.updated_at || new Date().toISOString(),
    lastLoginAt: backendUser.last_login_at,
    
    // Authentication context
    sub: backendUser.id, // Use ID as sub
    firebaseUid: backendUser.firebase_uid,
    
    // Permissions and tier  
    permissions: backendUser.permissions || [],
    packageTier: backendUser.package_tier || backendUser.subscription_tier || 'free',
    
    // Platform context
    platforms,
    primaryPlatform: platforms[0] || 'epsx',
    platformContext: 'epsx',
  };
}

/**
 * Transform backend users response to frontend format
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
 * Map backend role to frontend role enum
 */
function mapBackendRole(backendRole: string): 'admin' | 'user' | 'premium_user' {
  switch (backendRole.toLowerCase()) {
    case 'admin':
      return 'admin';
    case 'premium':
      return 'premium_user';
    case 'user':
    default:
      return 'user';
  }
}

/**
 * Map backend status to frontend status enum
 */
function mapBackendStatus(backendStatus?: string, isActive?: boolean): 'active' | 'inactive' | 'suspended' | 'deleted' {
  // If we have explicit status, use it
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
  
  // Fallback to isActive field
  return isActive ? 'active' : 'inactive';
}

/**
 * Derive platforms from user permissions
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
  
  // If no specific platforms found, default to epsx
  return Array.from(platformSet).length > 0 ? Array.from(platformSet) : ['epsx'];
}

/**
 * Create mock user data for development/testing
 */
export function createMockUser(overrides: Partial<BackendUserSummary> = {}): User {
  const mockBackendUser: BackendUserSummary = {
    id: 'mock-user-id',
    wallet_address: '0x1234567890123456789012345678901234567890',
    firebase_uid: 'mock-firebase-uid',
    email: 'user@example.com',
    display_name: 'Mock User',
    role: 'user',
    status: 'active',
    is_active: true,
    email_verified: true,
    permissions: ['epsx:analytics:view'],
    package_tier: 'free',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    last_login_at: new Date().toISOString(),
    ...overrides,
  };
  
  return transformBackendUser(mockBackendUser);
}

/**
 * Validate backend user data
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
    (data.firebase_uid === undefined || typeof data.firebase_uid === 'string') &&
    (data.display_name === undefined || typeof data.display_name === 'string') &&
    (data.role === undefined || typeof data.role === 'string') &&
    (data.status === undefined || typeof data.status === 'string') &&
    (data.email_verified === undefined || typeof data.email_verified === 'boolean') &&
    (data.package_tier === undefined || typeof data.package_tier === 'string') &&
    (data.subscription_tier === undefined || typeof data.subscription_tier === 'string') &&
    (data.last_login_at === undefined || typeof data.last_login_at === 'string')
  );
  
  if (!optionalFieldsValid) {
    return false;
  }
  
  return true;
}