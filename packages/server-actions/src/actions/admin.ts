'use server';

import { serverGet, serverPost, serverPut } from '../core/request';

// User Management Actions
export async function getAdminUsers(filters?: {
  role?: string;
  status?: string;
  packageTier?: string;
  limit?: number;
  offset?: number;
}) {
  try {
    return await serverGet('/api/admin/users', filters);
  } catch (error) {
    console.error('Error fetching admin users:', error);
    return { users: [], total: 0 };
  }
}

export async function getUserStats() {
  try {
    return await serverGet('/api/admin/analytics/user-statistics', {
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
    return await serverPost(`/api/admin/users/${uid}/role`, { role });
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
    return await serverPut(`/api/admin/users/${uid}/package-tier`, {
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
    return await serverGet('/api/admin/permission-profiles');
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
    return await serverPost('/api/admin/permission-profiles/assign', {
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
    return await serverGet('/api/admin/stock-rankings/packages');
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
    return await serverPost('/api/admin/stock-rankings/assign', {
      userIds,
      packageId,
      assignedBy
    });
  } catch (error) {
    console.error('Error assigning stock ranking package:', error);
    throw error;
  }
}

// Analytics Actions
export async function getAnalyticsData(type: string, filters?: any) {
  try {
    return await serverGet(`/api/admin/analytics/${type}`, filters);
  } catch (error) {
    console.error(`Error fetching ${type} analytics:`, error);
    return null;
  }
}