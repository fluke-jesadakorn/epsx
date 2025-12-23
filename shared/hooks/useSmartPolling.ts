'use client';

/**
 * SHARED Smart Polling Hook
 * Provides intelligent polling with adaptive intervals, exponential backoff,
 * tab visibility awareness, and connection quality detection.
 * 
 * Single source of truth for polling logic across all apps.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import useSWR, { SWRConfiguration } from 'swr';

// ============================================================================
// TYPES
// ============================================================================

export type PollingPriority = 'critical' | 'important' | 'normal' | 'background';

export interface SmartPollingConfig {
    intervals: Record<PollingPriority, number>;
    pauseWhenInactive: boolean;
    retryOnError: boolean;
    maxRetries: number;
}

export interface UseSmartPollingOptions extends SWRConfiguration {
    priority?: PollingPriority;
    customInterval?: number;
    pauseWhenHidden?: boolean;
    exponentialBackoff?: boolean;
    retryLimit?: number;
    onError?: (error: unknown) => void;
    onSuccess?: (data: unknown) => void;
}

export interface PollingState {
    isActive: boolean;
    currentInterval: number;
    retryCount: number;
    lastError: unknown;
    isPaused: boolean;
    isTabActive: boolean;
    connectionQuality: 'good' | 'poor' | 'offline';
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
    retryOnError: true,
    maxRetries: 3
};

// ============================================================================
// GLOBAL STATE
// ============================================================================

// Global state for managing polling across all hooks
const globalPollingState = {
    activeConnections: 0,
    isTabActive: true,
    connectionQuality: 'good' as 'good' | 'poor' | 'offline',
    lastActivity: Date.now()
};

// Track tab visibility across all hooks
if (typeof document !== 'undefined') {
    document.addEventListener('visibilitychange', () => {
        globalPollingState.isTabActive = !document.hidden;
        globalPollingState.lastActivity = Date.now();
    });
}

// Network quality detection
if (typeof navigator !== 'undefined' && 'connection' in navigator) {
    const updateConnection = () => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const conn = (navigator as any).connection;
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

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (navigator as any).connection?.addEventListener('change', updateConnection);
    updateConnection();
}

// ============================================================================
// MAIN HOOK
// ============================================================================

export function useSmartPolling<T = unknown>(
    key: string | null,
    fetcher: () => Promise<T>,
    options: UseSmartPollingOptions = {},
    config: SmartPollingConfig = DEFAULT_POLLING_CONFIG
) {
    const {
        priority = 'normal',
        customInterval,
        pauseWhenHidden = true,
        exponentialBackoff = true,
        retryLimit = 3,
        onError,
        onSuccess,
        ...swrOptions
    } = options;

    const [pollingState, setPollingState] = useState<PollingState>({
        isActive: true,
        currentInterval: customInterval || config.intervals[priority],
        retryCount: 0,
        lastError: null,
        isPaused: false,
        isTabActive: globalPollingState.isTabActive,
        connectionQuality: globalPollingState.connectionQuality
    });

    const retryTimeoutRef = useRef<NodeJS.Timeout | null>(null);
    const lastFetchTimeRef = useRef<number>(0);

    // Calculate adaptive interval based on multiple factors
    const calculateInterval = useCallback(() => {
        let baseInterval = customInterval || config.intervals[priority];

        // Adjust based on tab visibility
        if (!globalPollingState.isTabActive && pauseWhenHidden) {
            baseInterval = Math.min(baseInterval * 3, 900000); // Max 15 minutes
        }

        // Adjust based on connection quality
        if (globalPollingState.connectionQuality === 'poor') {
            baseInterval = Math.min(baseInterval * 2, 900000);
        } else if (globalPollingState.connectionQuality === 'offline') {
            baseInterval = 0; // Stop polling when offline
        }

        // Exponential backoff for errors
        if (exponentialBackoff && pollingState.retryCount > 0) {
            baseInterval = Math.min(baseInterval * Math.pow(2, pollingState.retryCount), 900000);
        }

        // Minimum interval for critical priority
        if (priority === 'critical') {
            baseInterval = Math.max(baseInterval, 15000); // Never less than 15s for critical
        }

        return baseInterval;
    }, [priority, customInterval, pauseWhenHidden, exponentialBackoff, pollingState.retryCount, config.intervals]);

    // Update polling state when global state changes
    useEffect(() => {
        const interval = setInterval(() => {
            const newState = {
                isTabActive: globalPollingState.isTabActive,
                connectionQuality: globalPollingState.connectionQuality
            };

            setPollingState(prev => {
                if (prev.isTabActive !== newState.isTabActive ||
                    prev.connectionQuality !== newState.connectionQuality) {
                    return { ...prev, ...newState, currentInterval: calculateInterval() };
                }
                return prev;
            });
        }, 5000);

        return () => clearInterval(interval);
    }, [calculateInterval]);

    // Enhanced fetcher with error handling and retry logic
    const enhancedFetcher = useCallback(async (): Promise<T> => {
        try {
            globalPollingState.activeConnections++;
            lastFetchTimeRef.current = Date.now();

            const data = await fetcher();

            // Reset retry count on success
            setPollingState(prev => ({
                ...prev,
                retryCount: 0,
                lastError: null,
                isPaused: false
            }));

            onSuccess?.(data);
            return data;
        } catch (_error) {
            // eslint-disable-next-line no-console
            console.error(`Polling error for ${key}:`, _error);

            setPollingState(prev => {
                const newRetryCount = prev.retryCount + 1;
                const shouldPause = newRetryCount >= retryLimit;

                return {
                    ...prev,
                    retryCount: newRetryCount,
                    lastError: _error,
                    isPaused: shouldPause
                };
            });

            onError?.(_error);

            // Don't retry immediately if we've hit the limit
            if (pollingState.retryCount >= retryLimit) {
                throw _error;
            }

            // Exponential backoff retry
            if (exponentialBackoff) {
                const retryDelay = Math.min(1000 * Math.pow(2, pollingState.retryCount), 30000);
                await new Promise(resolve => {
                    retryTimeoutRef.current = setTimeout(resolve, retryDelay);
                });

                return enhancedFetcher();
            }

            throw _error;
        } finally {
            globalPollingState.activeConnections = Math.max(0, globalPollingState.activeConnections - 1);
        }
    }, [key, fetcher, onSuccess, onError, retryLimit, exponentialBackoff, pollingState.retryCount]);

    // SWR configuration with adaptive polling
    const swrConfig: SWRConfiguration<T> = {
        refreshInterval: pollingState.isPaused ? 0 : pollingState.currentInterval,
        revalidateOnFocus: priority === 'critical' || priority === 'important',
        revalidateOnReconnect: true,
        shouldRetryOnError: false, // We handle retries manually
        dedupingInterval: Math.min(pollingState.currentInterval / 2, 30000),
        focusThrottleInterval: 5000,
        errorRetryCount: 0, // We handle retries manually
        ...swrOptions
    };

    const swr = useSWR<T>(
        pollingState.isPaused ? null : key,
        enhancedFetcher,
        swrConfig
    );

    // Manual retry function
    const retry = useCallback(() => {
        setPollingState(prev => ({
            ...prev,
            retryCount: 0,
            lastError: null,
            isPaused: false
        }));
        swr.mutate();
    }, [swr]);

    // Pause/resume polling
    const pause = useCallback(() => {
        setPollingState(prev => ({ ...prev, isPaused: true }));
    }, []);

    const resume = useCallback(() => {
        setPollingState(prev => ({
            ...prev,
            isPaused: false,
            retryCount: 0,
            lastError: null
        }));
    }, []);

    // Force refresh with burst mode (ignore current interval)
    const refresh = useCallback(async () => {
        setPollingState(prev => ({ ...prev, retryCount: 0, lastError: null }));
        return swr.mutate();
    }, [swr]);

    // Cleanup
    useEffect(() => {
        return () => {
            if (retryTimeoutRef.current) {
                clearTimeout(retryTimeoutRef.current);
            }
        };
    }, []);

    return {
        ...swr,
        // Enhanced properties
        pollingState,
        retry,
        pause,
        resume,
        refresh,
        // Computed values
        isPollingActive: !pollingState.isPaused && pollingState.currentInterval > 0,
        timeSinceLastFetch: Date.now() - lastFetchTimeRef.current,
        canRetry: pollingState.retryCount < retryLimit && pollingState.lastError,
        connectionStatus: globalPollingState.connectionQuality,
        activeConnections: globalPollingState.activeConnections
    };
}

// ============================================================================
// POLLING MANAGER HOOK
// ============================================================================

export function usePollingManager() {
    const [stats, setStats] = useState({
        activeConnections: 0,
        isTabActive: true,
        connectionQuality: 'good' as 'good' | 'poor' | 'offline',
        totalRequests: 0,
        failedRequests: 0,
        lastActivity: Date.now()
    });

    useEffect(() => {
        const interval = setInterval(() => {
            setStats({
                activeConnections: globalPollingState.activeConnections,
                isTabActive: globalPollingState.isTabActive,
                connectionQuality: globalPollingState.connectionQuality,
                totalRequests: 0, // Would track this in production
                failedRequests: 0, // Would track this in production  
                lastActivity: globalPollingState.lastActivity
            });
        }, 1000);

        return () => clearInterval(interval);
    }, []);

    return stats;
}

// ============================================================================
// OPTIMISTIC UPDATE HOOK
// ============================================================================

export function useOptimisticUpdate<T>(
    key: string,
    updateFn: (data: T) => Promise<T>,
    options: { onError?: (error: unknown, rollback: () => void) => void } = {}
) {
    const { mutate } = useSWR(key);

    return useCallback(async (optimisticData: T) => {
        // Store current data for rollback
        const currentData = await mutate();

        // Optimistically update UI immediately
        mutate(optimisticData, false);

        try {
            // Perform actual update
            const updatedData = await updateFn(optimisticData);

            // Update with real data
            mutate(updatedData, false);

            return updatedData;
        } catch (_error) {
            // Rollback on error
            const rollback = () => mutate(currentData, false);
            options.onError?.(_error, rollback);
            rollback();
            throw _error;
        }
    }, [key, mutate, updateFn, options]);
}
