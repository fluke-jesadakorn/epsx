// New admin service using only server actions
import type { UserLevel, UserLevelAssignment } from '@/types/admin/userLevels';
import { adminLogger } from '@/lib/logger';
import { 
  getAdminUsers, 
  getUserStats,
  updateUserRole,
  updateUserPackageTier,
  getPermissionProfiles,
  assignPermissionProfile,
  getStockRankingPackages,
  assignStockRankingPackage,
  getAnalyticsData,
  getCustomPermissions,
  getIAMRoles,
  evaluatePermission,
  getCurrentUser,
  updateSettings
} from '@epsx/server-actions';

export interface AdminUser {
  uid: string;
  email: string;
  emailVerified: boolean;
  displayName?: string;
  disabled: boolean;
  customClaims?: {
    role?: string;
    tokenBalance?: number;
    emailVerified?: boolean;
    permissions?: string[];
    createdAt?: number;
    lastUpdated?: number;
  };
  // User level data from Firestore
  userLevel?: UserLevel;
  numericLevel?: number;
  levelAssignedBy?: string;
  levelAssignedAt?: string;
  levelHistory?: UserLevelAssignment[];
}

export interface UserStats {
  totalUsers: number;
  activeUsers: number;
  newUsersThisMonth: number;
  usersByRole: Record<string, number>;
  usersByTier: Record<string, number>;
}

export class AdminService {
  /**
   * Get all admin users with optional filtering
   */
  static async getUsers(filters?: {
    role?: string;
    status?: string;
    packageTier?: string;
    limit?: number;
    offset?: number;
  }): Promise<AdminUser[]> {
    try {
      const response = await getAdminUsers(filters);
      return response.users || [];
    } catch (error) {
      adminLogger.error('Failed to fetch users', error);
      throw error;
    }
  }

  /**
   * Get user statistics
   */
  static async getUserStats(): Promise<UserStats> {
    try {
      return await getUserStats();
    } catch (error) {
      adminLogger.error('Failed to fetch user stats', error);
      throw error;
    }
  }

  /**
   * Update user role
   */
  static async updateUserRole(uid: string, role: string): Promise<void> {
    try {
      await updateUserRole(uid, role);
    } catch (error) {
      adminLogger.error('Failed to update user role', { uid, role, error });
      throw error;
    }
  }

  /**
   * Update user package tier
   */
  static async updateUserPackageTier(
    uid: string, 
    tier: string, 
    updatedBy: string
  ): Promise<void> {
    try {
      await updateUserPackageTier(uid, tier, updatedBy);
    } catch (error) {
      adminLogger.error('Failed to update package tier', { uid, tier, error });
      throw error;
    }
  }

  /**
   * Get permission profiles
   */
  static async getPermissionProfiles(): Promise<any> {
    try {
      return await getPermissionProfiles();
    } catch (error) {
      adminLogger.error('Failed to fetch permission profiles', error);
      throw error;
    }
  }

  /**
   * Assign permission profile to users
   */
  static async assignPermissionProfile(
    userIds: string[], 
    profileId: string, 
    assignedBy: string
  ): Promise<void> {
    try {
      await assignPermissionProfile(userIds, profileId, assignedBy);
    } catch (error) {
      adminLogger.error('Failed to assign permission profile', { 
        userIds, 
        profileId, 
        error 
      });
      throw error;
    }
  }

  /**
   * Get stock ranking packages
   */
  static async getStockRankingPackages(): Promise<any[]> {
    try {
      return await getStockRankingPackages();
    } catch (error) {
      adminLogger.error('Failed to fetch stock ranking packages', error);
      return [];
    }
  }

  /**
   * Assign stock ranking package to users
   */
  static async assignStockRankingPackage(
    userIds: string[], 
    packageId: string, 
    assignedBy: string
  ): Promise<void> {
    try {
      await assignStockRankingPackage(userIds, packageId, assignedBy);
    } catch (error) {
      adminLogger.error('Failed to assign stock ranking package', { 
        userIds, 
        packageId, 
        error 
      });
      throw error;
    }
  }

  /**
   * Get analytics data
   */
  static async getAnalyticsData(type: string, filters?: any): Promise<any> {
    try {
      return await getAnalyticsData(type, filters);
    } catch (error) {
      adminLogger.error(`Failed to fetch ${type} analytics`, error);
      return null;
    }
  }

  /**
   * Search users by email or name
   */
  static async searchUsers(query: string): Promise<AdminUser[]> {
    try {
      const allUsers = await this.getUsers();
      return allUsers.filter(user => 
        user.email.toLowerCase().includes(query.toLowerCase()) ||
        user.displayName?.toLowerCase().includes(query.toLowerCase())
      );
    } catch (error) {
      adminLogger.error('Failed to search users', { query, error });
      return [];
    }
  }

  /**
   * Get custom permissions
   */
  static async getCustomPermissions(): Promise<any[]> {
    try {
      return await getCustomPermissions();
    } catch (error) {
      adminLogger.error('Failed to fetch custom permissions', error);
      throw error;
    }
  }

  /**
   * Get IAM roles
   */
  static async getIAMRoles(): Promise<any[]> {
    try {
      return await getIAMRoles();
    } catch (error) {
      adminLogger.error('Failed to fetch IAM roles', error);
      throw error;
    }
  }

  /**
   * Evaluate user permission
   */
  static async evaluatePermission(params: {
    userId: string;
    action: string;
    resource: string;
  }): Promise<{ allowed: boolean }> {
    try {
      return await evaluatePermission(params);
    } catch (error) {
      adminLogger.error('Failed to evaluate permission', { params, error });
      return { allowed: false };
    }
  }

  /**
   * Get current user
   */
  static async getCurrentUser(): Promise<any> {
    try {
      return await getCurrentUser();
    } catch (error) {
      adminLogger.error('Failed to get current user', error);
      return null;
    }
  }

  /**
   * Update settings
   */
  static async updateSettings(settings: any): Promise<void> {
    try {
      await updateSettings(settings);
    } catch (error) {
      adminLogger.error('Failed to update settings', { settings, error });
      throw error;
    }
  }
}