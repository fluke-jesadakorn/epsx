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
} from './use-api-client';

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
} from './use-smart-polling';

// ============================================================================
// NOTIFICATION HOOKS
// ============================================================================

export {
  useNotificationBell
} from './use-notification-bell';

// ============================================================================
// CHAT SSE HOOKS
// ============================================================================

export {
  useChatSSE, type ChatSSEEvent
} from './use-chat-sse';
