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

// Fetch data for PublicRankingPreview (StockFinancialData format)
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

    // Transform API data to match StockFinancialData format for PublicRankingPreview
    const transformedData = apiData.data.map(stock => {
      return {
        symbol: stock.symbol,
        currentPrice: stock.value,
        quarters: stock.quarterly_performance.map(q => ({
          quarter: q.quarter,
          price: q.price,
          eps: q.eps,
          date: q.date,
          eps_growth: q.eps_growth,
          price_growth: q.price_growth,
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

// Fetch data for ClientEpsCardSection (TableDataMetrics format)  
export async function fetchEpsCardData(page = 1, limit = 3) {
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

    // Transform API data to match TableDataMetrics format for ClientEpsCardSection
    const transformedData = apiData.data.map(stock => {
      const latestQuarter = stock.quarterly_performance[0];
      return {
        symbol: stock.symbol,
        name: `Company ${stock.symbol}`, // Placeholder name
        valueIndex: stock.value.toString(),
        growthRate: `${stock.avg_growth}%`,
        activityScore: "0",
        marketSize: "0",
        growthFactor: "0",
        sector: "Technology", // Placeholder sector
        country: "US", // Placeholder country
        exchange: "NASDAQ", // Placeholder exchange
        currency: "USD",
        entryPhase: {
          date: stock.latest_date,
          active: true,
        },
        phaseStatus: {
          date: stock.latest_date,
          type: 'monitor' as const,
          active: true,
        },
        metricScore: "0",
        growthIndicator: stock.avg_growth > 0 ? "up" : "down",
        currentMetric: latestQuarter?.eps?.toString() || "0",
        predictedMetric: "0",
        lastAnalysisDate: stock.latest_date,
        nextAnalysisDate: stock.latest_date,
        // Required fields for ClientEpsCardSection
        startBuy: {
          active: stock.avg_growth > 5,
        },
        startAction: {
          type: stock.avg_growth < -5 ? 'sell' as const : 'hold' as const,
          active: Math.abs(stock.avg_growth) > 5,
        },
        epsGrowth: `${stock.avg_growth.toFixed(2)}%`,
        lastEarningsDate: stock.latest_date,
        currentQuarterEps: latestQuarter?.eps?.toString() || "0",
        nextEps: "0",
        dataValue: stock.value.toString(),
        changePercent: `${stock.avg_growth.toFixed(2)}%`,
        volume: "0",
        nextEarningsDate: stock.latest_date,
      };
    });

    return transformedData;
  } catch (error) {
    console.error('Failed to fetch EPS card data:', error);
    // Return empty array on error to prevent breaking the UI
    return [];
  }
}