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
  useApiClient,
  useAdminApiClient,
  useFrontendApiClient,
  type Platform,
  type ApiClients
} from './useApiClient';

// ============================================================================
// USER HOOKS
// ============================================================================

export {
  useUserProfile,
  useUpdateProfile,
  useUserSettings,
  useUpdateSettings,
  useSubscriptions,
  useSubscribeToPlan,
  useApiKeys,
  useCreateApiKey,
  useDeleteApiKey
} from './useUsers';

// ============================================================================
// PERMISSION HOOKS
// ============================================================================

export {
  useCurrentUserPermissions,
  useWalletPermissions,
  usePermissionStats,
  useGrantPermission,
  useRevokePermission,
  useCheckPermission,
  usePermissionDisplay
} from './usePermissions';

// ============================================================================
// WALLET HOOKS
// ============================================================================

export {
  useWallet,
  useWalletSearch,
  useRecentWallets,
  useUpdateWalletStatus,
  useUpdateWalletTier,
  useWalletStats
} from './useWallets';

// ============================================================================
// COMPLIANCE HOOKS
// ============================================================================

export {
  useKYCStatuses,
  useApproveKYC,
  useRejectKYC,
  useRiskAssessments,
  useUpdateRiskAssessment,
  useSuspiciousActivities,
  useFlagUser,
  useBlockUser,
  useComplianceMetrics
} from './useCompliance';

// ============================================================================
// CONVENIENCE RE-EXPORTS
// ============================================================================

// Most commonly used hooks for quick access
export {
  useUserProfile as useProfile,
  useCurrentUserPermissions as usePermissions,
  useWallet as useWalletInfo,
  useWalletSearch as useSearchWallets,
  useKYCStatuses as useKYC
} from './';
