'use server';

import { serverGet, serverPost } from '../core/request';

// Permission Types
export interface UserPermission {
  id: string;
  name: string;
  description?: string;
  resource: string;
  action: string;
  conditions?: Record<string, any>;
}

export interface PermissionProfile {
  id: string;
  name: string;
  description?: string;
  permissions: string[];
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

// Permission Actions
export async function getUserPermissions(): Promise<UserPermission[]> {
  try {
    const response = await serverGet('/api/v1/user/permissions');
    return response || [];
  } catch (error) {
    console.error('Error fetching user permissions:', error);
    return [];
  }
}

export async function checkPermission(permission: string): Promise<boolean> {
  try {
    const response = await serverGet('/api/v1/user/permissions/check', { permission });
    return response?.allowed || false;
  } catch (error) {
    console.error('Error checking permission:', error);
    return false;
  }
}

export async function checkFeatureAccess(featureId: string): Promise<{
  allowed: boolean;
  reason?: string;
  requiredTier?: string;
}> {
  try {
    const response = await serverGet('/api/v1/user/features/access', { featureId });
    return response || { allowed: false };
  } catch (error) {
    console.error('Error checking feature access:', error);
    return { allowed: false };
  }
}

export async function checkRankingAccess(): Promise<{
  allowed: boolean;
  tier: string;
  expiresAt?: string;
}> {
  try {
    const response = await serverGet('/api/v1/user/ranking-access');
    return response || { allowed: false, tier: 'BRONZE' };
  } catch (error) {
    console.error('Error checking ranking access:', error);
    return { allowed: false, tier: 'BRONZE' };
  }
}

export async function getPermissionProfiles(): Promise<PermissionProfile[]> {
  try {
    const response = await serverGet('/api/v1/permission-profiles');
    return response || [];
  } catch (error) {
    console.error('Error fetching permission profiles:', error);
    return [];
  }
}

export async function assignPermissionProfile(data: {
  profileId: string;
  userId: string;
  expiresAt?: string;
}) {
  try {
    return await serverPost('/api/v1/permission-profiles/assign', data);
  } catch (error) {
    console.error('Error assigning permission profile:', error);
    throw error;
  }
}

export async function revokePermissionProfile(data: {
  profileId: string;
  userId: string;
}) {
  try {
    return await serverPost('/api/v1/permission-profiles/revoke', data);
  } catch (error) {
    console.error('Error revoking permission profile:', error);
    throw error;
  }
}

export async function getPermissionMatrix(userId?: string): Promise<Record<string, string[]>> {
  try {
    const params = userId ? { userId } : undefined;
    const response = await serverGet('/api/v1/permissions/matrix', params);
    return response || {};
  } catch (error) {
    console.error('Error fetching permission matrix:', error);
    return {};
  }
}

// Pagination-aware feature access
export async function getPaginatedFeatureAccess(params: {
  page?: number;
  limit?: number;
  features?: string[];
}): Promise<{
  data: Array<{ featureId: string; allowed: boolean; reason?: string }>;
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
}> {
  try {
    // Convert features array to a comma-separated string for the API and filter undefined values
    const processedParams: Record<string, string | number | boolean> = {};
    if (params.page !== undefined) processedParams.page = params.page;
    if (params.limit !== undefined) processedParams.limit = params.limit;
    if (params.features && params.features.length > 0) {
      processedParams.features = params.features.join(',');
    }
    
    const response = await serverGet('/api/v1/user/features/access/paginated', processedParams);
    return response || { data: [], pagination: { page: 1, limit: 10, total: 0, totalPages: 0 } };
  } catch (error) {
    console.error('Error fetching paginated feature access:', error);
    return { data: [], pagination: { page: 1, limit: 10, total: 0, totalPages: 0 } };
  }
}