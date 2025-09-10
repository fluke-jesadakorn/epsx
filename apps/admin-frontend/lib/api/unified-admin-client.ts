/**
 * Unified Admin API Client
 * Consolidates all API functionality into a single, consistent interface
 * Replaces: api-client.ts, admin-client.ts, adminApiService.ts, and related files
 */

import { env } from '@/config/env';

// Core API Types
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  status: number;
}

export interface ApiError {
  message: string;
  status: number;
  code?: string;
}

// Request Configuration
export interface RequestConfig extends RequestInit {
  timeout?: number;
  serverSide?: boolean;
}

// Base API Client Class
export class UnifiedAdminClient {
  private baseURL: string;
  private token?: string;
  private isServerSide: boolean;

  constructor(baseURL?: string, token?: string, serverSide = false) {
    this.baseURL = baseURL || env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    this.token = token;
    this.isServerSide = serverSide;
  }

  // Core HTTP Methods
  private async makeRequest<T>(url: string, config: RequestConfig = {}): Promise<ApiResponse<T>> {
    const fullUrl = `${this.baseURL}${url}`;
    const { timeout = 30000, serverSide, ...options } = config;

    // Handle authentication
    let authToken = this.token;
    if (!authToken && (serverSide || this.isServerSide)) {
      try {
        // For server-side requests, try to get token from cookies
        if (typeof window === 'undefined') {
          const { cookies } = await import('next/headers');
          const cookieStore = await cookies();
          authToken = cookieStore.get('access_token')?.value;
        }
      } catch (error) {
        console.warn('Could not access cookies for server-side request');
      }
    }

    const headers = {
      'Content-Type': 'application/json',
      ...(authToken && { Authorization: `Bearer ${authToken}` }),
      ...options.headers,
    };

    const requestConfig: RequestInit = {
      timeout,
      cache: serverSide ? 'no-store' : 'default',
      credentials: serverSide ? undefined : 'include',
      ...options,
      headers,
    };

    try {
      const response = await fetch(fullUrl, requestConfig);
      
      // Handle authentication errors
      if (response.status === 401) {
        if (typeof window !== 'undefined') {
          window.location.href = '/login';
        }
        return {
          success: false,
          error: 'Unauthorized - please log in again',
          status: 401
        };
      }
      
      const data = response.ok ? await response.json() : null;
      
      if (!response.ok) {
        const errorMessage = data?.message || `HTTP error: ${response.status} ${response.statusText}`;
        return {
          success: false,
          error: errorMessage,
          status: response.status
        };
      }

      return {
        success: true,
        data,
        status: response.status
      };
    } catch (error) {
      console.error('API request failed:', { url: fullUrl, error });
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error',
        status: 0
      };
    }
  }

  async get<T>(url: string, params?: Record<string, any>, config?: RequestConfig): Promise<ApiResponse<T>> {
    const queryString = params ? '?' + new URLSearchParams(params).toString() : '';
    return this.makeRequest<T>(`${url}${queryString}`, { method: 'GET', ...config });
  }

  async post<T>(url: string, data?: any, config?: RequestConfig): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
      ...config,
    });
  }

  async put<T>(url: string, data?: any, config?: RequestConfig): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
      ...config,
    });
  }

  async delete<T>(url: string, config?: RequestConfig): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, { method: 'DELETE', ...config });
  }

  // Authentication Management
  setAuthToken(token: string) {
    this.token = token;
  }

  removeAuthToken() {
    this.token = undefined;
  }

  // User Management API
  async getUsers(params: { offset?: number; limit?: number; search?: string } = {}) {
    const { offset = 0, limit = 50, search } = params;
    const queryParams: Record<string, any> = { offset, limit };
    
    // Add search parameter if provided
    if (search && search.trim()) {
      queryParams.search = search;
    }
    
    return this.get('/api/v1/admin/users', queryParams);
  }

  async getUser(userId: string) {
    return this.get(`/api/v1/admin/users/${userId}`);
  }

  async createUser(userData: {
    email: string;
    permissions: string[];
    display_name?: string;
  }) {
    return this.post('/api/v1/admin/users', userData);
  }

  async updateUser(userId: string, userData: any) {
    return this.put(`/api/v1/admin/users/${userId}`, userData);
  }

  async deleteUser(userId: string) {
    return this.delete(`/api/v1/admin/users/${userId}`);
  }

  async searchUsers(query: string) {
    return this.get('/api/v1/admin/users/search', { q: query });
  }

  async getUserStats() {
    return this.get('/api/v1/admin/analytics/user-statistics');
  }

  // Permission Management API
  async getUserPermissions(userId: string) {
    return this.get(`/api/v1/admin/users/${userId}/permissions`);
  }

  async grantPermission(userId: string, permission: string, expiresAt?: string) {
    return this.post('/api/v1/admin/permissions/grant', {
      user_id: userId,
      permission,
      expires_at: expiresAt
    });
  }

  async revokePermission(userId: string, permission: string) {
    return this.post('/api/v1/admin/permissions/revoke', {
      user_id: userId,
      permission
    });
  }

  async bulkGrantPermissions(userIds: string[], permissions: string[], expiresAt?: string) {
    return this.post('/api/v1/admin/permissions/bulk-grant', {
      user_ids: userIds,
      permissions,
      expires_at: expiresAt
    });
  }

  async getPermissionAnalytics() {
    return this.get('/api/v1/admin/analytics/permissions');
  }

  async getPermissionExpiryStatus(userId: string) {
    return this.get(`/api/v1/admin/users/${userId}/permissions/expiry-status`);
  }

  // Notification Management API  
  async getNotifications(params?: {
    page?: number;
    limit?: number;
    type?: string;
    priority?: string;
    read?: boolean;
    userId?: string;
    startDate?: string;
    endDate?: string;
  }) {
    return this.get('/api/v1/notifications', params);
  }

  async getNotification(id: string) {
    return this.get(`/api/v1/notifications/${id}`);
  }

  async createNotification(data: {
    title: string;
    message: string;
    type: string;
    priority: string;
    userId?: string;
    userIds?: string[];
    actionUrl?: string;
    metadata?: Record<string, any>;
  }) {
    return this.post('/api/v1/notifications', data);
  }

  async updateNotification(id: string, data: any) {
    return this.put(`/api/v1/notifications/${id}`, data);
  }

  async deleteNotification(id: string) {
    return this.delete(`/api/v1/notifications/${id}`);
  }

  async markNotificationRead(id: string) {
    return this.put(`/api/v1/notifications/${id}/read`);
  }

  async broadcastNotification(data: {
    title: string;
    message: string;
    type: string;
    priority: string;
    userIds?: string[];
    allUsers?: boolean;
  }) {
    return this.post('/api/v1/notifications/broadcast', data);
  }

  async getNotificationStats(userId?: string) {
    return this.get('/api/v1/notifications/stats', userId ? { userId } : {});
  }

  // Analytics API
  async getEPSRankings() {
    return this.get('/api/v1/analytics/eps-rankings');
  }

  async getEPSHealth() {
    return this.get('/api/v1/analytics/eps-rankings/health');
  }

  async getPerformanceMetrics() {
    return this.get('/api/v1/admin/analytics/performance');
  }

  async getDashboardData() {
    return this.get('/api/v1/admin/analytics/dashboard');
  }

  async getCacheStats() {
    return this.get('/api/v1/admin/cache/stats');
  }

  // System Management API
  async getSystemConfig() {
    return this.get('/api/v1/settings/system');
  }

  async getFeatureFlags() {
    return this.get('/api/v1/settings/feature-flags');
  }

  async updateSystemConfig(config: Record<string, any>) {
    return this.put('/api/v1/settings/system', config);
  }

  // Stock Ranking API
  async getStockRankingPackages() {
    return this.get('/api/v1/admin/stock-ranking/packages');
  }

  async assignStockRankingPackage(userId: string, packageId: string, expiresAt?: string) {
    return this.post('/api/v1/admin/stock-ranking/assign', {
      userId,
      packageId,
      expiresAt
    });
  }

  async getStockRankingAssignments() {
    return this.get('/api/v1/admin/stock-ranking/assignments');
  }

  async extendStockRankingAssignment(assignmentId: string, newExpiresAt: string) {
    return this.put(`/api/v1/admin/stock-ranking/assignments/${assignmentId}/extend`, {
      expires_at: newExpiresAt
    });
  }

  async revokeStockRankingAssignment(assignmentId: string) {
    return this.delete(`/api/v1/admin/stock-ranking/assignments/${assignmentId}`);
  }
}

// Factory Functions
export function createAdminClient(baseURL?: string, token?: string): UnifiedAdminClient {
  return new UnifiedAdminClient(baseURL, token, false);
}

export function createServerAdminClient(baseURL?: string, token?: string): UnifiedAdminClient {
  return new UnifiedAdminClient(baseURL, token, true);
}

// Default Instances
export const adminClient = createAdminClient();
export const serverAdminClient = createServerAdminClient();

// Type Guards
export function isApiError(error: any): error is ApiError {
  return error && typeof error.message === 'string' && typeof error.status === 'number';
}

export function isApiSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { success: true; data: T } {
  return response.success && !!response.data;
}

// Error Handler Utility
export class APIError extends Error {
  constructor(public status: number, message: string, public code?: string) {
    super(message);
    this.name = 'APIError';
  }
}