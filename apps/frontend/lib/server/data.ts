/**
 * Unified Server-Only Data Fetching Library
 * Provides centralized data fetching for Server Components
 */

import type { AnalyticsFiltersResponse } from '@/shared/api/analytics';
import { createPlatformAnalyticsClient } from '@/shared/api/analytics';
import { COOKIES } from '@/shared/auth/cookies';
import type { CardDashboardResponse, FilterOptions, SymbolCardData } from '@/shared/types/analytics';
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

        // Backend already returns SymbolCardData[] with correct rank values (offset-adjusted)
        // Apply fallback rank only if rank is missing (shouldn't happen)
        const rankings: SymbolCardData[] = response.data.map((item, index) => ({
            ...item,
            rank: item.rank || ((params.page - 1) * params.limit) + index + 1,
        }));

        return {
            rankings,
            pagination: response.pagination,
            access_info: response.access_info,
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
            countries: response.data.countries.map((c: unknown) => {
                if (typeof c === 'string') return { value: c, label: c };
                const obj = c as { value?: string; label?: string };
                return { value: String(obj.value ?? ''), label: String(obj.label ?? obj.value ?? '') };
            }),
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

