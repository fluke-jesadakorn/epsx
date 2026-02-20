'use server';

import type {
    ApiKeysResponse,
    DeveloperPortalStats,
    PermissionAnalytics,
    PlanStats,
    RecentWalletsData,
    SystemMetrics,
    UserStats
} from '@/hooks/use-analytics-data';
import { redirectOnForbidden, rethrowRedirect } from '@/lib/api-error';
import { logout } from '@/lib/auth/auth';
import { createAdminApiClient, createPlansClient } from '@/shared/api';
import type { ApiResponse } from '@/shared/types/api';
import type { UnifiedApiClient } from '@/shared/utils/api-client';
import { logger } from '@/shared/utils/logger';
import { redirect } from 'next/navigation';


async function processApiResponse<T>(
    res: ApiResponse<T>,
    errorMessage: string,
    defaultValue?: T
): Promise<T> {
    if (!res.success) {
        redirectOnForbidden(res, '/analytics');

        if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
            await logout();
            redirect('/auth');
        }

        logger.error(`${errorMessage}: ${res.error?.message} (${res.error?.code})`, { error: res.error });

        if (defaultValue !== undefined) {
            return defaultValue;
        }

        throw new Error(res.error?.message ?? errorMessage);
    }

    return res.data ?? (defaultValue as T);
}

function processApiError<T>(
    error: unknown,
    errorMessage: string,
    defaultValue?: T
): T {
    rethrowRedirect(error);

    logger.error(`${errorMessage}:`, error instanceof Error ? error.message : String(error));

    if (defaultValue !== undefined) {
        return defaultValue;
    }

    throw error as Error;
}

async function handleAction<T>(
    requestFn: (apiClient: UnifiedApiClient) => Promise<ApiResponse<T>>,
    errorMessage: string,
    defaultValue?: T
): Promise<T> {
    const apiClient = createAdminApiClient({ serverSide: true });

    try {
        const res = await requestFn(apiClient);
        return await processApiResponse(res, errorMessage, defaultValue);
    } catch (error: unknown) {
        return processApiError(error, errorMessage, defaultValue);
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
