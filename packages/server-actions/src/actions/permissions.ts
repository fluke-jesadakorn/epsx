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
    // Use existing auth/profile endpoint which includes permissions
    const response = await serverGet('/api/v1/auth/profile');
    if (response?.permissions) {
      // Convert string permissions to UserPermission objects
      return response.permissions.map((perm: string) => ({
        id: perm,
        name: perm,
        resource: perm.split('.')[0] || perm,
        action: perm.split('.').slice(1).join('.') || perm
      }));
    }
    return [];
  } catch (error) {
    console.error('Error fetching user permissions:', error);
    return [];
  }
}

export async function checkPermission(permission: string): Promise<boolean> {
  try {
    // Use IAM evaluate endpoint for permission checking
    const response = await serverPost('/api/v1/iam/evaluate', {
      action: permission.split('.').pop() || permission,
      resource: permission.split('.').slice(0, -1).join('.') || permission
    });
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
    // Use IAM evaluate endpoint for feature access checking
    const response = await serverPost('/api/v1/iam/evaluate', {
      action: 'access',
      resource: `feature.${featureId}`
    });
    return {
      allowed: response?.allowed || false,
      reason: response?.reason,
      requiredTier: response?.required_tier
    };
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
    // Get profile first to get user_id for IAM evaluation
    const profileResponse = await serverGet('/api/v1/auth/profile');
    
    if (!profileResponse?.user_id) {
      return { allowed: false, tier: 'BRONZE' };
    }
    
    // Use IAM evaluate for ranking access with user_id
    const evaluateResponse = await serverPost('/api/v1/iam/evaluate', {
      user_id: profileResponse.user_id,
      action: 'access',
      resource: 'feature.rankings'
    });
    
    return {
      allowed: evaluateResponse?.allowed || false,
      tier: profileResponse?.package_tier || 'BRONZE',
      expiresAt: profileResponse?.expires_at
    };
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
    if (userId) {
      // For specific user, get their roles and overrides
      const [rolesResponse, overridesResponse] = await Promise.all([
        serverGet(`/api/v1/iam/users/${userId}/roles`),
        serverGet(`/api/v1/iam/users/${userId}/overrides`)
      ]);
      
      const matrix: Record<string, string[]> = {};
      if (rolesResponse?.roles) {
        rolesResponse.roles.forEach((role: any) => {
          if (role.permissions) {
            matrix[role.name] = role.permissions;
          }
        });
      }
      if (overridesResponse?.overrides) {
        matrix['user_overrides'] = overridesResponse.overrides;
      }
      return matrix;
    } else {
      // For current user, use profile endpoint
      const response = await serverGet('/api/v1/auth/profile');
      return response?.permissions ? { current_user: response.permissions } : {};
    }
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
    const features = params.features || ['trading', 'analytics', 'premium', 'rankings'];
    const limit = params.limit || 10;
    const page = params.page || 1;
    
    // Use IAM evaluate for each feature
    const evaluationPromises = features.map(async (featureId) => {
      try {
        const response = await serverPost('/api/v1/iam/evaluate', {
          action: 'access',
          resource: `feature.${featureId}`
        });
        return {
          featureId,
          allowed: response?.allowed || false,
          reason: response?.reason
        };
      } catch (error) {
        return {
          featureId,
          allowed: false,
          reason: 'Evaluation failed'
        };
      }
    });
    
    const allResults = await Promise.all(evaluationPromises);
    
    // Apply pagination
    const startIndex = (page - 1) * limit;
    const endIndex = startIndex + limit;
    const paginatedData = allResults.slice(startIndex, endIndex);
    
    return {
      data: paginatedData,
      pagination: {
        page,
        limit,
        total: allResults.length,
        totalPages: Math.ceil(allResults.length / limit)
      }
    };
  } catch (error) {
    console.error('Error fetching paginated feature access:', error);
    return { data: [], pagination: { page: 1, limit: 10, total: 0, totalPages: 0 } };
  }
}