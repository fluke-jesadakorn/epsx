// Using native fetch for lightweight API calls
import { clientConfig } from '@/config/env';
import { 
  ApiResponse, 
  ApiError, 
  JsonRequestBody,
  isApiError
} from '@/types/api';
import { apiLogger, safeError } from '@/lib/logger';

// API types are now imported from @/types/api

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

// Payment types are now centralized in @/types/api

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
  metadata?: Record<string, unknown>;
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
  private baseURL: string;
  private token?: string;

  constructor(baseURL: string, token?: string) {
    this.baseURL = baseURL;
    this.token = token;
  }

  private async makeRequest<T>(url: string, options: RequestInit = {}): Promise<ApiResponse<T>> {
    const fullUrl = `${this.baseURL}${url}`;
    
    const defaultHeaders: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (this.token) {
      defaultHeaders.Authorization = `Bearer ${this.token}`;
    }

    const config: RequestInit = {
      ...options,
      headers: {
        ...defaultHeaders,
        ...options.headers,
      },
    };

    try {
      const response = await fetch(fullUrl, config);
      
      if (response.status === 401) {
        // Handle unauthorized - redirect to login
        if (typeof window !== 'undefined') {
          window.location.href = '/login';
        }
      }

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      return { data, status: response.status, success: true };
    } catch (error) {
      const apiError: ApiError = {
        message: error instanceof Error ? error.message : 'Request failed',
        status: isApiError(error) ? error.status : 500,
        code: isApiError(error) ? error.code : 'UNKNOWN_ERROR',
      };
      apiLogger.error('API request failed', apiError);
      throw apiError;
    }
  }

  async get<T>(url: string, params?: Record<string, unknown>): Promise<ApiResponse<T>> {
    let queryString = '';
    if (params) {
      const searchParams = new URLSearchParams();
      Object.keys(params).forEach(key => {
        if (params[key] !== undefined && params[key] !== null) {
          searchParams.append(key, String(params[key]));
        }
      });
      queryString = searchParams.toString();
    }
    
    const finalUrl = queryString ? `${url}?${queryString}` : url;
    return this.makeRequest<T>(finalUrl, { method: 'GET' });
  }

  async post<T>(url: string, data?: JsonRequestBody): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async put<T>(url: string, data?: JsonRequestBody): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async delete<T>(url: string): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, { method: 'DELETE' });
  }

  // Notification API methods
  async getNotifications(params?: NotificationListParams): Promise<ApiResponse<NotificationListResponse>> {
    return this.get('/api/v1/notifications', params);
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
    this.token = token;
  }

  removeAuthToken() {
    this.token = undefined;
  }
}

function getDefaultApiUrl(): string {
  if (process.env.NODE_ENV === 'production') {
    throw new Error('NEXT_PUBLIC_API_URL or NEXT_PUBLIC_BACKEND_URL is required in production environment');
  }
  return 'http://localhost:8080';
}

export function createApiClient(baseURL?: string, token?: string): ApiClient {
  const url = baseURL || process.env.NEXT_PUBLIC_API_URL || clientConfig.apiUrl || getDefaultApiUrl();
  return new ApiClient(url, token);
}

// Re-export isApiError for compatibility
export { isApiError } from '@/types/api';

// EPS Analytics interfaces matching backend DTOs
export interface EPSPaginationResponse {
  page: number;
  limit: number;
  total: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export interface QuarterlyData {
  quarter: string; // e.g., "Q3 '25"
  date: string;
  price: number;
  eps: number;
  eps_growth: number; // Growth factor percentage
  price_growth: number; // Price growth percentage
  volume?: number;
}

export interface MarketData {
  market_cap?: number;
  volume_24h?: number;
  country: string;
  sector: string;
  exchange: string;
}

export interface AnalyticsMetrics {
  growth_factor: number;
  ranking_score: number;
  trend: string; // bullish, bearish, neutral, etc.
  volatility: number;
}

export interface UnifiedRankingItem {
  symbol: string;
  company_name: string;
  ranking_position: number;
  current_price: number;
  current_price_date: string;
  quarterly_data: QuarterlyData[];
  market_data: MarketData;
  analytics: AnalyticsMetrics;
}

export interface UnifiedFilters {
  country?: string;
  sector?: string;
  sort_by: string;
  min_eps?: number;
  min_growth?: number;
}

export interface UnifiedAnalyticsMetadata {
  available_countries: string[];
  available_sectors: string[];
  current_filters: UnifiedFilters;
  request_timestamp: string;
  data_source: string;
  enhanced_with_websocket: boolean;
}

export interface UnifiedAnalyticsRankingsResponse {
  success: boolean;
  data: UnifiedRankingItem[];
  pagination: EPSPaginationResponse;
  metadata: UnifiedAnalyticsMetadata;
  message?: string;
  processing_time_ms: number;
}

export interface QuarterlyPerformanceData {
  quarter: string; // "Q1", "Q0", etc.
  date: string; // "Aug 8, 2025"
  price: number;
  eps: number;
  eps_growth: number; // Growth factor %
  price_growth: number; // Price % growth
}

export interface SymbolCardData {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number; // Current price
  active_status: string; // Active or Non Active based on surplus
  quarterly_performance: QuarterlyPerformanceData[];
}

export interface CardDashboardMetadata {
  available_countries: string[];
  available_sectors: string[];
  request_timestamp: string;
  data_source: string;
}

export interface CardDashboardResponse {
  success: boolean;
  data: SymbolCardData[];
  pagination: EPSPaginationResponse;
  metadata: CardDashboardMetadata;
  message?: string;
  processing_time_ms: number;
}

export interface EPSQueryParams {
  page: number;
  limit: number;
  country?: string;
  sector?: string;
  sort_by?: string;
  min_eps?: number;
  min_growth?: number;
}

// Analytics Client for unified analytics endpoints
export class AnalyticsClient {
  private baseURL: string;

  constructor(baseURL?: string) {
    this.baseURL = baseURL || process.env.NEXT_PUBLIC_API_URL || clientConfig.apiUrl || getDefaultApiUrl();
  }

  private async makeRequest<T>(url: string, params?: Record<string, unknown>): Promise<ApiResponse<T>> {
    let queryString = '';
    if (params) {
      const searchParams = new URLSearchParams();
      Object.keys(params).forEach(key => {
        if (params[key] !== undefined && params[key] !== null) {
          searchParams.append(key, String(params[key]));
        }
      });
      queryString = searchParams.toString();
    }
    
    const fullUrl = queryString ? `${this.baseURL}${url}?${queryString}` : `${this.baseURL}${url}`;
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    // Add authentication token if available
    if (typeof window !== 'undefined') {
      const token = document.cookie
        .split('; ')
        .find(row => row.startsWith('epsx_frontend_jwt='))
        ?.split('=')[1];
      
      if (token) {
        headers.Authorization = `Bearer ${token}`;
      }
    }

    try {
      const response = await fetch(fullUrl, {
        method: 'GET',
        headers,
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      return {
        data,
        status: response.status,
        success: true,
      };
    } catch (error) {
      const apiError: ApiError = {
        message: error instanceof Error ? error.message : 'Request failed',
        status: isApiError(error) ? error.status : 500,
        code: isApiError(error) ? error.code : 'UNKNOWN_ERROR',
      };
      apiLogger.error('API request failed', apiError);
      throw apiError;
    }
  }

  async getUnifiedAnalyticsRankings(params: EPSQueryParams): Promise<ApiResponse<UnifiedAnalyticsRankingsResponse>> {
    return this.makeRequest<UnifiedAnalyticsRankingsResponse>('/api/v1/analytics/eps-rankings', params);
  }

  async getCardDashboard(params: EPSQueryParams): Promise<ApiResponse<CardDashboardResponse>> {
    return this.makeRequest<CardDashboardResponse>('/api/v1/analytics/card-dashboard', params);
  }

  async getAvailableCountries(): Promise<ApiResponse<{ countries: Array<{ value: string; label: string }>; count: number }>> {
    return this.makeRequest<{ countries: Array<{ value: string; label: string }>; count: number }>('/api/v1/analytics/eps-rankings/countries');
  }

  async getSectorsByCountry(country?: string): Promise<ApiResponse<{ sectors: string[]; count: number; country?: string }>> {
    const params = country ? { country } : undefined;
    return this.makeRequest<{ sectors: string[]; count: number; country?: string }>('/api/v1/analytics/eps-rankings/sectors', params);
  }
}

// Default instance
export const apiClient = createApiClient();