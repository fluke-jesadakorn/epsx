import { NextRequest, NextResponse } from 'next/server';
import { URL, URLContext, Service } from '../../../../../../shared/utils/url-resolver';

interface StockRanking {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number;
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

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new globalThis.URL(request.url);
    const page = parseInt(searchParams.get('page') || '1');
    const limit = parseInt(searchParams.get('limit') || '5');
    const type = searchParams.get('type') || 'preview'; // 'preview' or 'cards'
    
    const apiUrl = URL.get(Service.BACKEND, URLContext.SERVER);
    
    // Show real top stocks instead of fake ranks 101+ 
    const url = `${apiUrl}/api/v1/public/analytics/eps-rankings?page=${page}&limit=${limit}&sort_by=market_cap`;
    
    let response;
    try {
      response = await fetch(url, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
          'Cache-Control': 'no-cache',
        },
        cache: 'no-store', // NO CACHE: Force fresh data
        signal: AbortSignal.timeout(5000), // 5 second timeout
      });
    } catch (fetchError) {
      console.error('Backend unavailable, returning empty data:', fetchError);
      // Return empty data instead of failing
      return NextResponse.json([]);
    }

    if (!response.ok) {
      console.error(`Failed to fetch public rankings: ${response.status}`);
      // Return empty data instead of failing
      return NextResponse.json([]);
    }

    let apiData: ApiResponse;
    try {
      apiData = await response.json();
    } catch (parseError) {
      console.error('Failed to parse backend response:', parseError);
      return NextResponse.json([]);
    }
    
    if (!apiData.success || !Array.isArray(apiData.data)) {
      console.error('Invalid API response format:', apiData);
      return NextResponse.json({ error: 'Invalid API response' }, { status: 500 });
    }

    // Transform based on type requested
    if (type === 'cards') {
      // Transform for ClientEpsCardSection (TableDataMetrics format)
      const transformedData = apiData.data.map(stock => {
        const latestQuarter = stock.quarterly_performance[0];
        const epsGrowth = latestQuarter?.eps_growth || 0;
        const priceGrowth = latestQuarter?.price_growth || 0;
        
        return {
          symbol: stock.symbol,
          name: `Company ${stock.symbol}`,
          valueIndex: stock.value.toString(),
          growthRate: `${epsGrowth.toFixed(2)}%`,
          activityScore: "0",
          marketSize: "0",
          growthFactor: "0",
          sector: "Technology",
          country: "US",
          exchange: "NASDAQ",
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
          growthIndicator: epsGrowth > 0 ? "up" : "down",
          currentMetric: latestQuarter?.eps?.toString() || "0",
          predictedMetric: "0",
          lastAnalysisDate: stock.latest_date,
          nextAnalysisDate: stock.latest_date,
          startBuy: {
            active: epsGrowth > 5,
          },
          startAction: {
            type: epsGrowth < -5 ? 'sell' as const : 'hold' as const,
            active: Math.abs(epsGrowth) > 5,
          },
          epsGrowth: `${epsGrowth.toFixed(2)}%`,
          lastEarningsDate: stock.latest_date,
          currentQuarterEps: latestQuarter?.eps?.toString() || "0",
          nextEps: "0",
          dataValue: stock.value.toString(),
          changePercent: `${priceGrowth.toFixed(2)}%`,
          volume: "0",
          nextEarningsDate: stock.latest_date,
        };
      });

      return NextResponse.json(transformedData);
    } else {
      // Transform for PublicRankingPreview (StockFinancialData format)
      const transformedData = apiData.data.map(stock => {
        const latestGrowth = stock.quarterly_performance[0]?.eps_growth || 0;
        
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
          growth: latestGrowth,
          rank: stock.rank,
        };
      });

      return NextResponse.json(transformedData);
    }
  } catch (error) {
    console.error('Error fetching public rankings:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}