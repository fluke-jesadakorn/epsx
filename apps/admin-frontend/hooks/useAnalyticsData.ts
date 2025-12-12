'use client';

import { useCallback } from 'react';
import useSWR from 'swr';

import { adminApiClient } from '@/lib/api-client';

// Client-side interfaces for analytics data
interface AnalyticsDashboardData {
  user_stats?: any;
  permission_analytics?: any;
  system_metrics?: any;
  [key: string]: any;
}

interface UserStats {
  total_users: number;
  active_users: number;
  deleted_users: number;
  recent_users_30_days: number;
  by_permissions: Record<string, number>;
  by_tier: Record<string, number>;
  user_creation_by_month: Record<string, number>;
  generated_at: string;
}

interface PermissionAnalytics {
  total_permissions: number;
  users_with_permissions: number;
  expiring_soon: number;
  expired: number;
  health_score: number;
  recent_activity: number;
}

interface SystemMetrics {
  api_response_time: number;
  database_query_time: number;
  memory_usage: number;
  active_users: number;
  peak_users_today: number;
  new_signups: number;
}

interface ApiKeysResponse {
  keys: Array<{
    id: string;
    name: string;
    created_at: string;
    last_used?: string;
    status: string;
  }>;
}

// Fetcher function for SWR - uses adminApiClient
const fetcher = async <T>(url: string): Promise<T> => {
  const response = await adminApiClient.get<T>(url);
  if (response.data === undefined) {
    throw new Error('No data received from API');
  }
  return response.data;
};

// Real-time data fetching hooks
export function useUserStats() {
  const { data, error, isLoading, mutate } = useSWR<UserStats>(
    '/api/admin/users/stats',
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
    {
      refreshInterval: 60000, // Refresh every minute
      revalidateOnFocus: true,
      errorRetryCount: 3,
    }
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
    {
      refreshInterval: 10000, // Refresh every 10 seconds for real-time system metrics
      revalidateOnFocus: true,
      errorRetryCount: 3,
    }
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
    {
      refreshInterval: 60000, // Refresh every minute
      revalidateOnFocus: true,
      errorRetryCount: 3,
    }
  );

  return {
    dashboardData: data,
    isLoading,
    error,
    refresh: mutate,
  };
}

// API Key Management - Server-side data fetching
export function useApiKeys() {
  const { data, error, isLoading, mutate } = useSWR<ApiKeysResponse>(
    '/api/admin/api-keys',
    fetcher,
    {
      refreshInterval: 300000, // Refresh every 5 minutes
      revalidateOnFocus: true,
      errorRetryCount: 3,
    }
  );

  return {
    apiKeys: data?.keys || [],
    isLoading,
    error,
    refresh: mutate,
  };
}

// Consolidated analytics hook for the main dashboard
export function useAnalyticsOverview() {
  const { userStats, isLoading: userStatsLoading, error: userStatsError } = useUserStats();
  const { permissionAnalytics, isLoading: permissionLoading, error: permissionError } = usePermissionAnalytics();
  const { systemMetrics, isLoading: systemLoading, error: systemError } = useSystemMetrics();
  const { dashboardData, isLoading: dashboardLoading, error: dashboardError } = useAnalyticsDashboard();

  const refreshAll = useCallback(() => {
    // Refresh all data sources by triggering SWR revalidation
    // This will be automatically implemented by SWR when we have the mutate functions
  }, []);

  return {
    // Data
    userStats,
    permissionAnalytics,
    systemMetrics,
    dashboardData,

    // Loading states
    isLoading: userStatsLoading || permissionLoading || systemLoading || dashboardLoading,

    // Error states
    hasError: !!(userStatsError || permissionError || systemError || dashboardError),
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

// Utility hook for real-time updates
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