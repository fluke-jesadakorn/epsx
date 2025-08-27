// Using native fetch instead of axios to avoid bundling issues
import { env } from '@/config/env';
import type {
  Notification,
  NotificationListParams,
  NotificationListResponse,
  NotificationCreateRequest,
  NotificationUpdateRequest,
  NotificationPreferences,
  NotificationStats
} from '@/types';

export interface ApiError {
  message: string;
  status: number;
  code?: string;
}

export interface ApiResponse<T = any> {
  data: T;
  status: number;
  success: boolean;
}

export interface WatchlistAddRequest {
  symbol: string;
  name?: string;
}

export interface PriceAlertCreateRequest {
  symbol: string;
  targetPrice: number;
  condition: 'above' | 'below';
}

export interface PushSubscriptionRequest {
  endpoint: string;
  keys: {
    p256dh: string;
    auth: string;
  };
}

export interface PaymentStatus {
  id: string;
  status: 'pending' | 'completed' | 'failed' | 'cancelled';
  amount: number;
  currency: string;
  createdAt: string;
}

export interface PaymentTransaction {
  id: string;
  amount: number;
  currency: string;
  status: string;
  createdAt: string;
  description?: string;
}

// Admin API types
export interface ActionResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface AssignmentResult {
  id: string;
  userId: string;
  assignedAt: string;
  status: 'success' | 'failed';
  error?: string;
}

export interface StockRankingAssignmentUpdateRequest {
  userId: string;
  packageTierId: string;
  stockRankingTypeId: string;
  expiresAt?: string;
  metadata?: Record<string, any>;
}

export class ApiClient {
  private baseURL: string;
  private token?: string;

  constructor(baseURL: string, token?: string) {
    this.baseURL = baseURL;
    this.token = token;
  }

  private async makeRequest<T>(url: string, options: RequestInit = {}): Promise<ApiResponse<T>> {
    const fullUrl = `${this.baseURL}${url}`;
    const config: RequestInit = {
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
        ...(this.token && { Authorization: `Bearer ${this.token}` }),
        ...options.headers,
      },
      ...options,
    };

    try {
      const response = await fetch(fullUrl, config);
      
      if (response.status === 401) {
        // Handle unauthorized - redirect to login
        if (typeof window !== 'undefined') {
          window.location.href = '/login';
        }
        throw new Error('Unauthorized');
      }
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      return { data, status: response.status, success: true };
    } catch (error) {
      console.error('API request failed:', error);
      throw error;
    }
  }

  async get<T>(url: string, params?: any): Promise<ApiResponse<T>> {
    const queryString = params ? '?' + new URLSearchParams(params).toString() : '';
    return this.makeRequest<T>(`${url}${queryString}`, { method: 'GET' });
  }

  async post<T>(url: string, data?: any): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async put<T>(url: string, data?: any): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async delete<T>(url: string): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, { method: 'DELETE' });
  }

  setAuthToken(token: string) {
    this.token = token;
  }

  removeAuthToken() {
    this.token = undefined;
  }

  // Notification API methods
  async getNotifications(params?: NotificationListParams): Promise<ApiResponse<NotificationListResponse>> {
    return this.get('/api/v1/notifications', params);
  }

  async getNotification(id: string): Promise<ApiResponse<Notification>> {
    return this.get(`/api/v1/notifications/${id}`);
  }

  async createNotification(data: NotificationCreateRequest): Promise<ApiResponse<Notification>> {
    return this.post('/api/v1/notifications', data);
  }

  async updateNotification(id: string, data: NotificationUpdateRequest): Promise<ApiResponse<Notification>> {
    return this.put(`/api/v1/notifications/${id}`, data);
  }

  async deleteNotification(id: string): Promise<ApiResponse<void>> {
    return this.delete(`/api/v1/notifications/${id}`);
  }

  async markNotificationRead(id: string): Promise<ApiResponse<Notification>> {
    return this.put(`/api/v1/notifications/${id}/read`);
  }

  async markNotificationUnread(id: string): Promise<ApiResponse<Notification>> {
    return this.put(`/api/v1/notifications/${id}/unread`);
  }

  async markAllNotificationsRead(userId?: string): Promise<ApiResponse<{ updatedCount: number }>> {
    return this.post('/api/v1/notifications/mark-all-read', userId ? { userId } : {});
  }

  async getNotificationStats(userId?: string): Promise<ApiResponse<NotificationStats>> {
    return this.get('/api/v1/notifications/stats', { params: userId ? { userId } : {} });
  }

  async getNotificationPreferences(userId?: string): Promise<ApiResponse<NotificationPreferences>> {
    return this.get('/api/v1/notifications/preferences', { params: userId ? { userId } : {} });
  }

  async updateNotificationPreferences(preferences: Partial<NotificationPreferences>, userId?: string): Promise<ApiResponse<NotificationPreferences>> {
    return this.put('/api/v1/notifications/preferences', { ...preferences, ...(userId ? { userId } : {}) });
  }

  async broadcastNotification(data: Omit<NotificationCreateRequest, 'userId'> & { userIds?: string[]; allUsers?: boolean }): Promise<ApiResponse<{ notificationIds: string[]; userCount: number }>> {
    return this.post('/api/v1/notifications/broadcast', data);
  }

  async deleteNotifications(ids: string[]): Promise<ApiResponse<{ deletedCount: number }>> {
    return this.post('/api/v1/notifications/bulk-delete', { ids });
  }

  async markNotificationsRead(ids: string[]): Promise<ApiResponse<{ updatedCount: number }>> {
    return this.post('/api/v1/notifications/bulk-mark-read', { ids });
  }
}

export function createApiClient(baseURL?: string, token?: string): ApiClient {
  const url = baseURL || env.NEXT_PUBLIC_BACKEND_URL;
  return new ApiClient(url, token);
}

export function isApiError(error: any): error is ApiError {
  return error && typeof error.message === 'string' && typeof error.status === 'number';
}

// Default instance
export const apiClient = createApiClient();