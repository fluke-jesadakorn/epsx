/**
 * SHARED HOOKS INDEX
 *
 * Unified exports for all React hooks.
 * Simplifies imports across applications.
 */

// ============================================================================
// BASE API CLIENT
// ============================================================================

export {
  useAdminApiClient, useApiClient, useFrontendApiClient, type ApiClients, type Platform
} from './useApiClient';

// ============================================================================
// USER HOOKS
// ============================================================================

export {
  useApiKeys,
  useCreateApiKey,
  useDeleteApiKey, useSubscribeToPlan, useSubscriptions, useUpdateProfile, useUpdateSettings, useUserProfile, useUserSettings
} from './useUsers';

// ============================================================================
// PERMISSION HOOKS
// ============================================================================

export {
  useCheckPermission, useCurrentUserPermissions, useGrantPermission, usePermissionDisplay, usePermissionStats, useRevokePermission, useWalletPermissions
} from './usePermissions';

// ============================================================================
// WALLET HOOKS
// ============================================================================

export {
  useRecentWallets,
  useUpdateWalletStatus,
  useUpdateWalletTier, useWallet,
  useWalletSearch, useWalletStats
} from './useWallets';

// ============================================================================
// COMPLIANCE HOOKS
// ============================================================================

export {
  useApproveKYC, useBlockUser,
  useComplianceMetrics, useFlagUser, useKYCStatuses, useRejectKYC,
  useRiskAssessments, useSuspiciousActivities, useUpdateRiskAssessment
} from './useCompliance';

// ============================================================================
// CONVENIENCE RE-EXPORTS
// ============================================================================

// Most commonly used hooks for quick access
export {
  useKYCStatuses as useKYC, useCurrentUserPermissions as usePermissions, useUserProfile as useProfile, useWalletSearch as useSearchWallets, useWallet as useWalletInfo
} from './';

// ============================================================================
// ANALYTICS HOOKS
// ============================================================================

export {
  // Constants
  DEFAULT_ANALYTICS_CONFIG,
  DEFAULT_FILTER_OPTIONS,
  REALTIME_ANALYTICS_CONFIG,
  SLOW_ANALYTICS_CONFIG,
  // Utilities
  buildQueryString,
  combineErrorStates,
  combineLoadingStates,
  // Factories
  createAnalyticsHook,
  createParameterizedAnalyticsHook,
  // Types
  type AnalyticsConfig,
  type AnalyticsDashboardData,
  type AnalyticsPaginationParams,
  type FetcherFunction,
  type FilterOption,
  type PermissionAnalytics,
  type RichFilterOptions,
  type SystemMetrics,
  type UserStats
} from './useAnalyticsData';

