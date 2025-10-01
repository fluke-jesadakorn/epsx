/**
 * Unified API Client
 * Consolidates analytics, notifications, payments, and core API functionality
 */

import { env } from '../../../../shared/env/schema';
import { 
  ApiResponse, 
  ApiError, 
  PaginatedResponse,
  JsonRequestBody,
  isApiError
} from '../../../../shared/types/api';
import { apiLogger, safeError } from '@/lib/utils/logging';
import { getBackendUrl } from '../../../../shared/utils/url-resolver';
import { usePureWeb3AuthStore } from '@/lib/auth/pure-web3-service';

// ============================================================================
// Core Types and Interfaces
// ============================================================================

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
  type: 'system' | 'admin' | 'data' | 'feature' | 'security';
  priority: 'low' | 'normal' | 'high' | 'urgent';
  sender: 'system' | 'admin' | 'automated';
  imageUrl?: string;
  actionUrl?: string;
  customData?: Record<string, any>;
  createdAt: string;
  readAt?: string;
  clickedAt?: string;
  deliveredAt?: string;
  expiresAt?: string;
}

export interface Notification {
  id: string;
  title: string;
  body: string;
  type: 'system' | 'admin' | 'data' | 'feature' | 'security';
  priority: 'low' | 'normal' | 'high' | 'urgent';
  sender: 'system' | 'admin' | 'automated';
  imageUrl?: string;
  actionUrl?: string;
  customData?: Record<string, any>;
  createdAt: string;
  readAt?: string;
  clickedAt?: string;
  deliveredAt?: string;
  expiresAt?: string;
}

export interface NotificationStats {
  totalSent: number;
  delivered: number;
  failed: number;
  pending: number;
  successRate: number;
  unreadCount: number;
}

// Analytics interfaces
export interface UnifiedRankingItem {
  symbol: string;
  companyName: string;
  eps: number;
  epsGrowth: number;
  revenueGrowth: number;
  score: number;
  rank: number;
  sector?: string;
  country?: string;
  marketCap?: number;
  pe?: number;
  pb?: number;
  roe?: number;
  debt_equity?: number;
  current_ratio?: number;
  quick_ratio?: number;
  gross_margin?: number;
  operating_margin?: number;
  net_margin?: number;
  asset_turnover?: number;
  inventory_turnover?: number;
  receivables_turnover?: number;
  cash_cycle?: number;
  fcf_yield?: number;
  dividend_yield?: number;
  payout_ratio?: number;
  beta?: number;
  volatility?: number;
  momentum_1m?: number;
  momentum_3m?: number;
  momentum_6m?: number;
  momentum_1y?: number;
  rsi?: number;
  technical_score?: number;
}

export interface UnifiedAnalyticsRankingsResponse {
  rankings: UnifiedRankingItem[];
  pagination: {
    page: number;
    per_page: number;
    total_pages: number;
    total_items: number;
  };
  metadata: {
    query_time: number;
    cached: boolean;
    last_updated: string;
  };
}

// PaginatedResponse is imported from shared types

export interface CountResponse {
  count: number;
}

export interface StockFinancialData {
  symbol: string;
  company_name: string;
  country: string;
  sector: string;
  market_cap: number;
  quarterly_data: Array<{
    quarter: string;
    revenue: number;
    earnings: number;
    eps: number;
  }>;
}

// ============================================================================
// Core API Client Class
// ============================================================================

class ApiClient {
  private baseUrl: string;

  constructor() {
    this.baseUrl = env.BACKEND_URL || getBackendUrl('client');
  }

  private async getAuthHeaders(endpoint: string, method: string = 'GET', body?: any): Promise<HeadersInit> {
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
        // Cookie access might fail in some contexts
        const safeErr = safeError(error);
        const errorDetails = {
          message: safeErr.message,
          stack: safeErr.stack,
          code: safeErr.code,
          status: safeErr.status,
          type: error instanceof Error ? error.constructor.name : typeof error,
          context: 'server-side-auth-token-retrieval',
          timestamp: new Date().toISOString()
        };
        
        apiLogger.warn('Failed to get server-side auth token', errorDetails);
      }
    } else {
      // Client-side: check Web3 auth store first for Bearer token
      try {
        const authStore = usePureWeb3AuthStore.getState();
        
        // Use Bearer token from Web3 auth if available and not expired
        if (authStore.isConnected && authStore.walletAddress && authStore.bearerToken) {
          if (authStore.tokenExpiresAt && new Date(authStore.tokenExpiresAt) > new Date()) {
            headers['Authorization'] = `Bearer ${authStore.bearerToken}`;
            return headers;
          }
        }
        
        // Fallback to Web3 signature-based authentication
        if (authStore.isConnected && authStore.walletAddress) {
          const signedHeaders = await authStore.signRequest(endpoint, method, body);
          // Merge signed headers with base headers
          Object.assign(headers, signedHeaders);
        }
      } catch (error) {
        // Log Web3 auth errors but don't fail the request
        apiLogger.warn('Failed to get Web3 authentication headers', error);
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
      const method = options.method || 'GET';
      const body = options.body;
      const headers = await this.getAuthHeaders(endpoint, method, body);

      const config: RequestInit = {
        ...options,
        headers: {
          ...headers,
          ...options.headers,
        },
        credentials: 'include', // Always include cookies
      };

      apiLogger.debug('API Request', { url, method: config.method || 'GET' });

      const response = await fetch(url, config);
      
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        const apiError: ApiError = {
          status: response.status,
          message: errorData.message || `Request failed with status ${response.status}`,
          code: errorData.code || 'UNKNOWN_ERROR',
          details: errorData.details
        };
        throw apiError;
      }

      const data = await response.json();
      
      return {
        data,
        success: true,
        status: response.status,
        timestamp: new Date().toISOString()
      };

    } catch (error) {
      const safeErr = safeError(error);
      const errorDetails = {
        endpoint,
        url: `${this.baseUrl}${endpoint}`,
        method: options.method || 'GET',
        message: safeErr.message,
        stack: safeErr.stack,
        code: safeErr.code,
        status: safeErr.status,
        type: error instanceof Error ? error.constructor.name : typeof error,
        isApiError: isApiError(error),
        headers: options.headers,
        timestamp: new Date().toISOString()
      };

      apiLogger.error('API Request failed', errorDetails);

      if (isApiError(error)) {
        throw error;
      }

      const apiError: ApiError = {
        status: safeErr.status || 500,
        message: safeErr.message,
        code: safeErr.code || 'NETWORK_ERROR'
      };
      
      throw apiError;
    }
  }

  // ============================================================================
  // Analytics API Methods
  // ============================================================================

  async getUnifiedAnalyticsRankings(params: {
    page?: number;
    per_page?: number;
    country?: string;
    sector?: string;
    min_market_cap?: number;
    sort_by?: string;
    sort_order?: 'asc' | 'desc';
  } = {}): Promise<UnifiedAnalyticsRankingsResponse | undefined> {
    const searchParams = new URLSearchParams();
    
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        searchParams.append(key, String(value));
      }
    });

    const queryString = searchParams.toString();
    const endpoint = queryString ? `/api/v1/analytics/rankings?${queryString}` : '/api/v1/analytics/rankings';

    const response = await this.request<UnifiedAnalyticsRankingsResponse>(endpoint);
    return response?.data;
  }

  async getAnalyticsHealth(): Promise<{ status: string; timestamp: string } | undefined> {
    const response = await this.request<{ status: string; timestamp: string }>('/api/v1/analytics/health');
    return response?.data;
  }

  // ============================================================================
  // Notification API Methods
  // ============================================================================

  async getNotifications(params: NotificationListParams = {}): Promise<PaginatedResponse<NotificationResponse> | undefined> {
    const searchParams = new URLSearchParams();
    
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        searchParams.append(key, String(value));
      }
    });

    const queryString = searchParams.toString();
    const endpoint = queryString ? `/api/v1/notifications?${queryString}` : '/api/v1/notifications';

    const response = await this.request<PaginatedResponse<NotificationResponse>>(endpoint);
    return response?.data;
  }

  async markNotificationRead(notificationId: string): Promise<void> {
    await this.request(`/api/v1/notifications/${notificationId}/read`, {
      method: 'POST'
    });
  }

  async markAllNotificationsRead(): Promise<void> {
    await this.request('/api/v1/notifications/read-all', {
      method: 'POST'
    });
  }

  async getNotificationStats(): Promise<NotificationStats | undefined> {
    const response = await this.request<NotificationStats>('/api/v1/notifications/stats');
    return response?.data;
  }

  async deleteNotification(notificationId: string): Promise<void> {
    await this.request(`/api/v1/notifications/${notificationId}`, {
      method: 'DELETE'
    });
  }

  // Server-side notifications API
  async getNotificationsServer(userId: string): Promise<Notification[] | undefined> {
    const response = await this.request<Notification[]>(`/api/notifications/${userId}`);
    return response?.data;
  }

  async markNotificationReadServer(userId: string, notificationId: string): Promise<void> {
    await this.request(`/api/notifications/${userId}/${notificationId}/read`, {
      method: 'POST'
    });
  }

  async getUnreadNotificationCount(userId: string): Promise<{ count: number } | undefined> {
    const response = await this.request<{ count: number }>(`/api/notifications/${userId}/unread/count`);
    return response?.data;
  }

  // ============================================================================
  // User Management API Methods
  // ============================================================================

  async getUserProfile(): Promise<any | undefined> {
    const response = await this.request('/api/v1/user/profile');
    return response?.data;
  }

  async updateUserProfile(data: any): Promise<any | undefined> {
    const response = await this.request('/api/v1/user/profile', {
      method: 'PUT',
      body: JSON.stringify(data)
    });
    return response?.data;
  }

  // ============================================================================
  // Watchlist API Methods
  // ============================================================================

  async getWatchlist(): Promise<any[] | undefined> {
    const response = await this.request<any[]>('/api/v1/user/watchlist');
    return response?.data;
  }

  async addToWatchlist(request: WatchlistAddRequest): Promise<void> {
    await this.request('/api/v1/user/watchlist', {
      method: 'POST',
      body: JSON.stringify(request)
    });
  }

  async removeFromWatchlist(symbol: string): Promise<void> {
    await this.request(`/api/v1/user/watchlist/${symbol}`, {
      method: 'DELETE'
    });
  }

  // ============================================================================
  // Price Alerts API Methods
  // ============================================================================

  async getPriceAlerts(): Promise<any[] | undefined> {
    const response = await this.request<any[]>('/api/v1/user/alerts');
    return response?.data;
  }

  async createPriceAlert(request: PriceAlertCreateRequest): Promise<void> {
    await this.request('/api/v1/user/alerts', {
      method: 'POST',
      body: JSON.stringify(request)
    });
  }

  async deletePriceAlert(alertId: string): Promise<void> {
    await this.request(`/api/v1/user/alerts/${alertId}`, {
      method: 'DELETE'
    });
  }

  // ============================================================================
  // Push Notifications API Methods
  // ============================================================================

  async subscribeToPushNotifications(subscription: PushSubscriptionRequest): Promise<void> {
    await this.request('/api/v1/user/push-subscription', {
      method: 'POST',
      body: JSON.stringify(subscription)
    });
  }

  async unsubscribeFromPushNotifications(): Promise<void> {
    await this.request('/api/v1/user/push-subscription', {
      method: 'DELETE'
    });
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
// Analytics Client Class
// ============================================================================

export class AnalyticsClient {
  constructor(private apiClient: ApiClient) {}

  async getRankings(params?: {
    page?: number;
    per_page?: number;
    country?: string;
    sector?: string;
    min_market_cap?: number;
    sort_by?: string;
    sort_order?: 'asc' | 'desc';
  }): Promise<UnifiedAnalyticsRankingsResponse | undefined> {
    return this.apiClient.getUnifiedAnalyticsRankings(params);
  }

  async getHealth(): Promise<{ status: string; timestamp: string } | undefined> {
    return this.apiClient.getAnalyticsHealth();
  }
}

// ============================================================================
// Exports
// ============================================================================

// Export singleton instance
export const apiClient = new ApiClient();
export const analyticsClient = new AnalyticsClient(apiClient);

// Types are imported from their respective modules as needed