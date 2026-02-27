'use server';

import type { AnalyticsFilters } from '@/shared/api/analytics';
import { createAnalyticsClient } from '@/shared/api/analytics';
import { logger } from '@/lib/logger';
import { getServerActionClient } from '@/shared/utils/server-fetch';

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
    const client = getServerActionClient();
    const analytics = createAnalyticsClient(client);

    const mergedFilters: AnalyticsFilters = {
        ...DEFAULT_ANALYTICS_FILTERS,
        ...filters,
    };

    if ((mergedFilters.sort_by as string) === 'ranking_position') {
        mergedFilters.sort_by = 'eps_growth';
    }

    try {
        return await analytics.getAuthenticatedRankings(mergedFilters);
    } catch (error) {
        logger.action.error('getRankingsAction', error);
        return await analytics.getPublicRankings(mergedFilters);
    }
}

/**
 * Fetch available filter options (countries, sectors, etc.)
 */
export async function getAnalyticsFiltersAction() {
    const client = getServerActionClient();
    const analytics = createAnalyticsClient(client);

    try {
        return await analytics.getAuthenticatedFilters();
    } catch (_error) {
        return await analytics.getPublicFilters();
    }
}
