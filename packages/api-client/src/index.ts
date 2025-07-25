// Main exports
import { ApiClient } from './api-client';

export { ApiClient } from './api-client';
export { CookieManager, COOKIE_NAMES, COOKIE_CONFIG } from './cookie-manager';

// Type exports
export type {
  ApiResponse,
  ApiError,
  AuthCookies,
  LoginRequest,
  RegisterRequest,
  UserProfile,
  PasswordResetRequest,
  ProfileUpdateRequest,
  PasswordChangeRequest,
  AdminUser,
  UserListOptions,
  UserListResult,
  CreatePaymentRequest,
  CreatePaymentResponse,
  PaymentStatusResponse,
  StockFinancialData,
  PaginatedResponse,
  CountResponse,
  PermissionProfile,
  PermissionProfileAssignmentRequest,
  AssignmentResult,
  Permission,
  Role,
  UserPermissionStatus,
  PermissionCheckRequest,
  PermissionCheckResponse,
  StockRankingAssignment,
  StockRankingAssignmentRequest,
  StockRankingAssignmentExtendRequest,
  StockRankingAssignmentUpdateRequest,
  AnalyticsStatistics,
  StockRankingAnalytics,
  UserSoftDeleteRequest,
  AdminProfile,
  ActionResult,
  HttpMethod,
  RequestConfig,
  StockItem,
  PortfolioItem,
  StockRanking,
  PriceAlert,
  WatchlistAddRequest,
  PriceAlertCreateRequest,
  Notification,
  NotificationPreferences,
  NotificationListResponse,
  PushSubscriptionRequest,
} from './types';

// Type guards
export { isApiError, isApiSuccess } from './types';

// Create default instances for convenience
const apiClient = new ApiClient();

// Named exports for common use
export { apiClient };

// Helper functions
export const createApiClient = (baseUrl?: string) => new ApiClient(baseUrl);

// Utility exports for backward compatibility
export const loginWithCredentials = async (email: string, password: string) => {
  return apiClient.login({ type: 'credentials', email, password });
};