'use server';

interface StockRanking {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number;
  avg_growth: number;
  quarterly_performance: Array<{
    quarter: string;
    date: string;
    price: number;
    eps: number;
    eps_growth: number;
    price_growth: number;
  }>;
}

interface ApiResponse {
  success: boolean;
  data: StockRanking[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
}

export async function fetchPublicRankingData(page = 1, limit = 12) {
  try {
    const apiUrl = process.env.API_URL || process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
    const url = `${apiUrl}/api/v1/analytics/rankings?page=${page}&limit=${limit}&sort_by=market_cap`;
    
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'application/json',
        'Cache-Control': 'no-cache',
      },
      next: { revalidate: 300 }, // Cache for 5 minutes
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const apiData: ApiResponse = await response.json();
    
    if (!apiData.success || !Array.isArray(apiData.data)) {
      throw new Error('Invalid API response format');
    }

    // Transform API data to match frontend expected format
    const transformedData = apiData.data.map(stock => {
      const latestQuarter = stock.quarterly_performance[0];
      return {
        symbol: stock.symbol,
        currentPrice: stock.value,
        quarters: stock.quarterly_performance.map(q => ({
          quarter: q.quarter,
          price: q.price,
          eps: q.eps,
          date: q.date,
        })),
        growth: stock.avg_growth,
        rank: stock.rank,
      };
    });

    return transformedData;
  } catch (error) {
    console.error('Failed to fetch public ranking data:', error);
    // Return empty array on error to prevent breaking the UI
    return [];
  }
}