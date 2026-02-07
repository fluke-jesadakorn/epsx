'use client';

import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback } from 'react';

// ============================================================================
// FETCHER (Legacy fetcher removed in favor of Server Action)
// ============================================================================

// ============================================================================
// INDIVIDUAL DATA HOOKS (using shared types)
// ============================================================================

import {
  getApiKeysAction,
  getDeveloperPortalStatsAction,
  getPermissionAnalyticsAction,
  getPlanStatsAction,
  getSystemMetricsAction,
  getUserStatsAction
} from '@/app/analytics/actions';


// ============================================================================
// TYPES & CONFIG (Restored locally after shared cleanup)
// ============================================================================

export interface UserStats {
  total?: number;
  active?: number;
  today_connections?: number;
  total_users?: number;
  active_users?: number;
  inactive_users?: number;
  new_users_30_days?: number;
  growth_rate?: number;
}

export interface PermissionAnalytics {
  total?: number;
  pending_notifications?: number;
  total_groups?: number;
  total_permissions?: number;
  active_permissions?: number;
  permission_usage?: unknown[];
  expiring_soon?: number;
  expired?: number;
  health_score?: number;
}

export interface SystemMetrics {
  health_percentage?: number;
  uptime?: string;
  avg_response_time?: string;
  api_response_time?: number;
  memory_usage?: number;
  active_users?: number;
  peak_users_today?: number;
  database_query_time?: number;
}

export interface AnalyticsDashboardData {
  summary: unknown;
  trends: unknown[];
  metrics?: {
    totalRequests?: number;
  };
}

export interface DeveloperPortalStats {
  total_api_keys: number;
  active_api_keys: number;
  revoked_api_keys: number;
  expired_api_keys: number;
  total_modules: number;
  active_modules: number;
  total_requests_today: number;
  total_requests_this_month: number;
  top_modules_by_usage: unknown[];
}

export interface PlanStats {
  total_plans: number;
  active_plans: number;
  system_plans?: number;
  total_memberships: number;
  active_memberships: number;
  avg_permissions_per_plan?: number;
  by_plan: Record<string, number>;
  recent_assignments: number;
  recent_removals: number;
}

export const DEFAULT_ANALYTICS_CONFIG = {
  refreshInterval: 60000,
  revalidateOnFocus: false,
};

export const REALTIME_ANALYTICS_CONFIG = {
  refreshInterval: 10000,
  revalidateOnFocus: true,
};

export const SLOW_ANALYTICS_CONFIG = {
  refreshInterval: 300000,
  revalidateOnFocus: false,
};

// Simple utilities for combining states
/**
 *
 * @param {...any} states
 */
export const combineLoadingStates = (...states: boolean[]) => states.some(Boolean);
/**
 *
 * @param {...any} errors
 */
export const combineErrorStates = (...errors: unknown[]) => errors.some(Boolean);

// ============================================================================
// API KEYS TYPE (Admin-specific)
// ============================================================================

export interface ApiKey {
  id: string;
  client_name: string;
  total_requests: number;
  rate_limit_per_minute: number;
  last_used_at?: string;
  status: 'active' | 'revoked' | 'expired';
}

export interface ApiKeysResponse {
  keys: ApiKey[];
  api_keys?: ApiKey[];
}

export interface WalletConnection {
  wallet_address: string;
  metadata: Record<string, unknown>;
  created_at: string;
  last_auth_at?: string;
  is_active: boolean;
  active_permissions_count: number;
  connection_info: {
    is_new: boolean;
    last_seen?: number;
  };
}

export interface WalletAnalytics {
  total_in_period: number;
  daily_breakdown: Array<{
    date: string;
    connections: number;
  }>;
  period_days: number;
  avg_daily: number;
}

export interface RecentWalletsData {
  recent_wallets: WalletConnection[];
  analytics: WalletAnalytics;
  metadata: {
    limit: number;
    total_count: number;
    has_more: boolean;
    generated_at: string;
  };
}

/**
 *
 */
export function useUserStats() {
  const { data, error, isLoading, refetch } = useQuery<UserStats>({
    queryKey: ['user-stats'],
    queryFn: getUserStatsAction,
    refetchInterval: 30000, // Refresh every 30 seconds
    refetchOnWindowFocus: true,
  });

  return {
    userStats: data,
    isLoading,
    error,
    refresh: refetch,
  };
}

/**
 *
 */
export function usePlanStats() {
  const { data, error, isLoading, refetch } = useQuery<PlanStats>({
    queryKey: ['plan-stats'],
    queryFn: getPlanStatsAction,
    refetchInterval: DEFAULT_ANALYTICS_CONFIG.refreshInterval,
    refetchOnWindowFocus: DEFAULT_ANALYTICS_CONFIG.revalidateOnFocus,
  });

  return {
    planStats: data,
    isLoading,
    error,
    refresh: refetch,
  };
}

/**
 *
 */
export function usePermissionAnalytics() {
  const { data, error, isLoading, refetch } = useQuery<PermissionAnalytics>({
    queryKey: ['permission-analytics'],
    queryFn: getPermissionAnalyticsAction,
    refetchInterval: DEFAULT_ANALYTICS_CONFIG.refreshInterval,
    refetchOnWindowFocus: DEFAULT_ANALYTICS_CONFIG.revalidateOnFocus,
  });

  return {
    permissionAnalytics: data,
    isLoading,
    error,
    refresh: refetch,
  };
}

/**
 *
 */
export function useSystemMetrics() {
  const { data, error, isLoading, refetch } = useQuery<SystemMetrics>({
    queryKey: ['system-metrics'],
    queryFn: getSystemMetricsAction,
    refetchInterval: REALTIME_ANALYTICS_CONFIG.refreshInterval,
    refetchOnWindowFocus: REALTIME_ANALYTICS_CONFIG.revalidateOnFocus,
  });

  return {
    systemMetrics: data,
    isLoading,
    error,
    refresh: refetch,
  };
}

/**
 *
 * @param _dateRange
 * @param _selectedModule
 */
export function useAnalyticsDashboard(_dateRange = '7d', _selectedModule = 'all') {
  const { data, error, isLoading, refetch } = useQuery<DeveloperPortalStats>({
    queryKey: ['developer-portal-stats'],
    queryFn: getDeveloperPortalStatsAction,
    refetchInterval: DEFAULT_ANALYTICS_CONFIG.refreshInterval,
    refetchOnWindowFocus: DEFAULT_ANALYTICS_CONFIG.revalidateOnFocus,
  });

  const dashboardData: AnalyticsDashboardData | undefined = data ? {
    summary: data,
    trends: [],
    metrics: {
      totalRequests: data.total_requests_this_month
    }
  } : undefined;

  return {
    dashboardData,
    isLoading,
    error,
    refresh: refetch,
  };
}

// API Key Management
/**
 *
 */
export function useApiKeys() {
  const { data, error, isLoading, refetch } = useQuery<ApiKeysResponse>({
    queryKey: ['api-keys'],
    queryFn: getApiKeysAction,
    refetchInterval: SLOW_ANALYTICS_CONFIG.refreshInterval,
    refetchOnWindowFocus: SLOW_ANALYTICS_CONFIG.revalidateOnFocus,
  });

  return {
    apiKeys: data?.api_keys || data?.keys || [],
    isLoading,
    error,
    refresh: refetch,
  };
}

// ============================================================================
// CONSOLIDATED HOOKS
// ============================================================================

/**
 *
 */
export function useAnalyticsOverview() {
  const queryClient = useQueryClient();
  const { userStats, isLoading: userStatsLoading, error: userStatsError } = useUserStats();
  const { permissionAnalytics, isLoading: permissionLoading, error: permissionError } = usePermissionAnalytics();
  const { planStats, isLoading: planStatsLoading, error: planStatsError } = usePlanStats();
  const { systemMetrics, isLoading: systemLoading, error: systemError } = useSystemMetrics();
  const { dashboardData, isLoading: dashboardLoading, error: dashboardError } = useAnalyticsDashboard();

  const refreshAll = useCallback(() => {
    // Refresh all data sources using queryClient
    queryClient.invalidateQueries({ queryKey: ['user-stats'] });
    queryClient.invalidateQueries({ queryKey: ['permission-analytics'] });
    queryClient.invalidateQueries({ queryKey: ['plan-stats'] });
    queryClient.invalidateQueries({ queryKey: ['system-metrics'] });
    queryClient.invalidateQueries({ queryKey: ['developer-portal-stats'] });
  }, [queryClient]);

  return {
    // Data
    userStats,
    permissionAnalytics,
    planStats,
    systemMetrics,
    dashboardData,

    // Loading states (using shared utility)
    isLoading: combineLoadingStates(userStatsLoading, permissionLoading, planStatsLoading, systemLoading, dashboardLoading),

    // Error states (using shared utility)
    hasError: combineErrorStates(userStatsError, permissionError, planStatsError, systemError, dashboardError),
    errors: {
      userStats: userStatsError,
      permissions: permissionError,
      planStats: planStatsError,
      system: systemError,
      dashboard: dashboardError,
    },

    // Actions
    refreshAll,
  };
}

// Real-time metrics utility hook
/**
 *
 */
export function useRealTimeMetrics() {
  const { systemMetrics, isLoading, error } = useSystemMetrics();

  return {
    responseTime: systemMetrics?.api_response_time || 0,
    memoryUsage: systemMetrics?.memory_usage || 0,
    activeUsers: systemMetrics?.active_users || 0,
    peakUsers: systemMetrics?.peak_users_today || 0,
    dbQueryTime: systemMetrics?.database_query_time || 0,
    isLoading,
    error,
  };
}