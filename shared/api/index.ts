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
  UnifiedApiClient,
  APIError,
  createAdminApiClient,
  createFrontendApiClient,
  createApiClient,
  handlePaginatedRequest,
  handleSimpleRequest,
  retryRequest,
  isApiError,
  isApiResponse,
  isPaginatedResponse,
  type ApiResponse,
  type ApiError,
  type RequestConfig,
  type PaginatedResponse,
  type Platform
} from '../utils/api-client';

// ============================================================================
// DOMAIN API CLIENTS
// ============================================================================

export {
  UsersApi,
  createUsersClient,
  createAdminUsersClient,
  type UserProfile,
  type UserSettings,
  type UpdateProfileRequest,
  type UpdateSettingsRequest,
  type SubscriptionInfo,
  type UserApiKey
} from './users';

export {
  PermissionsApi,
  createPermissionsClient,
  createAdminPermissionsClient,
  type Permission,
  type PermissionEntry,
  type GrantPermissionRequest,
  type RevokePermissionRequest,
  type UserPermissionsResponse
} from './permissions';

export {
  GroupsApi,
  createGroupsClient,
  type Group,
  type GroupMembership,
  type AssignGroupRequest,
  type RemoveGroupRequest
} from './groups';

export {
  WalletsApi,
  createWalletsClient,
  type WalletInfo,
  type WalletSearchFilters,
  type WalletStats,
  type RecentWallet
} from './wallets';

export {
  ComplianceApi,
  createComplianceClient,
  type KYCStatus,
  type RiskAssessment,
  type SuspiciousActivity,
  type ComplianceMetrics
} from './compliance';

export {
  AnalyticsAPIClient as AnalyticsApi,
  createAnalyticsClient,
  type EPSRanking,
  type AnalyticsFilters,
  type EPSRankingsResponse
} from './analytics';

export {
  AuthAPIClient as AuthApi,
  createAuthClient,
  type Web3Challenge,
  type Web3VerifyRequest,
  type Web3VerifyResponse,
  type SessionInfo
} from './auth';

export {
  NotificationsAPIClient as NotificationsApi,
  createNotificationsClient,
  type NotificationsResponse as NotificationResponse,
  type NotificationStats
} from './notifications';

export {
  PlansAPIClient as PlansApi,
  createPlansClient,
  type PlanResponse as Plan,
  type PlanFeatureResponse as PlanFeature
} from './plans';
