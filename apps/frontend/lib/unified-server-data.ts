/**
 * Unified Server-Only Data Fetching Library
 * Provides centralized data fetching for Server Components
 */

import { createPlatformAnalyticsClient } from '@/shared/api/analytics';
import type { FilterOptions, SymbolCardData } from '@/shared/types/analytics';

export type EPSQueryParams = {
    page: number;
    limit: number;
    country?: string;
    sector?: string;
    sort_by: string;
    min_eps?: number;
    min_growth?: number;
    search?: string;
};

/**
 * Get analytics rankings data for the dashboard
 */
export async function getAnalyticsData(params: EPSQueryParams) {
    try {
        const analyticsClient = createPlatformAnalyticsClient('frontend');

        // Sort by type in shared/api/analytics is restricted, so we cast to the expected type
        const sort_by = params.sort_by as any;

        // Use the unified rankings endpoint which returns CardDashboardResponse
        // NOTE: The backend returns EPSRanking format which needs to be mapped to SymbolCardData
        const response = await analyticsClient.getAuthenticatedRankings({
            page: params.page,
            limit: params.limit,
            country: params.country === 'all' ? undefined : params.country,
            sector: params.sector === 'all' ? undefined : params.sector,
            sort_by: sort_by,
            min_eps: params.min_eps,
            min_growth: params.min_growth,
        }) as any;

        if (!response || (response.success === false)) {
            console.warn('⚠️ Analytics data fetch failed or returned empty:', response?.message);
            return {
                rankings: [],
                pagination: {
                    page: params.page || 1,
                    limit: params.limit || 10,
                    total: 0,
                    totalPages: 0,
                    hasNext: false,
                    hasPrev: false,
                }
            };
        }

        // Map backend EPSRanking data to frontend SymbolCardData
        // The backend returns loose structure that needs to be normalized
        const rawRankings = response.data || [];
        const mappedRankings: SymbolCardData[] = rawRankings.map((item: any, index: number) => {
            // Determine rank (use existing or calculate)
            const rank = item.ranking_position || item.rank || ((params.page - 1) * params.limit) + index + 1;

            // Map quarterly data if available
            const quarterlyPerformance = (item.quarterly_data || item.quarterly_performance || []).map((q: any) => ({
                quarter: q.quarter,
                date: q.date,
                price: q.price,
                eps: q.eps,
                eps_growth: q.eps_growth,
                price_growth: q.price_growth,
                is_estimated: false
            }));

            // Get latest data point for display
            const latestData = quarterlyPerformance[0] || {};

            return {
                rank: rank,
                symbol: item.symbol,
                company_name: item.name || item.company_name,
                latest_date: latestData.date || new Date().toISOString(),
                value: item.price_current || latestData.price || 0,
                active_status: item.active_status || 'Active',
                quarterly_performance: quarterlyPerformance,
                currency: item.currency || 'USD',

                // Mapped fields
                current_eps: item.current_eps,
                growth_factor: item.growth_factor,
                price_current: item.price_current,

                // Defaults for missing data
                next_quarter_estimate: item.next_quarter_estimate,
                next_earnings_date: item.next_earnings_date,
                days_until_next_earnings: item.days_until_next_earnings,
                progress_percentage: item.progress_percentage
            };
        });

        return {
            rankings: mappedRankings,
            pagination: response.pagination,
            processing_time_ms: response.processing_time_ms
        };
    } catch (error) {
        console.error('❌ Error in getAnalyticsData:', error);
        return {
            rankings: [],
            pagination: {
                page: params.page || 1,
                limit: params.limit || 10,
                total: 0,
                totalPages: 0,
                hasNext: false,
                hasPrev: false,
            }
        };
    }
}

/**
 * Get portfolio data - currently using the same rankings logic
 * but can be extended for portfolio-specific filtering
 */
export async function getPortfolioData(params: EPSQueryParams) {
    // For now, portfolio data uses the same underlying analytics engine
    // This can be modified to filter by user watchlist symbols if needed
    return getAnalyticsData(params);
}

/**
 * Get all available filter options (countries, sectors, etc.)
 */
export async function getServerFilterOptions(): Promise<FilterOptions> {
    try {
        const analyticsClient = createPlatformAnalyticsClient('frontend');
        const response = await analyticsClient.getAuthenticatedFilters();

        if (!response || !response.success) {
            throw new Error('No filter options returned from API');
        }

        // Transform from AnalyticsFiltersResponse to the expected FilterOptions format
        return {
            countries: (response.data.countries || []).map(c => ({ value: c, label: c })),
            sectors: response.data.sectors || [],
            exchanges: response.data.exchanges || [],
            stock_types: [], // Backend doesn't return this yet in the unified client
        };
    } catch (error) {
        console.error('❌ Error in getServerFilterOptions:', error);
        // Fallback options
        return {
            countries: [{ value: 'america', label: 'United States' }],
            sectors: ['Technology', 'Financial', 'Healthcare'],
            exchanges: ['NASDAQ', 'NYSE'],
            stock_types: ['common'],
        };
    }
}

export type { FilterOptions, SymbolCardData };

