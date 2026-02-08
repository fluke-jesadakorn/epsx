'use client';

/**
 * Simplified Smart Polling Hook using TanStack Query
 * Provides polling with adaptive intervals and tab visibility awareness.
 */

import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useState } from 'react';

// ============================================================================
// TYPES
// ============================================================================

export type PollingPriority = 'critical' | 'important' | 'normal' | 'background';

export interface SmartPollingConfig {
    intervals: Record<PollingPriority, number>;
    pauseWhenInactive: boolean;
}

export interface UseSmartPollingOptions {
    priority?: PollingPriority;
    customInterval?: number;
    pauseWhenHidden?: boolean;
    retry?: number;
    staleTime?: number;
    config?: SmartPollingConfig;
}

// ============================================================================
// DEFAULT CONFIG
// ============================================================================

export const DEFAULT_POLLING_CONFIG: SmartPollingConfig = {
    intervals: {
        critical: 30000,    // 30 seconds
        important: 60000,   // 1 minute
        normal: 300000,     // 5 minutes
        background: 900000  // 15 minutes
    },
    pauseWhenInactive: true,
};

// ============================================================================
// GLOBAL STATE
// ============================================================================

const globalPollingState = {
    isTabActive: true,
    connectionQuality: 'good' as 'good' | 'poor' | 'offline',
};

// Track tab visibility
if (typeof document !== 'undefined') {
    document.addEventListener('visibilitychange', () => {
        globalPollingState.isTabActive = !document.hidden;
    });
}

// Network quality detection
interface NetworkInformation {
    effectiveType: string;
    addEventListener: (type: string, listener: () => void) => void;
}

interface NavigatorWithConnection extends Navigator {
    connection?: NetworkInformation;
}

if (typeof navigator !== 'undefined' && 'connection' in navigator) {
    const updateConnection = () => {
        const nav = navigator as NavigatorWithConnection;
        const conn = nav.connection;
        if (conn) {
            if (conn.effectiveType === '4g') {
                globalPollingState.connectionQuality = 'good';
            } else if (conn.effectiveType === '3g' || conn.effectiveType === '2g') {
                globalPollingState.connectionQuality = 'poor';
            } else {
                globalPollingState.connectionQuality = 'offline';
            }
        }
    };
    const nav = navigator as NavigatorWithConnection;
    nav.connection?.addEventListener('change', updateConnection);
    updateConnection();
}

// ============================================================================
// MAIN HOOK
// ============================================================================

export function useSmartPolling<T = unknown>(
    queryKey: string[],
    fetcher: () => Promise<T>,
    options: UseSmartPollingOptions = {}
) {
    const queryClient = useQueryClient();
    const {
        priority = 'normal',
        customInterval,
        pauseWhenHidden = true,
        retry = 3,
        staleTime = 0,
        config = DEFAULT_POLLING_CONFIG
    } = options;

    const [isPaused, setIsPaused] = useState(false);
    const [_, setRetryCount] = useState(0);

    // Calculate interval based on priority and conditions
    const calculateInterval = useCallback(() => {
        let baseInterval = customInterval ?? config.intervals[priority];

        // Adjust based on tab visibility
        if (!globalPollingState.isTabActive && pauseWhenHidden) {
            baseInterval = Math.min(baseInterval * 3, 900000);
        }

        // Adjust based on connection quality
        if (globalPollingState.connectionQuality === 'poor') {
            baseInterval = Math.min(baseInterval * 2, 900000);
        } else if (globalPollingState.connectionQuality === 'offline') {
            baseInterval = 0;
        }

        return baseInterval;
    }, [priority, customInterval, pauseWhenHidden, config.intervals]);

    const interval = calculateInterval();

    const query = useQuery({
        queryKey,
        queryFn: async () => {
            if (isPaused) {
                throw new Error('Polling paused');
            }
            return fetcher();
        },
        refetchInterval: isPaused ? false : interval,
        refetchIntervalInBackground: false,
        refetchOnWindowFocus: priority === 'critical' || priority === 'important',
        refetchOnReconnect: true,
        retry,
        staleTime,
        enabled: !isPaused && interval > 0,
    });

    // Reset retry count on success
    useEffect(() => {
        if (query.data !== null && query.data !== undefined && query.error === null) {
            setRetryCount(0);
        }
    }, [query.data, query.error]);

    const manualRetry = useCallback(() => {
        setRetryCount(0);
        void queryClient.invalidateQueries({ queryKey });
    }, [queryClient, queryKey]);

    const pause = useCallback(() => {
        setIsPaused(true);
    }, []);

    const resume = useCallback(() => {
        setIsPaused(false);
        setRetryCount(0);
    }, []);

    const refresh = useCallback(() => {
        void query.refetch();
    }, [query]);

    return {
        ...query,
        // Enhanced properties
        retry: manualRetry,
        pause,
        resume,
        refresh,
        // Computed values
        isPollingActive: !isPaused && interval > 0,
        connectionStatus: globalPollingState.connectionQuality,
    };
}

// ============================================================================
// POLLING MANAGER HOOK
// ============================================================================

export function usePollingManager() {
    const [stats, setStats] = useState({
        isTabActive: true,
        connectionQuality: 'good' as 'good' | 'poor' | 'offline',
        lastActivity: Date.now(),
    });

    useEffect(() => {
        const interval = setInterval(() => {
            setStats({
                isTabActive: globalPollingState.isTabActive,
                connectionQuality: globalPollingState.connectionQuality,
                lastActivity: Date.now(),
            });
        }, 1000);

        return () => clearInterval(interval);
    }, []);

    return stats;
}
