'use server';

import {
    ApiKeysResponse,
    DeveloperPortalStats,
    PermissionAnalytics,
    PlanStats,
    RecentWalletsData,
    SystemMetrics,
    UserStats
} from '@/hooks/useAnalyticsData';
import { logout } from '@/lib/auth/auth';
import { createAdminApiClient, createPlansClient } from '@/shared/api';
import { ApiResponse } from '@/shared/types/api';
import { UnifiedApiClient } from '@/shared/utils/api-client';
import { redirect } from 'next/navigation';

/**
 * Generic helper to execute an API request with standard error handling and 401 redirect
 */
async function handleAction<T>(
    requestFn: (apiClient: UnifiedApiClient) => Promise<ApiResponse<T>>,
    errorMessage: string,
    defaultValue?: T
): Promise<T> {
    const apiClient = createAdminApiClient({ serverSide: true });

    try {
        const res = await requestFn(apiClient);

        if (!res.success) {
            if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
                await logout();
                redirect('/auth');
            }

            console.error(`${errorMessage}: ${res.error?.message} (${res.error?.code})`);

            if (defaultValue !== undefined) {
                return defaultValue;
            }

            throw new Error(res.error?.message || errorMessage);
        }

        return res.data || (defaultValue as T);
    } catch (error: unknown) {
        // Allow Next.js redirects to bubble up
        if (error instanceof Error && (error as any).digest?.startsWith('NEXT_REDIRECT')) {
            throw error;
        }

        console.error(`${errorMessage}:`, error);

        if (defaultValue !== undefined) {
            return defaultValue;
        }

        throw error;
    }
}

export async function getUserStatsAction(): Promise<UserStats> {
    return handleAction(
        (apiClient) => apiClient.get<UserStats>('/api/admin/wallets/stats'),
        'Failed to fetch user stats',
        {} as UserStats
    );
}

export async function getPermissionAnalyticsAction(): Promise<PermissionAnalytics> {
    return handleAction(
        (apiClient) => apiClient.get<PermissionAnalytics>('/api/admin/analytics/permissions'),
        'Failed to fetch permission analytics',
        {} as PermissionAnalytics
    );
}

export async function getSystemMetricsAction(): Promise<SystemMetrics> {
    return handleAction(
        (apiClient) => apiClient.get<SystemMetrics>('/api/admin/analytics/metrics'),
        'Failed to fetch system metrics',
        {} as SystemMetrics
    );
}

export async function getDeveloperPortalStatsAction(): Promise<DeveloperPortalStats> {
    return handleAction(
        (apiClient) => apiClient.get<DeveloperPortalStats>('/api/admin/developer-portal/stats'),
        'Failed to fetch developer portal stats',
        {
            total_api_keys: 0,
            active_api_keys: 0,
            revoked_api_keys: 0,
            expired_api_keys: 0,
            total_modules: 0,
            active_modules: 0,
            total_requests_today: 0,
            total_requests_this_month: 0,
            top_modules_by_usage: []
        }
    );
}

export async function getApiKeysAction(): Promise<ApiKeysResponse> {
    return handleAction(
        (apiClient) => apiClient.get<ApiKeysResponse>('/api/admin/developer-portal/api-keys'),
        'Failed to fetch API keys',
        { keys: [] }
    );
}

export async function getRecentWalletsAction(limit = 10, days = 30): Promise<RecentWalletsData> {
    return handleAction(
        (apiClient) => apiClient.get<RecentWalletsData>(`/api/admin/web3/recent-wallets?limit=${limit}&days=${days}`),
        'Failed to fetch recent wallets'
    );
}

export async function getPlanStatsAction(): Promise<PlanStats> {
    return handleAction(
        async (apiClient) => {
            const plansClient = createPlansClient(apiClient);
            return await plansClient.getStats();
        },
        'Failed to fetch plan stats',
        {
            total_plans: 0,
            active_plans: 0,
            total_memberships: 0,
            active_memberships: 0,
            by_plan: {},
            recent_assignments: 0,
            recent_removals: 0
        }
    );
}
