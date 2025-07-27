// Main exports
import { ApiClient } from './api-client';

export { ApiClient } from './api-client';

// Server actions exports (Next.js server components only)
export {
  serverRegister,
  serverGetAdminConfig,
  serverGetVapidKey,
  serverGetStockSymbols,
  serverGetIndividualStock,
  serverBatchStocks,
  serverGetPremiumRankings,
  serverGetSystemCache,
  serverGetCurrentUser,
  serverGetAuditLogs,
  serverCreateMusePayPayment,
  serverCreateCryptoPayment,
  serverCreateCryptoDepositAddress,
  serverLogin,
  serverListUsers,
  serverGetUser,
  serverSetUserRole,
  serverGetUserStats,
  serverBulkUpdateUserRoles,
  serverGetAdminUsers,
  serverGetAdminUser,
  serverGetAdminPermissionProfiles,
  serverGetAdminPermissionProfile,
  serverAssignAdminPermissionProfile,
  serverGetStockRankingAssignments,
  serverGetStockRankingAssignment,
  serverAssignBulkStockRanking,
  serverRevokeStockRankingAssignment,
  serverExtendStockRankingAssignment,
  serverUpdateStockRankingAssignment,
  serverGetAnalyticsStatistics,
  serverGetStockRankingAnalytics,
  serverGetAdminProfile,
  serverSoftDeleteUser,
  serverAssignPermissionProfile,
} from './api-server';
export { COOKIE_CONFIG, COOKIE_NAMES, CookieManager } from './cookie-manager';

// Type exports
export type {
  ActionResult,
  AdminProfile,
  AdminUser,
  AnalyticsStatistics,
  ApiError,
  ApiResponse,
  AssignmentResult,
  AuthCookies,
  CountResponse,
  CreatePaymentRequest,
  CreatePaymentResponse,
  HttpMethod,
  LoginRequest,
  Notification,
  NotificationListResponse,
  NotificationPreferences,
  PaginatedResponse,
  PasswordChangeRequest,
  PasswordResetRequest,
  PaymentStatusResponse,
  Permission,
  PermissionCheckRequest,
  PermissionCheckResponse,
  PermissionProfile,
  PermissionProfileAssignmentRequest,
  PortfolioItem,
  PriceAlert,
  PriceAlertCreateRequest,
  ProfileUpdateRequest,
  PushSubscriptionRequest,
  RegisterRequest,
  RequestConfig,
  Role,
  StockFinancialData,
  StockItem,
  StockRanking,
  StockRankingAnalytics,
  StockRankingAssignment,
  StockRankingAssignmentExtendRequest,
  StockRankingAssignmentRequest,
  StockRankingAssignmentUpdateRequest,
  UserListOptions,
  UserListResult,
  UserPermissionStatus,
  UserProfile,
  UserSoftDeleteRequest,
  WatchlistAddRequest,
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
