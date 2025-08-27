// Server component for fetching analytics filter options and initial data
import { AnalyticsClient } from '@/lib/api-client';

// Server-side data fetching for filter options
export async function getFilterOptions() {
  try {
    const analyticsClient = new AnalyticsClient();
    
    const [countriesResponse, sectorsResponse] = await Promise.all([
      analyticsClient.getAvailableCountries(),
      analyticsClient.getSectorsByCountry(),
    ]);

    return {
      countries: countriesResponse.data.countries,
      sectors: sectorsResponse.data.sectors,
      exchanges: ['NASDAQ', 'NYSE', 'LSE', 'TSX', 'ASX', 'HKEX', 'TSE', 'EURONEXT'],
      stock_types: ['common', 'preferred', 'reit', 'etf'],
    };
  } catch (error) {
    console.error('Error fetching filter options:', error);
    
    // Return fallback data
    return {
      countries: [
        { value: 'america', label: 'United States' },
        { value: 'canada', label: 'Canada' },
        { value: 'united_kingdom', label: 'United Kingdom' },
        { value: 'germany', label: 'Germany' },
        { value: 'france', label: 'France' },
        { value: 'japan', label: 'Japan' },
        { value: 'australia', label: 'Australia' }
      ],
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
      exchanges: ['NASDAQ', 'NYSE', 'LSE', 'TSX', 'ASX', 'HKEX', 'TSE', 'EURONEXT'],
      stock_types: ['common', 'preferred', 'reit', 'etf'],
    };
  }
}

// Server-side data fetching for initial analytics data
export async function getInitialAnalyticsData(filters: {
  page: number;
  limit: number;
  sort_by: string;
  country?: string;
  sector?: string;
  min_eps?: number;
  min_growth?: number;
}) {
  try {
    const analyticsClient = new AnalyticsClient();
    
    const response = await analyticsClient.getUnifiedAnalyticsRankings({
      page: filters.page,
      limit: filters.limit,
      sort_by: filters.sort_by,
      country: filters.country,
      sector: filters.sector,
      min_eps: filters.min_eps,
      min_growth: filters.min_growth,
    });

    return response.data;
  } catch (error) {
    console.error('Error fetching initial analytics data:', error);
    return null;
  }
}