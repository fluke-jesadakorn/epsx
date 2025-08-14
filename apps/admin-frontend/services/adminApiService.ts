/**
 * Client-side Admin API Service
 * Provides API calls for admin operations without server actions
 */
'use client';

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

/**
 * Client-side admin service using fetch API
 */
export class AdminApiService {
  private static baseUrl = '/api/v1/admin';

  /**
   * Generic API call helper
   */
  private static async apiCall<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        ...options,
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
      });

      if (!response.ok) {
        const error = await response.text();
        return { success: false, error };
      }

      const data = await response.json();
      return { success: true, data };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Get IAM roles
   */
  static async getIAMRoles() {
    return this.apiCall('/iam/roles');
  }

  /**
   * Get user statistics
   */
  static async getUserStats() {
    return this.apiCall('/users/stats');
  }

  /**
   * Get current admin user
   */
  static async getCurrentUser() {
    return this.apiCall('/auth/me');
  }

  /**
   * Get permission profiles
   */
  static async getPermissionProfiles() {
    return this.apiCall('/permission-profiles');
  }

  /**
   * Get analytics data
   */
  static async getAnalyticsData() {
    return this.apiCall('/analytics');
  }

  /**
   * Update user role
   */
  static async updateUserRole(userId: string, role: string) {
    return this.apiCall(`/users/${userId}/role`, {
      method: 'PUT',
      body: JSON.stringify({ role }),
    });
  }

  /**
   * Update user package tier
   */
  static async updateUserPackageTier(userId: string, packageTier: string) {
    return this.apiCall(`/users/${userId}/package-tier`, {
      method: 'PUT',
      body: JSON.stringify({ packageTier }),
    });
  }

  /**
   * Get stock ranking packages
   */
  static async getStockRankingPackages() {
    return this.apiCall('/stock-ranking/packages');
  }

  /**
   * Assign stock ranking package
   */
  static async assignStockRankingPackage(userId: string, packageId: string) {
    return this.apiCall('/stock-ranking/assign', {
      method: 'POST',
      body: JSON.stringify({ userId, packageId }),
    });
  }
}