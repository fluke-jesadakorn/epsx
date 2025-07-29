import { createApiClient, isApiError } from '@epsx/api-client';
import { config } from '@/lib/config';
import type {
  CustomPermission,
  EffectivePermission,
  Permission,
  UserWithPermissions,
  Role,
  Policy,
  Group,
} from '../types/admin/iam';
import { PackageTier } from '../types/admin/iam';

// Get API client - will automatically use backend URL
const getApi = () => {
  return createApiClient(); // Will use backend URL from environment
};

export class IAMService {
  /**
   * Get all users with IAM information
   */
  async getUsers(filters?: {
    packageTier?: PackageTier;
    subscriptionStatus?: string;
    hasCustomPermissions?: boolean;
  }): Promise<UserWithPermissions[]> {
    try {
      const api = getApi();
      const res = await api.getIamUsers(filters);
      
      if (isApiError(res)) {
        throw new Error(`Failed to fetch users: ${res.error}`);
      }
      
      return res.data || [];
    } catch (error) {
      console.error('Error fetching users:', error);
      return [];
    }
  }

  /**
   * Get specific user with all permission details
   */
  async getUserWithPermissions(uid: string): Promise<UserWithPermissions> {
    try {
      const api = getApi();
      const res = await api.getIamUser(uid);
      
      if (isApiError(res)) {
        throw new Error(`Failed to fetch user: ${res.error}`);
      }
      
      return res.data;
    } catch (error) {
      console.error('Error fetching user:', error);
      throw error;
    }
  }

  /**
   * Update user's package tier
   */
  async updateUserPackageTier(
    uid: string,
    tier: PackageTier,
    by: string
  ): Promise<void> {
    try {
      const api = getApi();
      const res = await api.updateUserPackageTier(uid, {
        packageTier: tier,
        updatedBy: by
      });
      
      if (isApiError(res)) {
        throw new Error(`Failed to update package tier: ${res.error}`);
      }
    } catch (error) {
      console.error('Error updating package tier:', error);
      throw error;
    }
  }

  /**
   * Apply package permissions to a user
   */
  async applyPackagePermissions(
    uid: string,
    tier: PackageTier
  ): Promise<void> {
    try {
      const api = getApi();
      const res = await api.applyPackagePermissions({
        userId: uid,
        packageTier: tier
      });
      
      if (isApiError(res)) {
        throw new Error(`Failed to apply package permissions: ${res.error}`);
      }
    } catch (error) {
      console.error('Error applying package permissions:', error);
      throw error;
    }
  }

  /**
   * Grant custom permission to user
   */
  async grantCustomPermission(
    uid: string,
    fid: string,
    perm: Permission,
    by: string,
    opts?: { expiresAt?: Date; reason?: string }
  ): Promise<CustomPermission> {
    try {
      const api = getApi();
      const res = await api.grantCustomPermission({
        userId: uid,
        featureId: fid,
        permission: perm,
        grantedBy: by,
        ...opts
      });
      
      if (isApiError(res)) {
        throw new Error(`Failed to grant permission: ${res.error}`);
      }
      
      return res.data;
    } catch (error) {
      console.error('Error granting permission:', error);
      throw error;
    }
  }

  /**
   * Revoke custom permission
   */
  async revokeCustomPermission(
    pid: string,
    by: string,
    reason?: string
  ): Promise<void> {
    try {
      const api = getApi();
      const res = await api.revokeCustomPermission({
        permissionId: pid,
        revokedBy: by,
        reason
      });
      
      if (isApiError(res)) {
        throw new Error(`Failed to revoke permission: ${res.error}`);
      }
    } catch (error) {
      console.error('Error revoking permission:', error);
      throw error;
    }
  }

  /**
   * Check if user has access to a specific feature
   */
  async hasFeatureAccess(uid: string, fid: string): Promise<boolean> {
    try {
      const api = getApi();
      const res = await api.evaluatePermission({
        userId: uid,
        action: `access:${fid}`,
        resource: `feature:${fid}`
      });
      
      if (isApiError(res)) {
        console.error('Error checking feature access:', res.error);
        return false;
      }
      
      return res.data?.allowed || false;
    } catch (error) {
      console.error('Error checking feature access:', error);
      return false;
    }
  }

  /**
   * Get user's effective permissions
   */
  async getUserEffectivePermissions(
    uid: string
  ): Promise<EffectivePermission[]> {
    try {
      const api = getApi();
      const res = await api.getUserEffectivePermissions(uid);
      
      if (isApiError(res)) {
        throw new Error(`Failed to fetch effective permissions: ${res.error}`);
      }
      
      return res.data || [];
    } catch (error) {
      console.error('Error fetching effective permissions:', error);
      return [];
    }
  }

  /**
   * Bulk apply permission profile to multiple users
   */
  async bulkApplyPermissionProfile(
    uids: string[],
    pid: string,
    by: string
  ): Promise<void> {
    try {
      const api = getApi();
      const res = await api.bulkApplyPermissionProfile({
        userIds: uids,
        profileId: pid,
        appliedBy: by
      });
      
      if (isApiError(res)) {
        throw new Error(`Failed to bulk apply permission profile: ${res.error}`);
      }
    } catch (error) {
      console.error('Error bulk applying permission profile:', error);
      throw error;
    }
  }

  /**
   * Preview what would happen if user upgrades package
   */
  async previewPackageUpgrade(
    uid: string,
    tier: PackageTier
  ): Promise<{
    currentPermissions: EffectivePermission[];
    newPermissions: any[];
    addedPermissions: any[];
    removedPermissions: any[];
  }> {
    try {
      const api = getApi();
      const res = await api.previewPackageUpgrade({
        userId: uid,
        targetTier: tier
      });
      
      if (isApiError(res)) {
        throw new Error(`Failed to preview package upgrade: ${res.error}`);
      }
      
      return res.data || {
        currentPermissions: [],
        newPermissions: [],
        addedPermissions: [],
        removedPermissions: []
      };
    } catch (error) {
      console.error('Error previewing package upgrade:', error);
      throw error;
    }
  }

  /**
   * Get audit logs for a user
   */
  async getUserAuditLogs(uid: string, limit: number = 50): Promise<any[]> {
    try {
      const api = getApi();
      const res = await api.getUserAuditLogs(uid, limit);
      
      if (isApiError(res)) {
        throw new Error(`Failed to fetch audit logs: ${res.error}`);
      }
      
      return res.data || [];
    } catch (error) {
      console.error('Error fetching audit logs:', error);
      return [];
    }
  }

  /**
   * Get all audit logs with optional filters
   */
  async getAllAuditLogs(limit: number = 100): Promise<any[]> {
    try {
      const api = getApi();
      const res = await api.getAllAuditLogs(limit);
      
      if (isApiError(res)) {
        throw new Error(`Failed to fetch audit logs: ${res.error}`);
      }
      
      return res.data || [];
    } catch (error) {
      console.error('Error fetching audit logs:', error);
      return [];
    }
  }

  /**
   * Get available permission profiles
   */
  async getPermissionProfiles() {
    try {
      const api = getApi();
      const res = await api.getPermissionProfiles();
      
      if (isApiError(res)) {
        throw new Error(`Failed to fetch permission profiles: ${res.error}`);
      }
      
      return res.data || {};
    } catch (error) {
      console.error('Error fetching permission profiles:', error);
      return {};
    }
  }

  /**
   * Create IAM role
   */
  async createRole(data: {
    name: string;
    description?: string;
    packageTier: string;
    policies: string[];
    inlinePermissions: any[];
    assignable: boolean;
  }): Promise<Role> {
    try {
      const api = getApi();
      const res = await api.createIamRole(data);
      
      if (isApiError(res)) {
        throw new Error(`Failed to create role: ${res.error}`);
      }
      
      return res.data;
    } catch (error) {
      console.error('Error creating role:', error);
      throw error;
    }
  }

  /**
   * Get all IAM roles
   */
  async getRoles(): Promise<Role[]> {
    try {
      const api = getApi();
      const res = await api.getIamRoles();
      
      if (isApiError(res)) {
        throw new Error(`Failed to fetch roles: ${res.error}`);
      }
      
      return res.data || [];
    } catch (error) {
      console.error('Error fetching roles:', error);
      return [];
    }
  }

  /**
   * Get all IAM policies
   */
  async getPolicies(): Promise<Policy[]> {
    try {
      const api = getApi();
      const res = await api.getIamPolicies();
      
      if (isApiError(res)) {
        throw new Error(`Failed to fetch policies: ${res.error}`);
      }
      
      return res.data || [];
    } catch (error) {
      console.error('Error fetching policies:', error);
      return [];
    }
  }

  /**
   * Clean up expired permissions (maintenance function)
   */
  async cleanupExpiredPermissions(): Promise<void> {
    try {
      const api = getApi();
      const res = await api.cleanupExpiredPermissions();
      
      if (isApiError(res)) {
        throw new Error(`Failed to cleanup permissions: ${res.error}`);
      }
    } catch (error) {
      console.error('Error cleaning up permissions:', error);
      throw error;
    }
  }
}

export const iamService = new IAMService();
