// Client-side admin service - uses server actions for backend communication
import type { UserLevel, UserLevelAssignment } from '@/types/admin/userLevels';
import { adminLogger } from '@/lib/logger';
import { 
  createApiClient, 
  type AdminUser as ApiAdminUser,
  serverListUsers,
  serverGetUser,
  serverSetUserRole,
  serverBulkUpdateUserRoles
} from '@epsx/api-client';

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
  levelUpdateReason?: string;
  maxTokens?: number;
  tokenMultiplier?: number;
  lastUpdated?: string;
  metadata: {
    creationTime?: string;
    lastSignInTime?: string;
    lastRefreshTime?: string | null;
  };
}

export interface UserListOptions {
  maxResults?: number;
  pageToken?: string;
}

export interface UserListResult {
  users: AdminUser[];
  pageToken?: string;
}

export interface UserStats {
  totalUsers: number;
  verifiedUsers: number;
  disabledUsers: number;
  adminUsers: number;
  verificationRate: number;
}

// All client-side calls use API client which handles routing properly

export class AdminService {
  // Get API client instance
  private static getApi() {
    return createApiClient();
  }

  // List users - Updated to use server action
  static async listUsers(opts: UserListOptions = {}): Promise<UserListResult> {
    try {
      const response = await serverListUsers({
        limit: opts.maxResults || 50,
        offset: opts.pageToken ? parseInt(opts.pageToken) : 0,
      });
      
      if (response.error) {
        throw new Error(response.error);
      }
      
      return {
        users: response.data?.users || [],
        pageToken: response.data?.pageToken,
      };
    } catch (error) {
      adminLogger.error('Failed to list users', { error: error instanceof Error ? error.message : error }, 'AdminService.listUsers');
      throw error;
    }
  }

  // Get user by UID - Updated to use server action
  static async getUser(uid: string): Promise<AdminUser> {
    try {
      const response = await serverGetUser(uid);
      
      if (response.error) {
        throw new Error(response.error);
      }
      
      // Map API response to local AdminUser type
      const apiUser = response.data!;
      return {
        ...apiUser,
        metadata: apiUser.metadata || {
          creationTime: undefined,
          lastSignInTime: undefined,
          lastRefreshTime: null,
        },
      };
    } catch (error) {
      adminLogger.error(`Failed to get user ${uid}`, { error: error instanceof Error ? error.message : error, uid }, 'AdminService.getUser');
      throw error;
    }
  }

  // Set user role - Updated to use server action
  static async setUserRole(uid: string, role: string): Promise<void> {
    try {
      const response = await serverSetUserRole(uid, role, 'Role updated via admin panel');

      if (response.error) {
        throw new Error(response.error);
      }
    } catch (error) {
      adminLogger.error(`Failed to set role for user ${uid}`, { error: error instanceof Error ? error.message : error, uid, role }, 'AdminService.setUserRole');
      throw error;
    }
  }

  // Update user status (enable/disable) - Note: Not currently implemented in Rust backend
  static async updateUserStatus(uid: string, disabled: boolean): Promise<void> {
    adminLogger.warn('updateUserStatus: This functionality needs to be implemented in the Rust backend', { uid, disabled }, 'AdminService.updateUserStatus');
    throw new Error('User status updates not yet implemented in Rust backend');
  }

  // Delete user - Note: Not currently implemented in Rust backend
  static async deleteUser(uid: string): Promise<void> {
    adminLogger.warn('deleteUser: This functionality needs to be implemented in the Rust backend', { uid }, 'AdminService.deleteUser');
    throw new Error('User deletion not yet implemented in Rust backend');
  }

  // Send password reset email - Use API client
  static async sendResetEmail(email: string): Promise<string> {
    try {
      const api = this.getApi();
      const response = await api.resetPassword({ email });

      if (response.error) {
        throw new Error(response.error);
      }

      return response.data?.message || 'Password reset email sent';
    } catch (error) {
      adminLogger.error(`Failed to generate password reset for ${email}`, { error: error instanceof Error ? error.message : error, email }, 'AdminService.sendResetEmail');
      throw error;
    }
  }

  // Get user statistics - Uses server action directly
  static async getUserStats(): Promise<UserStats> {
    try {
      const { getUserStats } = await import('@/app/actions/admin-server');
      return await getUserStats();
    } catch (error) {
      adminLogger.error('Failed to get user stats', { error: error instanceof Error ? error.message : error }, 'AdminService.getUserStats');
      throw error;
    }
  }

  // Set user level - Updated to use server action (same as setUserRole)
  static async setLevel(uid: string, level: UserLevel, reason?: string): Promise<void> {
    try {
      const response = await serverSetUserRole(uid, level, reason || 'User level updated via admin panel');

      if (response.error) {
        throw new Error(response.error);
      }
    } catch (error) {
      adminLogger.error(`Failed to set user level for user ${uid}`, { error: error instanceof Error ? error.message : error, uid, level }, 'AdminService.setLevel');
      throw error;
    }
  }

  // Get user level history - No API method available, return empty for now
  static async getLevelHistory(uid: string): Promise<UserLevelAssignment[]> {
    try {
      // TODO: Implement role history tracking in the backend
      adminLogger.warn('getLevelHistory: Role history tracking not yet implemented in backend', { uid }, 'AdminService.getLevelHistory');
      return [];
    } catch (error) {
      adminLogger.error(`Failed to get user level history for ${uid}`, { error: error instanceof Error ? error.message : error, uid }, 'AdminService.getLevelHistory');
      throw error;
    }
  }

  // Bulk update user levels - Updated to use server action
  static async bulkUpdateLevels(updates: Array<{uid: string, level: UserLevel, reason?: string}>): Promise<any> {
    try {
      const response = await serverBulkUpdateUserRoles(
        updates.map(update => ({
          uid: update.uid,
          role: update.level,
          reason: update.reason || 'Bulk update via admin panel'
        }))
      );

      if (response.error) {
        throw new Error(response.error);
      }
      
      return response.data;
    } catch (error) {
      adminLogger.error('Failed to bulk update user levels', { error: error instanceof Error ? error.message : error, updatesCount: updates.length }, 'AdminService.bulkUpdateLevels');
      throw error;
    }
  }

  // Permission checking methods
  static hasPermission(role: string, _resource: string, _action: string): boolean {
    // Simple role-based permission check
    if (role === 'ADMIN') {
      return true; // Admins have all permissions
    }
    
    // For other roles, implement specific permission logic
    return false;
  }

  static canManageUsers(role: string): boolean {
    return this.hasPermission(role, 'users', 'manage');
  }

  static canAssignLevels(role: string): boolean {
    return this.hasPermission(role, 'users', 'assign_levels');
  }

  static canViewAnalytics(role: string): boolean {
    return this.hasPermission(role, 'analytics', 'view');
  }

  static canViewPayments(role: string): boolean {
    return this.hasPermission(role, 'payments', 'view');
  }

  static canManageSystem(role: string): boolean {
    return this.hasPermission(role, 'system', 'manage');
  }

  static getAvailableActions(role: string, resource: string): string[] {
    if (role === 'ADMIN') {
      // Admins can do everything
      switch (resource) {
        case 'users':
          return ['create', 'read', 'update', 'delete', 'assign_levels'];
        case 'payments':
          return ['read', 'refund', 'process'];
        case 'system':
          return ['configure', 'backup', 'restore'];
        case 'analytics':
          return ['view', 'export'];
        default:
          return [];
      }
    }
    
    // For other roles, return empty array or specific permissions
    return [];
  }

  static getRolePriority(role: string): number {
    switch (role) {
      case 'ADMIN':
        return 100;
      case 'USER':
        return 1;
      default:
        return 0;
    }
  }

  // IAM Methods - Updated to use API client
  static async listRoles(): Promise<{ roles: any[] }> {
    try {
      const api = this.getApi();
      const response = await api.getRoles();
      
      if (response.error) {
        throw new Error(response.error);
      }
      
      return { roles: response.data || [] };
    } catch (error) {
      adminLogger.error('Failed to list roles', { error: error instanceof Error ? error.message : error }, 'AdminService.listRoles');
      throw error;
    }
  }

  static async listPolicies(): Promise<{ policies: any[] }> {
    try {
      const api = this.getApi();
      const response = await api.getPermissions();
      
      if (response.error) {
        throw new Error(response.error);
      }
      
      return { policies: response.data || [] };
    } catch (error) {
      adminLogger.error('Failed to list policies', { error: error instanceof Error ? error.message : error }, 'AdminService.listPolicies');
      throw error;
    }
  }

  static async listGroups(): Promise<{ groups: any[] }> {
    try {
      // Note: Using permission profiles instead of groups
      const api = this.getApi();
      const response = await api.listPermissionProfiles();
      
      if (response.error) {
        throw new Error(response.error);
      }
      
      return { groups: response.data?.permission_profiles || [] };
    } catch (error) {
      adminLogger.error('Failed to list groups', { error: error instanceof Error ? error.message : error }, 'AdminService.listGroups');
      throw error;
    }
  }

  // Permission Profile Assignment Methods
  static async listPermProfiles(opts: {
    category?: string;
    package_tier?: string;
    active_only?: boolean;
    limit?: number;
    offset?: number;
  } = {}): Promise<{ permission_profiles: any[]; total: number; limit: number; offset: number }> {
    try {
      const api = this.getApi();
      const response = await api.listPermissionProfiles(opts);
      
      if (response.error) {
        throw new Error(response.error);
      }
      
      return response.data!;
    } catch (error) {
      adminLogger.error('Failed to list permission profiles', { error: error instanceof Error ? error.message : error, opts }, 'AdminService.listPermProfiles');
      throw error;
    }
  }

  static async getPermProfile(profileId: string): Promise<any> {
    try {
      const api = this.getApi();
      const response = await api.getPermissionProfile(profileId);
      
      if (response.error) {
        throw new Error(response.error);
      }
      
      return response.data;
    } catch (error) {
      adminLogger.error(`Failed to get permission profile ${profileId}`, { error: error instanceof Error ? error.message : error, profileId }, 'AdminService.getPermProfile');
      throw error;
    }
  }

  static async assignPermProfile(req: {
    permission_profile_id: string;
    user_ids: string[];
    reason?: string;
    merge_permissions?: boolean;
    expires_at?: string;
    notify_users?: boolean;
  }): Promise<{
    permission_profile_id: string;
    successful_assignments: Array<{
      user_id: string;
      features_unlocked: string[];
      permissions_added: string[];
      assignment_type: string;
    }>;
    failed_assignments: Array<{
      user_id: string;
      error: string;
      error_code: string;
    }>;
    total_assigned: number;
    total_failed: number;
    applied_at: string;
  }> {
    try {
      const api = this.getApi();
      const response = await api.assignPermissionProfile(req);

      if (response.error) {
        throw new Error(response.error);
      }
      
      return response.data!;
    } catch (error) {
      adminLogger.error('Failed to assign permission profile', { error: error instanceof Error ? error.message : error, profileId: req.permission_profile_id, userCount: req.user_ids.length }, 'AdminService.assignPermProfile');
      throw error;
    }
  }

  // Permission Profile permission checking
  static canAssignProfiles(role: string): boolean {
    return this.hasPermission(role, 'permission_profiles', 'assign');
  }

  static canManageProfiles(role: string): boolean {
    return this.hasPermission(role, 'permission_profiles', 'manage');
  }
}
