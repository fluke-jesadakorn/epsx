/**
 * Frontend API Client Module
 *
 * Direct exports from the API module.
 * Uses shared UnifiedApiClient for core functionality.
 */

// Export all API functionality
export {
  analyticsClient,
  AnalyticsClient, apiClient,
  // Types
  type ApiResponse, type CountResponse, type Notification, type NotificationListParams,
  type NotificationResponse, type NotificationStats, type PaginatedResponse, type PriceAlertCreateRequest,
  type PushSubscriptionRequest, type StockFinancialData, type UnifiedAnalyticsRankingsResponse, type UnifiedRankingItem, type WatchlistAddRequest
} from './api/client';

// Export shared utilities
export {
  createFrontendApiClient, isApiError,
  isApiResponse,
  isPaginatedResponse
} from '@/shared/api';

