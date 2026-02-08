/**
 * Frontend API Client
 * 
 * Uses shared UnifiedApiClient for core HTTP functionality.
 * Provides domain-specific methods for frontend application.
 */

import type {
  ApiResponse,
  PaginatedResponse,
  UnifiedApiClient} from '@/shared/api';
import {
  createFrontendApiClient
} from '@/shared/api';
import { API_ROUTES } from '@/shared/config/route-constants';

// ============================================================================
// Types
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
  current_price?: number;
  price_current?: number;
  price?: number;
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
// Frontend API Client
// ============================================================================

class FrontendApiClient {
  private client: UnifiedApiClient;

  constructor() {
    this.client = createFrontendApiClient({
      serverSide: typeof window === 'undefined',
    });
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
    const filteredParams = Object.fromEntries(
      Object.entries(params).filter(([, v]) => v !== undefined)
    );
    const response = await this.client.get<UnifiedAnalyticsRankingsResponse>(
      API_ROUTES.ANALYTICS.RANKINGS,
      filteredParams
    );
    return response.data;
  }

  async getAnalyticsHealth(): Promise<{ status: string; timestamp: string } | undefined> {
    const response = await this.client.get<{ status: string; timestamp: string }>(API_ROUTES.HEALTH);
    return response.data;
  }

  // ============================================================================
  // Notification API Methods
  // ============================================================================

  async getNotifications(params: NotificationListParams = {}): Promise<PaginatedResponse<NotificationResponse> | undefined> {
    const filteredParams = Object.fromEntries(
      Object.entries(params).filter(([, v]) => v !== undefined)
    );
    const response = await this.client.get<PaginatedResponse<NotificationResponse>>(
      '/api/notifications',
      filteredParams
    );
    return response.data;
  }

  async markNotificationRead(notificationId: string): Promise<void> {
    await this.client.post(`/api/notifications/${notificationId}/read`);
  }

  async markAllNotificationsRead(): Promise<void> {
    await this.client.post('/api/notifications/read-all');
  }

  async getNotificationStats(): Promise<NotificationStats | undefined> {
    const response = await this.client.get<NotificationStats>('/api/notifications/unread-count');
    return response.data;
  }

  async deleteNotification(notificationId: string): Promise<void> {
    await this.client.delete(`/api/notifications/${notificationId}`);
  }

  async getNotificationsServer(userId: string): Promise<Notification[] | undefined> {
    const response = await this.client.get<Notification[]>(`/api/notifications/${userId}`);
    return response.data;
  }

  async markNotificationReadServer(userId: string, notificationId: string): Promise<void> {
    await this.client.post(`/api/notifications/${userId}/${notificationId}/read`);
  }

  async getUnreadNotificationCount(userId: string): Promise<{ count: number } | undefined> {
    const response = await this.client.get<{ count: number }>(`/api/notifications/${userId}/unread/count`);
    return response.data;
  }

  // ============================================================================
  // User Management API Methods
  // ============================================================================

  async getUserProfile(): Promise<Record<string, unknown> | undefined> {
    const response = await this.client.get<Record<string, unknown>>(API_ROUTES.USERS.PROFILE);
    return response.data;
  }

  async updateUserProfile(data: Record<string, unknown>): Promise<Record<string, unknown> | undefined> {
    const response = await this.client.put<Record<string, unknown>>(API_ROUTES.USERS.PROFILE, data);
    return response.data;
  }

  // ============================================================================
  // Watchlist API Methods
  // ============================================================================

  async getWatchlist(): Promise<Array<Record<string, unknown>> | undefined> {
    const response = await this.client.get<Array<Record<string, unknown>>>(API_ROUTES.USERS.WATCHLIST);
    return response.data;
  }

  async addToWatchlist(request: WatchlistAddRequest): Promise<void> {
    await this.client.post(API_ROUTES.USERS.WATCHLIST, request);
  }

  async removeFromWatchlist(symbol: string): Promise<void> {
    await this.client.delete(`/api/user/watchlist/${symbol}`);
  }

  // ============================================================================
  // Price Alerts API Methods
  // ============================================================================

  async getPriceAlerts(): Promise<Array<Record<string, unknown>> | undefined> {
    const response = await this.client.get<Array<Record<string, unknown>>>(API_ROUTES.USERS.ALERTS);
    return response.data;
  }

  async createPriceAlert(request: PriceAlertCreateRequest): Promise<void> {
    await this.client.post(API_ROUTES.USERS.ALERTS, request);
  }

  async deletePriceAlert(alertId: string): Promise<void> {
    await this.client.delete(`/api/user/alerts/${alertId}`);
  }

  // ============================================================================
  // Push Notifications API Methods
  // ============================================================================

  async subscribeToPushNotifications(subscription: PushSubscriptionRequest): Promise<void> {
    await this.client.post(API_ROUTES.USERS.PUSH_SUBSCRIPTION, subscription);
  }

  async unsubscribeFromPushNotifications(): Promise<void> {
    await this.client.delete(API_ROUTES.USERS.PUSH_SUBSCRIPTION);
  }

  // ============================================================================
  // Generic HTTP Methods (delegate to UnifiedApiClient)
  // ============================================================================

  async get<T = unknown>(endpoint: string, params?: Record<string, any>): Promise<ApiResponse<T>> {
    return this.client.get<T>(endpoint, params);
  }

  async post<T = unknown>(endpoint: string, data?: unknown): Promise<ApiResponse<T>> {
    return this.client.post<T>(endpoint, data);
  }

  async put<T = unknown>(endpoint: string, data?: unknown): Promise<ApiResponse<T>> {
    return this.client.put<T>(endpoint, data);
  }

  async delete<T = unknown>(endpoint: string): Promise<ApiResponse<T>> {
    return this.client.delete<T>(endpoint);
  }
}

// ============================================================================
// Analytics Client Class
// ============================================================================

export class AnalyticsClient {
  constructor(private frontendClient: FrontendApiClient) { }

  async getRankings(params?: {
    page?: number;
    per_page?: number;
    country?: string;
    sector?: string;
    min_market_cap?: number;
    sort_by?: string;
    sort_order?: 'asc' | 'desc';
  }): Promise<UnifiedAnalyticsRankingsResponse | undefined> {
    return this.frontendClient.getUnifiedAnalyticsRankings(params);
  }

  async getHealth(): Promise<{ status: string; timestamp: string } | undefined> {
    return this.frontendClient.getAnalyticsHealth();
  }
}

// ============================================================================
// Exports
// ============================================================================

// Export singleton instances
export const apiClient = new FrontendApiClient();
export const analyticsClient = new AnalyticsClient(apiClient);

// Export types from shared
export type { ApiResponse, PaginatedResponse };
