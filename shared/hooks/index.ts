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

// NOTE: useUsers.ts was removed in prior cleanup - these exports are now unavailable

// ============================================================================
// PERMISSION HOOKS
// ============================================================================

// NOTE: usePermissions.ts was removed in prior cleanup

// ============================================================================
// WALLET HOOKS
// ============================================================================

// NOTE: useWallets.ts was removed in prior cleanup

// ============================================================================
// COMPLIANCE HOOKS
// ============================================================================

// NOTE: useCompliance.ts was removed in prior cleanup

// ============================================================================
// SMART POLLING HOOKS
// ============================================================================

export {
  DEFAULT_POLLING_CONFIG, usePollingManager, useSmartPolling, type PollingPriority, type SmartPollingConfig, type UseSmartPollingOptions
} from './useSmartPolling';

// ============================================================================
// NOTIFICATION HOOKS
// ============================================================================

export {
  useNotificationBell
} from './useNotificationBell';
