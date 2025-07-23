// Client-side admin service - makes API calls to server endpoints
import type { UserLevel, UserLevelAssignment } from '@/types/admin/userLevels';

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

const BACKEND_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

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
      console.error('Failed to list users:', error);
      throw error;
    }
  }

  // Get user by UID - Updated to use Rust backend
  static async getUser(uid: string): Promise<AdminUser> {
    try {
      const response = await fetch(`${BACKEND_URL}/admin/users/${uid}`, {
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
      console.error(`Failed to get user ${uid}:`, error);
      throw error;
    }
  }

  // Set user role - Updated to use Rust backend
  static async setUserRole(uid: string, role: string): Promise<void> {
    try {
      const response = await fetch(`${BACKEND_URL}/admin/users/${uid}`, {
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
      console.error(`Failed to set role for user ${uid}:`, error);
      throw error;
    }
  }

  // Update user status (enable/disable) - Note: Not currently implemented in Rust backend
  static async updateUserStatus(uid: string, disabled: boolean): Promise<void> {
    console.warn('updateUserStatus: This functionality needs to be implemented in the Rust backend');
    throw new Error('User status updates not yet implemented in Rust backend');
  }

  // Delete user - Note: Not currently implemented in Rust backend
  static async deleteUser(uid: string): Promise<void> {
    console.warn('deleteUser: This functionality needs to be implemented in the Rust backend');
    throw new Error('User deletion not yet implemented in Rust backend');
  }

  // Send password reset email - Available in Rust backend auth endpoints
  static async sendPasswordResetEmail(email: string): Promise<string> {
    try {
      const response = await fetch(`${BACKEND_URL}/auth/password-reset`, {
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
      console.error(`Failed to generate password reset for ${email}:`, error);
      throw error;
    }
  }

  // Get user statistics - Uses Next.js API proxy to Rust backend
  static async getUserStats(): Promise<UserStats> {
    try {
      const response = await fetch('/api/admin/stats?include_roles=true&include_tiers=true', {
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
      console.error('Failed to get user stats:', error);
      throw error;
    }
  }

  // Set user level - Updated to use Rust backend (same as setUserRole)
  static async setUserLevel(uid: string, userLevel: UserLevel, reason?: string): Promise<void> {
    try {
      const response = await fetch(`${BACKEND_URL}/admin/users/${uid}`, {
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
      console.error(`Failed to set user level for user ${uid}:`, error);
      throw error;
    }
  }

  // Get user level history - Updated to use Rust backend
  static async getUserLevelHistory(uid: string): Promise<UserLevelAssignment[]> {
    try {
      const response = await fetch(`${BACKEND_URL}/admin/users/${uid}/level-history`, {
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
      console.error(`Failed to get user level history for ${uid}:`, error);
      throw error;
    }
  }

  // Bulk update user levels - Updated to use Rust backend
  static async bulkUpdateUserLevels(updates: Array<{uid: string, userLevel: UserLevel, reason?: string}>): Promise<any> {
    try {
      const response = await fetch(`${BACKEND_URL}/admin/users/bulk-update-levels`, {
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
      console.error('Failed to bulk update user levels:', error);
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
      console.error('Failed to list roles:', error);
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
      console.error('Failed to list policies:', error);
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
      console.error('Failed to list groups:', error);
      throw error;
    }
  }
}
