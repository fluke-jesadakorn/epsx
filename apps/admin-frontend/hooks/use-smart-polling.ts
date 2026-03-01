'use client';

/**
 * Admin Frontend Smart Polling Hook
 * Re-exports from shared with admin-specific config
 */

import {
  DEFAULT_POLLING_CONFIG as SHARED_DEFAULT_CONFIG,
  useSmartPolling as sharedUseSmartPolling,
  usePollingManager,
  type PollingPriority,
  type SmartPollingConfig,
  type UseSmartPollingOptions
} from '@/shared/hooks/use-smart-polling';

// Re-export types
export type { PollingPriority, SmartPollingConfig, UseSmartPollingOptions };

// Compatibility alias
export type PollingState = ReturnType<typeof sharedUseSmartPolling>;

// Re-export utility hooks
export { usePollingManager };

// Stub for backward compatibility
export function useOptimisticUpdate<T>(initialValue: T) {
  return { value: initialValue, setOptimistic: (_v: T) => { /* noop */ } };
}

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
/**
 *
 * @param key
 * @param fetcher
 * @param options
 */
export function useSmartPolling<T = unknown>(
  key: string | null,
  fetcher: () => Promise<T>,
  options: UseSmartPollingOptions = {}
) {
  const queryKey = key !== null ? [key] : null;
  return sharedUseSmartPolling(
    queryKey as string[],
    fetcher,
    { ...options, config: DEFAULT_POLLING_CONFIG }
  );
}
