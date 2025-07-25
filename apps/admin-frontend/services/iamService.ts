// import { buildPackagePermissions } from '../config/packagePermissions'; // Config removed
import type {
  CustomPermission,
  EffectivePermission,
  Permission,
  UserWithPermissions,
} from '../types/admin/iam';
import { PackageTier } from '../types/admin/iam';
// import { firebaseIAMService } from './firebaseIAMService'; // Service removed

// Placeholder for removed dependencies
const buildPackagePermissions = () => ({});

// Mock admin service
const adminService = {
  createAdminUser: async (...args: any[]) => {},
};
const firebaseIAMService = {
  getUsers: async (...args: any[]) => [],
  getUser: async (...args: any[]) => null,
  getUserWithPermissions: async (
    ...args: any[]
  ): Promise<UserWithPermissions> => ({
    id: '',
    email: '',
    displayName: '',
    name: '',
    emailVerified: false,
    disabled: false,
    roles: [],
    groups: [],
    attachedPolicies: [],
    status: 'active',
    packageTier: PackageTier.FREE,
    subscriptionStatus: 'inactive' as any,
    effectivePermissions: [],
    customPermissions: [],
    packagePermissions: [],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    lastActivity: new Date().toISOString(),
  }),
  updateUserPackageTier: async (...args: any[]) => {},
  setUserCustomPermissions: async (...args: any[]) => {},
  getUserCustomPermissions: async (...args: any[]) => [],
  getActivityLogs: async (...args: any[]) => [],
  bulkApplyProfile: async (...args: any[]) => {},
  applyPackagePermissions: async (...args: any[]) => {},
  grantCustomPermission: async (...args: any[]): Promise<CustomPermission> => ({
    id: '',
    userId: '',
    featureId: '',
    permission: {
      id: '',
      action: '',
      resource: '',
      service: '',
      effect: 'Allow',
    },
    grantedBy: '',
    grantedAt: new Date(),
    isActive: true,
  }),
  revokeCustomPermission: async (...args: any[]) => {},
  hasFeatureAccess: async (...args: any[]) => false,
  getUserEffectivePermissions: async (...args: any[]) => [],
  previewPackageUpgrade: async (...args: any[]) => ({
    currentPermissions: [],
    newPermissions: [],
    addedPermissions: [],
    removedPermissions: [],
  }),
  getUserAuditLogs: async (...args: any[]) => [],
  getAllAuditLogs: async (...args: any[]) => [],
  cleanupExpiredPermissions: async (...args: any[]) => {},
  createAuditLog: async (...args: any[]) => {},
  createUser: async (...args: any[]) => {},
  createAdminUser: async (...args: any[]) => {},
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
    return firebaseIAMService.getUsers(filters);
  }

  /**
   * Get specific user with all permission details
   */
  async getUserWithPermissions(userId: string): Promise<UserWithPermissions> {
    return firebaseIAMService.getUserWithPermissions(userId);
  }

  /**
   * Update user's package tier
   */
  async updateUserPackageTier(
    userId: string,
    newTier: PackageTier,
    updatedBy: string
  ): Promise<void> {
    return firebaseIAMService.updateUserPackageTier(userId, newTier, updatedBy);
  }

  /**
   * Apply package permissions to a user
   */
  async applyPackagePermissions(
    userId: string,
    packageTier: PackageTier
  ): Promise<void> {
    return firebaseIAMService.applyPackagePermissions(userId, packageTier);
  }

  /**
   * Grant custom permission to user
   */
  async grantCustomPermission(
    userId: string,
    featureId: string,
    permission: Permission,
    grantedBy: string,
    options?: { expiresAt?: Date; reason?: string }
  ): Promise<CustomPermission> {
    return firebaseIAMService.grantCustomPermission(
      userId,
      featureId,
      permission,
      grantedBy,
      options
    );
  }

  /**
   * Revoke custom permission
   */
  async revokeCustomPermission(
    permissionId: string,
    revokedBy: string,
    reason?: string
  ): Promise<void> {
    return firebaseIAMService.revokeCustomPermission(
      permissionId,
      revokedBy,
      reason
    );
  }

  /**
   * Check if user has access to a specific feature
   */
  async hasFeatureAccess(userId: string, featureId: string): Promise<boolean> {
    return firebaseIAMService.hasFeatureAccess(userId, featureId);
  }

  /**
   * Get user's effective permissions
   */
  async getUserEffectivePermissions(
    userId: string
  ): Promise<EffectivePermission[]> {
    return firebaseIAMService.getUserEffectivePermissions(userId);
  }

  /**
   * Bulk apply permission profile to multiple users
   */
  async bulkApplyPermissionProfile(
    userIds: string[],
    permissionProfileId: string,
    appliedBy: string
  ): Promise<void> {
    return firebaseIAMService.bulkApplyProfile(
      userIds,
      permissionProfileId,
      appliedBy
    );
  }

  /**
   * Preview what would happen if user upgrades package
   */
  async previewPackageUpgrade(
    userId: string,
    newTier: PackageTier
  ): Promise<{
    currentPermissions: EffectivePermission[];
    newPermissions: any[];
    addedPermissions: any[];
    removedPermissions: any[];
  }> {
    return firebaseIAMService.previewPackageUpgrade(userId, newTier);
  }

  /**
   * Get audit logs for a user
   */
  async getUserAuditLogs(userId: string, limit: number = 50): Promise<any[]> {
    return firebaseIAMService.getUserAuditLogs(userId, limit);
  }

  /**
   * Get all audit logs with optional filters
   */
  async getAllAuditLogs(limit: number = 100): Promise<any[]> {
    return firebaseIAMService.getAllAuditLogs(limit);
  }

  /**
   * Get available permission profiles
   */
  getPermissionProfiles() {
    // Return the permission profiles from config
    return buildPackagePermissions();
  }

  /**
   * Clean up expired permissions (maintenance function)
   */
  async cleanupExpiredPermissions(): Promise<void> {
    return firebaseIAMService.cleanupExpiredPermissions();
  }
}

export const iamService = new IAMService();
