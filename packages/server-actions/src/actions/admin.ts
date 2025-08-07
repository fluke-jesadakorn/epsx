'use server';

import { serverGet, serverPost, serverPut, serverDelete } from '../core/request';

// User Management Actions
export async function getAdminUsers(filters?: {
  role?: string;
  status?: string;
  packageTier?: string;
  limit?: number;
  offset?: number;
}) {
  console.log('🔄 getAdminUsers: Starting request with filters:', filters);
  
  try {
    console.log('🌐 getAdminUsers: Making API request to /api/v1/admin/users');
    const result = await serverGet('/api/v1/admin/users', filters);
    
    console.log('✅ getAdminUsers: Raw API response:', {
      type: typeof result,
      isArray: Array.isArray(result),
      hasUsers: result?.users ? 'yes' : 'no',
      resultKeys: result && typeof result === 'object' ? Object.keys(result) : 'not object'
    });
    
    // Handle different response structures
    let processedResult;
    if (Array.isArray(result)) {
      processedResult = result;
      console.log('📋 getAdminUsers: Using direct array result');
    } else if (result && result.users) {
      processedResult = result.users;
      console.log('📋 getAdminUsers: Using result.users array');
    } else if (result && result.data) {
      processedResult = Array.isArray(result.data) ? result.data : result.data.users || [];
      console.log('📋 getAdminUsers: Using result.data structure');
    } else {
      processedResult = result || [];
      console.log('📋 getAdminUsers: Using fallback empty array');
    }
    
    console.log('✅ getAdminUsers: Final processed result:', {
      type: typeof processedResult,
      isArray: Array.isArray(processedResult),
      length: Array.isArray(processedResult) ? processedResult.length : 'not array',
    });
    
    return processedResult;
  } catch (error) {
    console.error('❌ getAdminUsers: Error fetching admin users:', {
      error: error instanceof Error ? error.message : String(error),
      stack: error instanceof Error ? error.stack : undefined,
      filters: filters
    });
    // Return mock data for development when backend is not available
    console.warn('⚠️ getAdminUsers: Backend not available, returning mock data for development');
    return [
      {
        id: 'mock-user-1',
        uid: 'mock-user-1',
        email: 'user1@example.com',
        display_name: 'Mock User 1',
        email_verified: true,
        disabled: false,
        metadata: {
          creation_time: new Date().toISOString(),
          last_sign_in_time: new Date().toISOString()
        },
        packageTier: 'FREE',
        subscriptionStatus: 'ACTIVE',
        roles: [],
        groups: [],
        perms: []
      },
      {
        id: 'mock-user-2', 
        uid: 'mock-user-2',
        email: 'user2@example.com',
        display_name: 'Mock User 2',
        email_verified: false,
        disabled: false,
        metadata: {
          creation_time: new Date().toISOString(),
          last_sign_in_time: new Date().toISOString()
        },
        packageTier: 'GOLD',
        subscriptionStatus: 'ACTIVE',
        roles: [],
        groups: [],
        perms: []
      }
    ];
  }
}

export async function getUserStats() {
  try {
    return await serverGet('/api/v1/admin/analytics/user-statistics', {
      include_roles: true,
      include_tiers: true
    });
  } catch (error) {
    console.error('Error fetching user stats:', error);
    throw error;
  }
}

export async function updateUserRole(uid: string, role: string) {
  try {
    return await serverPost(`/api/v1/admin/users/${uid}/role`, { role });
  } catch (error) {
    console.error('Error updating user role:', error);
    throw error;
  }
}

export async function updateUserPackageTier(
  uid: string, 
  tier: string, 
  updatedBy: string
) {
  try {
    return await serverPut(`/api/v1/admin/users/${uid}/package-tier`, {
      packageTier: tier,
      updatedBy
    });
  } catch (error) {
    console.error('Error updating package tier:', error);
    throw error;
  }
}

// Permission Management Actions
export async function getPermissionProfiles() {
  try {
    return await serverGet('/api/v1/permission-profiles');
  } catch (error) {
    console.error('Error fetching permission profiles:', error);
    return {};
  }
}

export async function assignPermissionProfile(
  userIds: string[], 
  profileId: string, 
  assignedBy: string
) {
  try {
    return await serverPost('/api/v1/admin/permission-profiles/assign', {
      userIds,
      profileId,
      assignedBy
    });
  } catch (error) {
    console.error('Error assigning permission profile:', error);
    throw error;
  }
}

// Stock Ranking Actions
export async function getStockRankingPackages() {
  try {
    // NOTE: Stock ranking endpoints are temporarily disabled during Casbin migration
    console.warn('Stock ranking packages endpoint not available');
    return [];
  } catch (error) {
    console.error('Error fetching stock ranking packages:', error);
    return [];
  }
}

export async function assignStockRankingPackage(
  userIds: string[], 
  packageId: string, 
  assignedBy: string
) {
  try {
    // NOTE: Stock ranking endpoints are temporarily disabled during Casbin migration
    console.warn('Stock ranking assign endpoint not available');
    return { error: 'Stock ranking functionality temporarily unavailable' };
  } catch (error) {
    console.error('Error assigning stock ranking package:', error);
    throw error;
  }
}

// Analytics Actions
export async function getAnalyticsData(type: string, filters?: any) {
  try {
    return await serverGet(`/api/v1/admin/analytics/${type}`, filters);
  } catch (error) {
    console.error(`Error fetching ${type} analytics:`, error);
    return null;
  }
}

// CRUD User Operations
export async function createUser(userData: {
  email: string;
  password?: string;
  displayName?: string;
  role?: string;
  packageTier?: string;
  emailVerified?: boolean;
}) {
  try {
    return await serverPost('/api/v1/admin/users', userData);
  } catch (error) {
    console.error('Error creating user:', error);
    throw error;
  }
}

export async function updateUser(userId: string, updateData: {
  displayName?: string;
  role?: string;
  packageTier?: string;
  emailVerified?: boolean;
  disabled?: boolean;
}) {
  try {
    return await serverPut(`/api/v1/admin/users/${userId}`, updateData);
  } catch (error) {
    console.error('Error updating user:', error);
    throw error;
  }
}

export async function deleteUser(userId: string) {
  try {
    return await serverDelete(`/api/v1/admin/users/${userId}`);
  } catch (error) {
    console.error('Error deleting user:', error);
    throw error;
  }
}