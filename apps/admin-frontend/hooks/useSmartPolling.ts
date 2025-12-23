'use client';

/**
 * Admin Frontend Smart Polling Hook
 * Re-exports from shared with admin-specific config
 */

import {
  DEFAULT_POLLING_CONFIG as SHARED_DEFAULT_CONFIG,
  useSmartPolling as sharedUseSmartPolling,
  useOptimisticUpdate,
  usePollingManager,
  type PollingPriority,
  type PollingState,
  type SmartPollingConfig,
  type UseSmartPollingOptions
} from '@/shared/hooks/useSmartPolling';

// Re-export types
export type { PollingPriority, PollingState, SmartPollingConfig, UseSmartPollingOptions };

// Re-export utility hooks
export { useOptimisticUpdate, usePollingManager };

// Admin-specific polling config
export const DEFAULT_POLLING_CONFIG: SmartPollingConfig = {
  ...SHARED_DEFAULT_CONFIG,
  intervals: {
    critical: 30000,    // 30s - Security alerts, system errors
    important: 60000,   // 1min - User stats, active sessions
    normal: 300000,     // 5min - Analytics, reports  
    background: 900000  // 15min - Historical data, trends
  }
};

// Legacy type alias for backward compatibility
export type TilePriority = PollingPriority;

// Re-export with admin default config
export function useSmartPolling<T = unknown>(
  key: string | null,
  fetcher: () => Promise<T>,
  options: UseSmartPollingOptions = {}
) {
  return sharedUseSmartPolling(key, fetcher, options, DEFAULT_POLLING_CONFIG);
}