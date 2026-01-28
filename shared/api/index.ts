/**
 * SHARED API INDEX
 *
 * Unified exports for all API clients.
 * Simplifies imports across applications.
 */

// ============================================================================
// BASE API CLIENT
// ============================================================================

export {
  APIError, createAdminApiClient, createApiClient, createFrontendApiClient, handlePaginatedRequest,
  handleSimpleRequest, isApiError,
  isApiResponse,
  isPaginatedResponse, retryRequest, UnifiedApiClient, type ApiError, type ApiResponse, type PaginatedResponse,
  type Platform, type RequestConfig
} from '../utils/api-client';

export { isApiSuccess } from '../types/api';

// ============================================================================
// RESPONSE UTILITIES
// ============================================================================

export {
  assertSuccess,
  extractArray,
  extractArrayOrEmpty,
  extractData,
  extractDataOrDefault,
  extractError,
  extractMessage,
  extractObject,
  extractPagination,
  isSuccess
} from '../utils/response-utils';

// ============================================================================
// DOMAIN API CLIENTS
// ============================================================================

export {
  createAdminUsersClient, createUsersClient, UsersApi, type SubscriptionInfo, type UpdateProfileRequest,
  type UpdateSettingsRequest, type UserApiKey, type UserProfile,
  type UserSettings
} from './users';

export {
  createAdminPermissionsClient, createPermissionsClient, PermissionsApi, type GrantPermissionRequest, type Permission,
  type PermissionEntry, type RevokePermissionRequest,
  type UserPermissionsResponse
} from './permissions';

export {
  createWalletsClient, WalletsApi, type RecentWallet, type WalletInfo,
  type WalletSearchFilters,
  type WalletStats
} from './wallets';

export {
  ComplianceApi,
  createComplianceClient, type ComplianceMetrics, type KYCStatus,
  type RiskAssessment,
  type SuspiciousActivity
} from './compliance';

export {
  AnalyticsAPIClient as AnalyticsApi,
  createAnalyticsClient, type AnalyticsFilters, type EPSRanking, type EPSRankingsResponse
} from './analytics';

export {
  AuthAPIClient as AuthApi,
  createAuthClient, type SessionInfo, type Web3Challenge,
  type Web3VerifyRequest,
  type Web3VerifyResponse
} from './auth';

export {
  createNotificationsClient, NotificationsAPIClient as NotificationsApi, type NotificationsResponse as NotificationResponse,
  type NotificationStats
} from './notifications';

export {
  createPlansClient, PlansApi, type Plan, type PlanMembership,
  type PlanStats
} from './plans';

export {
  createSecurityClient, formatThreatScore,
  getEventTypeIcon, getSeverityBadgeColor, getSeverityColor, SecurityApi, type SecurityAlert, type SecurityEvent, type SecurityEventFilters, type SecurityEventsResponse,
  type SecurityMetrics, type SecurityMetricsResponse, type SecurityTrends, type SecurityTrendSummary, type UserThreatResponse
} from './security';

export {
  createSettingsClient, DEFAULT_SETTINGS, SettingsApi, type AppearanceSettings, type GeneralSettings,
  type NotificationSettings,
  type SecuritySettings, type SettingUpdate, type SystemSettings, type UpdateSettingsResponse
} from './settings';

export { createAdminPaymentsClient, createPaymentsClient, PaymentsApi, type PaymentConfirmRequest, type PaymentConfirmResult, type PaymentSubmitRequest, type PaymentValidateRequest, type PaymentValidationResult, type TransactionStatusData } from './payments';

export {
  ChatApi,
  chatApi,
  createChatClient, type ChatHistoryResponse,
  type ChatOptions, type ChatRequest,
  type ChatResponse, type Message
} from './chat';

