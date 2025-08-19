'use server';

import type { EPSRankingsResponse, AnalyticsFilters, FilterOptions, EPSRanking } from '@/types/analytics';
import type { CardDashboardResponse } from '@/types/financialChartData';

export async function fetchEPSRankings(filters: AnalyticsFilters): Promise<EPSRankingsResponse | null> {
  try {
    const apiUrl = process.env.API_URL || process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
    
    // Build query parameters
    const params = new URLSearchParams({
      page: filters.page.toString(),
      limit: filters.limit.toString(),
      sort_by: filters.sort_by,
    });

    if (filters.country) params.append('country', filters.country);
    if (filters.sector) params.append('sector', filters.sector);
    if (filters.min_eps) params.append('min_eps', filters.min_eps.toString());
    if (filters.min_growth) params.append('min_growth', filters.min_growth.toString());

    const url = `${apiUrl}/api/v1/analytics/rankings?${params.toString()}`;
    
    console.log('Fetching EPS rankings from:', url);

    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'application/json',
        'Cache-Control': 'no-cache',
      },
      next: { revalidate: 60 }, // Cache for 1 minute
    });

    if (!response.ok) {
      console.error(`Failed to fetch EPS rankings: ${response.status} ${response.statusText}`);
      return null;
    }

    const apiResponse: CardDashboardResponse = await response.json();
    
    // Validate response structure
    if (!apiResponse.success || !apiResponse.data || !Array.isArray(apiResponse.data)) {
      console.error('Invalid API response structure:', apiResponse);
      return null;
    }

    // Transform CardDashboardResponse to EPSRankingsResponse format
    const epsRankings: EPSRanking[] = apiResponse.data.map((card, index) => {
      const latestQuarterly = card.quarterly_performance[0] || {};
      
      return {
        symbol: card.symbol,
        name: `Company ${card.symbol}`, // Placeholder - API doesn't return company name
        country: 'US', // Default - will be extracted from metadata if available
        sector: 'Technology', // Default - API doesn't return sector in this format
        exchange: 'NASDAQ', // Default - API doesn't return exchange
        current_eps: latestQuarterly.eps || null,
        qoq_growth: latestQuarterly.eps_growth || null,
        price_current: card.value || null,
        market_cap: null, // Not available in card format
        volume: null, // Not available in card format
        ranking_position: card.rank || index + 1,
        active_status: card.active_status, // Active or Non Active from backend
        quarterly_data: card.quarterly_performance.map(q => ({
          quarter: q.quarter,
          date: q.date,
          price: q.price,
          eps: q.eps,
          eps_growth: q.eps_growth,
          price_growth: q.price_growth,
          volume: 0, // Not available
        })),
      };
    });

    const epsResponse: EPSRankingsResponse = {
      data: epsRankings,
      pagination: {
        page: apiResponse.pagination.page,
        limit: apiResponse.pagination.limit,
        total: apiResponse.pagination.total,
        totalPages: apiResponse.pagination.totalPages,
        hasNext: apiResponse.pagination.hasNext,
        hasPrev: apiResponse.pagination.hasPrev,
      },
    };

    return epsResponse;
  } catch (error) {
    console.error('Error fetching EPS rankings:', error);
    return null;
  }
}

export async function fetchFilterOptions(): Promise<FilterOptions> {
  try {
    const apiUrl = process.env.API_URL || process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
    
    // Fetch countries and sectors in parallel
    const [countriesResponse, sectorsResponse] = await Promise.all([
      fetch(`${apiUrl}/api/v1/analytics/eps-rankings/countries`, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
        next: { revalidate: 3600 }, // Cache for 1 hour
      }),
      fetch(`${apiUrl}/api/v1/analytics/eps-rankings/sectors`, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
        next: { revalidate: 3600 }, // Cache for 1 hour
      }),
    ]);

    let countries: string[] = [];
    let sectors: string[] = [];

    if (countriesResponse.ok) {
      const countriesData = await countriesResponse.json();
      countries = Array.isArray(countriesData.countries) ? countriesData.countries : [];
    }

    if (sectorsResponse.ok) {
      const sectorsData = await sectorsResponse.json();
      sectors = Array.isArray(sectorsData.sectors) ? sectorsData.sectors : [];
    }

    // Fallback data if API calls fail
    if (countries.length === 0) {
      countries = ['US', 'Canada', 'UK', 'Germany', 'France', 'Japan', 'Australia'];
    }

    if (sectors.length === 0) {
      sectors = [
        'Technology',
        'Healthcare',
        'Financial Services',
        'Consumer Discretionary',
        'Industrials',
        'Energy',
        'Telecommunications',
        'Real Estate',
      ];
    }

    return { countries, sectors };
  } catch (error) {
    console.error('Error fetching filter options:', error);
    
    // Return fallback data
    return {
      countries: ['US', 'Canada', 'UK', 'Germany', 'France', 'Japan', 'Australia'],
      sectors: [
        'Technology',
        'Healthcare', 
        'Financial Services',
        'Consumer Discretionary',
        'Industrials',
        'Energy',
        'Telecommunications',
        'Real Estate',
      ],
    };
  }
}