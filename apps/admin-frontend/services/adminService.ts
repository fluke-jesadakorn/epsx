// Client-side admin service - makes API calls to server endpoints
import type { UserLevel, UserLevelAssignment } from '@/types/admin/userLevels';
import { adminLogger } from '@/lib/logger';

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

// All client-side calls should use Next.js API routes for security

export class AdminService {
  // List users - Updated to use Rust backend
  static async listUsers(options: UserListOptions = {}): Promise<UserListResult> {
    try {
      const params = new URLSearchParams();
      if (options.maxResults) params.set('limit', options.maxResults.toString());
      if (options.pageToken) params.set('offset', options.pageToken);

      const response = await fetch(`/api/admin/users?${params.toString()}`, {
        method: 'GET',
        credentials: 'include', // Include cookies for session auth
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to fetch users');
      }
      
      return await response.json();
    } catch (error) {
      adminLogger.error('Failed to list users', { error: error instanceof Error ? error.message : error }, 'AdminService.listUsers');
      throw error;
    }
  }

  // Get user by UID - Updated to use Next.js API route
  static async getUser(uid: string): Promise<AdminUser> {
    try {
      const response = await fetch(`/api/admin/users/${uid}`, {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to fetch user');
      }
      
      return await response.json();
    } catch (error) {
      adminLogger.error(`Failed to get user ${uid}`, { error: error instanceof Error ? error.message : error, uid }, 'AdminService.getUser');
      throw error;
    }
  }

  // Set user role - Updated to use Next.js API route
  static async setUserRole(uid: string, role: string): Promise<void> {
    try {
      const response = await fetch(`/api/admin/users/${uid}`, {
        method: 'PUT',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          role,
          reason: 'Role updated via admin panel'
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to update user role');
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

  // Send password reset email - Use Next.js API route
  static async sendPasswordResetEmail(email: string): Promise<string> {
    try {
      const response = await fetch('/api/auth/password-reset', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to send password reset');
      }

      const result = await response.json();
      return result.link || result.message || 'Password reset email sent';
    } catch (error) {
      adminLogger.error(`Failed to generate password reset for ${email}`, { error: error instanceof Error ? error.message : error, email }, 'AdminService.sendPasswordResetEmail');
      throw error;
    }
  }

  // Get user statistics - Uses Next.js API proxy to Rust backend
  static async getUserStats(): Promise<UserStats> {
    try {
      const response = await fetch('/api/admin/analytics/user-statistics?include_roles=true&include_tiers=true', {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to fetch user statistics');
      }
      
      return await response.json();
    } catch (error) {
      adminLogger.error('Failed to get user stats', { error: error instanceof Error ? error.message : error }, 'AdminService.getUserStats');
      throw error;
    }
  }

  // Set user level - Updated to use Next.js API route (same as setUserRole)
  static async setUserLevel(uid: string, userLevel: UserLevel, reason?: string): Promise<void> {
    try {
      const response = await fetch(`/api/admin/users/${uid}`, {
        method: 'PUT',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          role: userLevel,
          reason: reason || 'User level updated via admin panel'
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to update user level');
      }
    } catch (error) {
      adminLogger.error(`Failed to set user level for user ${uid}`, { error: error instanceof Error ? error.message : error, uid, userLevel }, 'AdminService.setUserLevel');
      throw error;
    }
  }

  // Get user level history - Updated to use Next.js API route
  static async getUserLevelHistory(uid: string): Promise<UserLevelAssignment[]> {
    try {
      const response = await fetch(`/api/admin/users/${uid}/role-history`, {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to fetch user level history');
      }
      
      return await response.json();
    } catch (error) {
      adminLogger.error(`Failed to get user level history for ${uid}`, { error: error instanceof Error ? error.message : error, uid }, 'AdminService.getUserLevelHistory');
      throw error;
    }
  }

  // Bulk update user levels - Updated to use Next.js API route
  static async bulkUpdateUserLevels(updates: Array<{uid: string, userLevel: UserLevel, reason?: string}>): Promise<any> {
    try {
      const response = await fetch('/api/admin/users/batch-update-roles', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          updates: updates.map(update => ({
            user_id: update.uid,
            role: update.userLevel,
            reason: update.reason || 'Bulk update via admin panel'
          }))
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to bulk update user levels');
      }
      
      const result = await response.json();
      
      if (!result || typeof result !== 'object') {
        throw new Error('Invalid response from server');
      }
      
      return result;
    } catch (error) {
      adminLogger.error('Failed to bulk update user levels', { error: error instanceof Error ? error.message : error, updatesCount: updates.length }, 'AdminService.bulkUpdateUserLevels');
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

  static canAssignUserLevels(role: string): boolean {
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

  // IAM Methods
  static async listRoles(): Promise<{ roles: any[] }> {
    try {
      const response = await fetch('/api/admin/iam/roles', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        throw new Error('Failed to fetch roles');
      }
      
      return await response.json();
    } catch (error) {
      adminLogger.error('Failed to list roles', { error: error instanceof Error ? error.message : error }, 'AdminService.listRoles');
      throw error;
    }
  }

  static async listPolicies(): Promise<{ policies: any[] }> {
    try {
      const response = await fetch('/api/admin/iam/policies', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        throw new Error('Failed to fetch policies');
      }
      
      return await response.json();
    } catch (error) {
      adminLogger.error('Failed to list policies', { error: error instanceof Error ? error.message : error }, 'AdminService.listPolicies');
      throw error;
    }
  }

  static async listGroups(): Promise<{ groups: any[] }> {
    try {
      const response = await fetch('/api/admin/iam/groups', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        throw new Error('Failed to fetch groups');
      }
      
      return await response.json();
    } catch (error) {
      adminLogger.error('Failed to list groups', { error: error instanceof Error ? error.message : error }, 'AdminService.listGroups');
      throw error;
    }
  }

  // Permission Profile Assignment Methods
  static async listPermissionProfiles(options: {
    category?: string;
    package_tier?: string;
    active_only?: boolean;
    limit?: number;
    offset?: number;
  } = {}): Promise<{ permission_profiles: any[]; total: number; limit: number; offset: number }> {
    try {
      const params = new URLSearchParams();
      if (options.category) params.set('category', options.category);
      if (options.package_tier) params.set('package_tier', options.package_tier);
      if (options.active_only !== undefined) params.set('active_only', options.active_only.toString());
      if (options.limit) params.set('limit', options.limit.toString());
      if (options.offset) params.set('offset', options.offset.toString());

      const response = await fetch(`/api/admin/permission-profiles?${params.toString()}`, {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to fetch permission profiles');
      }
      
      return await response.json();
    } catch (error) {
      adminLogger.error('Failed to list permission profiles', { error: error instanceof Error ? error.message : error, options }, 'AdminService.listPermissionProfiles');
      throw error;
    }
  }

  static async getPermissionProfile(permissionProfileId: string): Promise<any> {
    try {
      const response = await fetch(`/api/admin/permission-profiles/${permissionProfileId}`, {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to fetch permission profile');
      }
      
      return await response.json();
    } catch (error) {
      adminLogger.error(`Failed to get permission profile ${permissionProfileId}`, { error: error instanceof Error ? error.message : error, permissionProfileId }, 'AdminService.getPermissionProfile');
      throw error;
    }
  }

  static async assignPermissionProfileDirectly(request: {
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
      const response = await fetch('/api/admin/permission-profiles/assign', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to assign permission profile');
      }
      
      return await response.json();
    } catch (error) {
      adminLogger.error('Failed to assign permission profile', { error: error instanceof Error ? error.message : error, profileId: request.permission_profile_id, userCount: request.user_ids.length }, 'AdminService.assignPermissionProfileDirectly');
      throw error;
    }
  }

  // Permission Profile permission checking
  static canAssignPermissionProfiles(role: string): boolean {
    return this.hasPermission(role, 'permission_profiles', 'assign');
  }

  static canManagePermissionProfiles(role: string): boolean {
    return this.hasPermission(role, 'permission_profiles', 'manage');
  }
}
