// Server component for fetching analytics filter options and initial data
// Server-side data fetching for filter options
export async function getFilterOptions() {
  // Return static filter options data
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
    const { analyticsClient } = await import('@/lib/api-client');
    
    return await analyticsClient.getRankings({
      page: filters.page,
      per_page: filters.limit,
      sort_by: filters.sort_by,
      country: filters.country,
      sector: filters.sector,
    });
  } catch (error) {
    console.error('Error fetching initial analytics data:', error);
    return null;
  }
}