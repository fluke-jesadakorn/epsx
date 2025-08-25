import axios from 'axios';
import type { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';

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

// Notification interfaces
export interface NotificationListParams {
  page?: number;
  per_page?: number;
  status?: 'read' | 'unread' | 'all';
  category?: string;
  priority?: string;
  from_date?: string;
  to_date?: string;
  notification_type?: string;
}

export interface NotificationResponse {
  id: string;
  user_id: string;
  title: string;
  message: string;
  notification_type: string;
  category: string;
  priority: string;
  status: string;
  channel: string;
  metadata?: Record<string, any>;
  created_at: string;
  updated_at?: string;
  read_at?: string;
  expires_at?: string;
  scheduled_for?: string;
  sent_at?: string;
  delivery_status?: string;
  error_message?: string;
  retry_count: number;
}

export interface NotificationListResponse {
  notifications: NotificationResponse[];
  pagination: {
    page: number;
    per_page: number;
    total_pages: number;
    has_next: boolean;
    has_prev: boolean;
  };
  unread_count: number;
  total_count: number;
  fetched_at: string;
}

export interface NotificationPreferences {
  email_enabled: boolean;
  push_enabled: boolean;
  in_app_enabled: boolean;
  categories: CategoryPreference[];
  quiet_hours?: QuietHours;
}

export interface CategoryPreference {
  category: string;
  enabled: boolean;
  channels: string[];
  min_priority: string;
}

export interface QuietHours {
  enabled: boolean;
  start_time: string;
  end_time: string;
  timezone: string;
  days: string[];
}

export interface UnreadCountResponse {
  user_id: string;
  unread_count: number;
  by_category: CategoryCount[];
  by_priority: PriorityCount[];
  last_checked: string;
}

export interface CategoryCount {
  category: string;
  count: number;
}

export interface PriorityCount {
  priority: string;
  count: number;
}

export interface DeviceTokenRequest {
  token: string;
  device_type: 'android' | 'ios' | 'web';
  device_id?: string;
  app_version?: string;
}

export interface MarkNotificationsReadRequest {
  notification_ids: string[];
  mark_all?: boolean;
}

export class ApiClient {
  private instance: AxiosInstance;

  constructor(baseURL: string, token?: string) {
    this.instance = axios.create({
      baseURL,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
        ...(token && { Authorization: `Bearer ${token}` }),
      },
    });

    this.instance.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          // Handle unauthorized - redirect to login
          if (typeof window !== 'undefined') {
            window.location.href = '/login';
          }
        }
        return Promise.reject(error);
      }
    );
  }

  async get<T>(url: string, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.instance.get(url, config);
    return { data: response.data, status: response.status, success: true };
  }

  async post<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.instance.post(url, data, config);
    return { data: response.data, status: response.status, success: true };
  }

  async put<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.instance.put(url, data, config);
    return { data: response.data, status: response.status, success: true };
  }

  async delete<T>(url: string, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.instance.delete(url, config);
    return { data: response.data, status: response.status, success: true };
  }

  // Notification API methods
  async getNotifications(params?: NotificationListParams): Promise<ApiResponse<NotificationListResponse>> {
    return this.get('/api/v1/notifications', { params });
  }

  async getNotification(id: string): Promise<ApiResponse<NotificationResponse>> {
    return this.get(`/api/v1/notifications/${id}`);
  }

  async markNotificationRead(id: string): Promise<ApiResponse<void>> {
    return this.post(`/api/v1/notifications/read/${id}`);
  }

  async markAllNotificationsRead(request?: MarkNotificationsReadRequest): Promise<ApiResponse<void>> {
    return this.post('/api/v1/notifications/read-all', request || { notification_ids: [], mark_all: true });
  }

  async deleteNotification(id: string): Promise<ApiResponse<void>> {
    return this.delete(`/api/v1/notifications/${id}`);
  }

  async getUnreadCount(): Promise<ApiResponse<UnreadCountResponse>> {
    return this.get('/api/v1/notifications/unread-count');
  }

  async getNotificationPreferences(): Promise<ApiResponse<NotificationPreferences>> {
    return this.get('/api/v1/notifications/preferences');
  }

  async updateNotificationPreferences(preferences: Partial<NotificationPreferences>): Promise<ApiResponse<NotificationPreferences>> {
    return this.put('/api/v1/notifications/preferences', preferences);
  }

  async registerDeviceToken(request: DeviceTokenRequest): Promise<ApiResponse<void>> {
    return this.post('/api/v1/notifications/device-token', request);
  }

  // Push notification methods (updated to v1 endpoints)
  async subscribeToPushNotifications(subscription: PushSubscriptionRequest): Promise<ApiResponse<void>> {
    return this.post('/api/v1/notifications/subscribe', subscription);
  }

  async unsubscribeFromPushNotifications(): Promise<ApiResponse<void>> {
    return this.post('/api/v1/notifications/unsubscribe');
  }

  setAuthToken(token: string) {
    this.instance.defaults.headers.Authorization = `Bearer ${token}`;
  }

  removeAuthToken() {
    delete this.instance.defaults.headers.Authorization;
  }
}

export function createApiClient(baseURL?: string, token?: string): ApiClient {
  const url = baseURL || process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
  return new ApiClient(url, token);
}

export function isApiError(error: any): error is ApiError {
  return error && typeof error.message === 'string' && typeof error.status === 'number';
}

// Analytics Client for unified analytics endpoints
export class AnalyticsClient {
  private client: AxiosInstance;

  constructor(baseURL?: string) {
    this.client = axios.create({
      baseURL: baseURL || process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080',
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Add request interceptor for authentication
    this.client.interceptors.request.use((config) => {
      if (typeof window !== 'undefined') {
        const token = document.cookie
          .split('; ')
          .find(row => row.startsWith('epsx_frontend_jwt='))
          ?.split('=')[1];
        
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }
      }
      return config;
    });
  }

  async getUnifiedAnalyticsRankings(params: {
    page: number;
    limit: number;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: number;
    min_growth?: number;
  }): Promise<ApiResponse<any>> {
    try {
      const response = await this.client.get('/api/v1/analytics/eps-rankings', { params });
      return {
        data: response.data,
        status: response.status,
        success: true,
      };
    } catch (error: any) {
      throw {
        message: error.response?.data?.message || 'Failed to fetch analytics data',
        status: error.response?.status || 500,
        code: error.response?.data?.code,
      } as ApiError;
    }
  }
}

// Default instance
export const apiClient = createApiClient();