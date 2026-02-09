/**
 * Unified Server-Only Data Fetching Library
 * Provides centralized data fetching for Server Components
 */

import type { AnalyticsFiltersResponse, EPSRanking } from '@/shared/api/analytics';
import { createPlatformAnalyticsClient } from '@/shared/api/analytics';
import { COOKIES } from '@/shared/auth/cookies';
import type { CardDashboardResponse, FilterOptions, QuarterlyPerformanceData, SymbolCardData } from '@/shared/types/analytics';
import { logger } from '@/shared/utils/logger';

export interface EPSQueryParams {
    page: number;
    limit: number;
    country?: string;
    sector?: string;
    sort_by: string;
    min_eps?: number;
    min_growth?: number;
    search?: string;
}

async function hasServerAccessToken(): Promise<boolean> {
    try {
        const { cookies } = await import('next/headers');
        const cookieStore = await cookies();
        return Boolean(cookieStore.get(COOKIES.access_token)?.value);
    } catch {
        return false;
    }
}

/**
 * Get analytics rankings data for the dashboard
 */
export async function getAnalyticsData(params: EPSQueryParams) {
    try {
        const analyticsClient = createPlatformAnalyticsClient('frontend');

        // Sort by type in shared/api/analytics is restricted, so we cast to the expected type
        const sort_by = params.sort_by as 'eps_growth' | 'market_cap' | 'volume' | 'price';

        const filters = {
            page: params.page,
            limit: params.limit,
            country: params.country === 'all' ? undefined : params.country,
            sector: params.sector === 'all' ? undefined : params.sector,
            sort_by,
            min_eps: params.min_eps,
            min_growth: params.min_growth,
        };

        // Use authenticated rankings when possible, fallback to public for guests
        // NOTE: The backend returns EPSRanking format which needs to be mapped to SymbolCardData
        let response: CardDashboardResponse | null = null;
        const hasToken = await hasServerAccessToken();
        if (hasToken) {
            try {
                response = await analyticsClient.getAuthenticatedRankings(filters);
            } catch {
                // Silent fallback
            }
        }

        if (!response) {
            try {
                response = await analyticsClient.getPublicRankings(filters);
            } catch {
                return {
                    rankings: [],
                    pagination: {
                        page: params.page,
                        limit: params.limit,
                        total: 0,
                        totalPages: 0,
                        hasNext: false,
                        hasPrev: false,
                    }
                };
            }
        }

        if (!response.success) {
            logger.warn('⚠️ Analytics data fetch failed or returned empty:', response.message);
            return {
                rankings: [],
                pagination: {
                    page: params.page,
                    limit: params.limit,
                    total: 0,
                    totalPages: 0,
                    hasNext: false,
                    hasPrev: false,
                }
            };
        }

        // Map backend EPSRanking data to frontend SymbolCardData
        // The backend returns loose structure that needs to be normalized
        // Map backend EPSRanking data to frontend SymbolCardData
        // The backend returns loose structure that needs to be normalized
        const rawRankings = (response.data as unknown as EPSRanking[]);
        const mappedRankings: SymbolCardData[] = rawRankings.map((item: EPSRanking, index: number) => {
            // Determine rank (use existing or calculate)
            const rank = item.ranking_position || ((params.page - 1) * params.limit) + index + 1;

            // Map quarterly data if available
            const quarterlyPerformance: QuarterlyPerformanceData[] = item.quarterly_data.map((q) => ({
                quarter: q.quarter,
                date: q.date,
                price: q.price,
                eps: q.eps,
                eps_growth: q.eps_growth,
                price_growth: q.price_growth,
                is_estimated: false
            }));

            // Get latest data point for display
            const latestData = quarterlyPerformance[0] ?? ({ date: new Date().toISOString(), price: 0 } as QuarterlyPerformanceData);

            return {
                rank,
                symbol: item.symbol,
                company_name: item.name,
                latest_date: latestData.date,
                value: item.price_current ?? latestData.price,
                active_status: item.active_status,
                quarterly_performance: quarterlyPerformance,
                currency: 'USD',

                // Mapped fields
                current_eps: item.current_eps ?? 0,
                growth_factor: item.growth_factor ?? 0,
                price_current: item.price_current ?? 0,

                // Defaults for missing data
                next_quarter_estimate: undefined,
                next_earnings_date: undefined,
                days_until_next_earnings: undefined,
                progress_percentage: 0
            };
        });

        return {
            rankings: mappedRankings,
            pagination: response.pagination,
            processing_time_ms: 0
        };
    } catch (error) {
        logger.error('❌ Error in getAnalyticsData:', String(error));
        return {
            rankings: [],
            pagination: {
                page: params.page,
                limit: params.limit,
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
        let response: AnalyticsFiltersResponse | null = null;
        const hasToken = await hasServerAccessToken();

        if (hasToken) {
            try {
                response = await analyticsClient.getAuthenticatedFilters();
            } catch {
                // Silent fallback
            }
        }

        response ??= await analyticsClient.getPublicFilters();

        if (!response.success) {
            throw new Error('No filter options returned from API');
        }

        // Transform from AnalyticsFiltersResponse to the expected FilterOptions format
        return {
            countries: response.data.countries.map((c: string) => ({ value: c, label: c })),
            sectors: response.data.sectors,
            exchanges: response.data.exchanges,
            stock_types: [], // Backend doesn't return this yet in the unified client
        };
    } catch (error) {
        logger.error('❌ Error in getServerFilterOptions:', String(error));
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

