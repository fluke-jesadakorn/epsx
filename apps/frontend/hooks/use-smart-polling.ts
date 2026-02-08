'use client';

/**
 * Frontend Smart Polling Hook
 * Re-exports from shared with frontend-specific config
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
} from '@/shared/hooks/use-smart-polling';

// Re-export types
export type { PollingPriority, PollingState, SmartPollingConfig, UseSmartPollingOptions };

// Re-export utility hooks
export { useOptimisticUpdate, usePollingManager };

// Frontend-specific polling config (typically longer intervals for user-facing content)
export const DEFAULT_POLLING_CONFIG: SmartPollingConfig = {
    ...SHARED_DEFAULT_CONFIG,
    intervals: {
        critical: 30000,    // 30s - Real-time price data
        important: 120000,  // 2min - Stock rankings
        normal: 300000,     // 5min - Market data
        background: 600000  // 10min - Static content
    }
};

// Re-export with frontend default config
export function useSmartPolling<T = unknown>(
    key: string | null,
    fetcher: () => Promise<T>,
    options: UseSmartPollingOptions = {}
) {
    return sharedUseSmartPolling(key, fetcher, options, DEFAULT_POLLING_CONFIG);
}
