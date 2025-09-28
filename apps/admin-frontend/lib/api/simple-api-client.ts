/**
 * Simplified Admin API Client - Backend-Only Permission Architecture
 * No client-side permission checking, only API calls and error handling
 */

import { env } from '@/config/env';
import { 
  ApiResponse, 
  ApiError,
  handleApiResponse
} from '@/lib/api/response-handler';

// ============================================================================
// Core Types and Interfaces  
// ============================================================================

export interface AdminUserData {
  id: string;
  email: string;
  wallet_address?: string;
  role: string;
  created_at: string;
  updated_at: string;
  last_login?: string;
}

export interface AdminUserListResponse {
  users: AdminUserData[];
  pagination: {
    page: number;
    per_page: number;
    total_pages: number;
    total_items: number;
  };
}

export interface CreateUserRequest {
  email: string;
  wallet_address?: string;
  role?: string;
}

export interface UpdateUserRequest {
  email?: string;
  role?: string;
}

export interface GrantPermissionRequest {
  user_id: string;
  permission: string;
  expires_at?: string;
  reason?: string;
}

export interface RevokePermissionRequest {
  user_id: string;
  permission: string;
  reason?: string;
}

export interface SystemHealthResponse {
  status: 'healthy' | 'degraded' | 'unhealthy';
  timestamp: string;
  services: Record<string, { status: string; response_time?: number }>;
  metrics: {
    total_users: number;
    active_sessions: number;
    system_load: number;
  };
}

// ============================================================================
// Simplified Admin API Client Class
// ============================================================================

class SimpleAdminApiClient {
  private baseUrl: string;

  constructor() {
    this.baseUrl = env.BACKEND_URL || 'http://localhost:8080';
  }

  private async getAuthHeaders(): Promise<HeadersInit> {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };

    // Server-side: get token from cookies
    if (typeof window === 'undefined') {
      try {
        const { cookies } = await import('next/headers');
        const cookieStore = await cookies();
        const accessToken = cookieStore.get('access_token')?.value;
        
        if (accessToken) {
          headers['Authorization'] = `Bearer ${accessToken}`;
        }
      } catch (error) {
        console.warn('Failed to get server-side auth token:', error);
      }
    } else {
      // Client-side: check Web3 auth store first for Bearer token
      try {
        const { usePureWeb3AuthStore } = await import('@/lib/auth/pure-web3-service');
        const authStore = usePureWeb3AuthStore.getState();
        
        // Use Bearer token from Web3 auth if available and not expired
        if (authStore.isConnected && authStore.walletAddress && authStore.bearerToken) {
          if (authStore.tokenExpiresAt && new Date(authStore.tokenExpiresAt) > new Date()) {
            headers['Authorization'] = `Bearer ${authStore.bearerToken}`;
            return headers;
          }
        }
      } catch (error) {
        console.warn('Failed to get Web3 Bearer token:', error);
      }
      
      // Fallback to cookie-based token
      try {
        const cookies = document.cookie.split(';');
        const accessTokenCookie = cookies.find(cookie => 
          cookie.trim().startsWith('access_token=')
        );
        
        if (accessTokenCookie) {
          const token = accessTokenCookie.split('=')[1];
          headers['Authorization'] = `Bearer ${token}`;
        }
      } catch (error) {
        console.warn('Failed to get client-side auth token:', error);
      }
    }

    return headers;
  }

  private async request<T>(
    endpoint: string, 
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    try {
      const url = `${this.baseUrl}${endpoint}`;
      const headers = await this.getAuthHeaders();

      const config: RequestInit = {
        ...options,
        headers: {
          ...headers,
          ...options.headers,
        },
        credentials: 'include',
      };

      const response = await fetch(url, config);
      
      // Use our response handler to process the response
      return await handleApiResponse<T>(response, {
        operation: `${options.method || 'GET'} ${endpoint}`,
        component: 'SimpleAdminApiClient'
      });

    } catch (error) {
      console.error('API Request failed:', error);
      
      // Return structured error
      const apiError: ApiError = {
        success: false,
        error: {
          type: 'NETWORK_ERROR',
          code: 'NETWORK_ERROR',
          message: error instanceof Error ? error.message : 'Network request failed',
          user_message: 'Unable to connect to the server. Please check your connection.',
          suggested_actions: ['Check your internet connection', 'Try again in a moment']
        }
      };
      
      return apiError;
    }
  }

  // ============================================================================
  // User Management API Methods
  // ============================================================================

  async getUsers(params: {
    page?: number;
    per_page?: number;
    search?: string;
    role?: string;
  } = {}): Promise<ApiResponse<AdminUserListResponse>> {
    const searchParams = new URLSearchParams();
    
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        searchParams.append(key, String(value));
      }
    });

    const queryString = searchParams.toString();
    const endpoint = queryString ? `/api/v1/admin/users?${queryString}` : '/api/v1/admin/users';

    return this.request<AdminUserListResponse>(endpoint);
  }

  async getUser(userId: string): Promise<ApiResponse<AdminUserData>> {
    return this.request<AdminUserData>(`/api/v1/admin/users/${userId}`);
  }

  async createUser(userData: CreateUserRequest): Promise<ApiResponse<AdminUserData>> {
    return this.request<AdminUserData>('/api/v1/admin/users', {
      method: 'POST',
      body: JSON.stringify(userData)
    });
  }

  async updateUser(userId: string, userData: UpdateUserRequest): Promise<ApiResponse<AdminUserData>> {
    return this.request<AdminUserData>(`/api/v1/admin/users/${userId}`, {
      method: 'PUT',
      body: JSON.stringify(userData)
    });
  }

  async deleteUser(userId: string): Promise<ApiResponse<void>> {
    return this.request<void>(`/api/v1/admin/users/${userId}`, {
      method: 'DELETE'
    });
  }

  // ============================================================================
  // Permission Management API Methods (Backend-Only Validation)
  // ============================================================================

  async grantPermission(request: GrantPermissionRequest): Promise<ApiResponse<void>> {
    return this.request<void>('/api/v1/admin/permissions/grant', {
      method: 'POST',
      body: JSON.stringify(request)
    });
  }

  async revokePermission(request: RevokePermissionRequest): Promise<ApiResponse<void>> {
    return this.request<void>('/api/v1/admin/permissions/revoke', {
      method: 'POST',
      body: JSON.stringify(request)
    });
  }

  async getUserPermissions(userId: string): Promise<ApiResponse<{ permissions: string[] }>> {
    return this.request<{ permissions: string[] }>(`/api/v1/admin/users/${userId}/permissions`);
  }

  // ============================================================================
  // System Management API Methods
  // ============================================================================

  async getSystemHealth(): Promise<ApiResponse<SystemHealthResponse>> {
    return this.request<SystemHealthResponse>('/api/v1/admin/system/health');
  }

  async getSystemMetrics(): Promise<ApiResponse<any>> {
    return this.request<any>('/api/v1/admin/system/metrics');
  }

  // ============================================================================
  // Notification API Methods
  // ============================================================================

  async sendNotification(data: {
    user_ids?: string[];
    title: string;
    message: string;
    type?: string;
    priority?: string;
  }): Promise<ApiResponse<void>> {
    return this.request<void>('/api/v1/admin/notifications/send', {
      method: 'POST',
      body: JSON.stringify(data)
    });
  }

  async getNotificationStats(): Promise<ApiResponse<{
    total_sent: number;
    delivery_rate: number;
    error_rate: number;
  }>> {
    return this.request<{
      total_sent: number;
      delivery_rate: number;
      error_rate: number;
    }>('/api/v1/admin/notifications/stats');
  }

  // ============================================================================
  // Analytics API Methods
  // ============================================================================

  async getAnalytics(params: {
    start_date?: string;
    end_date?: string;
    metric?: string;
  } = {}): Promise<ApiResponse<any>> {
    const searchParams = new URLSearchParams();
    
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        searchParams.append(key, String(value));
      }
    });

    const queryString = searchParams.toString();
    const endpoint = queryString ? `/api/v1/admin/analytics?${queryString}` : '/api/v1/admin/analytics';

    return this.request<any>(endpoint);
  }

  // ============================================================================
  // Generic HTTP Methods
  // ============================================================================

  async get<T = any>(endpoint: string, options?: RequestInit): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { ...options, method: 'GET' });
  }

  async post<T = any>(endpoint: string, data?: any, options?: RequestInit): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      ...options,
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined
    });
  }

  async put<T = any>(endpoint: string, data?: any, options?: RequestInit): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      ...options,
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined
    });
  }

  async delete<T = any>(endpoint: string, options?: RequestInit): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { ...options, method: 'DELETE' });
  }
}

// ============================================================================
// Exports
// ============================================================================

// Export singleton instance
export const adminApiClient = new SimpleAdminApiClient();

// Export class for custom instances
export { SimpleAdminApiClient };

// ============================================================================
// SIMPLE ADMIN API CLIENT COMPLETE
// ============================================================================
//
// 🎉 SIMPLE ADMIN API CLIENT COMPLETE!
//
// Created backend-only API client for admin frontend:
// - No client-side permission checking or validation
// - All permission validation handled by backend
// - Proper error handling with structured responses
// - Clean separation of concerns
// - Type-safe API methods for admin operations
//
// Key Features:
// ✅ User management (CRUD operations)
// ✅ Permission management (grant/revoke)
// ✅ System health and metrics
// ✅ Notification management
// ✅ Analytics data access
// ✅ Generic HTTP methods
// ✅ Automatic authentication header handling
// ✅ Comprehensive error handling
//
// Backend handles ALL permission validation! 🎯
// ============================================================================