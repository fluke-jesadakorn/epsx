'use server';

import {
    ApiKeysResponse,
    DeveloperPortalStats,
    PermissionAnalytics,
    SystemMetrics,
    UserStats
} from '@/hooks/useAnalyticsData';
import { createAdminApiClient } from '@/shared/api';

export async function getUserStatsAction(): Promise<UserStats> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<UserStats>('/api/admin/wallets/stats');
    if (!res.success) {
        throw new Error(res.error?.message || 'Failed to fetch user stats');
    }
    return res.data || {};
}

export async function getPermissionAnalyticsAction(): Promise<PermissionAnalytics> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<PermissionAnalytics>('/api/admin/analytics/permissions');
    if (!res.success) {
        throw new Error(res.error?.message || 'Failed to fetch permission analytics');
    }
    return res.data || {};
}

export async function getSystemMetricsAction(): Promise<SystemMetrics> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<SystemMetrics>('/api/admin/analytics/metrics');
    if (!res.success) {
        throw new Error(res.error?.message || 'Failed to fetch system metrics');
    }
    return res.data || {};
}

export async function getDeveloperPortalStatsAction(): Promise<DeveloperPortalStats> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<DeveloperPortalStats>('/api/admin/developer-portal/stats');
    if (!res.success) {
        throw new Error(res.error?.message || 'Failed to fetch developer portal stats');
    }
    return res.data || {
        total_api_keys: 0,
        active_api_keys: 0,
        revoked_api_keys: 0,
        expired_api_keys: 0,
        total_modules: 0,
        active_modules: 0,
        total_requests_today: 0,
        total_requests_this_month: 0,
        top_modules_by_usage: []
    };
}

export async function getApiKeysAction(): Promise<ApiKeysResponse> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<ApiKeysResponse>('/api/admin/developer-portal/api-keys');
    if (!res.success) {
        throw new Error(res.error?.message || 'Failed to fetch API keys');
    }
    return res.data || { keys: [] };
}

export async function getRecentWalletsAction(limit = 10, days = 30): Promise<any> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<any>(`/api/admin/web3/recent-wallets?limit=${limit}&days=${days}`);
    if (!res.success) {
        throw new Error(res.error?.message || 'Failed to fetch recent wallets');
    }
    return res.data;
}
