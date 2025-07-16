// Client-side admin service - makes API calls to server endpoints
export interface AdminUser {
  uid: string;
  email: string;
  emailVerified: boolean;
  displayName?: string;
  disabled: boolean;
  customClaims?: {
    role?: string;
    emailVerified?: boolean;
    permissions?: string[];
    createdAt?: number;
    lastUpdated?: number;
  };
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

export class AdminService {
  // List users
  static async listUsers(options: UserListOptions = {}): Promise<UserListResult> {
    try {
      const params = new URLSearchParams();
      if (options.maxResults) params.set('maxResults', options.maxResults.toString());
      if (options.pageToken) params.set('pageToken', options.pageToken);

      const response = await fetch(`/api/admin/users?${params.toString()}`);
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

  // Get user by UID
  static async getUser(uid: string): Promise<AdminUser> {
    try {
      const response = await fetch(`/api/admin/users/${uid}`);
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

  // Set user role
  static async setUserRole(uid: string, role: string): Promise<void> {
    try {
      const response = await fetch('/api/admin/users', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          action: 'updateRole',
          uid,
          data: { role }
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

  // Update user status (enable/disable)
  static async updateUserStatus(uid: string, disabled: boolean): Promise<void> {
    try {
      const response = await fetch('/api/admin/users', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          action: 'updateStatus',
          uid,
          data: { disabled }
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to update user status');
      }
    } catch (error) {
      console.error(`Failed to update status for user ${uid}:`, error);
      throw error;
    }
  }

  // Delete user
  static async deleteUser(uid: string): Promise<void> {
    try {
      const response = await fetch('/api/admin/users', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          action: 'deleteUser',
          uid
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to delete user');
      }
    } catch (error) {
      console.error(`Failed to delete user ${uid}:`, error);
      throw error;
    }
  }

  // Send password reset email
  static async sendPasswordResetEmail(email: string): Promise<string> {
    try {
      const response = await fetch('/api/admin/users', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          action: 'sendPasswordReset',
          data: { email }
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to send password reset');
      }

      const result = await response.json();
      return result.link;
    } catch (error) {
      console.error(`Failed to generate password reset for ${email}:`, error);
      throw error;
    }
  }

  // Get user statistics
  static async getUserStats(): Promise<UserStats> {
    try {
      const response = await fetch('/api/admin/stats');
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
}
