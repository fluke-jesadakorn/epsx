'use server';

import { createAnalyticsClient } from '@/shared/api/analytics';
import { getServerActionClient } from '@/shared/utils/server-fetch';
import type { AnalyticsFilters } from '@/types/analytics';

const DEFAULT_ANALYTICS_FILTERS: AnalyticsFilters = {
    sort_by: 'eps_growth',
    page: 1,
    limit: 20,
};

/**
 * Fetch stock rankings with optional filters
 * Directly calls the backend from the server
 */
export async function getRankingsAction(filters: Partial<AnalyticsFilters> = {}) {
    const client = await getServerActionClient();
    const analytics = createAnalyticsClient(client);

    const mergedFilters = {
        ...DEFAULT_ANALYTICS_FILTERS,
        ...filters,
    } as any;

    // Map client-side 'ranking_position' to backend 'eps_growth'
    if (mergedFilters.sort_by === 'ranking_position') {
        mergedFilters.sort_by = 'eps_growth';
    }

    try {
        return await analytics.getAuthenticatedRankings(mergedFilters);
    } catch (error) {
        console.error('[getRankingsAction] Failed:', error);
        return await analytics.getPublicRankings(mergedFilters);
    }
}

/**
 * Fetch available filter options (countries, sectors, etc.)
 */
export async function getAnalyticsFiltersAction() {
    const client = await getServerActionClient();
    const analytics = createAnalyticsClient(client);

    try {
        return await analytics.getAuthenticatedFilters();
    } catch (error) {
        return await analytics.getPublicFilters();
    }
}
