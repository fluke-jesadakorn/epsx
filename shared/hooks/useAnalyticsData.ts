/**
 * SHARED ANALYTICS HOOKS
 *
 * Provides common analytics hook utilities for both frontend and admin-frontend.
 * Uses SWR for efficient data fetching with caching and revalidation.
 */

'use client';

import useSWR, { SWRConfiguration } from 'swr';

// ============================================================================
// TYPES
// ============================================================================

export interface AnalyticsConfig {
    /** Refresh interval in milliseconds */
    refreshInterval?: number;
    /** Whether to revalidate on window focus */
    revalidateOnFocus?: boolean;
    /** Number of retry attempts on error */
    errorRetryCount?: number;
}

export interface FilterOption {
    value: string;
    label: string;
}

export interface RichFilterOptions {
    countries: FilterOption[];
    sectors: string[];
    exchanges?: string[];
    stock_types?: string[];
}

export interface AnalyticsPaginationParams {
    page?: number;
    per_page?: number;
    sort_by?: string;
    sort_order?: 'asc' | 'desc';
}

// Admin-specific types
export interface UserStats {
    total_users: number;
    active_users: number;
    deleted_users: number;
    recent_users_30_days: number;
    by_permissions: Record<string, number>;
    by_tier: Record<string, number>;
    user_creation_by_month: Record<string, number>;
    generated_at: string;
}

export interface PermissionAnalytics {
    total_permissions: number;
    users_with_permissions: number;
    expiring_soon: number;
    expired: number;
    health_score: number;
    recent_activity: number;
}

export interface SystemMetrics {
    api_response_time: number;
    database_query_time: number;
    memory_usage: number;
    active_users: number;
    peak_users_today: number;
    new_signups: number;
}

export interface AnalyticsDashboardData {
    user_stats?: UserStats;
    permission_analytics?: PermissionAnalytics;
    system_metrics?: SystemMetrics;
    [key: string]: unknown;
}

// ============================================================================
// CONSTANTS
// ============================================================================

export const DEFAULT_ANALYTICS_CONFIG: Required<AnalyticsConfig> = {
    refreshInterval: 60000, // 1 minute
    revalidateOnFocus: true,
    errorRetryCount: 3,
};

export const REALTIME_ANALYTICS_CONFIG: Required<AnalyticsConfig> = {
    refreshInterval: 10000, // 10 seconds
    revalidateOnFocus: true,
    errorRetryCount: 3,
};

export const SLOW_ANALYTICS_CONFIG: Required<AnalyticsConfig> = {
    refreshInterval: 300000, // 5 minutes
    revalidateOnFocus: true,
    errorRetryCount: 3,
};

// Default filter options for fallback
export const DEFAULT_FILTER_OPTIONS: RichFilterOptions = {
    countries: [
        { value: 'america', label: 'United States' },
        { value: 'canada', label: 'Canada' },
        { value: 'united_kingdom', label: 'United Kingdom' },
        { value: 'germany', label: 'Germany' },
        { value: 'france', label: 'France' },
        { value: 'japan', label: 'Japan' },
        { value: 'australia', label: 'Australia' }
    ],
    sectors: [
        'Technology',
        'Healthcare',
        'Financial Services',
        'Consumer Discretionary',
        'Industrials',
        'Energy',
        'Telecommunications',
        'Real Estate',
    ],
    exchanges: ['NASDAQ', 'NYSE', 'LSE', 'TSX', 'ASX', 'HKEX', 'TSE', 'EURONEXT'],
    stock_types: ['common', 'preferred', 'reit', 'etf'],
};

// ============================================================================
// HOOK FACTORIES
// ============================================================================

export type FetcherFunction<T> = (url: string) => Promise<T>;

/**
 * Create a SWR-based analytics hook with the specified configuration
 */
export function createAnalyticsHook<T>(
    key: string | null,
    fetcher: FetcherFunction<T>,
    config: AnalyticsConfig = DEFAULT_ANALYTICS_CONFIG
) {
    const swrConfig: SWRConfiguration = {
        refreshInterval: config.refreshInterval ?? DEFAULT_ANALYTICS_CONFIG.refreshInterval,
        revalidateOnFocus: config.revalidateOnFocus ?? DEFAULT_ANALYTICS_CONFIG.revalidateOnFocus,
        errorRetryCount: config.errorRetryCount ?? DEFAULT_ANALYTICS_CONFIG.errorRetryCount,
    };

    return function useAnalytics() {
        const { data, error, isLoading, mutate } = useSWR<T>(key, fetcher, swrConfig);

        return {
            data,
            isLoading,
            error,
            refresh: mutate,
        };
    };
}

/**
 * Create a parameterized analytics hook (key changes based on params)
 */
export function createParameterizedAnalyticsHook<T, P extends Record<string, unknown>>(
    getKey: (params: P) => string,
    fetcher: FetcherFunction<T>,
    config: AnalyticsConfig = DEFAULT_ANALYTICS_CONFIG
) {
    const swrConfig: SWRConfiguration = {
        refreshInterval: config.refreshInterval ?? DEFAULT_ANALYTICS_CONFIG.refreshInterval,
        revalidateOnFocus: config.revalidateOnFocus ?? DEFAULT_ANALYTICS_CONFIG.revalidateOnFocus,
        errorRetryCount: config.errorRetryCount ?? DEFAULT_ANALYTICS_CONFIG.errorRetryCount,
    };

    return function useParameterizedAnalytics(params: P) {
        const key = getKey(params);
        const { data, error, isLoading, mutate } = useSWR<T>(key, fetcher, swrConfig);

        return {
            data,
            isLoading,
            error,
            refresh: mutate,
        };
    };
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Build URL query string from params object
 */
export function buildQueryString(params: Record<string, string | number | boolean | undefined>): string {
    const query = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== '') {
            query.set(key, String(value));
        }
    });
    const qs = query.toString();
    return qs ? `?${qs}` : '';
}

/**
 * Combine loading states from multiple hooks
 */
export function combineLoadingStates(...states: boolean[]): boolean {
    return states.some(state => state);
}

/**
 * Combine error states from multiple hooks
 */
export function combineErrorStates(...errors: (Error | undefined)[]): boolean {
    return errors.some(error => error !== undefined);
}
