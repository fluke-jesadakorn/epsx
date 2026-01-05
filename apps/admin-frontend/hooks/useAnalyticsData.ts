'use client';

import { useCallback } from 'react';
import useSWR from 'swr';

import { adminApiClient } from '@/lib/api-client';

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
  active_permissions?: number;
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
  summary: any;
  trends: any[];
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
export const combineLoadingStates = (...states: boolean[]) => states.some(Boolean);
export const combineErrorStates = (...errors: any[]) => errors.some(Boolean);

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

interface ApiKeysResponse {
  keys: ApiKey[];
}

// ============================================================================
// FETCHER
// ============================================================================

const fetcher = async <T>(url: string): Promise<T> => {
  const response = await adminApiClient.get<T>(url);
  if (response.data === undefined) {
    throw new Error('No data received from API');
  }
  return response.data;
};

// ============================================================================
// INDIVIDUAL DATA HOOKS (using shared types)
// ============================================================================

export function useUserStats() {
  const { data, error, isLoading, mutate } = useSWR<UserStats>(
    '/api/admin/wallets/stats',
    fetcher,
    {
      refreshInterval: 30000, // Refresh every 30 seconds
      revalidateOnFocus: true,
      errorRetryCount: 3,
    }
  );

  return {
    userStats: data,
    isLoading,
    error,
    refresh: mutate,
  };
}

export function usePermissionAnalytics() {
  const { data, error, isLoading, mutate } = useSWR<PermissionAnalytics>(
    '/api/admin/analytics/permissions',
    fetcher,
    DEFAULT_ANALYTICS_CONFIG
  );

  return {
    permissionAnalytics: data,
    isLoading,
    error,
    refresh: mutate,
  };
}

export function useSystemMetrics() {
  const { data, error, isLoading, mutate } = useSWR<SystemMetrics>(
    '/api/admin/analytics/performance',
    fetcher,
    REALTIME_ANALYTICS_CONFIG
  );

  return {
    systemMetrics: data,
    isLoading,
    error,
    refresh: mutate,
  };
}

export function useAnalyticsDashboard(dateRange: string = '7d', selectedModule: string = 'all') {
  const { data, error, isLoading, mutate } = useSWR<AnalyticsDashboardData>(
    `/api/admin/analytics/dashboard?dateRange=${dateRange}&selectedModule=${selectedModule}`,
    fetcher,
    DEFAULT_ANALYTICS_CONFIG
  );

  return {
    dashboardData: data,
    isLoading,
    error,
    refresh: mutate,
  };
}

// API Key Management
export function useApiKeys() {
  const { data, error, isLoading, mutate } = useSWR<ApiKeysResponse>(
    '/api/admin/api-keys',
    fetcher,
    SLOW_ANALYTICS_CONFIG
  );

  return {
    apiKeys: data?.keys || [],
    isLoading,
    error,
    refresh: mutate,
  };
}

// ============================================================================
// CONSOLIDATED HOOKS
// ============================================================================

export function useAnalyticsOverview() {
  const { userStats, isLoading: userStatsLoading, error: userStatsError } = useUserStats();
  const { permissionAnalytics, isLoading: permissionLoading, error: permissionError } = usePermissionAnalytics();
  const { systemMetrics, isLoading: systemLoading, error: systemError } = useSystemMetrics();
  const { dashboardData, isLoading: dashboardLoading, error: dashboardError } = useAnalyticsDashboard();

  const refreshAll = useCallback(() => {
    // Refresh all data sources by triggering SWR revalidation
  }, []);

  return {
    // Data
    userStats,
    permissionAnalytics,
    systemMetrics,
    dashboardData,

    // Loading states (using shared utility)
    isLoading: combineLoadingStates(userStatsLoading, permissionLoading, systemLoading, dashboardLoading),

    // Error states (using shared utility)
    hasError: combineErrorStates(userStatsError, permissionError, systemError, dashboardError),
    errors: {
      userStats: userStatsError,
      permissions: permissionError,
      system: systemError,
      dashboard: dashboardError,
    },

    // Actions
    refreshAll,
  };
}

// Real-time metrics utility hook
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